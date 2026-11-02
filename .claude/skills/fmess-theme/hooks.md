# What a theme can hook into

Everything here was read from `src-tauri/src/inject.js`, `src-tauri/src/themes.rs`
and the shipped themes on 2026-11-02. Facebook and inject.js change: re-check with

```sh
cd ~/projects/herald && grep -n "data-fmess\|--fmess\|fmess-account\|UNDER_EVERY_THEME" src-tauri/src/inject.js
```

## How the CSS reaches the page

- fMess puts the chosen theme in `<style id="fmess-theme">`, appended last to `<html>`
  so it wins over Facebook's sheets. Its text is
  `[Facebook's light colours, if mode: light] + UNDER_EVERY_THEME + theme CSS` (`applyTheme`).
- Call windows (popups, `window.opener`) get no theme. Calls always look like Facebook's.
- **Opening comment** (`header()` in themes.rs):
  ```css
  /* fMess theme
     name: Teams
     preview: #rail #list #chat #your-bubble #their-bubble
  */
  ```
  `preview` needs exactly five hex colours or the Settings card has no preview, and the
  `built_in_themes_load_nothing_and_have_cards` test fails. `mode: light` is optional:
  in dark mode the page then also gets Facebook's light colours. Use it only for a
  theme with no dark design. Zalo and WhatsApp both have dark designs and leave it out.
- **Validation** (`check()`): at most 2 MB. No `@import`. `url()`/`src()` only with
  `data:`. No `image-set`, `-webkit-image-set`, `image`, `cross-fade`, `element`,
  `expression`. It is read with a real CSS tokenizer, so escapes and comments
  can't get around it.
- **UNDER_EVERY_THEME**: makes `[data-fmess-ui="composer-bar"]` (the reply / link preview
  bar above the box) and its two inner div levels `height: auto`, with 8px bottom padding,
  so larger type doesn't overflow. Don't set fixed heights there.

## Attributes and variables fMess sets

