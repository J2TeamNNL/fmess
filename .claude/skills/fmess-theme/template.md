# The WhatsApp theme as a template

`~/projects/fmess-themes/whatsapp/` is the model for every new theme. The newer
`~/projects/fmess-themes/teams/` follows the same order and adds what WhatsApp lacks:
`html[data-fmess-chats]` scoping, a card around `[role="main"]`, a 68px rail with only
`settings`/`account` as `--fmess-buttons`, row/avatar resizing, Messenger's menus and
tooltips restyled, reaction pills and the seen receipt, the like-button guard, and
`@container style()` flags instead of `:has()` left of a combinator. Read its NOTES.md.

WhatsApp's files:

| File | What |
|---|---|
| `whatsapp.src.css` | the theme, with `{{…}}` placeholders (~500 lines, 20 KB) |
| `icons.json` | WhatsApp web's own SVG icons by their own names |
| `doodle-min.svg` | the chat background pattern, drawn in `currentColor` |
| `roboto.woff2` | the font (Roboto, Apache 2.0, so it may be bundled) |
| `build.py` | puts it all in as `data:` URLs |

```sh
python3 ~/projects/fmess-themes/whatsapp/build.py ~/projects/herald/src-tauri/themes/whatsapp.css
# prints the path and the size; today 612,562 bytes, the same file as the one shipped
```

## build.py and icons.json

- `icons.json`: `{ "name": "viewBox|path|path…" }`. Each path is the `d` of one `<path>`.
  A leading `E` means `fill-rule="evenodd"`. Paths are filled only: stroke icons must be
  converted to outlines first, or build.py must learn to write strokes.
- Placeholders:
  - `{{icon:name}}` becomes the whole `url("data:image/svg+xml,…")`. Write it bare:
    `-webkit-mask: {{icon:ic-call}} center / 24px 24px no-repeat;`
  - `{{doodle:#hex}}` becomes a bare data URL with `currentColor` replaced. Write it
    inside `url("…")`.
  - `{{font}}` becomes a bare `data:font/woff2;base64,…`. Write it inside `url("…")`.
- For a new app, copy build.py, rename the files it reads, and drop the placeholders you
  don't need. Keep the output byte-identical between runs (no timestamps), so a rebuild of
  an unchanged theme changes nothing in git.
- Budget: 2 MB hard limit (themes.rs). URL-encoded SVG and base64 fonts grow about 1.3×.
  Check with `wc -c`.

## whatsapp.src.css, section by section

Sections are named by their comment headers. Write a new theme in the same order.

1. **Opening comment.** `name`, `preview` (rail, list, chat, your bubble, theirs). No `mode:`.
2. **`@font-face "fMess Roboto"`** from `{{font}}`, `font-weight: 100 900`. Give the font a
   name of its own, so a locally installed one doesn't clash.
3. **"WhatsApp's colours, light and, while Messenger is dark, dark."** The theme's own tokens
   `--wa-*` on `:root`, the same names again under `html[data-fmess-dark]`. Every rule below
   uses only these tokens, so dark mode costs nothing more. Name them after the target's
   roles (rail, panel, chat, bubble-in/out, input, hover, selected, text, text-2, icon,
   brand…). For a new app use its own prefix (`--teams-*`).
4. **"Messenger's own colours, in WhatsApp's."** All of Messenger's variables (hooks.md) on
   the four selectors, each pointing at a token, `!important`.
5. **Font stack** on `html, html body, html :is(div, span, a, h1, h2, h3, h4, input, textarea, button, label, p)`
   with emoji fonts at the end (`"Apple Color Emoji", "Segoe UI Emoji"`).
6. **"The strip under the window buttons"** → `html::before`, `position: fixed`, very high
   z-index, `pointer-events: none`. A gradient continues the rail colour for 64px, then the
   panel colour. It uses `--header-height`; use `--fmess-title-bar` in new themes (hooks.md).
7. **"The bar down the left"** → the app rail, faked on `[role="navigation"]`:
   - `padding-left: 64px` makes room, and `--fmess-buttons` lists chats, calls, status,
     channels, communities, media and **account** (only account works).
   - `::before` (z-index 3) draws, bottom layer first: the rail colour, a static round
     highlight behind the current item (Chats), and the hover highlight placed by
     `--fmess-hover-left/-top`.
   - `::after` (z-index 4, `pointer-events: none`) is a **single masked layer**. Its
     background is the avatar (`var(--fmess-avatar)`, placed at the bottom) over the icon
     colour. Its `-webkit-mask` stacks every icon at its position, plus a circle where the
     avatar sits. Icons and a round avatar in one pseudo-element, recoloured by one token.
8. **"Unread chats on the Chats button"** → `html[data-fmess-unread]::after`, fixed, with
   `content: attr(data-fmess-unread)`, top = strip height + 8px.
