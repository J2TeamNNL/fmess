<p align="center">
  <img src="site/assets/icon-256.png" width="128" alt="fMess">
</p>

<h1 align="center">fMess</h1>

<p align="center">
  <b>Messenger, back on your desktop.</b><br>
  A small, native app for your chats on macOS, Windows and Linux.
</p>

<p align="center">
  <a href="https://fmess.app">Website</a> ·
  <a href="https://github.com/toanbku/fmess/releases/latest">Download</a> ·
  <a href="README.vi.md">Tiếng Việt</a>
</p>

<p align="center">
  <a href="https://github.com/toanbku/fmess/releases/latest"><img src="https://img.shields.io/github/v/release/toanbku/fmess?label=release" alt="Latest release"></a>
  <a href="https://github.com/toanbku/fmess/actions/workflows/ci.yml"><img src="https://github.com/toanbku/fmess/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="LICENSE"><img src="https://img.shields.io/github/license/toanbku/fmess" alt="MIT license"></a>
</p>

<p align="center"><img src="site/assets/og.png" width="720" alt="fMess"></p>

Meta shut down the Messenger desktop apps in December 2025 and messenger.com in
April 2026; chats on the web now live at facebook.com/messages. fMess gives them
a window of their own again and makes it feel like the old app.

## Features

- **Real notifications.** Sender photos, group names and quick reply right in
  the banner (macOS). Reading a chat clears its notifications.
- **Voice and video calls**, end-to-end encrypted chats included on macOS.
- **Just your chats.** No feed, no reels, no top bar; the window buttons float
  over the page like the old app.
- **Block Typing.** Nobody sees "typing…" while you draft.
- **Messenger's settings in one place.** Sounds, Do Not Disturb, dark mode and
  active status next to fMess's own, synced with your phone.
- **Keyboard first.** Shortcuts below; opening a chat puts the cursor in its message box.
- **Zoomable photos.** Pinch or double-click in the photo viewer.
- **Always there.** Unread badge on the Dock or taskbar, a tray icon, keeps running
  when the window is closed.
- **Clean links.** Links in chats open in your browser without Facebook's
  redirect and tracking tags.
- **Updates itself.** fMess checks GitHub for new versions and installs them
  when you say so.
- **Small.** A few MB, because it uses the system webview instead of bundling Chromium.

## Install

