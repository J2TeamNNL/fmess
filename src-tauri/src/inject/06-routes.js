  // Facebook is a single-page app, so moving between chats and the rest of
  // Facebook happens through the History API rather than real navigations.
  const CHATS = 'data-fmess-chats';
  // The chats, and the photo/video viewer opened from them.
  const onChatsPage = () => /^\/(messages|messenger_media)(\/|$)/.test(location.pathname);
  let lastPath = null;
  // Counts the chats opened, so work for one stops when another opens.
  let chatVisit = 0;
  const MEDIA = 'data-fmess-media';
  let viewerOpen = false;
  let viewerCloseTimer;
  // Facebook doesn't always move to /messenger_media for the viewer (a
  // chat's photos open in a layer at the same address), so the viewer on
  // screen counts too.
  const isViewerOpen = () => location.pathname.startsWith('/messenger_media') || Boolean(viewerCloseBox() || viewerBar());
  const syncRoute = () => {
    document.documentElement.toggleAttribute(CHATS, onChatsPage());
    const inViewer = isViewerOpen();
    document.documentElement.toggleAttribute(MEDIA, inViewer);
    if (inViewer !== viewerOpen) {
      viewerOpen = inViewer;
      clearInterval(viewerCloseTimer);
      if (inViewer) viewerCloseTimer = setInterval(() => (placeViewerClose(), dropStaleZoom()), 100);
      else resetZoom();
      listenForViewerWheel(inViewer);
      // On macOS fMess gives the photo/video viewer a regular title bar, so
      // the viewer lays itself out below the window buttons as in a browser.
      setTimeout(() => window.__TAURI_INTERNALS__?.invoke('set_viewer_open', { open: inViewer }).catch(() => {}));
    }
    if (location.pathname !== lastPath) {
      // /t/<id> and /e2ee/t/<id> are the same chat (Facebook goes from one
      // to the other as an encrypted chat opens).
      const chatOf = (path) => path?.match(/^\/messages\/(?:e2ee\/)?t\/([^/]+)/)?.[1] ?? path;
      const anotherChat = chatOf(location.pathname) !== chatOf(lastPath);
      lastPath = location.pathname;
      if (anotherChat) chatVisit += 1;
      if (/^\/messages\/(e2ee\/)?t\//.test(lastPath)) {
        focusComposer();
        // Defined further down, after this first runs.
        setTimeout(() => clearOpenChatNotifications());
        if (anotherChat) {
          // The box still on screen belongs to the chat that was open; a
          // draft kept for this chat goes back only during this visit.
          const previousBox = document.querySelector('[role="main"] [contenteditable="true"][role="textbox"]');
          const visit = chatVisit;
          setTimeout(() => window.__fmessReturnDraft?.(previousBox, () => visit === chatVisit));
        }
      }
    }
  };

  // Opening a chat puts the cursor in its message box, as the desktop app
  // did, instead of needing another click. The old chat's box is still on
  // screen for a moment, so wait for a different one (or give up waiting).
  const composer = () => document.querySelector('[role="main"] [contenteditable="true"][role="textbox"]');
  let focusTimer;
  function focusComposer() {
    clearInterval(focusTimer);
    const previous = composer();
    let tries = 0;
    focusTimer = setInterval(() => {
      tries += 1;
      const box = composer();
      if ((box && (box !== previous || tries > 10)) || tries > 60) {
        clearInterval(focusTimer);
        if (box && document.activeElement !== box) box.focus();
      }
    }, 50);
  }
  for (const method of ['pushState', 'replaceState']) {
    const original = history[method];
    history[method] = function (...args) {
      const result = original.apply(this, args);
      syncRoute();
      return result;
    };
  }
  window.addEventListener('popstate', syncRoute);
  window.navigation?.addEventListener('currententrychange', syncRoute);
  // The viewer can open and close without a route change.
  setInterval(() => document.documentElement && viewerOpen !== isViewerOpen() && syncRoute(), 200);
  withRoot(syncRoute);

  // Facebook links in a chat (reels, posts, profiles) open in the browser, so
  // fMess stays on the conversation. Keep this list in step with
  // IN_APP_PATHS in links.rs, which handles the window.open() below.
  const IN_APP_PATHS = ['/messages', '/messenger_media', '/groupcall', '/videocall'];
  const isFacebookHost = (host) =>
    ['facebook.com', 'messenger.com'].some((domain) => host === domain || host.endsWith(`.${domain}`));
  document.addEventListener(
    'click',
    (event) => {
      if (!onChatsPage() || event.button !== 0 || event.metaKey || event.ctrlKey || event.shiftKey) return;
      const link = event.target instanceof Element ? event.target.closest('a[href]') : null;
      if (!link) return;
      const url = new URL(link.href, location.href);
      if (!isFacebookHost(url.hostname) || IN_APP_PATHS.some((path) => url.pathname.startsWith(path))) return;
      event.preventDefault();
      event.stopImmediatePropagation();
      window.open(url.href, '_blank', 'noopener');
    },
    true,
  );

