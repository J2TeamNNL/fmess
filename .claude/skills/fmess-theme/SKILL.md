---
name: fmess-theme
description: >-
  Make an fMess theme that copies another chat app's look as closely as possible
  (layout, light and dark colours, type, official icons, emoji, bubbles, composer,
  tooltips, menus, the avatar/account button) and ship it as a built-in theme with
  its own app icon. Use for "make an fMess theme like Teams/Slack/Telegram/Discord",
  "theme giống <app>", "làm theme <app> cho fMess", "clone <app>'s look", or when
  continuing or fixing the Zalo, WhatsApp or Teams themes. Repo ~/projects/herald,
  private workspace ~/projects/fmess-themes.
---

# fMess theme like <app>

fMess wraps facebook.com/messages. A theme is **CSS only**: `inject.js` puts it in the
page, and the user picks it in Settings. Messenger stays underneath. The goal: someone who
uses `<app>` every day sees `<app>` at a glance, in light and dark, at 880×600.

Zalo and WhatsApp are shipped. Teams is built (`~/projects/fmess-themes/teams/`, NOTES.md
there) and not released yet. The user has explained the
same things for each one; this skill holds them, so run it end to end without asking again.

**Read before writing selectors:**
- [hooks.md](hooks.md): every hook fMess gives themes, Messenger's variables, DOM
  landmarks, the CSS and performance gotchas.
- [template.md](template.md): `whatsapp.src.css` section by section, the build.py and
  icons.json format, the decoration tricks.
- [extraction.md](extraction.md): recon snippets, the five agent briefs, safety and
  measuring rules.
- [testing.md](testing.md): probes, dark and language checks, screenshots, restoring the
  user's app.

## Where things are

| What | Path |
|---|---|
| Theme workspace (private, not in git) | `~/projects/fmess-themes/<app>/`: `spec/`, `icons.json`, `<app>.src.css`, `build.py`, assets |
| The template | `~/projects/fmess-themes/whatsapp/` (and `zalo.css`, built by hand) |
| Built-in themes | `src-tauri/themes/<id>.css`, listed in `BUILT_IN` in `src-tauri/src/themes.rs` |
| Hooks | `src-tauri/src/inject/` (the page script in parts: 09-marks.js, 17-theme-hooks.js, 20-theme.js; `--fmess-buttons`, `#fmess-account`, `applyTheme`) |
| Probe/snapshot | `src-tauri/src/probe.rs`; demo content: `~/projects/fmess-themes/demo-shot.js`, `shoot.sh` |
| App icon per theme | `branding/icon.py --themes`, `branding/brands/<id>.svg`, `src-tauri/icons/themes/<id>-{256,512}.png`, `src-tauri/src/icon.rs` |
| User settings | `~/Library/Application Support/io.github.toanbku.fmess/settings.json` (`theme`, `zoom`, `themeIcon`) |
| Gotcha notes | `~/.claude/projects/-Users-toanho/memory/fmess-*.md`, `herald-messenger-desktop.md` |

## Don't make the user say these again

- Cover **all** of: layout, emoji, text and type, sending files (attach), the typing
  indicator, dark mode, tooltips, the settings/account menu at the avatar, and the app's
  **official** icon set (its own SVG paths, not lookalikes). Each gets a row in the mapping
  table, even when the row says "can't".
- Dark mode is not optional. Every colour is a token with a light and a dark value.
- Marks first, English aria-label only as a fallback. Facebook translates labels.
- Read-only in every real account: the target app and Messenger alike.
- Restore the user's `settings.json` (shoot.sh and the zoom-1 step rewrite its `theme` and
  `zoom`) and reopen `/Applications/fMess.app` when testing ends. The debug and installed
  apps can't run side by side.
- Don't commit, tag or release unless the user asks.

## 1. Recon (you, in Chrome)

1. If `~/projects/fmess-themes/<app>/spec/` exists, read it and continue from there.
2. Load the `anthropic-skills:chrome-browser` skill, then the Chrome tools in one
   ToolSearch call. `tabs_context_mcp`, then `tabs_create_mcp`: work only in your own
   tab. Open the target's web app (Teams `teams.microsoft.com` or `teams.live.com`,
   Slack `app.slack.com`, Telegram `web.telegram.org`, Discord `discord.com/app`). If it
   isn't logged in, stop and ask the user to log in. Never type credentials.