Download the build for your system from the
[latest release](https://github.com/toanbku/fmess/releases/latest).

| System | Download |
|---|---|
| macOS 12+, Apple silicon (M1 and later) | [fMess-mac-apple-silicon.dmg](https://github.com/toanbku/fmess/releases/latest/download/fMess-mac-apple-silicon.dmg) |
| macOS 12+, Intel | [fMess-mac-intel.dmg](https://github.com/toanbku/fmess/releases/latest/download/fMess-mac-intel.dmg) |
| Windows 10 and 11 | [fMess-windows-setup.exe](https://github.com/toanbku/fmess/releases/latest/download/fMess-windows-setup.exe) |
| Linux | [AppImage](https://github.com/toanbku/fmess/releases/latest/download/fMess-linux.AppImage) · [.deb](https://github.com/toanbku/fmess/releases/latest/download/fMess-linux.deb) · [.rpm](https://github.com/toanbku/fmess/releases/latest/download/fMess-linux.rpm) |

On a Mac you can also use Homebrew: `brew install --cask toanbku/tap/fmess`.

The Mac app is signed and notarized by Apple, so it opens like any other app.

If Windows says it protected your PC, click **More info**, then **Run anyway**:
fMess isn't code-signed for Windows yet, so SmartScreen doesn't know it.

If fMess says it can't find the WebView2 Runtime, install it from [Microsoft's WebView2 page](https://developer.microsoft.com/microsoft-edge/webview2) (Evergreen): Windows 10 and 11 include it, but some trimmed-down builds remove it.

## Keyboard shortcuts

On Windows and Linux, use Ctrl instead of ⌘.

| Shortcut | Action |
|---|---|
| `⌘K` | Search chats |
| `⌘N` | New message |
| `⌘1` … `⌘9` | Open the 1st … 9th chat |
| `⌘⇧]` / `⌘⇧[`, `⌃Tab` / `⌃⇧Tab` | Next / previous chat |
| `⌘⇧H` | Back to the chat list |
| `⌘+` / `⌘−` / `⌘0` | Zoom in / out / actual size |
| `⌘,` | Settings |

## Compared with other apps

Checked on November 1, 2026: each project's GitHub page, its latest Mac download (Apple silicon) and macOS's own signature check.

| | fMess | [Messenger Next](https://github.com/rozsazoltan/messenger) | [Caprine](https://github.com/sindresorhus/caprine) | [Ferdium](https://github.com/ferdium/ferdium-app) | A browser tab |
|---|---|---|---|---|---|
| Made for | Messenger | Messenger | Messenger | Many services in one app | All of Facebook, feed included |
| Source code | Open (MIT) | Closed | Open (MIT) | Open (Apache 2.0) | – |
| Mac download | 4.3 MB | 9.6 MB | 98 MB | 160 MB | – |
| Built on | System webview (Tauri) | System webview (Tauri) | Electron | Electron | Your browser |
| Mac app signed | Yes, notarized | No | Yes | Yes, notarized | – |
| Latest release | November 2026 | September 2026 | January 2026, before messenger.com closed | September 2026 | – |

Why the source matters: a Messenger app holds your Facebook login and sees every message. fMess's code is public, so anyone can check what it does with them and build it themselves; a closed-source app has to be taken on its developer's word.

## Privacy

fMess shows facebook.com/messages in your system's own webview. You sign in on
Facebook's page; fMess never sees your password, has no server and sends no
analytics. The page can ask fMess for only a handful of things, like showing a
notification (`src-tauri/capabilities/messenger-web.json`); it can't read your
files or run programs.

## Build from source

You need Rust (stable), Node.js and the
[Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for your system.

```sh
npm install
npm run tauri dev       # run with live logs
npm run tauri build     # .app/.dmg, .msi/.exe, .deb/.rpm/.AppImage
```

Run the tests with `cargo test` in `src-tauri/`.

Sender photos and quick reply in macOS notifications need a signed build with
the Communication Notifications entitlement: put the provisioning profile in
`src-tauri/signing/` and build with `--config src-tauri/tauri.signed.conf.json`.
Other builds show plain notifications.

## Releasing

1. Bump the version in `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml` and `package.json`.
2. Push a tag such as `v0.2.0`. The [Release workflow](.github/workflows/release.yml)
   builds every platform into a draft release, with signed update archives and `latest.json`.
3. Publish the draft. Installed copies offer the update within a few hours, or
   right away from **Check for Updates…**.

The website in `site/` is deployed to [fmess.app](https://fmess.app) by
[GitHub Pages](.github/workflows/pages.yml) on every push to `main`.

## How it works

The window loads facebook.com/messages directly. fMess adds one script to the
page (`src-tauri/src/inject/`, joined into one script at build time) that restyles it through Facebook's own CSS
variables, adds the shortcuts and settings, and hands notifications to the
operating system. Calls in encrypted chats use Facebook's own WebKit path,
which Facebook keeps behind a flag; fMess turns it on (Settings → Calls).

## Known limitations

- Facebook changes its web app often; an update on their side can break something
  until fMess catches up.
- On Linux, calls depend on WebKitGTK's WebRTC support.
- On Windows and Linux, the webview may throttle a hidden window.

## Credits

Made with ♥ by [Toan Ho](https://github.com/toanbku).

fMess is not affiliated with, endorsed by or sponsored by Meta Platforms, Inc.
Messenger and Facebook are trademarks of Meta Platforms, Inc.

## License

[MIT](LICENSE)
