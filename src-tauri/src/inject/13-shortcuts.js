  // Keyboard shortcuts in the spirit of the old desktop app (it showed
  // "Search (⌘K)"). Selectors avoid aria-labels, which Facebook translates.
  const isMac = /Mac/.test(navigator.platform);
  const isVisible = (el) => el && el.getBoundingClientRect().width > 0;
  const chatLinks = () =>
    [...document.querySelectorAll('[role="grid"] [role="row"] a[href*="/messages/"]')].filter(isVisible);
  const shortcuts = {
    search() {
      const inputs = [...document.querySelectorAll('input[type="search"]')];
      // A theme may fold the chat list's field away until it is used (see the
      // search box below).
      const input = inputs.find(isVisible) ?? inputs.find((el) => el.closest('[role="navigation"]'));
      input?.focus();
      input?.select();
    },
    newMessage() {
      document.querySelector('a[href="/messages/new/"]')?.click();
    },
    openChat(index) {
      chatLinks()[index]?.click();
    },
    moveChat(step) {
      const links = chatLinks();
      const current = links.findIndex((link) => new URL(link.href).pathname === location.pathname);
      links[Math.max(0, Math.min(links.length - 1, current + step))]?.click();
    },
  };
  // Also called from fMess's menu bar on macOS.
  window.__fmessShortcut = (name, arg) => onChatsPage() && shortcuts[name]?.(arg);
  // The menu bar's shortcuts (menu.rs), by key, with Ctrl.
  const MENU_KEYS = {
    Comma: 'settings',
    Equal: 'zoom-in',
    NumpadAdd: 'zoom-in',
    Minus: 'zoom-out',
    NumpadSubtract: 'zoom-out',
    Digit0: 'zoom-reset',
    Numpad0: 'zoom-reset',
    KeyR: 'reload',
    BracketLeft: 'back',
    BracketRight: 'forward',
  };
  // With Ctrl+Shift (Ctrl+Shift+= is Ctrl++ on most keyboards).
  const MENU_SHIFT_KEYS = { KeyH: 'messages', Equal: 'zoom-in' };
  window.addEventListener(
    'keydown',
    (event) => {
      const primary = isMac ? event.metaKey : event.ctrlKey;
      let action = null;
      if (primary && !event.shiftKey && !event.altKey && event.code === 'KeyK') action = ['search'];
      else if (primary && !event.shiftKey && !event.altKey && event.code === 'KeyN') action = ['newMessage'];
      else if (primary && !event.shiftKey && !event.altKey && /^Digit[1-9]$/.test(event.code))
        action = ['openChat', Number(event.code.slice(5)) - 1];
      else if (primary && event.shiftKey && event.code === 'BracketRight') action = ['moveChat', 1];
      else if (primary && event.shiftKey && event.code === 'BracketLeft') action = ['moveChat', -1];
      else if (event.ctrlKey && event.code === 'Tab') action = ['moveChat', event.shiftKey ? -1 : 1];
      if (!action && !isMac && event.ctrlKey && !event.altKey && !event.metaKey) {
        // Windows and Linux have no menu bar, so its shortcuts start here.
        const menu = event.shiftKey ? MENU_SHIFT_KEYS[event.code] : MENU_KEYS[event.code];
        if (menu) {
          event.preventDefault();
          event.stopPropagation();
          invoke('menu_shortcut', { id: menu }).catch(() => {});
          return;
        }
      }
      if (!action || !onChatsPage()) return;
      event.preventDefault();
      event.stopPropagation();
      shortcuts[action[0]](action[1]);
    },
    true,
  );