3. `resize_window` until `innerWidth × innerHeight` is 880×600 at 100% zoom. If the window
   is fullscreen it won't resize: constrain the page instead (testing.md, "Reference at
   880×600"). Apps that size panes by container queries (Teams) then lay out as at 880.
4. Run the recon snippet (extraction.md). Note the stable attributes (`data-tid`,
   `data-testid`, `data-icon`, roles), the token system and its version (Fluent
   `--colorNeutralBackground1…`, Material `--md-sys-…`), how dark mode is switched (class or
   attribute on html/body or a provider), and where dark values can come from without
   changing the user's settings.
5. Open a conversation that has no unread badge. Take a reference screenshot
   (`save_to_disk`) into `spec/ref/light.png`, and `dark.png` if dark can be seen without a
   settings change.
6. Write `spec/recon.md`: URL, token system and version, dark mechanism, a list of regions
   with their stable selectors, the reference size. This is the agents' brief.

## 2. Parallel extraction (five agents, one message)

Spawn five `general-purpose` agents in a single message, each with the shared preamble from
extraction.md plus its scope. Each writes one file in `~/projects/fmess-themes/<app>/spec/`:

| Agent | File | Scope |
|---|---|---|
| Tokens and type | `tokens.md` | palette light + dark (and its source), fonts with fallback stacks, sizes per role, radii, shadows |
| Layout | `layout.md` | rail, title bar, list, header, message area, composer, details: geometry at 880×600 |
| Icons | `icons.md` + `../icons.json` | official SVG paths per action, reaction emoji set, official logo + licence |
| Messages | `messages.md` | bubbles, grouping, receipts, reactions, replies, link previews, attachments, emoji, dates, typing indicator, emoji picker |
| Popups and states | `popups.md` | tooltips, menus, the account/settings card at the avatar, hover/focus/selected, scrollbars, toasts |

The preamble carries the rules: read-only; never send, react, call, type in a message box,
change settings, or click a menu item; Escape after opening a popup; open only read
conversations; own tab only; ask before downloading anything; exact computed values, or
`UNMEASURED` with the reason. When they finish, read all five and list the UNMEASURED gaps.
Fill the ones that matter yourself, or leave them as known gaps.

## 3. Mapping table → `spec/mapping.md`

One row per region of the target: **target region → fMess/Messenger hook → technique →
can't**. Start from this (details in hooks.md):

| Target region | Hook | Notes |
|---|---|---|
| Title bar / window strip | `html::before`, height `var(--fmess-title-bar)`; a taller band by raising `--header-height` in the chats | 28px macOS, 0 on Windows/Linux; `pointer-events: none`. Real controls in it: Messenger's search field moved there, `--fmess-window-buttons` (hooks.md) |
| App rail with app icons | `[role="navigation"]` `padding-left`, `::before` (colour, current item, hover), `::after` (masked icons + avatar) | items are pictures; only `account`/`settings` in `--fmess-buttons` act |
| Avatar + account/settings card | `--fmess-avatar`, `account` in `--fmess-buttons` (on the list) or `--fmess-window-buttons` (anywhere), or `#fmess-account` | opens fMess's **native** menu (name, Settings…, Log Out). The target's card can't be copied. |
| Unread count | `html[data-fmess-unread]::after` | `content: attr(data-fmess-unread)` |
| List title / logo | `[role="navigation"] h1 a` masked with a wordmark | |
| New chat, list ⋯ | `a[href="/messages/new/"]`, `[data-fmess-ui="list-menu"]` | svg mask |
| Search | `label:has(input[type="search"])`, `::placeholder` | written placeholder = one language |
| Filters | `[role="tablist"]`, `[role="tab"][aria-selected]` | can't add tabs Messenger lacks |
| Chat rows | `[role="row"] a[role="link"]`, `:hover`, `[aria-current="page"]` | |
| Chat background / pattern | `[data-mw-file-dropzone]` + its `::before` layer | per-chat picture host transparent |
| Chat header + buttons | `chat-header`, `audio-call`, `video-call`, `info` (`[aria-expanded="true"]`) | extra target buttons only as decoration |
| Bubbles | `--chat-*` vars, `[data-message-id] [role="presentation"]`, `[dir="auto"]` | no group-position mark: no tails, no first/last radii |
| Date dividers | `[data-scope="date_break"] > span > span` | |
| Reactions | `--reaction-pill-*`, `reactions`, `more-emojis`, `img[alt="…"] { content: url() }` | the img swap is global |
| Composer, send files, emoji, like/send | `composer`, `attach`, `more-actions`, `emoji`, `sticker`, `gif`, `voice-clip`, `like`, `send`; flex `order`, `display: contents` | rich-text toolbars, formatting: can't. The default 👍 is marked `send` (hooks.md): guard it |
| Reply / link preview bar | `composer-bar`, `--chat-replied-message-background-color` | UNDER_EVERY_THEME makes it grow |
| Typing indicator | **no mark**, can't be triggered without typing | colours via Messenger vars |
| Read receipt, reaction pills, group sender names | structural selectors (hooks.md, "No mark, but a structure") | the img swap / `@container style()` flag |
| Details pane | `details` + `::before` title; inner rules with `@container style(--flag: 1)` | |
| Tooltips, menus, popovers | `[role="tooltip"]`, `[role="menu"]`, `--popover-background` + the two island blocks | |
| Photo viewer | `html[data-fmess-media]` | hide decorations |
| Calls | none: call windows get no theme | |
| Scroll bars | `--chat-scrollbar-thumb-color` | fMess owns the scroll bars |
| Type | font stack on `html :is(div, span, …)`, `@font-face` from a data URL | only openly licensed fonts bundled |

