<p align="center">
  <img src="site/assets/icon-256.png" width="128" alt="fMess">
</p>

<h1 align="center">fMess</h1>

<p align="center">
  <b>Messenger trở lại trên máy tính.</b><br>
  Ứng dụng native gọn nhẹ cho macOS, Windows và Linux.
</p>

<p align="center">
  <a href="https://fmess.app/?lang=vi">Website</a> ·
  <a href="https://github.com/toanbku/fmess/releases/latest">Tải về</a> ·
  <a href="README.md">English</a>
</p>

<p align="center">
  <a href="https://github.com/toanbku/fmess/releases/latest"><img src="https://img.shields.io/github/v/release/toanbku/fmess?label=release" alt="Bản mới nhất"></a>
  <a href="https://github.com/toanbku/fmess/actions/workflows/ci.yml"><img src="https://github.com/toanbku/fmess/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="LICENSE"><img src="https://img.shields.io/github/license/toanbku/fmess" alt="Giấy phép MIT"></a>
</p>

<p align="center"><img src="site/assets/og.png" width="720" alt="fMess"></p>

Meta đã chính thức khai tử ứng dụng Messenger cho máy tính vào tháng 12/2025 và
Messenger.com vào tháng 4/2026; toàn bộ đoạn chat trên web được chuyển về
facebook.com/messages. fMess đưa đoạn chat trở lại một cửa sổ riêng, mang lại
trải nghiệm như ứng dụng gốc.

## Tính năng

- **Thông báo native.** Hiển thị ảnh đại diện người gửi, tên nhóm và trả lời
  nhanh ngay trên banner (macOS). Đọc đoạn chat là thông báo tự biến mất.
- **Gọi thoại và video**, hỗ trợ cả cuộc trò chuyện mã hoá đầu cuối trên macOS.
- **Giao diện tập trung.** Lược bỏ News Feed, Reels và thanh công cụ; nút cửa sổ
  nổi trên trang như bản gốc.
- **Ẩn trạng thái đang nhập.** Đối phương không thấy "đang nhập…" khi bạn soạn tin.
- **Cài đặt Messenger tập trung.** Âm thanh, Không làm phiền, Dark Mode và trạng
  thái hoạt động nằm cạnh cài đặt của fMess, đồng bộ với điện thoại.
- **Tối ưu bàn phím.** Phím tắt bên dưới; mở đoạn chat là con trỏ nằm sẵn trong ô soạn tin.
- **Zoom ảnh mượt mà.** Chụm hai ngón hoặc nhấp đúp trong trình xem ảnh.
- **Luôn chạy nền.** Badge tin chưa đọc trên Dock hoặc taskbar, biểu tượng trên
  thanh hệ thống, đóng cửa sổ vẫn nhận tin.
- **Link sạch.** Link trong chat mở bằng trình duyệt, đã gỡ trang chuyển hướng và
  mã theo dõi của Facebook.
- **Tự cập nhật.** fMess kiểm tra phiên bản mới trên GitHub và cài khi bạn đồng ý.
- **Nhẹ.** Chỉ vài MB nhờ dùng webview có sẵn của hệ điều hành thay vì kèm Chromium.

## Cài đặt

