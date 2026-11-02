  // A few of Facebook's own modules are adjusted as its loader (window.__d)
  // defines them. Each patch checks the module's shape first and leaves it
  // alone if Facebook changed it.
  //
  // Calls in end-to-end encrypted chats need a frame transform. Facebook ships
  // one for WebKit (RTCRtpScriptTransform) but keeps it behind gatekeeper
  // 16078, and otherwise only accepts Chrome's createEncodedStreams, so every
  // WebKit call button says "not supported on this browser". fMess answers
  // that gatekeeper itself; its settings can turn this off.
  //
  // Notifications: Messenger raises full ones (sender, text, picture; clicking
  // opens the chat) only when it runs as an installed app on messenger.com and
  // the page is hidden. fMess is such an app, and like the old desktop app it
  // counts as away whenever its window isn't the active one. Only that one
  // check changes (FACTORY_REWRITES); patching isMessengerPWA or VisibilityAPI
  // themselves changed far more of the page and made it sluggish.
  // fMess passes its settings in before this script (window.__fmessSettings);
  // a change to this one reloads the page and comes through localStorage.
  const settings = window.__fmessSettings ?? {};
  let macCalls = settings.macCalls !== false;
  try {
    const changed = localStorage.getItem('fmess.macCalls');
    if (changed !== null) macCalls = changed !== '0';
  } catch {}
  const FORCED_GATES = macCalls ? { 16078: true } : {};
  const MODULE_PATCHES = {
    gkx(exports) {
      const gkx = exports.default;
      if (typeof gkx?.add !== 'function') return false;
      exports.default = Object.assign(
        (id) => (Object.hasOwn(FORCED_GATES, id) ? FORCED_GATES[id] : gkx(id)),
        gkx,
      );
      return true;
    },
    // Facebook pops its own notifications (group posts, likes) over the page
    // unless the route asks it not to; in fMess no route wants them. The
    // notifications under the bell in Facebook's top bar stay.
    useHideNotificationsToasts(exports) {
      if (typeof exports.default !== 'function') return false;
      exports.default = () => true;
      return true;
    },
    // Messenger's notification for a new message carries only a title, text
    // and one picture. The old desktop app showed the sender and, in a group,
    // the group with its picture, and kept each chat's notifications
    // together; fMess reads those details the way this module does.
    MWDesktopNotifications(exports) {
      const trigger = exports.triggerLocalNotification;
      if (typeof trigger !== 'function') return false;
      exports.triggerLocalNotification = function (db, message, ...rest) {
        const notify = () => {
          const details = readMessageDetails(db, message).catch(() => null);
          messageDetails.set(String(message?.messageId), details);
          if (messageDetails.size > 50) messageDetails.delete(messageDetails.keys().next().value);
          return trigger.call(this, db, message, ...rest);
        };
        // A computer back from sleep gets the messages it missed one by one,
        // and Messenger would notify about each, read elsewhere or not.
        return isOldNews(db, message).then((old) => (old ? undefined : notify()), notify);
      };
      return true;
    },
    // Messenger's dark mode is React state at the root of the page. fMess's
    // Settings change it with the setter Messenger's own Preferences use, read
    // from inside that provider, so the page switches at once.
    'makeCometDarkModeStateProvider.react'(exports) {
      const make = exports.default;
      if (typeof make !== 'function' || make.length !== 1) return false;
      exports.default = (options) => {
        const Provider = make(options);
        const React = window.require('react');
        const contextModule = window.require('CometDarkModeContext');
        const context = contextModule?.$$typeof ? contextModule : contextModule?.default;
        const ReadSetter = () => {
          window.__fmessSetDarkMode = React.useContext(context).setDarkModeSetting;
          return null;
        };
        return (props) => React.createElement(Provider, props, React.createElement(ReadSetter), props.children);
      };
      return true;
    },
  };
  // Rewrites of a module's source, each replacing exactly one match. The page
  // forbids eval but allows blob: scripts, so the rewritten factory loads as
  // one; modules run only when first required, by which time it is there.
  // Until then, or if a pattern no longer matches, the original runs.
  const FACTORY_REWRITES = {
    useMWLSChatPopperUpper: [
      [/(\w+)\("VisibilityAPI"\)\.isVisibilityHidden\(\)===!0/, '($1("VisibilityAPI").isVisibilityHidden()===!0||!document.hasFocus())'],
      [/\w+\("isMessengerPWA"\)\(\)&&\w+\("justknobx"\)\._\("1465"\)/, '!0'],
    ],
  };
  const rewrittenFactories = {};
  const rewriteFactory = (name, factory) => {
    let source = factory.toString();
    for (const [pattern, replacement] of FACTORY_REWRITES[name]) {
      if (source.match(new RegExp(pattern.source, 'g'))?.length !== 1) return;
      source = source.replace(pattern, replacement);
    }
    const key = `fmess:${name}`;
    const script = document.createElement('script');
    script.src = URL.createObjectURL(
      new Blob([`window[${JSON.stringify(key)}] = (${source});`], { type: 'text/javascript' }),
    );
    script.onload = () => {
      rewrittenFactories[name] = window[key];
      delete window[key];
      URL.revokeObjectURL(script.src);
      script.remove();
    };
    (document.head || document.documentElement).append(script);
  };
  let defineModule;
  Object.defineProperty(window, '__d', {
    configurable: true,
    get: () => defineModule,
    set(loaderDefine) {
      defineModule = function (name, deps, factory, ...rest) {
        if (typeof factory === 'function' && Object.hasOwn(FACTORY_REWRITES, name)) {
          const originalFactory = factory;
          rewriteFactory(name, originalFactory);
          factory = function (...args) {
            return (rewrittenFactories[name] ?? originalFactory).apply(this, args);
          };
        } else if (typeof factory === 'function' && Object.hasOwn(MODULE_PATCHES, name)) {
          const patch = MODULE_PATCHES[name];
          const originalFactory = factory;
          factory = function (...args) {
            const result = originalFactory.apply(this, args);
            args.some((arg) => arg && typeof arg === 'object' && arg !== window && patch(arg));
            return result;
          };
          // The loader passes a factory's extra arguments (fbt, for one) only
          // when it declares them.
          Object.defineProperty(factory, 'length', { value: originalFactory.length });
        }
        return loaderDefine.call(this, name, deps, factory, ...rest);
      };
    },
  });

  // WKWebView ignores window.close(), so the Close button of a call window did
  // nothing. Navigate to a URL fMess watches for instead; it closes the window
  // (and ignores the request in the main window). See CLOSE_WINDOW_URL in webview.rs.
  window.close = () => {
    location.href = 'fmess://close-window';
  };

