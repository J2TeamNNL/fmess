  // Layout, closer to the old desktop app.
  //
  // Facebook's top bar is a zero-height role="banner" wrapper around fixed
  // navigation, and the page leaves room for it through --header-height.
  // Themed subtrees declare that variable again, so set it on every element.
  // In the chats the bar is hidden; elsewhere it stays, so there is always a
  // way back.
  //
  // On macOS the title bar overlays the page (set by fMess before this
  // script), so the page keeps a strip of __fmessTitleBarInset pixels free
  // for the window buttons and dragging, and the bar moves below it.
  //
  // The chats lose the inset, rounded cards Facebook draws, so the panes sit
  // edge to edge with a divider, as in the old app.
  const LOGGED_OUT = 'data-fmess-logged-out';
  const syncLoggedOut = () => {
    let out = false;
    try {
      out = String(window.require('CurrentUserInitialData').USER_ID) === '0';
    } catch {}
    document.documentElement?.toggleAttribute(LOGGED_OUT, out);
  };
  setInterval(syncLoggedOut, 1000);
  // Popups (call windows) keep a normal title bar.
  const titleBarInset = window.opener ? 0 : window.__fmessTitleBarInset || 0;
  const inset = `${titleBarInset}px`;
  const BAR_HIDDEN = `html[${CHATS}]`;
  const BAR_SHOWN = `html:not([${CHATS}])`;
  const layoutStyle = document.createElement('style');
  layoutStyle.textContent = `
    /* For themes that draw in the strip: 0 where the system draws the title bar. */
    :root { --fmess-title-bar: ${inset}; }
    ${BAR_HIDDEN} div[role="banner"] { display: none !important; }
    ${BAR_HIDDEN}, ${BAR_HIDDEN} * { --header-height: ${inset} !important; }
    :is(${BAR_SHOWN}) div[role="banner"] { transform: translateY(${inset}); }
    :is(${BAR_SHOWN}), :is(${BAR_SHOWN}) * { --header-height: calc(56px + ${inset}) !important; }
    /* Logged out, Facebook's pages have no top bar to make room for. */
    html[${LOGGED_OUT}], html[${LOGGED_OUT}] * { --header-height: ${inset} !important; }
    html[${CHATS}], html[${CHATS}] * {
      --messenger-card-spacing: 0px !important;
      --messenger-card-corner-radius: 0px !important;
      --messenger-card-box-shadow: none !important;
    }
    html[${CHATS}] [role="main"] { box-shadow: -1px 0 0 var(--divider, var(--card-border-color)); }
    /* The strip under the window buttons is the page's own background,
       Facebook's grey, over panes that are white (or dark): the panes'
       colour instead, so the top reads as one bar. */
    html[${CHATS}] body { background: var(--surface-background) !important; }
    /* The same divider before the chat's details (the pane beside the
       messages, with the Search button). */
    html[${CHATS}] [role="main"] [data-fmess-ui="details"] {
      box-shadow: -1px 0 0 var(--divider, var(--card-border-color));
    }
    /* The photo/video viewer centres its top bar (close, download, forward)
       in a band --header-height tall, made for Facebook's 56px bar; without
       it the buttons sit off the top. The viewer gets a regular title bar
       on macOS (see syncRoute), so it needs no drag strip either. */
    html[${MEDIA}] [role="dialog"], html[${MEDIA}] [role="dialog"] * { --header-height: 56px !important; }
    html[${MEDIA}] [data-tauri-drag-region] { display: none; }
    /* Facebook shows the viewer's close button and logo in its top bar,
       which fMess hides, and keeps the viewer's own copy transparent. Show
       that copy's close button (placeViewerClose moves it) without the logo. */
    html[${MEDIA}] [role="dialog"] > div:has(> a[href="/"]) {
      opacity: 1 !important;
      /* Above the dark gradient the viewer lays over the photo's top edge,
         which download and forward sit above too; under it, close looked grey. */
      z-index: 2147483000 !important;
    }
    html[${MEDIA}] [role="dialog"] > div:has(> a[href="/"]) > a[href="/"] { display: none !important; }
    /* Its thin X looks out of place beside the solid download and forward
       icons; draw a bold white one instead (the viewer is always dark; a
       masked currentColor rendered grey in WebKit). */
    html[${MEDIA}] [role="dialog"] > div:has(> a[href="/"]) [role="button"] svg { display: none !important; }
    html[${MEDIA}] [role="dialog"] > div:has(> a[href="/"]) [role="button"]::after {
      content: '';
      width: 20px;
      height: 20px;
      background: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 20 20'%3E%3Cpath d='M5.5 5.5l9 9M14.5 5.5l-9 9' stroke='white' stroke-width='6' stroke-linecap='round'/%3E%3C/svg%3E") center / contain no-repeat;
    }
    /* Download and forward show their name on hover; so does close, using
       the label Facebook gives it (translated with the rest of the page). */
    html[${MEDIA}] [role="dialog"] > div:has(> a[href="/"]) [role="button"]::before {
      content: attr(aria-label);
      position: absolute;
      top: calc(100% + 8px);
      left: 50%;
      transform: translateX(-50%);
      padding: 6px 10px;
      border-radius: 6px;
      background: rgba(0, 0, 0, 0.8);
      color: #fff;
      font: 12px/16px system-ui, -apple-system, sans-serif;
      white-space: nowrap;
      opacity: 0;
      pointer-events: none;
      transition: opacity 0.15s;
    }
    html[${MEDIA}] [role="dialog"] > div:has(> a[href="/"]) [role="button"]:hover::before { opacity: 1; }
    /* WebKit takes about 4 s per layout for a 100%-height image inside an
       aspect-ratio flex box (photos in shared-post previews), which froze
       whole chats. Auto height looks the same there and takes milliseconds. */
    html[${CHATS}] img[height="100%"][width="100%"] { height: auto !important; }
  `;
  withRoot((root) => root.append(layoutStyle));