9. **Viewer**: `html[data-fmess-media]::before, ::after { display: none }`.
10. **"Chat list"**:
    - `#fmess-account` hidden (the rail has the avatar).
    - The "Chats" `h1 a` becomes the wordmark: fixed size, `color: transparent`, children
      `visibility: hidden`, background = brand colour, mask = `{{icon:wa-wordmark}}`.
    - New chat: the button's `--secondary-button-background` is overridden locally, and the
      svg is masked with `ic-add`.
    - Search: `label:has(input[type=search])` becomes a pill. The placeholder is made
      transparent and redrawn by `::after` ("Search or start a new chat", English only;
      demo-shot.js patches it for VI shots).
    - `list-menu` ⋯ masked with `ic-more-vert`.
    - Tabs as chips: the `tablist` nowraps, non-tab children are hidden
      (`> :not(:has([role="tab"]))`), and `[aria-selected="true"]` is filled. Text colour goes
      through `--primary-text` / `--primary-deemphasized-button-text` set on the tab.
    - Rows: radius, `:hover`, `[aria-current="page"]`.
11. **"Chat: … doodles"**: the dropzone gets the chat colour; its `::before` (z-index −1,
    `isolation: isolate` on the dropzone, `will-change: transform`) repeats the doodle. A
    `--wa-doodle` token is swapped in dark mode. The per-chat background host is made
    transparent.
12. **"Header"** → `[data-fmess-ui="chat-header"]`: background, soft shadow, no border. The
    name is `h3 span`. Call, video and info icons are masked on their `svg`, with `svg *`
    hidden.
13. **"Dates"** → `[data-scope="date_break"] > span > span` as chips.
14. **"Bubbles"**: `[data-message-id] [role="presentation"]` radius 7.5px,
    `[dir="auto"]` 14.2px.
15. **"Composer"** (a floating pill):
    - The composer gets margin, radius, background and shadow; its parent goes transparent.
    - The tool group `> div:has(+ div [role="textbox"])` gets `display: contents`. Its
      buttons become direct flex items, so `order` can put + and the smiley before the box
      and the mic after it.
    - GIF hidden. The like span is hidden while it holds `like`. When there is text the
      mark turns into `send`, the span matches no more, and the round green send button
      appears.
    - Each icon is the svg-mask pattern below.
16. **"Details"** → `[data-fmess-ui="details"]` background, and `::before` writes the title
    "Contact info" (fixed text).
17. **Tooltips + dark islands**: the two blocks from hooks.md, verbatim.

## Rail and Instagram sections (since the rail became its own web view)

The same file also dresses the rail and Instagram (hooks.md, "One theme, three web
views"). The shipped themes add, after the Messenger sections:

1. **Rail** under `html[data-fmess-service="rail"]`: background on `#fmess-rail`, item
   size and `color` on `[data-fmess-rail]`, the selected look on `[data-active]`,
   `[data-fmess-badge]`, and `[data-fmess-avatar]`. The old picture-only rail on
   `[role="navigation"]` (`padding-left`, `::before/::after`, `--fmess-buttons`) is gone
   from the built-in themes; user themes that still draw it keep working.
2. **Instagram** under `html[data-fmess-service="instagram"]`: `--ig-*` remapped to the
   theme's tokens (on `html` and on `.__fb-dark-mode`/`.__fb-light-mode`), `ig-nav`
   hidden and its 72px taken back (shipped themes pull `list` left with
   `margin-left: -72px`; zeroing `ig-shell` is the cleaner way), then list, rows, header, bubbles, dates,
   composer and details through their marks.
3. Every bare Messenger rule that names a shared mark (`composer`, `emoji`, `attach`…)
   gets `:where(html:not([data-fmess-service="instagram"]))` in front.

These sections were written straight into `src-tauri/themes/*.css`: the theme sources
are not on this machine, so a `build.py` run from an old source would drop them.

## Tricks that work (and keep clicks working)

- **Icon swap, one colour** (preferred: one set serves light and dark):
  ```css
  X svg { width: 24px !important; height: 24px !important; background: var(--app-icon);
          -webkit-mask: {{icon:name}} center / 24px 24px no-repeat; }
  X svg * { visibility: hidden; }
  ```
- **Icon swap, multicolour** (Zalo): `X { background: url(data:…) center / 24px 24px no-repeat !important }`
  with `X svg { visibility: hidden }`. This needs a second set under `html[data-fmess-dark]`.
- **Decorations** (rails, title bars, badges, avatars) go on pseudo-elements of `html` or
  `[role="navigation"]`, with `pointer-events: none` on anything over a clickable area.
  Clickable fakes go through `--fmess-buttons`.
- **Several icons in one pseudo-element**: one background colour plus a stacked
  `-webkit-mask` with one layer per icon at its own position. The avatar is a background
  layer cut round by a `radial-gradient(circle …)` mask layer.
- **Reorder** with flex `order`, and flatten wrappers with `display: contents`.
- **Hide** only what has another way in (WhatsApp hides GIF). Never hide the like/send span
  as a whole without making sure send still shows when there is text.
- **Emoji**: `img[alt="👍"] { content: url(data:…) }` replaces Facebook's emoji image
  (global, see hooks.md). Zalo did this for its six reaction faces, with Like moved first
  via `order: -1` on the menu item.
- **Inner styles of a pane** without `:has()`: put a flag variable on the marked pane and use
  `@container style(--flag: 1) { … }` (Zalo's details and reply quote). Check on Linux too:
  WebKitGTK may lag.
- **Selection colour**: `--text-highlight`.
