# What a theme can hook into

Everything here was read from the page script (`src-tauri/src/inject/`, joined into inject.js at build time), `src-tauri/src/themes.rs`
and the shipped themes on 2026-11-02. Facebook and inject.js change: re-check with

```sh
cd ~/projects/herald && grep -rn "data-fmess\|--fmess\|fmess-account\|UNDER_EVERY_THEME" src-tauri/src/inject/
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
| `html[data-fmess-media]` | photo/video viewer open (`/messenger_media`, or the viewer on screen: in chats it is a layer at the same address) | Hide `html::before/::after` and other overlays. On macOS the viewer has its own title bar. |
| `html[data-fmess-unread="N"]` | number of unread chats in the list, checked every 2 s; removed at 0 | `content: attr(data-fmess-unread)` badge on the rail |
| `html[data-fmess-name]` | your full name | `content: attr(data-fmess-name)` (Zalo's title bar: `"Zalo - " attr(data-fmess-name)`) |
| `--fmess-avatar` (inline on `<html>`) | `url("…")` of your profile picture; Facebook URLs expire, so it is refreshed every 10 min | a background layer. Always give a fallback: `var(--fmess-avatar, <gradient>)` |
| `--fmess-title-bar` (on `:root`) | 28px in the main window on macOS (the strip under the window buttons); 0 on Windows/Linux and in popups | height of a drawn title strip (`html::before`) and anything placed under it |
| `--header-height` | forced on every element: in the chats = the same inset; elsewhere 56px + inset (Facebook's top bar); 56px inside the viewer dialog | WhatsApp's strip uses it; prefer `--fmess-title-bar`, which stays the strip's height outside the chats too. A theme may **raise** it in the chats for a taller title band: `html[data-fmess-chats], html[data-fmess-chats] * { --header-height: 48px !important }` (Teams); Messenger's page then starts under the band, and the viewer keeps its 56px (a more specific rule) |
| `[data-tauri-drag-region]` | macOS only: fMess's transparent strip, `--fmess-title-bar` tall, z-index max, that drags the window | A taller band can make it taller (`height: 48px !important`) and cut a hole where a real control sits (`clip-path: polygon(evenodd, outer…, hole…)`), so the control gets the clicks. Window buttons (below) need no hole |
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
| Header picture (40px) | `[data-fmess-ui="chat-header"] a[role="link"] > div > div > div:first-child` |
| Composer placeholder ("Aa") | `[role="textbox"] + div[aria-hidden="true"] > div`, there only while the box is empty (the 1×1 "Aa" before the editor is for screen readers) |
| A chat row's time | `div[style*="height: 8px"] + div > span:last-child:has(abbr)` (line 2: preview, " · ", time) |
| The list's title row, pills row | `[data-fmess-list] div:has(> div > div > #fmess-account)`; `div:has(> div > div > [role="tablist"])` (the tablist and its two wrappers are 36px by class) |
| Page language | `html[lang]` (`en` for an English Facebook): the place to write the target's own words for English only | To style one, find a structural selector with a probe (roles, `dir`,
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

- While the pointer is over one, fMess sets `--fmess-hover-left` / `--fmess-hover-top`,
  `data-fmess-hover="<name>"` and `cursor: pointer` on the nav (all removed when none is
  lit). The name lets a theme label the lit button (Teams' tooltips: the nav's free
  pseudo-element with `content: var(--label)`, the label set per
  `[data-fmess-hover="…"]`) or treat one differently (Teams keeps the open app's slot
  clear). Paint a highlight at that position (default off-screen):
  `radial-gradient(…) var(--fmess-hover-left, -100px) var(--fmess-hover-top, -100px) / 40px 40px no-repeat`.
- Clicks: **only `account` and `settings` do anything.** `account` opens fMess's native
  account menu (your name, Settings…, Log Out) to the right of the button. `settings` opens
  fMess's Settings. Every other name only lights up on hover.
- The native menu is not HTML. A theme can't restyle it or reproduce the target's profile
  card. Put the `account` button where the target shows your avatar, and stop there.

## Buttons anywhere in the window: `--fmess-window-buttons`

The same idea, for buttons drawn outside the chat list (Teams' "…" and avatar at the
right of its title bar). Declare on `:root` (scope it, e.g.
`html[data-fmess-chats]:not([data-fmess-media])`): `name left top size, …` in px from the
window's top left. A negative `left` counts from the right edge and a negative `top` from
the bottom, as `--fmess-buttons` does: left = window width + left − size.

```css
html[data-fmess-chats]:not([data-fmess-media]) {
  --fmess-window-buttons: settings -53 8 32, account -12 8 32;
}
```

- Hover: fMess sets `--fmess-window-hover-left` / `--fmess-window-hover-top` (window px),
  `data-fmess-window-hover="<name>"` and `cursor: pointer` on `:root`. Draw the highlight
  in a fixed pseudo-element at those positions (default off-screen).
- Clicks: `account` opens fMess's native account menu **under** the button (kept inside the
  window), `settings` opens Settings, `sidebar` folds the theme's sidebar away and back
  (below); other names only light up. A press on one never starts a window drag, even over
  the macOS drag strip.
