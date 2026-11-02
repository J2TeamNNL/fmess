  // The old desktop app showed whose account it was on: a profile picture
  // above the chats, opening a menu with Log Out. Facebook keeps both in its
  // top bar, which fMess hides in the chats, so the picture goes before
  // "Chats" and opens fMess's native account menu (show_account_menu).
  const ACCOUNT = 'fmess-account';
  const accountStyle = document.createElement('style');
  accountStyle.textContent = `
    #${ACCOUNT} {
      flex: none;
      width: 32px;
      height: 32px;
      /* As tall as the header row, which is shorter than "Chats" itself. */
      margin: -8px 12px -8px 0;
      border-radius: 50%;
      background: var(--secondary-button-background, #e4e6eb) center / cover no-repeat;
      color: var(--primary-text, #050505);
      font: 600 15px/32px system-ui, -apple-system, sans-serif;
      text-align: center;
      cursor: pointer;
    }
    #${ACCOUNT}:hover { filter: brightness(0.9); }
  `;
  withRoot((root) => root.append(accountStyle));

  // The picture comes from Messenger's own contact entry for you. Facebook's
  // hidden top bar has it too, but also every profile of its account menu.
  let contactPicture = null;
  const readContactPicture = async () => {
    const me = currentUserId();
    if (!me) return;
    await loadModules(['LSDatabaseSingleton']);
    const db = await window.require('LSDatabaseSingleton').LSDatabaseSingleton;
    contactPicture = (await db.tables.contacts.get(int64Key(me)))?.profilePictureUrl || contactPicture;
    // A theme can show your picture and name (var(--fmess-avatar), attr(data-fmess-name)).
    const root = document.documentElement;
    if (contactPicture) root.style.setProperty('--fmess-avatar', `url("${contactPicture}")`);
    root.setAttribute('data-fmess-name', window.require('CurrentUserInitialData').NAME || '');
  };
  const currentAccount = () => {
    try {
      const user = window.require('CurrentUserInitialData');
      const barPicture = [...document.querySelectorAll('[role="banner"] svg image')].find((image) => !image.closest('[role="dialog"]'));
      return {
        name: user.NAME,
        initial: [...(user.SHORT_NAME || user.NAME || '?')][0],
        picture: contactPicture || barPicture?.getAttribute('xlink:href') || barPicture?.getAttribute('href') || '',
      };
    } catch {
      return null;
    }
  };
  if (!window.opener) {
    window.addEventListener('load', () => setTimeout(() => readContactPicture().catch(() => {}), 2000));
    // Picture links expire; Messenger refreshes the entry.
    setInterval(() => readContactPicture().catch(() => {}), 10 * 60 * 1000);
  }

  let account = null;
  function openAccountMenu() {
    const box = document.getElementById(ACCOUNT).getBoundingClientRect();
    invoke('show_account_menu', { name: account.name, x: box.left, y: box.bottom + 4 }).catch(() => {});
  }

  // Facebook redraws the chat list header now and then, so check each second.
  setInterval(() => {
    if (window.opener || !onChatsPage()) return;
    account = currentAccount() || account;
    // The header row holds "Chats" and the new message button.
    let header = document.querySelector('a[href="/messages/new/"]');
    while (header && !header.querySelector('h1')) header = header.parentElement;
    if (!header || !account) return;
    let button = document.getElementById(ACCOUNT);
    if (!button) {
      button = document.createElement('div');
      button.id = ACCOUNT;
      button.setAttribute('role', 'button');
      button.tabIndex = 0;
      button.addEventListener('click', openAccountMenu);
      button.addEventListener('keydown', (event) => {
        if (event.key !== 'Enter' && event.key !== ' ') return;
        event.preventDefault();
        openAccountMenu();
      });
    }
    if (header.firstElementChild !== button) header.prepend(button);
    if (button.title !== account.name) {
      button.title = account.name;
      button.setAttribute('aria-label', account.name);
    }
    if (button.dataset.picture !== account.picture) {
      button.dataset.picture = account.picture;
      button.style.backgroundImage = account.picture ? `url("${account.picture}")` : '';
      button.textContent = account.picture ? '' : account.initial;
    }
  }, 1000);

  // Called from the account menu. Facebook's own Log Out item posts this
  // form, which carries the logout token, and Facebook signs out from there.
  window.__fmessLogOut = () => {
    const form = document.querySelector('form[action^="/logout.php"]');
    if (!form) return false;
    if (form.requestSubmit) form.requestSubmit();
    else form.submit();
    return true;
  };