**Decorations** that Messenger lacks (rails, extra tabs, title bars, badges) are drawn on
`html`/`[role="navigation"]` pseudo-elements with `pointer-events: none`, and icons go in
as `-webkit-mask` layers. Clickable fakes go through `--fmess-buttons`. Never hide a real
button's box: hide its `svg *` and paint in its place (template.md).

## 4. Build

1. `mkdir ~/projects/fmess-themes/<app>` if needed. Copy `teams/build.py` (or
   `whatsapp/build.py` if you bundle a font) and adapt the file names and placeholders.
   Teams' adds `{{svg:name:#hex}}` (an icon filled in a colour, for backgrounds and `img`
   `content`) and `{{emoji}}` (another agent's `emoji/emoji.css`, if present). Keep
   `icons.json` in the same `viewBox|d|Ed` format.
2. Write `<app>.src.css` in WhatsApp's section order (template.md): opening comment (name,
   five preview colours) → `@font-face` (if allowed) → `--<app>-*` tokens on `:root` and
   `html[data-fmess-dark]` → Messenger variable remap → font stack → strip → rail → unread
   → viewer → list → chat background → header → dates → bubbles → composer → details →
   the two island blocks.
3. Values come from the spec files, not from memory. Where the spec says UNMEASURED, choose
   and leave a `/* UNMEASURED: … */` comment.