- `sidebar` toggles `data-fmess-sidebar-collapsed` on `<html>`, remembered in localStorage
  (`fmess.sidebarCollapsed`) for the next start. fMess only keeps the flag; the theme does
  the folding. Teams' sidebar button hides the **app bar**, not the chat list: folded, the
  Teams theme drops the rail drawn in the list's padding (`padding-left: 0`, the
  pseudo-elements `display: none`, `--fmess-buttons: none` so nothing there lights or
  clicks, the unread badge hidden) and the list starts at the window's edge.
- fMess reads the list at most once a second.
- Draw the pictures in fixed pseudo-elements with `pointer-events: none`: `body::before` and
  `body::after` (the Teams theme uses both), `html::before` is the strip/band.

## A search box of the theme's own: `--fmess-search-box: 1`

Moving Messenger's own search field out of the list (`position: fixed` in a title band)
looked right in WebKit snapshots and passed hit tests, but **on screen WebKit clipped it**
(the list's panel is transformed so it can slide). Don't move it. Ask fMess for a box
instead: `--fmess-search-box: 1` on `:root` makes fMess add `#fmess-search-box` to
`<body>`, a `role=button` div that carries the field's (translated) placeholder and
focuses Messenger's field on click. Style the box where the target has its search; fold
Messenger's field until it has the focus (Teams puts its row over the title row then).

## Quick reactions in a message's hover actions: `--fmess-quick-reactions`

`--fmess-quick-reactions: 1f44d 2764 1f606 1f62e` on `:root` (codepoints, `_` between the
parts of a sequence). fMess adds a `.fmess-quick-reactions` group (`role=group`) first in
the hovered message's actions, with one `.fmess-quick-reaction` button per emoji
(`data-emoji`, `aria-label` = the emoji), each holding Messenger's own emoji picture (an
`<img>` from `emoji.php`, so a theme's emoji swap applies), or the emoji as text if fMess
hasn't seen that picture yet: Messenger's emoji URLs carry a checksum (`/v9/t55/2/32/…`),
so fMess knows the six of the reaction picker and learns others from the page. A click
opens Messenger's reaction picker for that message and picks the emoji there (which also
takes it back if it is already yours); an emoji the picker lacks just closes it. Style the
group inside the actions (`order`, sizes); hide it to opt out per place.

After a quick-reaction click the message row (`[data-scope="messages_table"]`) carries
`data-fmess-reacted` until the pointer leaves it: hide the actions box for such rows, as
Messenger's own actions close once a reaction is picked. Hide it with `visibility`, not
`display: none`: Messenger's picker is anchored to the react button while fMess picks.

## A message's hover actions

No marks; found by structure and icons (2026-11-02):

| What | Selector |
|---|---|
| The actions' box (present only while hovered) | `[role="toolbar"] > [role="presentation"] > [role="presentation"]:has([role="button"])` |
| React, reply, more | `[role="button"]:has(path[d^="M111 203a9.5"])`, `…"M101 936.5c"`, `…"M456 410a2"` (each in `div > span > div`, 28px) |
| The actions' column | `[data-scope="messages_table"] > [role="presentation"] > [role="presentation"]:has(> [role="presentation"] > [role="toolbar"])`; `[role="toolbar"]` itself is `position: relative` |
| Your message vs theirs | the row content (`[data-scope="messages_table"] > [role="presentation"]`) is `row-reverse` for yours; its first column is the read-receipt one, `> [role="presentation"]:first-child > [role="presentation"] > [role="none"]`; theirs holds the picture column. Flag it and use `@container style()` |
| The time shown on hover | a normal `[role="tooltip"]` |

To hang the box above the message (Teams): make the toolbar and its column `static`, the
box `absolute` from the row (`top: -40px`; theirs `left: 50px`, yours `right: 8px`),
`width: max-content`, `flex-direction: row` (yours are reversed), and `flex: none` on the
items. Rules inside `@container` need the same specificity as the rules they override.

## Replying and link previews: `composer-bar`

`[data-fmess-ui="composer-bar"]` (marked only while replying or showing a link's preview)
is the composer's previous sibling: `> div` (flex, the content) holds the text block
(`h3` "Replying to <name>", then the quoted text, 13px) and Messenger's 12px close button
(`[role="button"]`, the X path starts `M9.942 2.942`). To make bar and box one box (Teams'
quote inside the box): the bar gets the box's side margins, border and top corners
(`border-bottom: 0`), and `[data-fmess-ui="composer-bar"] + [data-fmess-ui="composer"]`
loses its top border and corners. Its content can become the quote card; mind
UNDER_EVERY_THEME's `padding-bottom: 8px !important` on `> div`.

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
- **One masked pseudo-element, one icon filled under the pointer:** `-webkit-mask-composite`
  across many layers didn't give "outline icons ∪ (strip of filled icons ∩ hovered slot)"
  in WebKit. What works: the strip as the last mask layer with `-webkit-mask-clip:
  content-box`, and padding that makes the content box the hovered slot (`padding-top:
  max(0px, calc(var(--fmess-hover-top, -300px) + 10px))`, the bottom likewise against a
  fixed height); the other layers keep `border-box` (Teams' app bar).
- **`zoom`** shrinks Messenger's nested picture boxes (avatar + ring + presence dot) in one
  rule. `getBoundingClientRect()` of zoomed children reports odd numbers; trust the snapshot.
- **Hit areas.** Never hide a button's box to restyle it. Hide its `svg *` (or the svg's
  `visibility`) and paint on the button or the svg, so the click target and the tooltip stay.
