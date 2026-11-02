  // Only text in the chat (messages, the pinned message, the details), in
  // dialogs and in the fields can be selected, as in a native app; the chat
  // list can't. (Only messages could at first, which kept the pinned
  // message's text, an address say, from being copied.)
  // A double or triple click selects a whole message and the break after it,
  // and WebKit then fills the space under its last line down to the edge of
  // the bubble, or across the chat; it fills only what can be selected
  // (Facebook marks every layer selectable, and wraps messages in more of
  // them now and then: the rule used to cover [data-message-id] only, then
  // rows gained a full-width role="article" around it). And Select All (⌘A)
  // outside a field selected the whole page: the chat list, the tabs, the
  // header, the details, all in bands.
  const selectStyle = document.createElement('style');
  selectStyle.textContent = `
    html[${CHATS}] body,
    html[${CHATS}] body * { -webkit-user-select: none !important; user-select: none !important; }
    html[${CHATS}] :is([role="main"], [role="dialog"]) [dir="auto"],
    html[${CHATS}] :is([role="main"], [role="dialog"]) [dir="auto"] *,
    html[${CHATS}] :is(input, textarea, [contenteditable="true"], [contenteditable="true"] *) {
      -webkit-user-select: text !important;
      user-select: text !important;
    }
  `;
  withRoot((root) => root.append(selectStyle));
  // Select All outside a field would select every bit of that text, from the
  // header down to the details; there it selects nothing, as in a native app.
  document.addEventListener(
    'keydown',
    (event) => {
      if (event.code !== 'KeyA' || !(event.metaKey || event.ctrlKey) || event.altKey || event.shiftKey) return;
      if (!document.documentElement.hasAttribute(CHATS)) return;
      if (event.target instanceof Element && event.target.closest('input, textarea, [contenteditable="true"]')) return;
      event.preventDefault();
    },
    true,
  );
  // Spell check (fMess Settings). It knows only the system's languages, so on
  // a Windows set up in English nearly every Vietnamese word was underlined
  // in red; turned off, fields don't check spelling. Fields are turned off as
  // they take the focus, so a message box Facebook draws again is too.
  let spellCheck = settings.spellCheck !== false;
  const UNCHECKED = 'data-fmess-unchecked';
  const FIELDS = 'input, textarea, [contenteditable="true"]';
  const uncheck = (field) => {
    if (!field?.spellcheck) return;
    field.spellcheck = false;
    field.setAttribute(UNCHECKED, '');
  };
  document.addEventListener(
    'focusin',
    (event) => !spellCheck && event.target instanceof Element && uncheck(event.target.closest(FIELDS)),
    true,
  );
  function setSpellCheck(on) {
    spellCheck = on;
    if (!on) return uncheck(document.activeElement?.closest?.(FIELDS));
    for (const field of document.querySelectorAll(`[${UNCHECKED}]`)) {
      field.removeAttribute(UNCHECKED);
      field.spellcheck = true;
    }
  }
  if (titleBarInset) {
    const dragStrip = document.createElement('div');
    dragStrip.setAttribute('data-tauri-drag-region', '');
    dragStrip.style.cssText = `position: fixed; inset: 0 0 auto 0; height: ${inset}; z-index: 2147483647;`;
    withRoot((root) => root.append(dragStrip));
  }

