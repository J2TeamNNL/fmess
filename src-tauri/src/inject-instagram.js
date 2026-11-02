// fMess on Instagram: marks the page for themes, keeps a strip under the
// macOS window buttons to drag the window by, counts unread chats from the
// page title and hands notifications to the OS notification center.
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
    const link = document.querySelector(`a[href="${path}"]`);
    if (link) link.click();
    else location.assign(path);
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

  // Instagram may never raise a web notification. When the unread count
  // rises in the background and it stayed quiet, fMess notifies about the
  // newest chat in the inbox. Its class names are hashed, so rows are found
  // by their links or the list's role.
  const chatRows = () => {
    const links = [...document.querySelectorAll('a[href*="/direct/t/"]')];
    if (links.length) return links;
    return [...document.querySelectorAll('[aria-label="Thread list" i] :is([role="listitem"], [role="button"])')];
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
  let lastFallback = '';
  const notifyNewestChat = () => {
    if (Date.now() - lastPageNotification < 4000) return;
    const row = chatRows()[0];
    const texts = row ? visibleTexts(row) : [];
    // "Name", "Preview", "·", "2m": the age goes.
    const preview = texts.slice(1).filter((text) => text !== '·' && !/^\d+\s?[a-z]{1,3}$/i.test(text)).join(' ');
    const thread = row && threadOf(row.getAttribute('href') ?? row.querySelector('a[href*="/direct/t/"]')?.getAttribute('href'));
    const key = `${thread}\n${texts.join('\n')}`;
    if (row && key === lastFallback) return;
    lastFallback = key;
    raisingFallback = true;
    try {
      new FMessNotification(texts[0] || 'Instagram', {
        body: preview || 'New message',
        icon: row?.querySelector('img')?.currentSrc,
        fmessThread: thread,
      });
    } finally {
      raisingFallback = false;
    }
  };

  // A read chat's notifications go, as on Messenger.
  const clearOpenChat = () => {
    const thread = threadOf(location.pathname);
    if (thread && document.hasFocus()) invoke('clear_notifications', { thread }).catch(() => {});
  };
  window.addEventListener('focus', clearOpenChat);

  // The title reads "(3) Instagram" with three unread chats.
  const unreadFromTitle = (title) => Number(/^\s*\((\d+)\+?\)/.exec(title ?? '')?.[1] ?? 0);
  let lastUnread = -1;
  let lastPath = '';
  let titleTimer = 0;
  const syncUnread = () => {
    clearTimeout(titleTimer);
    titleTimer = setTimeout(() => {
      if (location.pathname !== lastPath) {
        lastPath = location.pathname;
        clearOpenChat();
      }
      const count = unreadFromTitle(document.title);
      if (count === lastUnread) return;
      const rose = lastUnread >= 0 && count > lastUnread;
      lastUnread = count;
      // A theme can show it (attr(data-fmess-unread)).
      if (count) document.documentElement.setAttribute('data-fmess-unread', count);
      else document.documentElement.removeAttribute('data-fmess-unread');
      invoke('set_unread_count', { count }).catch(() => {});
      // Not selected in fMess, the web view is hidden and has no focus.
      if (rose && (!document.hasFocus() || document.hidden)) setTimeout(notifyNewestChat, 1500);
    }, 500);
  };
  const titleWatcher = new MutationObserver(syncUnread);
  const watchTitle = () => {
    if (!document.head) return false;
    titleWatcher.observe(document.head, { subtree: true, childList: true, characterData: true });
    syncUnread();
    return true;
  };
  if (!watchTitle()) {
    const wait = new MutationObserver(() => watchTitle() && wait.disconnect());
    withRoot((root) => wait.observe(root, { childList: true }));
  }

  if (window.__fmessDebug) window.__fmessInstagram = { unreadFromTitle, chatRows, notifyNewestChat };
})();
