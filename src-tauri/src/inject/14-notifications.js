  const shown = new Map();
  let nextId = Math.floor(Math.random() * 1e9);
  // When the page last raised a notification itself (see notifyNewestChat).
  let lastPageNotification = 0;
  let raisingFallback = false;
  // Messenger may raise the same message twice; browsers show one notification
  // per tag (Messenger uses the message id), so fMess does too.
  const shownTags = new Set();

  const invoke = (command, args) =>
    window.__TAURI_INTERNALS__
      ? window.__TAURI_INTERNALS__.invoke(command, args)
      : Promise.reject(new Error('fMess bridge unavailable'));

  // Sender and chat of a message notification, by message id (Messenger's
  // tag), read as MWDesktopNotifications does (see MODULE_PATCHES).
  const messageDetails = new Map();
  // A message sent over LATE_MS ago, or in a chat read past it (on another
  // device, say), needs no notification.
  async function isOldNews(db, message) {
    const sent = int64(message?.timestampMs);
    if (sent && serverNow() - Number(sent) > LATE_MS) return true;
    const ReQL = window.require('ReQL');
    const chat = await ReQL.firstAsync(ReQL.fromTableAscending(db.tables.threads).getKeyRange(message.threadKey));
    return Boolean(chat && sent) && int64(chat.lastReadWatermarkTimestampMs) >= sent;
  }
  async function readMessageDetails(db, message) {
    const ReQL = window.require('ReQL');
    const first = (table, ...key) => ReQL.firstAsync(ReQL.fromTableAscending(table).getKeyRange(...key));
    const [thread, contact, participant] = await Promise.all([
      first(db.tables.threads, message.threadKey),
      first(db.tables.contacts, message.senderId),
      first(db.tables.participants, message.threadKey, message.senderId),
    ]);
    if (!thread || !contact) return null;
    const picture = (name, row) => {
      try {
        return moduleFunction(window.require(name))(row) ?? null;
      } catch {
        return null;
      }
    };
    const isGroup = window.require('LSMessagingThreadTypeUtil').isGroup(thread.threadType);
    return {
      thread: String(int64(message.threadKey)),
      sender: participant?.nickname || contact.name || '',
      senderPicture:
        picture('getLSMediaContactProfilePictureLargeUrl', contact) ?? picture('getLSMediaContactProfilePictureUrl', contact),
      group: isGroup ? { name: thread.threadName || null, picture: picture('getLSMediaThreadPictureUrl', thread) } : null,
    };
  }
  // A picture for a native notification, as base64; none rather than a late
  // notification when it is slow or large.
  const pictureData = async (url) => {
    if (!url || typeof url !== 'string') return null;
    try {
      const response = await fetch(url);
      const blob = response.ok ? await response.blob() : null;
      if (!blob || blob.size > 4_000_000) return null;
      const bytes = new Uint8Array(await blob.arrayBuffer());
      let binary = '';
      for (let i = 0; i < bytes.length; i += 0x8000) binary += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
      return btoa(binary);
    } catch {
      return null;
    }
  };
  const within = (promise, ms) => Promise.race([promise, new Promise((resolve) => setTimeout(() => resolve(null), ms))]);
  const sendNotification = async (notification) => {
    const details = await within(messageDetails.get(String(notification.tag)) ?? Promise.resolve(null), 2000);
    // With people's pictures hidden (Privacy), notifications go without them.
    const pictures = !privacy.hideAvatars;
    const [icon, image, senderPicture, groupPicture] = await within(
      Promise.all([
        details || !pictures ? null : pictureData(notification.icon),
        pictureData(notification.image),
        details && pictures ? pictureData(details.senderPicture) : null,
        details?.group && pictures ? pictureData(details.group.picture) : null,
      ]),
      3000,
    ) ?? [];
    const message = details && {
      thread: details.thread,
      sender: details.sender,
      senderPicture: senderPicture ?? null,
      group: details.group && { name: details.group.name, picture: groupPicture ?? null },
    };
    return invoke('notify', {
      id: notification.fmessId,
      title: notification.title,
      body: notification.body,
      icon: icon ?? null,
      image: image ?? null,
      message: message || null,
    });
  };

  class FMessNotification extends EventTarget {
    static get permission() {
      return 'granted';
    }

    static requestPermission(callback) {
      callback?.('granted');
      return Promise.resolve('granted');
    }

    constructor(title, options = {}) {
      super();
      this.title = String(title ?? '');
      this.body = String(options.body ?? '');
      this.tag = options.tag ?? '';
      this.icon = options.icon ?? '';
      this.image = options.image ?? '';
      this.data = options.data ?? null;
      this.silent = Boolean(options.silent);
      this.onclick = this.onshow = this.onclose = this.onerror = null;
      if (!raisingFallback) lastPageNotification = Date.now();
      if (this.tag) {
        if (shownTags.has(this.tag)) return;
        shownTags.add(this.tag);
        if (shownTags.size > 500) shownTags.delete(shownTags.values().next().value);
      }
      // The native side takes an i32 id.
      this.fmessId = nextId++ % 0x7fffffff;
      shown.set(this.fmessId, this);
      if (shown.size > 100) shown.delete(shown.keys().next().value);
      sendNotification(this).then(
        () => this.fmessFire('show'),
        () => this.fmessFire('error'),
      );
    }

    close() {
      if (shown.delete(this.fmessId)) this.fmessFire('close');
    }

    fmessFire(type) {
      const event = new Event(type);
      this[`on${type}`]?.call(this, event);
      this.dispatchEvent(event);
    }
  }

  window.Notification = FMessNotification;

  // Some code paths notify through a service worker registration instead.
  if (window.ServiceWorkerRegistration) {
    ServiceWorkerRegistration.prototype.showNotification = function (title, options) {
      new FMessNotification(title, options);
      return Promise.resolve();
    };
  }

  // Keep navigator.permissions in agreement with Notification.permission.
  const query = navigator.permissions?.query?.bind(navigator.permissions);
  if (query) {
    navigator.permissions.query = (descriptor) =>
      descriptor?.name === 'notifications'
        ? Promise.resolve({
            name: 'notifications',
            state: 'granted',
            onchange: null,
            addEventListener() {},
            removeEventListener() {},
          })
        : query(descriptor);
  }

  // facebook.com only notifies through Web Push, which embedded WebKit lacks,
  // so new messages never reach the shim above. fMess calls this instead when
  // the unread count in the title goes up while the window is in the
  // background, and it notifies about the newest chat in the list.
  // A chat row reads "<name> <preview> · <age>" on screen. It also carries
  // text only screen readers get, clipped to 1px: "Active now" before the name,
  // and "Unread message:" after it when the chat is unread. Emoji are images.
  const rowTexts = (row) => {
    const texts = [];
    const walker = document.createTreeWalker(row, NodeFilter.SHOW_TEXT | NodeFilter.SHOW_ELEMENT);
    for (let node = walker.nextNode(); node; node = walker.nextNode()) {
      const isEmoji = node instanceof HTMLImageElement && node.alt && node.getBoundingClientRect().width <= 24;
      if (node.nodeType !== Node.TEXT_NODE && !isEmoji) continue;
      const text = isEmoji ? node.alt : node.textContent.trim();
      const box = (isEmoji ? node : node.parentElement)?.getBoundingClientRect();
      if (text && box) texts.push({ text, visible: box.width > 1 && box.height > 1 });
    }
    return texts;
  };
  const isUnreadChat = (row) => {
    const texts = rowTexts(row);
    const name = texts.findIndex((part) => part.visible);
    return name >= 0 && texts.slice(name + 1).some((part) => !part.visible);
  };
  const chatSummary = (row) => {
    const texts = rowTexts(row).filter((part) => part.visible).map((part) => part.text);
    const age = texts.lastIndexOf('·');
    const preview = (age > 0 ? texts.slice(1, age) : texts.slice(1)).join(' ').replace(/\s+/g, ' ');
    return { name: texts[0] ?? '', preview };
  };

  // Whether a chat row has nothing to notify about, as Messenger's copy of
  // the chat list says: its newest message is your own, or came in late.
  const nothingNew = async (row) => {
    const thread = row.getAttribute('href')?.match(/\/t\/(\d+)/)?.[1];
    const me = currentUserId();
    if (!thread || !me) return false;
    await loadModules(['ReQL', 'LSDatabaseSingleton']);
    const ReQL = window.require('ReQL');
    const db = await window.require('LSDatabaseSingleton').LSDatabaseSingleton;
    const chat = await ReQL.firstAsync(ReQL.fromTableAscending(db.tables.threads).getKeyRange(int64Key(thread)));
    if (!chat) return false;
    return int64(chat.snippetSenderContactId) === BigInt(me) || serverNow() - Number(int64(chat.lastActivityTimestampMs)) > LATE_MS;
  };

  // Facebook raises its own notifications (see MODULE_PATCHES). In case it
  // stops doing so, notify about the newest unread chat when the count rises
  // and Facebook stayed quiet.
  const notifyNewestChat = async () => {
    const links = chatLinks();
    const unread = links.filter(isUnreadChat);
    // A message you send from your phone in an encrypted chat can leave that
    // chat unread here, and a computer back from sleep catches up on chats
    // all at once; Messenger notifies about neither.
    let link;
    for (const row of unread.length ? unread : links.slice(0, 1)) {
      if (!(await nothingNew(row).catch(() => false))) {
        link = row;
        break;
      }
    }
    // Messenger itself stays quiet during Do Not Disturb.
    if (!link || isMuted() || Date.now() - lastPageNotification < 4000) return;
    const chat = chatSummary(link);
    const href = link.getAttribute('href');
    raisingFallback = true;
    try {
      const notification = new FMessNotification(chat.name || 'Messenger', {
        body: chat.preview || 'New message',
        icon: link.querySelector('img')?.currentSrc,
      });
      notification.onclick = () => chatLinks().find((row) => row.getAttribute('href') === href)?.click();
    } finally {
      raisingFallback = false;
    }
  };