Tải bản phù hợp với máy bạn ở
[bản phát hành mới nhất](https://github.com/toanbku/fmess/releases/latest).

| Hệ điều hành | Tải về |
|---|---|
| macOS 12+, Apple silicon (M1 trở lên) | [fMess-mac-apple-silicon.dmg](https://github.com/toanbku/fmess/releases/latest/download/fMess-mac-apple-silicon.dmg) |
| macOS 12+, Intel | [fMess-mac-intel.dmg](https://github.com/toanbku/fmess/releases/latest/download/fMess-mac-intel.dmg) |
| Windows 10 và 11 | [fMess-windows-setup.exe](https://github.com/toanbku/fmess/releases/latest/download/fMess-windows-setup.exe) |
| Linux | [AppImage](https://github.com/toanbku/fmess/releases/latest/download/fMess-linux.AppImage) · [.deb](https://github.com/toanbku/fmess/releases/latest/download/fMess-linux.deb) · [.rpm](https://github.com/toanbku/fmess/releases/latest/download/fMess-linux.rpm) |

Trên Mac cũng cài được bằng Homebrew: `brew install --cask toanbku/tap/fmess`.

Ứng dụng cho Mac đã được ký và Apple notarize, nên mở như mọi app bình thường.

Nếu Windows báo "Windows protected your PC", bấm **More info** rồi **Run anyway**:
fMess chưa được ký số cho Windows nên SmartScreen chưa nhận diện được.

Nếu fMess báo không tìm thấy WebView2 Runtime, bạn cài lại từ [trang WebView2 của Microsoft](https://developer.microsoft.com/microsoft-edge/webview2) (bản Evergreen): Windows 10 và 11 có sẵn, nhưng vài bản Windows rút gọn đã gỡ mất.

## Phím tắt

Trên Windows và Linux, thay ⌘ bằng Ctrl.

| Phím tắt | Chức năng |
|---|---|
| `⌘K` | Tìm kiếm đoạn chat |
| `⌘N` | Tin nhắn mới |
| `⌘1` … `⌘9` | Mở đoạn chat thứ 1 … 9 |
| `⌘⇧]` / `⌘⇧[`, `⌃Tab` / `⌃⇧Tab` | Đoạn chat kế tiếp / trước đó |
| `⌘⇧H` | Về danh sách chat |
| `⌘+` / `⌘−` / `⌘0` | Phóng to / thu nhỏ / mặc định |
| `⌘,` | Cài đặt |

## So với app khác

Kiểm tra ngày 1/11/2026: trang GitHub của từng dự án, bản tải về mới nhất cho Mac (Apple silicon) và kết quả kiểm tra chữ ký của chính macOS.

| | fMess | [Messenger Next](https://github.com/rozsazoltan/messenger) | [Caprine](https://github.com/sindresorhus/caprine) | [Ferdium](https://github.com/ferdium/ferdium-app) | Mở trong trình duyệt |
|---|---|---|---|---|---|
| Làm cho | Messenger | Messenger | Messenger | Nhiều dịch vụ trong một app | Cả Facebook, có luôn feed |
| Mã nguồn | Mở (MIT) | Đóng | Mở (MIT) | Mở (Apache 2.0) | – |
| Dung lượng tải về (Mac) | 4.3 MB | 9.6 MB | 98 MB | 160 MB | – |
| Nền tảng | Webview của hệ thống (Tauri) | Webview của hệ thống (Tauri) | Electron | Electron | Trình duyệt |
| Ký số trên Mac | Có, Apple đã xác thực | Không | Có | Có, Apple đã xác thực | – |
| Bản mới nhất | Tháng 11/2026 | Tháng 9/2026 | Tháng 1/2026, trước khi messenger.com đóng cửa | Tháng 9/2026 | – |

Vì sao mã nguồn quan trọng: app Messenger nào cũng giữ phiên đăng nhập Facebook của bạn và thấy mọi tin nhắn. Mã nguồn của fMess công khai, ai cũng kiểm tra được app làm gì với chúng và tự build lấy; với app đóng mã nguồn, bạn chỉ có thể tin lời nhà phát triển.

## Quyền riêng tư

fMess hiển thị facebook.com/messages bằng webview có sẵn của hệ điều hành. Bạn
đăng nhập trực tiếp trên trang của Facebook; fMess không thấy mật khẩu, không có
máy chủ riêng và không gửi dữ liệu phân tích. Trang web chỉ được nhờ fMess làm vài
việc như hiện thông báo (`src-tauri/capabilities/messenger-web.json`), không đọc
được file hay chạy chương trình trên máy bạn.

## Build từ mã nguồn

Cần Rust (stable), Node.js và các
[yêu cầu của Tauri](https://v2.tauri.app/start/prerequisites/) cho hệ điều hành của bạn.

```sh
npm install
npm run tauri dev       # chạy kèm log
npm run tauri build     # .app/.dmg, .msi/.exe, .deb/.rpm/.AppImage
```

Chạy test bằng `cargo test` trong `src-tauri/`.

Ảnh người gửi và trả lời nhanh trên thông báo macOS cần bản build đã ký với
entitlement Communication Notifications: đặt provisioning profile vào
`src-tauri/signing/` và build với `--config src-tauri/tauri.signed.conf.json`.
Các bản build khác hiển thị thông báo thường.

## Phát hành

1. Tăng phiên bản trong `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml` và `package.json`.
2. Push một tag như `v0.2.0`. [Workflow Release](.github/workflows/release.yml) build
   mọi nền tảng vào một bản nháp, kèm file cập nhật đã ký và `latest.json`.
3. Publish bản nháp. Các máy đã cài sẽ được mời cập nhật trong vài giờ, hoặc ngay
   lập tức qua **Check for Updates…**.

Website trong `site/` được [GitHub Pages](.github/workflows/pages.yml) triển khai lên
[fmess.app](https://fmess.app) mỗi lần push lên `main`.

## Cách hoạt động

Cửa sổ tải trực tiếp facebook.com/messages. fMess chèn một script vào trang
(`src-tauri/src/inject/`, ghép thành một script lúc build) để chỉnh giao diện qua chính các biến CSS của Facebook,
thêm phím tắt và cài đặt, rồi chuyển thông báo sang hệ điều hành. Cuộc gọi trong
chat mã hoá dùng đường WebKit có sẵn của Facebook, vốn đang bị Facebook khoá bằng
cờ; fMess bật cờ đó lên (Cài đặt → Calls).

## Hạn chế

- Facebook thay đổi trang web liên tục; một bản cập nhật phía Facebook có thể làm
  hỏng tính năng cho tới khi fMess kịp vá.
- Trên Linux, cuộc gọi phụ thuộc vào hỗ trợ WebRTC của WebKitGTK.
- Trên Windows và Linux, webview có thể giảm tốc khi cửa sổ bị ẩn.

## Ghi công

Made with ♥ by [Toan Ho](https://github.com/toanbku).

fMess không liên kết với, không được bảo trợ hay tài trợ bởi Meta Platforms, Inc.
Messenger và Facebook là nhãn hiệu của Meta Platforms, Inc.

## Giấy phép

[MIT](LICENSE)
