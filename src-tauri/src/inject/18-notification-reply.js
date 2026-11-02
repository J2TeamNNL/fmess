  // Called from Rust when the user clicks a native notification.
  window.__fmessNotificationClicked = (id, thread) => {
    const notification = shown.get(id);
    window.focus();
    if (notification) notification.fmessFire('click');
    else if (thread) chatLinks().find((row) => new RegExp(`/t/${thread}/?$`).test(row.getAttribute('href')))?.click();
  };

  // Replying from a notification, as in the old app. Messenger sends what is
  // typed in a chat's message box (encrypted chats too), so fMess opens the
  // chat, types the reply there and sends it, then goes back to the chat
  // that was open. The window stays as it was.
  const waitFor = async (check, ms) => {
    for (const end = Date.now() + ms; Date.now() < end; await new Promise((resolve) => setTimeout(resolve, 100))) {
      const value = check();
      if (value) return value;
    }
    return null;
  };
  const openChat = (id, thread) => {
    // Messenger's own handler for the notification knows the chat's kind.
    const notification = shown.get(id);
    if (notification) return notification.fmessFire('click');
    const row = chatLinks().find((link) => new RegExp(`/t/${thread}/?$`).test(link.getAttribute('href')));
    if (row) return row.click();
    history.pushState(null, '', `/messages/t/${thread}/`);
    dispatchEvent(new PopStateEvent('popstate'));
  };
  // One reply at a time: a second one opens its chat, which would take the
  // message box from a first one still typing.
  let replying = Promise.resolve();
  window.__fmessReply = (id, thread, text) => {
    const reply = replying.then(() => replyNow(id, thread, text));
    replying = reply.catch(() => {});
    return reply;
  };
  // A draft a reply took out of its chat and couldn't put back, because
  // another chat was opened meanwhile, goes back the next time the chat is
  // open (if nothing has been typed there since).
  const strayDrafts = new Map();
  // `previousBox` is the message box on screen when the address changed:
  // Messenger changes the address first, and the chat that was open keeps
  // its box a moment longer. Only a box that comes after it is this chat's.
  // `stillHere` turns false once another chat is opened, even if this one is
  // opened again after: a box found then could be the other chat's.
  const returnStrayDraft = async (previousBox, stillHere = () => true) => {
    const openThread = () => location.pathname.match(/^\/messages\/(?:e2ee\/)?t\/([^/]+)/)?.[1];
    const thread = openThread();
    const draft = thread && strayDrafts.get(thread);
    if (!draft) return;
    const box = await waitFor(() => stillHere() && openThread() === thread && composer() !== previousBox && composer(), 5000);
    if (!box || !stillHere() || openThread() !== thread || composer() !== box || box.textContent.trim()) return;
    box.focus();
    document.execCommand('insertText', false, draft);
    // Kept until it is really back.
    if (await waitFor(() => box.isConnected && box.textContent.includes(draft.trim()), 2000)) strayDrafts.delete(thread);
  };
  window.__fmessReturnDraft = returnStrayDraft;
  const replyNow = async (id, thread, text) => {
    const inChat = () => new RegExp(`^/messages/(?:e2ee/)?t/${thread}/?$`).test(location.pathname);
    const before = location.pathname;
    const opening = !inChat();
    const oldBox = composer();
    if (opening) openChat(id, thread);
    // Until the chat opens, the previous chat's box is still there.
    const box =
      (await waitFor(() => inChat() && (!opening || composer() !== oldBox) && composer(), 10000)) ?? (inChat() && composer());
    // Checked before each step, since each wait leaves time to open another
    // chat: the reply's chat is still open, with the same message box.
    const here = () => Boolean(box?.isConnected) && inChat() && composer() === box;
    let sent = false;
    let draft = '';
    let draftOut = false;
    if (here()) {
      box.focus();
      // A draft in the chat waits while the reply goes out, then comes back.
      draft = box.textContent.trim() ? box.textContent : '';
      if (draft) {
        document.execCommand('selectAll');
        document.execCommand('delete');
        draftOut = true;
        await waitFor(() => !box.textContent.trim(), 1000);
      }
      if (here()) {
        box.focus();
        document.execCommand('insertText', false, text);
        // The editor shows what was typed a moment later.
        if ((await waitFor(() => box.textContent.trim(), 1000)) && here()) {
          box.dispatchEvent(
            new KeyboardEvent('keydown', { key: 'Enter', code: 'Enter', keyCode: 13, which: 13, bubbles: true, cancelable: true }),
          );
          sent = Boolean(await waitFor(() => !box.textContent.trim(), 3000));
        }
      }
      if (draftOut && here()) {
        // An unsent reply stays in the notice below instead.
        if (!sent) {
          document.execCommand('selectAll');
          document.execCommand('delete');
        }
        box.focus();
        document.execCommand('insertText', false, draft);
        draftOut = false;
      }
    }
    if (draftOut) strayDrafts.set(thread, draft);
    if (!sent) {
      // Say so, and keep the reply: clicking opens the chat.
      const notice = new FMessNotification(say('Reply not sent', 'Chưa gửi được tin trả lời'), {
        body: draftOut ? `${text}\n\n${say('Your draft', 'Bản nháp của bạn')}: ${draft}` : text,
      });
      notice.onclick = () => openChat(id, thread);
      return false;
    }
    if (draftOut) {
      const notice = new FMessNotification(say('Your draft is kept', 'Bản nháp của bạn vẫn còn'), { body: draft });
      notice.onclick = () => openChat(id, thread);
    }
    // Back to the chat that was open, unless another one has been opened since.
    if (location.pathname !== before && inChat() && /^\/messages\/(e2ee\/)?t\//.test(before)) history.back();
    return true;
  };

  // As in the old app, reading a chat clears its notifications: opening it
  // in fMess, or coming back to fMess while it is open.
  const clearOpenChatNotifications = () => {
    const thread = location.pathname.match(/^\/messages\/(?:e2ee\/)?t\/(\d+)/)?.[1];
    if (thread && document.hasFocus()) invoke('clear_notifications', { thread }).catch(() => {});
  };
  window.addEventListener('focus', clearOpenChatNotifications);