4. **Fonts:** bundle only openly licensed ones (Roboto, Apache 2.0, was fine for
   WhatsApp). Proprietary fonts (Segoe UI for Teams, SF, the vendor's custom fonts) are not
   bundled. Use the target's own fallback stack with its font first, so users who have it
   (e.g. Windows) get it.
5. `python3 build.py ~/projects/fmess-themes/<app>/<app>.css`, then `wc -c`: must be under
   2 MB (WhatsApp is 612 KB). No `@import`, and `data:` URLs only.

## 5. Test loop (testing.md has the commands)

1. Back up `settings.json` into your scratchpad before anything else.
2. Debug build once: `cd ~/projects/herald && npx tauri build --debug --bundles app`.
   CSS changes need no rebuild: wrap the built CSS into `apply.js` and run it as a probe
   with `FMESS_WINDOW_SIZE=880x600` and `FMESS_SNAPSHOT`, zoom 1, theme none.
3. Snapshot light and dark (the page-only dark snippet), and compare them side by side with
   `spec/ref/`: picture first to find differences, then the probe numbers against the
   spec. Fix, rebuild CSS, repeat until the regions match.
4. Languages: run the aria-label rewrite probe. `changed` must be empty, and every
   aria-label selector must have a mark beside it.
5. Look at tooltips (light and dark), the ⋯ menu, the reaction picker, the photo viewer, the
   call buttons (look only), the reply bar (open, measure, cancel in the same probe), a group
   chat, a link preview, the details pane, and the `--fmess-title-bar: 0` case.
6. Performance: no `:has()` left of a combinator and none nested, full-chat pictures on a
   `will-change` layer, scroll fps the same as with no theme.
7. Restore: kill the debug app, copy the backup over `settings.json`,
   `open -a /Applications/fMess.app`, close your Chrome tabs.

## 6. Ship (in ~/projects/herald)

1. `python3 ~/projects/fmess-themes/<app>/build.py src-tauri/themes/<app>.css`.
2. `src-tauri/src/themes.rs`: add `("<app>", include_str!("../themes/<app>.css"))` to
   `BUILT_IN` (card order = array order), and update the module comment ("Zalo and WhatsApp
   come with fMess"). Run `cd src-tauri && cargo test themes`.
3. App icon:
   - Put the official logo in `branding/brands/<app>.svg`.
   - In `branding/icon.py`, add `<app>` to the `--themes` tuple `("zalo", "whatsapp")`.
     Give `brand()` a branch for how this logo sits in the 1024 square: Zalo's logo is the
     whole app icon; the default branch is WhatsApp-specific (bubble on a green squircle).
     Add its source and licence to the docstring.
   - Run `python3 branding/icon.py --themes $S/theme-icons`. Render the 256 and 512 SVGs
     with headless Chrome as the docstring says (`--headless=new
     --force-device-scale-factor=1 --default-background-color=00000000 --window-size=N,N
     --screenshot=…`, with its own `--user-data-dir`), into
     `src-tauri/icons/themes/<app>-256.png` and `-512.png`.
   - Add a match arm to `icon()` in `src-tauri/src/icon.rs`, and update its doc comment.
   - Update the hint in `ui/settings.html` ("With Zalo or WhatsApp, the icon shows…").
4. Debug build, choose the theme in Settings, check the Dock icon and the "App icon follows
   the theme" switch, then restore.
5. Docs and site: `grep -rn -i whatsapp README.md README.vi.md site/ ui/` and add the new
   theme everywhere it's listed:
   - `site/index.html`: carousel `<img data-shot>`, the theme buttons, `shot_title`,
     `shot_note`, `cmp_also`, the image list near the top.
   - `site/assets/site.js`: the VI strings for the same keys, plus `shot_<app>`.
   - `site/llms.txt`: the Themes line.
   - Screenshots `site/assets/shot-<app>{,-vi}{,-dark}.webp` (testing.md), and bump `?v=`.
   - As of 0.1.20 the READMEs don't list themes. Add a line only if a themes list exists.
6. Show the user the diff and screenshots. Commit only when asked: plain English subject
   ("Teams theme", like the earlier "WhatsApp theme; …"), Co-Authored-By trailer.
7. Release only when asked, using the memory note `fmess-release-and-linux-testing`: bump
   `package.json`, `package-lock.json`, `src-tauri/Cargo.toml`, `Cargo.lock`,
   `tauri.conf.json` ("Release 0.1.x"), tag `v0.1.x`, then release.yml makes one draft and
   the `check` job checks latest.json. Test locally first: the Mac dmg (`codesign --verify`,
   run 20 s), and the `.deb` in the OrbStack machine `fmess-linux`. Then publish via
   `gh api -X PATCH …/releases/<id> -F draft=false -f make_latest=true`. A bad pushed tag
   can't be deleted: bump the version.

## 7. Licensing

- Accepted so far, with the user's consent: the target's official icons and emoji inside
  a theme (Zalo's from its app's font and assets, WhatsApp's from its web app). The Zalo
  workspace README says "do not publish", and the user later chose to ship it, accepting
  the DMCA risk. Ask again for a new app if it is a different kind of risk.
- Brand logos for the app icon: Zalo's and WhatsApp's are public domain on Wikimedia
  Commons (icon.py docstring). Prefer such a source, and write the licence down.
- Fluent UI System Icons and Fluent Emoji are MIT: fine for Teams. Fonts under OFL or
  Apache are fine.
- Never bundle proprietary fonts (Segoe UI, SF Pro, vendor fonts). Use the fallback stack.
- Use no vendor screenshots or marketing pictures on the site: only fMess's own shots with
  demo content.

## Done when

- [ ] spec/ has tokens, layout, icons, messages, popups, mapping, with gaps marked
- [ ] Light and dark match the references region by region at 880×600
- [ ] No themed style depends on an English label; the rewrite probe shows no change
- [ ] Tooltips are readable in both modes; viewer, calls, reply bar and details are fine
- [ ] Under 2 MB, `cargo test themes` passes, no broad `:has()`
- [ ] Icon, Settings hint, site, llms.txt updated (if shipping)
- [ ] The user's settings (and `.window-state.json`) are restored, the installed app is
      reopened, Chrome tabs closed, the target app's own settings put back as found
