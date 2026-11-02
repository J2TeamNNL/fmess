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

