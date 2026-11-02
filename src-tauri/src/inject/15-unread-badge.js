  // The Dock badge counts unread chats in the list; the page title counts
  // Facebook's own notifications instead.
  let lastUnread = -1;
  // Reading the list lays the page out (about 45 ms), so it is read again only
  // after the list has changed: a message came in, a chat was read.
  let watchedList = null;
  let listChanged = true;
  const listWatcher = new MutationObserver(() => {
    listChanged = true;
  });
  setInterval(() => {
    // Wait until scrolling stops.
    if (!onChatsPage() || performance.now() - lastScrollAt < 1000) return;
    const list = document.querySelector('[role="grid"] [role="row"] a[href*="/messages/"]')?.closest('[role="grid"]') ?? null;
    if (list !== watchedList) {
      listWatcher.disconnect();
      if (list) listWatcher.observe(list, { subtree: true, childList: true, characterData: true, attributes: true });
      watchedList = list;
      listChanged = true;
    }
    if (!listChanged) return;
    listChanged = false;
    const count = chatLinks().filter(isUnreadChat).length;
    if (count === lastUnread) return;
    // The list fills in while the page loads; that isn't new mail.
    const rose = lastUnread >= 0 && count > lastUnread && performance.now() > 10000;
    lastUnread = count;
    // A theme can show it (attr(data-fmess-unread)).
    if (count) document.documentElement.setAttribute('data-fmess-unread', count);
    else document.documentElement.removeAttribute('data-fmess-unread');
    invoke('set_unread_count', { count }).catch(() => {});
    if (rose && !document.hasFocus()) setTimeout(notifyNewestChat, 1500);
  }, 2000);

