// fMess: keeps the window on Messenger chats, hides Facebook's top bar there,
// and hands web notifications to the OS notification center.
// Runs at document start, before the page's scripts.
(() => {
  if (window.top !== window || window.__fmessInstalled) return;
  window.__fmessInstalled = true;

  // Debug builds (window.__fmessDebug, see webview.rs): note who adds wheel,
  // touch and scroll listeners and whether they can cancel scrolling, for
  // probes (window.__fmessListenerLog). A wheel listener that isn't passive
  // makes WebKit hold the first event of every scroll gesture until the page
  // has run it.
  if (window.__fmessDebug) {
    const listenerLog = (window.__fmessListenerLog = []);
    const WATCHED = new Set(['wheel', 'mousewheel', 'touchstart', 'touchmove', 'scroll']);
    const describe = (target) => {
      if (target === window) return 'window';
      if (target === document) return 'document';
      if (!(target instanceof Element)) return String(target);
      const role = target.getAttribute('role');
      return `${target.tagName.toLowerCase()}${target.id ? `#${target.id}` : ''}${role ? `[role=${role}]` : ''}`;
    };
    const nativeAdd = EventTarget.prototype.addEventListener;
    EventTarget.prototype.addEventListener = function (type, listener, options) {
      if (WATCHED.has(type) && listenerLog.length < 500) {
        listenerLog.push({
          type,
          target: describe(this),
          passive: typeof options === 'object' && options !== null && 'passive' in options ? Boolean(options.passive) : null,
          capture: typeof options === 'boolean' ? options : Boolean(options?.capture),
          at: Math.round(performance.now()),
          stack: (new Error().stack || '').split('\n').slice(1, 4).join(' | ').slice(0, 300),
        });
      }
      return nativeAdd.call(this, type, listener, options);
    };

    // The page's errors, printed by probe.rs. Query strings and URL fragments
    // are cut, as they can carry tokens, and objects show only their type, as
    // they can hold names or message text.
    const errors = (window.__fmessErrors = []);
    const show = (part) => {
      if (part instanceof Error) return `${part.name}: ${part.message}`;
      if (typeof part === 'function') return '[function]';
      if (part === null || typeof part !== 'object') return String(part);
      try {
        return `[${part.constructor?.name || 'object'}]`;
      } catch {
        return '[object]';
      }
    };
    const note = (kind, parts) => {
      if (errors.length >= 200) return;
      const text = parts
        .map(show)
        .join(' ')
        .replace(/\?[^\s"')]*/g, '?…')
        .replace(/(\S)#[^\s"')]*/g, '$1#…')
        .slice(0, 500);
      errors.push(`${kind} ${text}`);
    };
    window.addEventListener('error', (event) => {
      const where = event.filename ? `(${event.filename}:${event.lineno})` : '';
      note('error', [event.error ?? event.message, where]);
    });
    window.addEventListener('unhandledrejection', (event) => note('unhandledrejection', [event.reason]));
    const nativeError = console.error;
    console.error = function (...args) {
      note('console.error', args);
      return nativeError.apply(this, args);
    };
  }

  // WebView2 (Windows) runs this before the page has its <html> element;
  // WebKit already has one. What needs it waits for it.
  const withRoot = (use) => {
    if (document.documentElement) return use(document.documentElement);
    const wait = new MutationObserver(() => {
      if (!document.documentElement) return;
      wait.disconnect();
      use(document.documentElement);
    });
    wait.observe(document, { childList: true });
  };

