// fMess on Instagram: marks the page for themes (data-fmess-*), keeps a strip
// under the macOS window buttons to drag the window by, counts unread chats
// and hands notifications to the OS notification center.
// Runs at document start, before the page's scripts.
(() => {
  if (window.top !== window || window.__fmessInstalled) return;
  window.__fmessInstalled = true;

  const popup = Boolean(window.opener);
  const settings = window.__fmessSettings ?? {};
  const invoke = (command, args) =>
    window.__TAURI_INTERNALS__
      ? window.__TAURI_INTERNALS__.invoke(command, args)
      : Promise.reject(new Error('fMess bridge unavailable'));
  // WebView2 (Windows) runs this before the page has its <html> element.
  const withRoot = (use) => {
    if (document.documentElement) return use(document.documentElement);
    const wait = new MutationObserver(() => {
      if (!document.documentElement) return;
      wait.disconnect();
      use(document.documentElement);
    });
    wait.observe(document, { childList: true });
  };

  // Below 900px wide Instagram draws an avatars-only inbox: its list reads
  // matchMedia('(max-width: 900px)') once, when the inbox mounts. fMess's
  // window is that narrow, so that one query says "no" and the full list shows.
  if (!popup) {
    const matchMedia = window.matchMedia.bind(window);
    const never = (media) => ({ matches: false, media, onchange: null, addEventListener() {}, removeEventListener() {}, addListener() {}, removeListener() {}, dispatchEvent: () => false });
    window.matchMedia = (query) => (/^\s*\(\s*max-width:\s*900px\s*\)\s*$/.test(query) ? never(query) : matchMedia(query));
  }

  // One theme serves every service and the rail: each picks its own part by
  // this attribute. On macOS the title bar overlays the page; as on
  // Messenger, its strip drags the window, and themes can draw in it.
  const titleBarInset = popup ? 0 : window.__fmessTitleBarInset || 0;
  withRoot((root) => {
    if (popup) return;
    root.setAttribute('data-fmess-service', 'instagram');
    const layout = document.createElement('style');
    layout.textContent = `:root { --fmess-title-bar: ${titleBarInset}px; }`;
    root.append(layout);
    if (!titleBarInset) return;
    const strip = document.createElement('div');
    strip.setAttribute('data-tauri-drag-region', '');
    strip.style.cssText = `position: fixed; inset: 0 0 auto 0; height: ${titleBarInset}px; z-index: 2147483647;`;
    root.append(strip);
  });

  // The chosen theme, last in the page so it wins over Instagram's styles.
  let theme = null;
  const applyTheme = (next) => {
    theme = next?.css && !popup ? next : null;
    withRoot((root) => {
      let style = document.getElementById('fmess-theme');
      if (!theme) return style?.remove();
      if (!style) {
        style = document.createElement('style');
        style.id = 'fmess-theme';
      }
      style.textContent = theme.css;
      root.append(style);
    });
  };
  applyTheme(settings.theme);
  window.addEventListener('load', () => theme && applyTheme(theme));
  // Called by fMess after every page load and when its settings change.
  window.__fmessApplySettings = (next) => {
    if (next.theme?.css !== theme?.css) applyTheme(next.theme);
  };

  // Windows and Linux have no menu bar: the shortcuts fMess's menu has there
  // that make sense here start in the page (menu.rs).
  if (!/Mac/.test(navigator.platform)) {
    const SERVICE_KEYS = { Digit1: 'messenger', Digit2: 'instagram' };
    const MENU_KEYS = { Comma: 'settings', Equal: 'zoom-in', NumpadAdd: 'zoom-in', Minus: 'zoom-out', NumpadSubtract: 'zoom-out', Digit0: 'zoom-reset', Numpad0: 'zoom-reset' };
    window.addEventListener(
      'keydown',
      (event) => {
        if (event.shiftKey || event.altKey || !event.ctrlKey) return;
        const id = event.metaKey ? SERVICE_KEYS[event.code] : MENU_KEYS[event.code];
        if (!id) return;
        event.preventDefault();
        event.stopPropagation();
        invoke('menu_shortcut', { id }).catch(() => {});
      },
      true,
    );
  }

  // Notifications go to fMess (notify.rs), which follows its settings.
  const shown = new Map();
  let nextId = Math.floor(Math.random() * 1e9);
  // When the page last raised a notification itself (see notifyNewestChat).
  let lastPageNotification = 0;
  let raisingFallback = false;
  const shownTags = new Set();
  const threadOf = (text) => /\/direct\/t\/(\d+)/.exec(String(text ?? ''))?.[1] ?? null;
  const pictureData = async (url) => {
    if (!url || typeof url !== 'string') return null;
    try {
      const response = await fetch(url);
      const blob = response.ok ? await response.blob() : null;
      if (!blob || blob.size > 4_000_000) return null;
      const bytes = new Uint8Array(await blob.arrayBuffer());
      let binary = '';
      for (let i = 0; i < bytes.length; i += 0x8000) binary += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
      return btoa(binary);
    } catch {
      return null;
    }
  };
  const within = (promise, ms) => Promise.race([promise, new Promise((resolve) => setTimeout(() => resolve(null), ms))]);

  class FMessNotification extends EventTarget {
    static get permission() {
      return 'granted';
    }

    static requestPermission(callback) {
      callback?.('granted');
      return Promise.resolve('granted');
    }

    constructor(title, options = {}) {
      super();
      this.title = String(title ?? '');
      this.body = String(options.body ?? '');
      this.tag = options.tag ?? '';
      this.icon = options.icon ?? '';
      this.data = options.data ?? null;
      this.onclick = this.onshow = this.onclose = this.onerror = null;
      let data = '';
      try {
        data = JSON.stringify(this.data) ?? '';
      } catch {}
      this.fmessThread = options.fmessThread ?? threadOf(`${this.tag} ${data}`);
      if (!raisingFallback) lastPageNotification = Date.now();
      if (this.tag) {
        if (shownTags.has(this.tag)) return;
        shownTags.add(this.tag);
        if (shownTags.size > 500) shownTags.delete(shownTags.values().next().value);
      }
      // The native side takes an i32 id.
      this.fmessId = nextId++ % 0x7fffffff;
      shown.set(this.fmessId, this);
      if (shown.size > 100) shown.delete(shown.keys().next().value);
      within(pictureData(this.icon), 3000)
        .then((icon) =>
          invoke('notify', {
            id: this.fmessId,
            title: this.title,
            body: this.body,
            icon: icon ?? null,
            image: null,
            message: this.fmessThread ? { thread: this.fmessThread, sender: this.title, senderPicture: null, group: null } : null,
          }),
        )
        .then(
          () => this.fmessFire('show'),
          () => this.fmessFire('error'),
        );
    }

    close() {
      if (shown.delete(this.fmessId)) this.fmessFire('close');
    }

    fmessFire(type) {
      const event = new Event(type);
      this[`on${type}`]?.call(this, event);
      this.dispatchEvent(event);
    }
  }
  window.Notification = FMessNotification;
  if (window.ServiceWorkerRegistration) {
    ServiceWorkerRegistration.prototype.showNotification = function (title, options) {
      new FMessNotification(title, options);
      return Promise.resolve();
    };
  }
  const query = navigator.permissions?.query?.bind(navigator.permissions);
  if (query) {
    navigator.permissions.query = (descriptor) =>
      descriptor?.name === 'notifications'
        ? Promise.resolve({ name: 'notifications', state: 'granted', onchange: null, addEventListener() {}, removeEventListener() {} })
        : query(descriptor);
  }

  // Called from Rust when the user clicks a native notification. The page's
  // own handler goes first; if it doesn't move, fMess opens the chat.
  const openThread = (thread) => {
    const path = thread ? `/direct/t/${thread}/` : '/direct/inbox/';
    if (location.pathname === path || (!thread && location.pathname.startsWith('/direct/'))) return;
    const row = thread && chatRows().find((candidate) => threadKeyOf(candidate) === thread);
    if (row) row.click();
    else {
      const link = document.querySelector(`a[href="${path}"]`);
      if (link) return link.click();
    }
    setTimeout(() => location.pathname !== path && !(!thread && location.pathname.startsWith('/direct/')) && location.assign(path), row ? 400 : 0);
  };
  window.__fmessNotificationClicked = (id, thread) => {
    window.focus();
    const notification = shown.get(id);
    const before = location.pathname;
    notification?.fmessFire('click');
    const target = thread || notification?.fmessThread;
    setTimeout(() => location.pathname === before && openThread(target), notification ? 300 : 0);
  };

  if (popup) return;

  // unread-pure:start
  // The title reads "(3) Instagram" with three unread chats, when it does.
  const unreadFromTitle = (title) => Number(/^\s*\((\d+)\+?\)/.exec(title ?? '')?.[1] ?? 0);
  // The count on Instagram's own Messages icon: "3", "9+".
  const badgeCount = (text) => Number(/^\s*(\d+)\s*\+?\s*$/.exec(text ?? '')?.[1] ?? 0);
  // A chat row is unread when its text is bold or it ends in a dot.
  const isUnreadRow = ({ weights = [], dot = false }) => dot || weights.some((weight) => weight >= 600);
  // Each signal sees only part of the inbox; the biggest is nearest the truth.
  const unreadTotal = ({ title = 0, badge = 0, rows = 0 }) => Math.max(title, badge, rows);
  // "Name", "Preview", "·", "2 giờ": the age after the dot goes.
  const rowTexts = (texts) => {
    const dot = texts.indexOf('·');
    const kept = dot < 0 ? texts : texts.slice(0, dot);
    return { name: kept[0] ?? '', preview: kept.slice(1).join(' ') };
  };
  // unread-pure:end

  // Instagram's class names are hashed and its labels translated, so the parts
  // themes style are found by position, role and structure, and marked with
  // data-fmess-ui (Messenger's names where the meaning is the same).
  const UI = 'data-fmess-ui';
  const mark = (el, name) => {
    if (el && el.getAttribute(UI) !== name) el.setAttribute(UI, name);
  };
  const flag = (el, attribute, on) => {
    if (el && el.hasAttribute(attribute) !== on) el.toggleAttribute(attribute, on);
  };
  const tidy = (el, name) => {
    if (el?.getAttribute(UI) === name) el.removeAttribute(UI);
  };
  const rectOf = (el) => el.getBoundingClientRect();
  const listPane = () =>
    [...document.querySelectorAll('[role="navigation"]')].find((nav) => nav.querySelector('a[href="/direct/requests/"]')) ??
    [...document.querySelectorAll('[role="navigation"]')].find((nav) => nav.querySelector('label input')) ??
    [...document.querySelectorAll('[role="navigation"]')].find((nav) => nav.querySelector('[role="button"] img'));
  // Thread ids in URLs are the thread keys React hands the list rows.
  const threadKeyOf = (el) => {
    try {
      const fiberKey = Object.keys(el).find((key) => key.startsWith('__reactFiber'));
      let fiber = fiberKey && el[fiberKey];
      for (let hops = 0; fiber && hops < 30; fiber = fiber.return, hops++) {
        const props = fiber.memoizedProps;
        const key = props?.threadKeyForSelection ?? props?.viewerThreadRef?.thread_key;
        if (key) return String(key);
      }
    } catch {}
    return null;
  };
  // The chats in the list: buttons with a picture, outside any other button
  // (the ⋯ that shows on hover is a button inside the row).
  const chatRows = () => {
    const list = listPane();
    const host = list?.querySelector('h1')?.parentElement?.nextElementSibling ?? list;
    if (!host) return [];
    return [...host.querySelectorAll('[role="button"]')].filter(
      (row) => row.querySelector('img') && !row.parentElement.closest('[role="button"]') && rectOf(row).height > 48,
    );
  };
  const visibleTexts = (row) => {
    const texts = [];
    const walker = document.createTreeWalker(row, NodeFilter.SHOW_TEXT);
    for (let node = walker.nextNode(); node; node = walker.nextNode()) {
      const text = node.textContent.trim();
      const box = node.parentElement?.getBoundingClientRect();
      if (text && box && box.width > 1 && box.height > 1) texts.push(text);
    }
    return texts;
  };
  const rowIsUnread = (row) => {
    const weights = [];
    for (const el of row.querySelectorAll('span')) {
      if (!el.firstElementChild && el.textContent.trim()) weights.push(Number(getComputedStyle(el).fontWeight));
    }
    const box = rectOf(row);
    const dot = [...row.querySelectorAll('div:empty, span:empty')].some((el) => {
      const rect = rectOf(el);
      return (
        rect.width >= 6 &&
        rect.width <= 14 &&
        Math.abs(rect.width - rect.height) < 1 &&
        rect.left + rect.width / 2 > box.right - 56 &&
        getComputedStyle(el).backgroundColor !== 'rgba(0, 0, 0, 0)'
      );
    });
    return isUnreadRow({ weights, dot });
  };
  // The red count on the Messages icon of Instagram's own navigation.
  const navBadge = () => {
    const link = document.querySelector('a[href="/direct/inbox/"]');
    for (const el of link?.querySelectorAll('span, div') ?? []) {
      if (el.firstElementChild) continue;
      const count = badgeCount(el.textContent);
      if (count) return count;
      const rect = rectOf(el);
      if (!el.textContent && rect.width >= 6 && rect.width <= 20 && getComputedStyle(el).backgroundColor !== 'rgba(0, 0, 0, 0)') return 1;
    }
    return 0;
  };

  const NAV_BY_HREF = { '/reels/': 'reels', '/direct/inbox/': 'direct', '/explore/': 'search' };
  const NAV_BY_ORDER = ['notifications', 'create', 'settings', 'meta'];
  const markNav = () => {
    const inbox = document.querySelector('a[href="/direct/inbox/"]');
    let rail = inbox;
    while (rail && !(rectOf(rail).width <= 100 && rectOf(rail).height >= innerHeight * 0.9)) rail = rail.parentElement;
    if (!rail) return;
    mark(rail, 'ig-nav');
    const links = [...rail.querySelectorAll('a[href]')];
    const hashed = links.filter((link) => link.getAttribute('href') === '#');
    let slashes = 0;
    for (const link of links) {
      const href = link.getAttribute('href');
      let name = NAV_BY_HREF[href];
      if (href === '/') name = ['logo', 'home'][slashes++];
      else if (href === '#') name = hashed.length === NAV_BY_ORDER.length ? NAV_BY_ORDER[hashed.indexOf(link)] : undefined;
      else if (!name && /^\/[^/]+\/$/.test(href)) name = 'profile';
      if (name && link.getAttribute('data-fmess-nav') !== name) link.setAttribute('data-fmess-nav', name);
    }
  };

  const LIST_MARKS = ['list', 'list-header', 'list-account', 'new-message', 'search', 'notes', 'list-title', 'requests', 'row'];
  const THREAD_MARKS = ['empty', 'chat', 'chat-header', 'audio-call', 'video-call', 'info', 'messages', 'composer', 'emoji', 'voice-clip', 'attach', 'sticker', 'send', 'more-actions', 'details'];
  // Marks on elements that no longer qualify (a closed chat, a list that
  // changed layout) would style the wrong thing.
  const clearMarks = (names) => {
    for (const el of document.querySelectorAll(names.map((name) => `[${UI}="${name}"]`).join())) el.removeAttribute(UI);
  };
  const markList = (list, view) => {
    mark(list, 'list');
    const title = list.querySelector('h1')?.parentElement;
    flag(document.documentElement, 'data-fmess-compact', !title);
    if (title) {
      const header = list.firstElementChild?.firstElementChild;
      mark(header, 'list-header');
      const buttons = header ? [...header.querySelectorAll('[role="button"]')] : [];
      mark(buttons[0], 'list-account');
      if (buttons.length > 1) mark(buttons[buttons.length - 1], 'new-message');
      mark(list.querySelector('label'), 'search');
      mark(title, 'list-title');
      mark(title.previousElementSibling, 'notes');
      mark(title.querySelector('a[href="/direct/requests/"]'), 'requests');
    }
    const current = view === 'thread' ? /\/direct\/t\/(\d+)/.exec(location.pathname)?.[1] : null;
    for (const row of chatRows()) {
      mark(row, 'row');
      flag(row, 'data-fmess-unread', rowIsUnread(row));
      flag(row, 'data-fmess-selected', Boolean(current) && threadKeyOf(row) === current);
    }
  };

  // Bubbles and date breaks are marked once, when they first have a size.
  const checked = new WeakSet();
  const markMessages = (area) => {
    const areaBox = rectOf(area);
    for (const bubble of area.querySelectorAll('[role="presentation"]')) {
      if (checked.has(bubble)) continue;
      const box = rectOf(bubble);
      if (!box.width) continue;
      checked.add(bubble);
      const style = getComputedStyle(bubble);
      if (style.backgroundColor === 'rgba(0, 0, 0, 0)' || parseFloat(style.borderTopLeftRadius) < 12 || !bubble.textContent.trim()) continue;
      mark(bubble, 'bubble');
      bubble.setAttribute('data-fmess-dir', box.left + box.width / 2 > areaBox.left + areaBox.width / 2 ? 'out' : 'in');
    }
    // A date or time line is a centred text across the whole chat, inside a row.
    for (const row of area.firstElementChild?.firstElementChild?.children ?? []) {
      if (checked.has(row) || !rectOf(row).height) continue;
      checked.add(row);
      for (const el of row.querySelectorAll('div, span')) {
        if (el.childElementCount || !el.textContent.trim() || rectOf(el).width < areaBox.width * 0.9) continue;
        if (getComputedStyle(el).textAlign === 'center') mark(el, 'date-break');
      }
    }
  };

  const markThread = (list, view) => {
    const shell = list.parentElement;
    const field = view === 'thread' ? shell.querySelector('[role="textbox"][contenteditable="true"]') : null;
    const pane = shell.children[1];
    if (!field) {
      clearMarks(THREAD_MARKS.filter((name) => name !== 'empty'));
      if (view === 'inbox') mark(pane, 'empty');
      else tidy(pane, 'empty');
      return;
    }
    tidy(pane, 'empty');
    // The body (messages + composer) follows the header, which holds the name.
    let body = field;
    while (body && !(body.previousElementSibling?.querySelector('h2') && !list.contains(body))) body = body.parentElement;
    if (!body) return;
    const header = body.previousElementSibling;
    mark(header, 'chat-header');
    mark(body.parentElement, 'chat');
    mark(body.firstElementChild, 'messages');
    const details = body.parentElement.nextElementSibling;
    mark(details, 'details');
    const calls = [...header.querySelectorAll('[role="button"]')].filter((button) => !button.closest('a') && button.querySelector('svg'));
    if (calls.length >= 3) {
      mark(calls[calls.length - 3], 'audio-call');
      mark(calls[calls.length - 2], 'video-call');
      mark(calls[calls.length - 1], 'info');
    }
    let composer = field;
    for (let hops = 0; composer && hops < 8 && !(parseFloat(getComputedStyle(composer).borderTopWidth) > 0); hops++) composer = composer.parentElement;
    if (composer && composer !== field) {
      mark(composer, 'composer');
      const buttons = [...composer.querySelectorAll('[role="button"]')];
      const after = buttons.filter((button) => field.compareDocumentPosition(button) & Node.DOCUMENT_POSITION_FOLLOWING);
      const typing = after.some((button) => !button.querySelector('svg'));
      mark(buttons.find((button) => !after.includes(button)), 'emoji');
      after.forEach((button, index) => mark(button, !button.querySelector('svg') ? 'send' : typing ? 'more-actions' : ['voice-clip', 'attach', 'sticker'][index] ?? 'more-actions'));
    }
    markMessages(body.firstElementChild);
  };

  const viewOf = (path) =>
    path.startsWith('/direct/t/') ? 'thread' : path.startsWith('/direct/requests') ? 'requests' : path.startsWith('/direct/') ? 'inbox' : 'other';
  let rowUnread = 0;
  const markPass = () => {
    const root = document.documentElement;
    const view = viewOf(location.pathname);
    if (root.getAttribute('data-fmess-view') !== view) root.setAttribute('data-fmess-view', view);
    markNav();
    const list = view === 'other' ? null : listPane();
    if (!list) {
      clearMarks([...LIST_MARKS, ...THREAD_MARKS]);
      return (rowUnread = 0);
    }
    markList(list, view);
    markThread(list, view);
    rowUnread = chatRows().filter((row) => row.hasAttribute('data-fmess-unread')).length;
  };

  // Themes key their dark colours on html[data-fmess-dark].
  const syncDark = () => withRoot((root) => flag(root, 'data-fmess-dark', root.classList.contains('__fb-dark-mode')));
  withRoot((root) => {
    syncDark();
    new MutationObserver(syncDark).observe(root, { attributes: true, attributeFilter: ['class'] });
  });

  // A read chat's notifications go, as on Messenger.
  const clearOpenChat = () => {
    const thread = threadOf(location.pathname);
    if (thread && document.hasFocus()) invoke('clear_notifications', { thread }).catch(() => {});
  };
  window.addEventListener('focus', clearOpenChat);

  // Instagram may never raise a web notification. When the unread count
  // rises in the background and it stayed quiet, fMess notifies about the
  // newest unread chat (the first rows can be pinned ones).
  let lastFallback = '';
  const notifyNewestChat = () => {
    if (Date.now() - lastPageNotification < 4000) return;
    const row = chatRows().find((candidate) => candidate.hasAttribute('data-fmess-unread'));
    if (!row) return;
    const { name, preview } = rowTexts(visibleTexts(row));
    const thread = threadKeyOf(row);
    const key = `${thread}\n${name}\n${preview}`;
    if (key === lastFallback) return;
    lastFallback = key;
    raisingFallback = true;
    try {
      new FMessNotification(name || 'Instagram', { body: preview || 'New message', icon: row.querySelector('img')?.currentSrc, fmessThread: thread });
    } finally {
      raisingFallback = false;
    }
  };

  // One throttled pass after the page changes: Instagram re-renders a lot,
  // and timers (not animation frames) still run while the view is hidden.
  let lastUnread = -1;
  let lastPath = '';
  let passTimer = 0;
  let lastPass = 0;
  const syncUnread = () => {
    if (location.pathname !== lastPath) {
      lastPath = location.pathname;
      clearOpenChat();
    }
    const count = unreadTotal({ title: unreadFromTitle(document.title), badge: navBadge(), rows: rowUnread });
    if (count === lastUnread) return;
    const rose = lastUnread >= 0 && count > lastUnread;
    lastUnread = count;
    // A theme can show it (attr(data-fmess-unread)).
    if (count) document.documentElement.setAttribute('data-fmess-unread', count);
    else document.documentElement.removeAttribute('data-fmess-unread');
    invoke('set_unread_count', { count }).catch(() => {});
    // Not selected in fMess, the web view is hidden and has no focus.
    if (rose && (!document.hasFocus() || document.hidden)) setTimeout(notifyNewestChat, 1500);
  };
  const schedulePass = () => {
    if (passTimer) return;
    passTimer = setTimeout(
      () => {
        passTimer = 0;
        lastPass = Date.now();
        try {
          markPass();
        } catch {}
        syncUnread();
      },
      Math.max(150, 500 - (Date.now() - lastPass)),
    );
  };
  withRoot((root) => {
    new MutationObserver(schedulePass).observe(root, { childList: true, subtree: true, characterData: true });
    window.addEventListener('popstate', schedulePass);
    schedulePass();
  });

  if (window.__fmessDebug) window.__fmessInstagram = { unreadFromTitle, chatRows, notifyNewestChat, markPass };
})();
