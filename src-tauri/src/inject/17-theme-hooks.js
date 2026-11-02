  // A theme can draw buttons on the chat list (the Zalo look's bar down its
  // left side) and list them in --fmess-buttons on the list: "name left top
  // size" each, comma-separated, a negative top counting from the bottom.
  // fMess lights the one under the pointer (--fmess-hover-left/-top and
  // data-fmess-hover="<name>" on the list, for a theme's label) and runs
  // those it knows: "account" opens the account menu, as the account button
  // does, and "settings" opens fMess's Settings.
  const themeButtonAt = (list, x, y) => {
    const box = list.getBoundingClientRect();
    for (const spec of getComputedStyle(list).getPropertyValue('--fmess-buttons').split(',')) {
      const [name, left, top, size] = spec.trim().replace(/"/g, '').split(/\s+/);
      if (!name || !(Number(size) > 0)) continue;
      const button = { name, left: box.left + Number(left), size: Number(size) };
      button.top = Number(top) < 0 ? box.bottom + Number(top) - button.size : box.top + Number(top);
      if (x >= button.left && x <= button.left + button.size && y >= button.top && y <= button.top + button.size) return button;
    }
    return null;
  };
  let hoveredList = null;
  document.addEventListener(
    'mousemove',
    (event) => {
      const list = event.target instanceof Element && event.target.closest('[role="navigation"]');
      const unlight = (el) => (el.style.removeProperty('--fmess-hover-top'), el.style.removeProperty('cursor'), el.removeAttribute('data-fmess-hover'));
      if (hoveredList && hoveredList !== list) unlight(hoveredList);
      hoveredList = list || null;
      if (!list) return;
      const button = themeButtonAt(list, event.clientX, event.clientY);
      if (!button) return unlight(list);
      const box = list.getBoundingClientRect();
      list.style.setProperty('--fmess-hover-left', `${button.left - box.left}px`);
      list.style.setProperty('--fmess-hover-top', `${button.top - box.top}px`);
      list.style.cursor = 'pointer';
      if (list.getAttribute('data-fmess-hover') !== button.name) list.setAttribute('data-fmess-hover', button.name);
    },
    { passive: true },
  );
  document.addEventListener(
    'click',
    (event) => {
      const list = event.target instanceof Element && event.target.closest('[role="navigation"]');
      const button = list && themeButtonAt(list, event.clientX, event.clientY);
      if (!button || !['account', 'settings'].includes(button.name)) return;
      event.preventDefault();
      event.stopPropagation();
      if (button.name === 'settings') invoke('open_settings').catch(() => {});
      else invoke('show_account_menu', { name: currentAccount()?.name ?? '', x: button.left + button.size + 6, y: button.top }).catch(() => {});
    },
    true,
  );

  // The same for buttons a theme draws anywhere in the window (the Teams
  // look's "…" and picture at the right of its title bar): --fmess-window-
  // buttons on :root, "name left top size" in px from the window's top left,
  // a negative left counting from the right edge and a negative top from the
  // bottom (left = width + left - size). fMess lights the one under the
  // pointer (--fmess-window-hover-left/-top and data-fmess-window-hover on
  // :root) and runs "account" and "settings" as above; the account menu
  // opens under the button, kept inside the window (about 200px wide).
  // "sidebar" folds the theme's sidebar (the Teams look's app bar) away and
  // back: data-fmess-sidebar-collapsed on <html>, kept for next time, for
  // the theme to fold it. A press there doesn't drag the window, though the
  // title strip is under it.
  // The list is read again at most once a second, not on every move.
  const SIDEBAR_COLLAPSED = 'data-fmess-sidebar-collapsed';
  const SIDEBAR_COLLAPSED_KEY = 'fmess.sidebarCollapsed';
  try {
    if (localStorage.getItem(SIDEBAR_COLLAPSED_KEY) === '1') withRoot((root) => root.setAttribute(SIDEBAR_COLLAPSED, ''));
  } catch {}
  const toggleSidebar = () => {
    const collapsed = document.documentElement.toggleAttribute(SIDEBAR_COLLAPSED);
    try {
      localStorage.setItem(SIDEBAR_COLLAPSED_KEY, collapsed ? '1' : '0');
    } catch {}
  };
  let windowButtons = { at: 0, specs: [] };
  const windowButtonAt = (x, y) => {
    if (performance.now() - windowButtons.at > 1000) {
      const value = getComputedStyle(document.documentElement).getPropertyValue('--fmess-window-buttons');
      windowButtons = { at: performance.now(), specs: value.split(',').map((spec) => spec.trim().replace(/"/g, '').split(/\s+/)) };
    }
    for (const [name, left, top, size] of windowButtons.specs) {
      if (!name || !(Number(size) > 0)) continue;
      const button = { name, size: Number(size) };
      button.left = Number(left) < 0 ? innerWidth + Number(left) - button.size : Number(left);
      button.top = Number(top) < 0 ? innerHeight + Number(top) - button.size : Number(top);
      if (x >= button.left && x <= button.left + button.size && y >= button.top && y <= button.top + button.size) return button;
    }
    return null;
  };
  let litWindowButton = '';
  document.addEventListener(
    'mousemove',
    (event) => {
      const button = windowButtonAt(event.clientX, event.clientY);
      const lit = button ? `${button.left} ${button.top}` : '';
      if (lit === litWindowButton) return;
      litWindowButton = lit;
      const root = document.documentElement;
      if (!button) return (root.style.removeProperty('--fmess-window-hover-top'), root.style.removeProperty('cursor'), root.removeAttribute('data-fmess-window-hover'));
      root.style.setProperty('--fmess-window-hover-left', `${button.left}px`);
      root.style.setProperty('--fmess-window-hover-top', `${button.top}px`);
      root.setAttribute('data-fmess-window-hover', button.name);
      root.style.cursor = 'pointer';
    },
    { passive: true },
  );
  const onWindowButton = (event) => {
    const button = windowButtonAt(event.clientX, event.clientY);
    if (!button) return null;
    event.preventDefault();
    event.stopImmediatePropagation();
    return button;
  };
  // Before Tauri's drag strip sees them (it starts a drag on mousedown, and
  // maximizes on a double click's mouseup).
  window.addEventListener('mousedown', onWindowButton, true);
  window.addEventListener('mouseup', onWindowButton, true);
  window.addEventListener(
    'click',
    (event) => {
      const button = onWindowButton(event);
      if (button?.name === 'settings') invoke('open_settings').catch(() => {});
      else if (button?.name === 'sidebar') toggleSidebar();
      else if (button?.name === 'account') invoke('show_account_menu', { name: currentAccount()?.name ?? '', x: Math.max(0, Math.min(button.left, innerWidth - 200)), y: button.top + button.size + 4 }).catch(() => {});
    },
    true,
  );

  // A theme can also ask for a search box of its own (--fmess-search-box: 1
  // on :root; the Teams look's is in its title bar): a button that focuses
  // Messenger's search field, as ⌘K does, for the theme to show. It carries
  // the field's placeholder, so it reads in Facebook's language.
  const SEARCH_BOX = 'fmess-search-box';
  setInterval(() => {
    let box = document.getElementById(SEARCH_BOX);
    if (getComputedStyle(document.documentElement).getPropertyValue('--fmess-search-box').trim() !== '1') return box?.remove();
    if (!box) {
      box = document.createElement('div');
      box.id = SEARCH_BOX;
      box.setAttribute('role', 'button');
      box.tabIndex = 0;
      // Keeps the focus where it is until the click moves it to the field.
      box.addEventListener('mousedown', (event) => event.preventDefault());
      box.addEventListener('click', () => shortcuts.search());
      document.body.append(box);
    }
    const placeholder = document.querySelector('[role="navigation"] input[type="search"]')?.placeholder ?? '';
    if (box.textContent !== placeholder) box.textContent = placeholder;
  }, 500);

  // A theme can put quick reactions in a message's hover actions (the Teams
  // look's emoji row): --fmess-quick-reactions on :root, emoji as codepoints
  // ("1f44d 2764 1f606 1f62e"). fMess adds them first in the hovered
  // message's actions, in a .fmess-quick-reactions group of buttons
  // (.fmess-quick-reaction, data-emoji) holding Messenger's own picture of
  // the emoji, so themes that swap Messenger's emoji swap these too. A click
  // opens Messenger's reaction picker for that message (its react button,
  // found by its icon) and picks the same emoji there, which also takes the
  // reaction back if it is already yours; an emoji the picker doesn't show
  // just closes it.
  const REACT_ICON = 'M111 203a9.5';
  // Messenger's emoji URLs carry a checksum, so they are learnt from the
  // pictures it shows (its reaction picker has these six); one not seen yet
  // is written as text.
  const emojiPictures = new Map(
    Object.entries({ '1f44d': 't55', 2764: 'tf3', '1f606': 't2d', '1f62e': 't1a', '1f622': 't67', '1f621': 'te6' }).map(
      ([emoji, sum]) => [emoji, `https://static.xx.fbcdn.net/images/emoji.php/v9/${sum}/2/32/${emoji}.png`],
    ),
  );
  const emojiName = (text) => [...(text ?? '')].map((c) => c.codePointAt(0).toString(16)).filter((c) => c !== 'fe0f').join('_');
  const learnEmoji = (root) => {
    for (const img of root.querySelectorAll('img[src*="/images/emoji.php/"]')) {
      const emoji = img.getAttribute('src').match(/\/([0-9a-f_]+)\.png/)?.[1];
      if (emoji) emojiPictures.set(emoji.replace(/_fe0f/g, ''), img.getAttribute('src'));
    }
  };
  let quickReactions = { at: 0, list: [] };
  const quickReactionList = () => {
    if (performance.now() - quickReactions.at > 1000) {
      const value = getComputedStyle(document.documentElement).getPropertyValue('--fmess-quick-reactions');
      const list = value.trim().toLowerCase().split(/\s+/).filter((emoji) => /^[0-9a-f_]+$/.test(emoji));
      // Another theme (or none): the buttons added for the last one go.
      if (list.join() !== quickReactions.list.join()) for (const group of document.querySelectorAll('.fmess-quick-reactions')) group.remove();
      quickReactions = { at: performance.now(), list };
    }
    return quickReactions.list;
  };
  const reactButton = (row) =>
    [...row.querySelectorAll('[role="toolbar"] [role="button"]')].find((button) =>
      button.querySelector('svg path')?.getAttribute('d')?.startsWith(REACT_ICON),
    );
  const addQuickReactions = (row) => {
    const list = quickReactionList();
    const react = list.length && reactButton(row);
    if (!react) return;
    // The actions' own box: the react button's ancestor that holds the others too.
    let item = react;
    while (item.parentElement && item.parentElement.querySelectorAll('[role="button"]').length === 1) item = item.parentElement;
    const actions = item.parentElement;
    if (!actions || actions.querySelector(':scope > .fmess-quick-reactions')) return;
    learnEmoji(row);
    const group = document.createElement('div');
    group.className = 'fmess-quick-reactions';
    group.setAttribute('role', 'group');
    for (const emoji of list) {
      const button = document.createElement('div');
      button.className = 'fmess-quick-reaction';
      button.setAttribute('role', 'button');
      button.setAttribute('tabindex', '0');
      button.dataset.emoji = emoji;
      const text = String.fromCodePoint(...emoji.split('_').map((c) => parseInt(c, 16)));
      button.setAttribute('aria-label', text);
      if (emojiPictures.has(emoji)) {
        const img = document.createElement('img');
        img.src = emojiPictures.get(emoji);
        img.alt = text;
        img.draggable = false;
        button.append(img);
      } else button.textContent = text;
      group.append(button);
    }
    actions.prepend(group);
  };
  document.addEventListener(
    'mouseover',
    (event) => {
      const row = event.target instanceof Element && event.target.closest('[role="main"] [data-scope="messages_table"]');
      if (!row || !quickReactionList().length) return;
      // The actions are drawn a moment after the pointer arrives.
      for (const ms of [0, 120, 400]) setTimeout(() => addQuickReactions(row), ms);
    },
    { capture: true, passive: true },
  );
  const pickReaction = async (row, emoji) => {
    reactButton(row)?.click();
    const picker = await waitFor(
      () => [...document.querySelectorAll('[role="menu"]')].find((menu) => menu.querySelector('[role="menuitemradio"] img[alt]')),
      1500,
    );
    if (!picker) return;
    learnEmoji(picker);
    const choice = [...picker.querySelectorAll('[role="menuitemradio"]')].find((option) => emojiName(option.querySelector('img')?.getAttribute('alt')) === emoji);
    if (choice) choice.click();
    else document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', code: 'Escape', keyCode: 27, bubbles: true }));
  };
  // The buttons fMess adds (the search box, quick reactions) take Enter and
  // Space as buttons do.
  document.addEventListener(
    'keydown',
    (event) => {
      if (event.key !== 'Enter' && event.key !== ' ') return;
      const button = event.target instanceof Element && event.target.closest('#fmess-search-box, .fmess-quick-reaction');
      if (!button) return;
      event.preventDefault();
      button.click();
    },
    true,
  );
  document.addEventListener(
    'click',
    (event) => {
      const button = event.target instanceof Element && event.target.closest('.fmess-quick-reaction');
      const row = button?.closest('[data-scope="messages_table"]');
      if (!row) return;
      event.preventDefault();
      event.stopPropagation();
      pickReaction(row, button.dataset.emoji).catch(() => {});
      // The actions close once a reaction is picked, as Messenger's own do:
      // the row carries data-fmess-reacted until the pointer leaves it.
      row.setAttribute('data-fmess-reacted', '');
      row.addEventListener('mouseleave', () => row.removeAttribute('data-fmess-reacted'), { once: true });
    },
    true,
  );

