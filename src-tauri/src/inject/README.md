# The page script

fMess runs one script in the Messenger page (and in the call windows it opens),
before the page's own scripts. It is written here in parts, one per concern, that
share one function's scope: `build.rs` joins every `.js` file in file-name order
into `inject.js`, which `webview.rs` puts in the page. So a part can use what an
earlier part declares, and the numbers set that order.

| Part | What it does |
|---|---|
| 01-start | Opens the script's function; debug listener log; `withRoot` |
| 02-splash | fMess's loading screen in place of Facebook's |
| 03-facebook-modules | Adjusts a few of Facebook's own modules as they load; Close in call windows |
| 04-privacy | Block Typing (fMess Settings) |
| 05-messenger-prefs | Loads Facebook modules on demand; Messenger's Preferences for the Settings window |
| 06-routes | The chats' route and its marks on `<html>`; the cursor in the message box; links |
| 07-layout | Layout closer to the old desktop app; the strip under the window buttons |
| 08-panes | Resizable chat list and details |
| 09-marks | `data-fmess-ui` marks for themes, found without the page's language |
| 10-selection | What can be selected; Select All |
| 11-scrollbars | Overlay scroll bars |
| 12-viewer | The photo/video viewer: its buttons and zoom |
| 13-shortcuts | Keyboard shortcuts |
| 14-notifications | Web notifications to the system's, with sender and chat; `say` for fMess's own words |
| 15-unread-badge | The unread count for the Dock and taskbar |
| 16-read-ahead | Reading recent chats ahead so they open at once |
| 17-theme-hooks | Buttons, search box and quick reactions themes can ask for |
| 18-notification-reply | Clicks on notifications and replying from them |
| 19-account | Your picture and the account menu |
| 20-theme | The theme picked in Settings; settings from fMess; closes the function |

The joined script is the same text the single file was; `node
tests/notification-reply.mjs` joins the parts the same way. To check the
script's syntax: `cat src-tauri/src/inject/*.js | node --check`