| Hook | Where / when | Use it for |
|---|---|---|
| `html[data-fmess-dark]` | Messenger is dark: its own setting `ENABLED`, or `USE_SYSTEM` while the system is dark | Every dark colour. Do **not** key on `.__fb-dark-mode` on the root: with `USE_SYSTEM`, Facebook marks only parts of the page. |
| `html[data-fmess-chats]` | path is `/messages…` or `/messenger_media…` | Scoping page decorations to the chats. The shipped themes don't scope, but new ones should. |
| `html[data-fmess-media]` | photo/video viewer open (`/messenger_media`, or its dialog on screen) | Hide `html::before/::after` and other overlays. On macOS the viewer has its own title bar. |
| `html[data-fmess-unread="N"]` | number of unread chats in the list, checked every 2 s; removed at 0 | `content: attr(data-fmess-unread)` badge on the rail |
| `html[data-fmess-name]` | your full name | `content: attr(data-fmess-name)` (Zalo's title bar: `"Zalo - " attr(data-fmess-name)`) |
| `--fmess-avatar` (inline on `<html>`) | `url("…")` of your profile picture; Facebook URLs expire, so it is refreshed every 10 min | a background layer. Always give a fallback: `var(--fmess-avatar, <gradient>)` |
| `--fmess-title-bar` (on `:root`) | 28px in the main window on macOS (the strip under the window buttons); 0 on Windows/Linux and in popups | height of a drawn title strip (`html::before`) and anything placed under it |
| `--header-height` | forced on every element: in the chats = the same inset; elsewhere 56px + inset (Facebook's top bar); 56px inside the viewer dialog | WhatsApp's strip uses it; prefer `--fmess-title-bar`, which stays the strip's height outside the chats too |
| `html[data-fmess-logged-out]` | login pages | (rarely needed) |
| `html[data-fmess-main]`, `[data-fmess-overlay-scrollbars]` | fMess's own scroll bars: thin native overlay ones on macOS, hidden plus a floating `#fmess-scroll-indicator` elsewhere | Don't restyle `::-webkit-scrollbar` against it. `--chat-scrollbar-thumb-color` is all a theme needs. |
| `[data-fmess-list]`, `[data-fmess-chat-row]`, `.fmess-pane-edge` | the chat list nav, the row holding chat + details, the 9px resize handles (z-index 5, line colour `--primary-button-background`) | Don't cover or hide the handles |

## `data-fmess-ui` marks

Facebook translates its aria-labels: a Vietnamese Facebook says "Tìm kiếm", not "Search".
A theme that matched only English labels broke for those users (commit 396bd90). fMess
therefore marks the parts themes style by where they are and by their icons. `markUi()`
runs every second (`syncPanes`) and after each click, and the composer is marked again on
input, so a fresh DOM may go up to a second without its marks.

| Mark | Element | Found by |
|---|---|---|
| `composer` | the message box row: `[role=region]` holding the contenteditable `[role=textbox]` in `[role=main]` | structure |
| `composer-bar` | the composer's previous sibling: the reply bar / link preview bar | structure |
| `voice-clip`, `attach`, `sticker`, `gif`, `emoji` | composer buttons | start of their `svg path d` (`COMPOSER_ICONS`). Breaks if Facebook redraws an icon. |
| `like` / `send` | the button in the composer's last `<span>`: `like` while it holds an `<img>` (the chat's quick reaction), `send` otherwise | structure, re-marked on input. **Trap (0.1.21):** Messenger's default thumbs-up is an `<svg>` ("Send a like"), so it is marked `send` on an empty box. A theme that shows `send` as a send button then sends a 👍. Guard: `> span:has(> [role="button"] path[d^="M16,9.1c"])` (its icon) besides the mark and the English label. |
| `more-actions` | any other composer button outside the text field. While typing, Messenger folds attach/sticker/gif into one "+", which gets this mark. | structure |
| `chat-header` | the header row above `[role=log]` | ancestor of `info` |
| `info` | header button with `aria-expanded` that opens the details. `[aria-expanded="true"]` while they're open. | structure |
| `audio-call`, `video-call` | the two svg buttons just before `info` on the same line (only marked when exactly two) | position |
| `details` | the details pane (2nd child of `[data-fmess-chat-row]`) | structure |
| `list-menu` | the ⋯ above the chat list (`aria-haspopup=menu`, not in the grid) | structure |
| `reactions` | the reaction picker `[role=menu]` (has `menuitemradio img[alt]`) | structure |
| `more-emojis` | its first `[role=button]` (the +) | structure |

**Selector rule:** mark first, English label as a fallback, never the label alone:

```css
:is([aria-label="Thread composer"], [data-fmess-ui="composer"]) :is([aria-label="Choose a sticker"], [data-fmess-ui="sticker"]) svg { … }
```

English labels used as fallbacks so far: `Thread composer`, `Attach a file up to 100 MB`,
`Open more actions`, `Choose a sticker`, `Choose a GIF`, `Choose an emoji`,
`Send a voice clip`, `Press enter to send`, `^="Send a"` (like), `Start a voice call`,
`*="audio call" i`, `Start a video call`, `Conversation information`,
`Settings, help and more`, `Messages reactions`, `Show more emojis`.

**There is no mark for:** the typing indicator, read receipts / seen avatars, link
previews and file cards inside messages, the first or last bubble of a group (so no
WhatsApp-style tails: `tail-in`/`tail-out` are in WhatsApp's icons.json but unused), or the
emoji picker.

**No mark, but a structure** (found for Teams, 2026-11-02):

| What | Selector |
|---|---|
| Seen receipt (reader's 14px picture under your last message) | `[data-scope="messages_table"] > div:not([role]) > div > span > img` (Teams swaps it for an eye with `content:`) |
| Reaction pill under a bubble (white, radius 10, shadow) | `[data-scope="messages_table"] [role="button"] > span > div[role="none"]:has(> div > div > span > img)`; style what is inside through a `@container style()` flag |
| Sender's name above a group's messages | `[data-scope="messages_table"] h4 span` |
| Unread chat row | the row's link holds Messenger's unread dot, a `[role="button"]`; read rows: `a[role="link"]:not(:has([role="button"]))` |
| Row picture box (68px, 56px picture inside) | `[role="row"] a[role="link"] > div > div > div:first-child`; resize the picture with `zoom` on its child (it holds the presence dot and group pictures too) |
| Header picture (40px) | `[data-fmess-ui="chat-header"] a[role="link"] > div > div > div:first-child` | To style one, find a structural selector with a probe (roles, `dir`,
`data-*`, inline `style*=` pieces), never text, and note it in the mapping table. If the
only way is new JS, that is an inject.js change for the user to decide on, not part of the
theme.

## Buttons a theme draws on the chat list: `--fmess-buttons`

Declare on `[role="navigation"]`: `name left top size, …`, in px from the nav's box. A
negative `top` counts from the bottom (top = bottom + top − size).

```css
[role="navigation"] {
  padding-left: 64px !important;
  --fmess-buttons: chats 12 10 40, calls 12 54 40, account 12 -8 40;
}
```

- While the pointer is over one, fMess sets `--fmess-hover-left` / `--fmess-hover-top` and
  `cursor: pointer` on the nav. Paint a highlight at that position (default off-screen):
  `radial-gradient(…) var(--fmess-hover-left, -100px) var(--fmess-hover-top, -100px) / 40px 40px no-repeat`.
- Clicks: **only `account` and `settings` do anything.** `account` opens fMess's native
  account menu (your name, Settings…, Log Out) to the right of the button. `settings` opens
  fMess's Settings. Every other name only lights up on hover.
- The native menu is not HTML. A theme can't restyle it or reproduce the target's profile
  card. Put the `account` button where the target shows your avatar, and stop there.

## `#fmess-account`

A 32px round `div[role=button]` that fMess puts first in the chat list header row (the row
with `h1` "Chats" and `a[href="/messages/new/"]`). It opens the same native menu. Hide it
(`display: none !important`) when the theme draws its own avatar button (WhatsApp does),
or restyle it in place.

## Messenger's DOM landmarks

| Selector | What |
|---|---|
| `[role="navigation"]` | the chat list pane (Facebook's own top bar is hidden in the chats). In a probe, `document.querySelector('[role="navigation"]')` finds an empty one first: use `[data-fmess-list]` |
| `[data-fmess-list] div:has(> #fmess-account)` | the list's header row ("Chats", ⋯, new message) |
| `[data-fmess-list] label:has(input[type="search"])` > `span > svg` | search field and its magnifier |
| `[role="row"]` > `[role="gridcell"]` > … `a[role="link"] > div` | a chat row; the inner div carries `--x-paddingBlock` / `--x-paddingInline` inline, overridable with `!important` |
| `[role="navigation"] h1 a` | the "Chats" title (WhatsApp masks a wordmark onto it) |
| `[role="navigation"] a[href="/messages/new/"]` | new message button |
| `[role="navigation"] label:has(input[type="search"])`, `input[type="search"]` | search box |
| `[role="navigation"] [role="tablist"]`, `[role="tab"][aria-selected="true"]` | filter tabs (All, Unread, Groups, Communities) |
| `[role="navigation"] [role="row"] a[role="link"]`, `a[aria-current="page"]` | chat rows, the open chat |
| `[role="main"]` | the chat pane (messages + details) |
| `[role="main"] [data-mw-file-dropzone]` | the message area (also the file drop zone): background goes here |
| `[data-mw-file-dropzone] [style*="background-image"]:has([role="log"])` | the per-chat background picture host: make it transparent |
| `[role="log"]` | the message list |
| `[role="log"] [role="article"]` | a message row (full width) |
| `[data-message-id]` | one message |
| `[data-message-id] [role="presentation"]` | the bubble |
| `[data-message-id] [dir="auto"]` | message text |
| `[data-scope="date_break"] > span > span` | date divider text |
| `[style*="--chat-outgoing-message-bubble-background-color"]` | each chat's colour host (redeclares the `--chat-*` colours) |
| `[role="tooltip"]` | tooltips. The element itself is the dark island: `background: rgba(0,0,0,.8)`, radius 12, padding 12, a box-shadow; its text is a trimmed `span` (cap height only, so pad more than the target's line box) |
| `[role="menu"]`, `[role="menuitem"]`, `[role="menuitemradio"]` | menus, the reaction picker. A menu: `[role="menu"] > div:first-child` is the card (bg, `border-radius: 0 12px 12px`, shadow), `> svg` the arrow, `div[style*="max-height"]` the 8px padding, `hr` the dividers; each item ends in a `div[role="none"]` overlay painted **over** the content with `--hover-overlay` |
| `[role="dialog"]` | dialogs, the photo viewer |
| `.__fb-dark-mode` / `.__fb-light-mode` | Facebook's mode islands: subtrees that redeclare every colour variable |

## Messenger's own variables (remap these)

Set them with `!important` on all four of
`:root, .__fb-dark-mode, .__fb-light-mode, [style*="--chat-outgoing-message-bubble-background-color"]`.
Facebook redeclares them on mode islands and on each chat's colour host. Point every value
at the theme's own tokens (`var(--wa-…)`), so dark mode only swaps the tokens.

| Variable | Paints |
|---|---|
| `--accent`, `--primary-button-background`, `--mwp-primary-theme-color` | brand colour: buttons, links in UI, unread dots, pane-edge line |
| `--primary-deemphasized-button-background` / `-text` | selected tab / soft buttons |
| `--text-highlight` | text selection |
| `--primary-text`, `--secondary-text`, `--placeholder-text` | text levels |
| `--primary-icon`, `--secondary-icon`, `--placeholder-icon` | Facebook's icon colours |
| `--blue-link` | links |
| `--divider` | lines between panes and sections (also used by fMess's pane divider) |
| `--web-wash`, `--surface-background`, `--card-background`, `--messenger-card-background` | page and pane backgrounds |
| `--comment-background`, `--input-background`, `--secondary-button-background` | fields and grey buttons |
| `--hover-overlay` | hover tint. Drawn on top of menu items and buttons: keep it translucent, or the hovered item's text disappears |
| `--focus-ring-blue`, `--focus-ring-shadow-default`, `--focus-ring-shadow-inset`, `--focus-ring-outline-link` | Facebook's keyboard focus ring (blue on white) |
| `--popover-background` | menus, popovers |
| `--notification-badge` | red badges |
| `--chat-bubble-border-radius`, `--chat-bubble-padding-vertical`, `--chat-bubble-padding-horizontal` | bubble shape |
| `--chat-outgoing-message-bubble-background-color`, `--chat-outgoing-message-background-gradient` (set `initial`) | your bubbles |
| `--chat-incoming-message-bubble-background-color` | theirs |
| `--chat-message-text-color`, `--chat-incoming-message-text-color`, `--chat-admin-text-color` | message text, system lines |
| `--chat-replied-message-background-color` | the quoted message in a reply |
| `--chat-composer-background-color`, `--chat-composer-input-background-color`, `--chat-composer-button-color`, `--chat-composer-caret-color` | composer |
| `--chat-edit-message-overlay-color` | editing a message |
| `--chat-scrollbar-thumb-color` | scroll thumb |
| `--mwp-header-background-color`, `--mwp-header-button-color`, `--mwp-header-text-color`, `--mwp-subheader-text-color` | chat header |
| `--mwp-message-row-background` | message row (set `transparent`) |
| `--reaction-pill-background-color`, `--reaction-pill-multireact-selected-color` | reaction pills under bubbles |

fMess itself sets `--messenger-card-spacing: 0`, `--messenger-card-corner-radius: 0`,
`--messenger-card-box-shadow: none` in the chats (edge-to-edge panes). Leave those alone.

## Facebook's mode islands: two blocks every theme ends with

Copy these verbatim from the end of `whatsapp.src.css`:

1. **Tooltips** are drawn in the opposite mode to the page (dark on light, light on dark).
   Give `[role="tooltip"] .__fb-dark-mode` light text and `.__fb-light-mode` dark text
   (commit ebb937b: before, dark mode had white-on-white tooltips).
2. **Dark islands on a light page** (toasts such as "All chats loaded.", the photo
   viewer): `html:not([data-fmess-dark]) .__fb-dark-mode` keeps light text (commit 0af239e).

To make tooltips look like the target's, style the tooltip's background and type too, but
keep the text colour following its own background.

## CSS and performance gotchas (measured, not guessed)

- **`:has()` to the left of a combinator** (`pane:has(x) h1`) makes WebKit restyle the whole
  page whenever an element like `x` comes or goes anywhere. Use a fMess mark, or put a flag
  variable on the pane and query it:
  ```css
  [role="main"] [data-fmess-ui="details"] { --z-details: 1; }
  @container style(--z-details: 1) { h1, h2, h3 { font-size: 16px !important; } }
  ```
  `:has()` in the last compound (`> span:has([data-fmess-ui="like"])`) is acceptable. Keep its
  argument narrow.
- **No `:has()` inside `:has()`.** WebKit drops the whole rule.
- **A picture over the whole chat** (pattern, doodles) as a `-webkit-mask` or a background on
  the scrolling area cut scrolling to 7–20 fps. Put it on a `::before` layer of the
  dropzone with `z-index: -1`, `isolation: isolate` on the dropzone, and
  `will-change: transform`.
- **`box-shadow` / `filter` on bubbles** (`[data-message-id] [role="presentation"]`) leaves
  ghost boxes and covers reactions. Change colours and radius only.
- **`img[alt="❤"] { content: url(…) }`** swaps that emoji **everywhere**, in message text
  too, not only in the picker.
- **Text the theme writes** (`content: "Contact info"`, a placeholder drawn by `::after`) is
  in one language. Facebook's own text is translated. Only write the target's brand words,
  or accept that the theme speaks its home language (Zalo writes Vietnamese).
- **Zoom.** The user runs fMess at zoom 1.25. Compare and measure at 1.0 (testing.md).
- **Flags for descendants.** Anything that would be `x:has(y) z` becomes a custom property on
  `x:has(y)` plus `@container style(--flag: 1) { z { … } }` (Teams: read rows, reaction
  pills, the like button, the composer field). The flag inherits, so the query matches every
  descendant.
- **`zoom`** shrinks Messenger's nested picture boxes (avatar + ring + presence dot) in one
  rule. `getBoundingClientRect()` of zoomed children reports odd numbers; trust the snapshot.
- **Hit areas.** Never hide a button's box to restyle it. Hide its `svg *` (or the svg's
  `visibility`) and paint on the button or the svg, so the click target and the tooltip stay.
