  // Opening a chat is instant only when Messenger already holds its newest
  // messages in memory (its LightSpeed database lives only in the page).
  // Otherwise the list waits on the server (plain chats) or its worker's
  // encrypted store (end-to-end encrypted chats). Messenger reads ahead only
  // for the chat under the pointer; fMess reads ahead, with the same calls,
  // for the chats at the top of the list. Reading ahead opens no chat, so nothing is marked
  // as seen, and the older messages it brings in raise no notification
  // (Messenger notifies only for messages newer than any it has seen).
  const WARM = {
    chats: 12,
    messages: 40, // a window's worth, and some to scroll up through
    plainRounds: 2, // a server page each
    plainGapMs: 1200,
    plainPerHour: 60,
    plainAgainMs: 5 * 60 * 1000, // per chat
    secureGapMs: 200, // the worker reads its own store
  };
  let warmChats = settings.warmChats !== false;
  try {
    const changed = localStorage.getItem('fmess.warmChats');
    if (changed !== null) warmChats = changed !== '0';
  } catch {}
  const WARM_MODULES = ['ReQL', 'LSDatabaseSingleton', 'LSFactory', 'LSIntEnum', 'LSIssueMessagesRangeQueryStoredProcedure', 'LSMessagingThreadTypeUtil'];
  const AUTHORITATIVE = 80n; // LSAuthorityLevel
  const HIDDEN_ROW = 2n; // LSMessageRenderingType
  const askedAt = new Map();
  const asks = [];
  let warming = false;
  let warmTimer = null;
  let lastWarm = 0;
  const pause = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

  // Friends' 1:1 and group chats at the top of the inbox, plain or encrypted.
  // Encrypted chats' rows are all marked isHidden (the chat list shows them
  // through other tables), so isHidden only rules out plain ones.
  // Disappearing-message chats are left alone: their timers are Messenger's.
  const recentChats = async (db, ReQL) => {
    const util = window.require('LSMessagingThreadTypeUtil');
    const rows = await ReQL.toArrayAsync(ReQL.fromTableDescending(db.tables.threads.index('lastActivityTimestampMs')).take(WARM.chats * 3));
    return rows
      .filter((chat) => util.isMessengerOrE2EEInbox(chat.folderName) && util.isOpenOrArmadilloFnFThread(chat))
      .filter((chat) => (!chat.isHidden || util.isArmadilloSecure(chat.threadType)) && !chat.isDisappearingMode)
      .filter((chat) => int64(chat.authorityLevel) === AUTHORITATIVE)
      .slice(0, WARM.chats)
      .map((chat) => ({ chat, secure: util.isArmadilloSecure(chat.threadType) }));
  };

  // A plain chat: the list shows rows from its newest range and asks for more
  // when it has fewer than a page. fMess asks the server for the page before
  // that range, as scrolling up (and Messenger's hover read-ahead) does.
  const newestRange = (db, ReQL, chat) =>
    ReQL.firstAsync(ReQL.fromTableDescending(db.tables.messages_ranges_v2__generated).getKeyRange(chat.threadKey));
  const rowsInRange = async (db, ReQL, chat, range) => {
    const rows = await ReQL.toArrayAsync(ReQL.fromTableDescending(db.tables.messages).getKeyRange(chat.threadKey).take(WARM.messages + 10));
    const from = int64(range.minTimestampMs);
    return rows.filter((row) => int64(row.timestampMs) >= from && row.isCollapsed !== true && int64(row.messageRenderingType) !== HIDDEN_ROW).length;
  };
  const warmPlainChat = async (db, ReQL, chat) => {
    for (let round = 0; round < WARM.plainRounds; round++) {
      // No range: the server hasn't sent the chat's newest message yet, and
      // Messenger can't ask before it either.
      const range = await newestRange(db, ReQL, chat);
      if (!range || !range.hasMoreBefore || range.isLoadingBefore) return;
      if ((await rowsInRange(db, ReQL, chat, range)) >= WARM.messages) return;
      while (asks.length && asks[0] < Date.now() - 3600e3) asks.shift();
      if (asks.length >= WARM.plainPerHour) return;
      asks.push(Date.now());
      askedAt.set(String(int64(chat.threadKey)), Date.now());
      const query = moduleFunction(window.require('LSIssueMessagesRangeQueryStoredProcedure'));
      const factory = moduleFunction(window.require('LSFactory'));
      const before = window.require('LSIntEnum').ofNumber(0); // LSMailboxMessagesRangeQueryDirection.BEFORE
      await db.runInTransaction(
        (tx) => query(factory(tx), { direction: before, referenceTimestampMs: range.minTimestampMs, threadKey: chat.threadKey }),
        'readwrite',
      );
      // The answer clears isLoadingBefore on the range.
      for (let wait = 0; wait < 20 && (await newestRange(db, ReQL, chat))?.isLoadingBefore; wait++) await pause(300);
      await pause(WARM.plainGapMs);
    }
  };

  // An encrypted chat: what pressing its row in the chat list does
  // (useMWPreloadThread): Messenger's worker loads its newest messages from
  // its own store into the page. It does nothing for a chat already loaded,
  // so asking again is cheap. The worker knows a chat only once it is mapped.
  const SECURE_MODULES = ['MAWPreloadSecureMessages'];
  const warmSecureChat = async (db, ReQL, chat) => {
    if (!(await ReQL.firstAsync(ReQL.fromTableAscending(db.tables.mi_act_mapping_table).getKeyRange(chat.threadKey)))) return;
    const { preloadSecureMessages } = window.require('MAWPreloadSecureMessages');
    await Promise.race([preloadSecureMessages(db, chat.threadKey, WARM.messages, chat.authorityLevel), pause(15000)]);
    await pause(WARM.secureGapMs);
  };

  const warmRecentChats = async () => {
    if (warming || !onChatsPage() || navigator.onLine === false) return;
    warming = true;
    try {
      await loadModules(WARM_MODULES);
      const ReQL = window.require('ReQL');
      // The singleton is the database the chat view reads: probes showed
      // chats opening from what was read ahead into it.
      const db = await window.require('LSDatabaseSingleton').LSDatabaseSingleton;
      const secureReady = await loadModules(SECURE_MODULES).then(() => true, () => false);
      for (const { chat, secure } of await recentChats(db, ReQL)) {
        const key = String(int64(chat.threadKey));
        // The open chat is loaded already.
        if (location.pathname.match(/\/t\/(\d+)/)?.[1] === key) continue;
        if (secure) {
          if (secureReady) await warmSecureChat(db, ReQL, chat);
        } else if (Date.now() - (askedAt.get(key) ?? 0) > WARM.plainAgainMs) {
          await warmPlainChat(db, ReQL, chat);
        }
      }
    } catch {
      // Facebook changed something this relies on; the next round tries again.
    } finally {
      warming = false;
    }
  };
  // At most a round every 20 s, however often it is asked for.
  const scheduleWarm = (delay) => {
    if (!warmChats || warmTimer) return;
    warmTimer = setTimeout(() => {
      warmTimer = null;
      const wait = lastWarm + 20000 - Date.now();
      if (wait > 0) return scheduleWarm(wait);
      lastWarm = Date.now();
      warmRecentChats();
    }, delay);
  };
  if (warmChats && !window.opener) {
    const start = setInterval(async () => {
      if (!chatLinks().length) return;
      clearInterval(start);
      // Once the first sync has settled; again when the window is left, every
      // few minutes, and when a chat changes (a new message can leave Messenger
      // with only that one, or move a chat into the top of the list).
      scheduleWarm(8000);
      window.addEventListener('blur', () => scheduleWarm(2000));
      setInterval(() => scheduleWarm(0), 4 * 60 * 1000);
      try {
        await loadModules(['ReQL', 'LSDatabaseSingleton']);
        const db = await window.require('LSDatabaseSingleton').LSDatabaseSingleton;
        window.require('ReQL').fromTableAscending(db.tables.threads).subscribe(() => scheduleWarm(5000));
      } catch {}
    }, 1000);
  }

