  // fMess's loading screen in place of Facebook's, in Messenger's colours
  // (set by fMess before this script), until the chat list is up. Login and
  // other pages show at once.
  if (!window.opener && window.__fmessSplashLogo && /^\/messages(\/|$)/.test(location.pathname)) {
    const dark = window.__fmessDark ?? matchMedia('(prefers-color-scheme: dark)').matches;
    const splashStyle = document.createElement('style');
    splashStyle.textContent = `
      #fmess-splash {
        position: fixed;
        inset: 0;
        /* Below only the window's drag strip. */
        z-index: 2147483646;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        gap: 16px;
        background: ${dark ? '#1c1c1d' : '#ffffff'};
        color: ${dark ? '#e4e6eb' : '#050505'};
        font: 600 22px/1 -apple-system, BlinkMacSystemFont, system-ui, sans-serif;
        letter-spacing: -0.01em;
        transition: opacity 0.3s ease;
      }
      #fmess-splash.done { opacity: 0; pointer-events: none; }
      #fmess-splash img { width: 112px; height: 112px; animation: fmess-breathe 1.8s ease-in-out infinite; }
      #fmess-splash i {
        position: relative;
        width: 120px;
        height: 3px;
        margin-top: 6px;
        border-radius: 2px;
        overflow: hidden;
        background: ${dark ? 'rgba(255, 255, 255, 0.12)' : 'rgba(0, 0, 0, 0.08)'};
      }
      #fmess-splash i::after {
        content: '';
        position: absolute;
        inset: 0;
        width: 40%;
        border-radius: 2px;
        background: #ff6a13;
        animation: fmess-slide 1.1s ease-in-out infinite;
      }
      @keyframes fmess-breathe { 50% { transform: scale(0.95); } }
      @keyframes fmess-slide { from { transform: translateX(-100%); } to { transform: translateX(250%); } }
    `;
    const splash = document.createElement('div');
    splash.id = 'fmess-splash';
    splash.innerHTML = `<img alt="" src="${window.__fmessSplashLogo}"><span>fMess</span><i></i>`;
    withRoot((root) => root.append(splashStyle, splash));
    const shownAt = Date.now();
    const waitForChats = setInterval(() => {
      const ready =
        document.querySelector('[role="grid"] [role="row"]') ||
        document.querySelector('input[name="email"], input[name="pass"]') ||
        !/^\/messages(\/|$)/.test(location.pathname) ||
        Date.now() - shownAt > 20000;
      if (!ready) return;
      clearInterval(waitForChats);
      splash.classList.add('done');
      setTimeout(() => (splash.remove(), splashStyle.remove()), 400);
    }, 100);
  }

