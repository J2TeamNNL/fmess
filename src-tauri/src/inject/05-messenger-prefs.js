  // Loads Facebook modules on demand; a name Facebook dropped would never call back.
  const loadModules = (names) =>
    new Promise((resolve, reject) => {
      const timer = setTimeout(() => reject(new Error(`could not load ${names.join(', ')}`)), 10000);
      window.require('Bootloader').loadModules(names, () => (clearTimeout(timer), resolve()), 'fmess');
    });
  const moduleFunction = (module) => (typeof module === 'function' ? module : module.default);
  // Lightspeed keeps 64-bit numbers as [high, low], and keys rows by their id
  // that way (an account's settings, a contact, a chat).
  const int64 = (value) => (Array.isArray(value) ? (BigInt(value[0]) << 32n) + BigInt(value[1] >>> 0) : BigInt(value ?? 0));
  const int64Key = (id) => [Number(BigInt(id) >> 32n), Number(BigInt(id) & 0xffffffffn)];
  const serverNow = () => {
    try {
      return window.require('ServerTime').getMillis();
    } catch {
      return Date.now();
    }
  };
  // Messages that arrive this late aren't news (see isOldNews).
  const LATE_MS = 2 * 60 * 1000;

  // The rest of Messenger's Preferences (⋯ → Preferences) belong to the
  // account as well. fMess's Settings show them: the page reads them here
  // and reports them when they may have changed, and changes them the way
  // that dialog does (__fmessSetMessengerPref).
  // Some of the modules involved come only with the code of that dialog, which
  // Messenger loads when it first opens; fMess loads it the same way.
  const loadPreferences = async () => {
    await loadModules(['MWInboxSettingsDialog.entrypoint']);
    await window.require('MWInboxSettingsDialog.entrypoint').root.load();
  };
  const PREF_MODULES = ['CometRelayEnvironment', 'relay-runtime', 'resolveActiveStatus', 'CometDarkMode', 'LSDatabaseSingleton'];
  const readMessengerPrefs = async () => {
    const me = currentUserId();
    if (!me) return null;
    await loadPreferences();
    await loadModules(PREF_MODULES);
    const [env, relay, resolveActiveStatus, darkMode, database] = PREF_MODULES.map((name) => window.require(name));
    const query = window.require('useCometHomeChatSettingsQuery.graphql');
    const { viewer } = (await relay.fetchQuery(env, query, {}, { fetchPolicy: 'store-or-network' }).toPromise()) ?? {};
    const db = await database.LSDatabaseSingleton;
    const notifications = await db.tables.notification_settings.get(int64Key(me));
    // Right after loading, Messenger may not have synced its settings yet.
    if (!notifications) return null;
    return {
      soundsOn: viewer?.chat_sounds_on ?? true,
      activeStatusOn: viewer
        ? moduleFunction(resolveActiveStatus)(viewer.chat_visibility, viewer.chat_visibility_on_messenger, viewer.chat_visibility_on_facebook)
        : null,
      // Seconds since 1970 when Do Not Disturb ends: 0 when off, -1 until turned off.
      muteUntil: Number(int64(notifications.globalMuteUntil)),
      darkMode: darkMode.getDarkModeSetting(),
    };
  };
  let messengerPrefs = null;
  let reportedPrefs = '';
  // Messenger also keeps Do Not Disturb for this computer's notifications in
  // IndexedDB, and leaves the end time there when Do Not Disturb runs out;
  // clear it the way turning it off in Messenger does.
  let clearedLocalMute = false;
  const clearLocalMute = async () => {
    clearedLocalMute = true;
    await loadModules(['MessengerNotificationsWriter']);
    window.require('MessengerNotificationsWriter').updateMute(currentUserId(), 0);
  };
  // Whether the preferences could be read.
  // Whether Messenger is dark: its own setting, or the system's when it follows
  // the system (Facebook then marks only parts of the page). Themes key their
  // dark colours on it: html[data-fmess-dark].
  const systemDark = matchMedia('(prefers-color-scheme: dark)');
  const syncDark = () => {
    const setting = messengerPrefs?.darkMode;
    const dark =
      setting === 'ENABLED' || (setting === 'DISABLED' ? false : setting === 'USE_SYSTEM' ? systemDark.matches : (window.__fmessDark ?? systemDark.matches));
    withRoot((root) => root.toggleAttribute('data-fmess-dark', Boolean(dark)));
  };
  syncDark();
  systemDark.addEventListener('change', syncDark);
  const reportMessengerPrefs = async () => {
    const prefs = await readMessengerPrefs().catch(() => null);
    if (!prefs) return false;
    messengerPrefs = prefs;
    syncDark();
    if (!isMuted() && !clearedLocalMute) clearLocalMute().catch(() => {});
    const json = JSON.stringify(prefs);
    if (json === reportedPrefs) return true;
    reportedPrefs = json;
    window.__TAURI_INTERNALS__?.invoke('sync_messenger_prefs', { prefs }).catch(() => {});
    return true;
  };
  // Right after loading they may not be there yet: try every few seconds.
  const reportOnceRead = async (tries = 0) => {
    if (!(await reportMessengerPrefs()) && tries < 20) setTimeout(() => reportOnceRead(tries + 1), 3000);
  };
  if (!window.opener) {
    window.addEventListener('load', () => {
      setTimeout(reportOnceRead, 2000);
      // A new dark mode setting (from fMess's Settings, Messenger's own
      // Preferences or the phone) recolours fMess's windows at once.
      loadModules(['CometDarkMode'])
        .then(() => window.require('CometDarkMode').onDarkModeChange(() => reportMessengerPrefs()))
        .catch(() => {});
    });
    window.addEventListener('focus', reportMessengerPrefs);
    setInterval(reportMessengerPrefs, 30000);
  }
  const isMuted = () => {
    const until = messengerPrefs?.muteUntil ?? 0;
    return until < 0 || until * 1000 > Date.now();
  };
  // Messenger still plays its message sound during Do Not Disturb; fMess
  // keeps it quiet. Its sounds are <audio> files (SoundPlayer); call audio is
  // a stream, and ringing loops, so calls still ring.
  const nativePlay = HTMLMediaElement.prototype.play;
  HTMLMediaElement.prototype.play = function (...args) {
    if (this instanceof HTMLAudioElement && this.src && !this.loop && isMuted()) return Promise.resolve();
    return nativePlay.apply(this, args);
  };
  // Do Not Disturb for ms milliseconds; -1 until turned off, 0 turns it off.
  const muteFor = async (ms) => {
    const names = ['I64', 'ServerTime', 'LSDatabaseSingleton', 'LSFactory', 'MessengerNotificationsWriter'];
    await loadPreferences();
    await loadModules(names);
    const [I64, serverTime, database, factory, writer] = names.map((name) => window.require(name));
    const update = window.require('LSOptimisticallyUpdateGlobalMuteUntilStoredProcedure');
    const until = ms === 0 ? I64.zero : ms < 0 ? I64.neg_one : I64.of_float((serverTime.getMillis() + ms) / 1e3);
    const db = await database.LSDatabaseSingleton;
    await db.runInTransaction((tx) => moduleFunction(update)(moduleFunction(factory)(tx), { globalMuteUntil: until }), 'readwrite');
    // What Messenger's push notifications on this computer check.
    writer.updateMute(currentUserId(), ms === 0 ? 0 : I64.to_float(until));
  };
  // Called by fMess's Settings.
  window.__fmessSetMessengerPref = async (name, value) => {
    try {
      if (name === 'soundsOn') {
        await loadPreferences();
        await loadModules(['CometRelayEnvironment']);
        await window.require('MWChatChangeSoundsSettingMutation').commit(window.require('CometRelayEnvironment'), { sounds_on: Boolean(value) });
      } else if (name === 'darkMode') {
        if (window.__fmessSetDarkMode) {
          window.__fmessSetDarkMode(value);
        } else {
          // Saved, but the page keeps its look until it reloads.
          await loadModules(['CometDarkMode']);
          window.require('CometDarkMode').saveDarkModeSetting(value, { onRevert() {} });
        }
      } else if (name === 'muteFor') {
        await muteFor(Number(value));
      }
    } catch (error) {
      console.warn('[fMess] could not change', name, error);
    }
    reportedPrefs = '';
    setTimeout(reportMessengerPrefs, 1000);
  };
  // Opens Messenger's own Preferences, or its Active Status page, for what
  // fMess's Settings leave to it (Active Status keeps lists of people).
  window.__fmessOpenMessengerPreferences = async (section) => {
    const wait = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
    const find = async (test) => {
      for (let waited = 0; waited < 8000; waited += 200) {
        const found = test();
        if (found) return found;
        await wait(200);
      }
      return null;
    };
    let header = document.querySelector('a[href="/messages/new/"]');
    while (header && !header.querySelector('h1')) header = header.parentElement;
    header && [...header.querySelectorAll('[role="button"]')].find((el) => !el.closest('a') && el.id !== 'fmess-account')?.click();
    // Preferences heads the ⋯ menu.
    const item = await find(() => {
      const items = [...document.querySelectorAll('[role="menuitem"], [role="menu"] [role="button"]')];
      return items.find((el) => /^(preferences|tùy chọn|tuỳ chọn)$/i.test((el.innerText || '').trim())) ?? items[0];
    });
    item?.click();
    if (section !== 'activeStatus') return;
    // The Preferences dialog is the one with the dark mode choices; Active
    // Status is its first button with text (after the profile link).
    const dialog = await find(() => [...document.querySelectorAll('[role="dialog"]')].find((el) => el.querySelector('input[type="radio"]')));
    if (!dialog) return;
    [...dialog.querySelectorAll('[role="button"]')].find((el) => (el.innerText || '').trim())?.click();
  };

