//! Windows shows the chats with Microsoft's WebView2. Windows 10 and 11 come
//! with it, but some trimmed-down builds remove it, and fMess's installer only
//! adds it back while it runs. Without it no window can open, so fMess offers
//! Microsoft's installer instead of Tauri's error, whose link can't be clicked.

use windows_sys::Win32::UI::{
    Shell::ShellExecuteW,
    WindowsAndMessaging::{MessageBoxW, IDYES, MB_ICONWARNING, MB_YESNO, SW_SHOWNORMAL},
};

use crate::i18n::{self, pick};

/// Microsoft's WebView2 installer (the Evergreen bootstrapper, a small .exe).
const INSTALLER: &str = "https://go.microsoft.com/fwlink/p/?LinkId=2124703";

/// Whether WebView2 is there; when it isn't, the user was offered its installer.
pub fn check() -> bool {
    if tauri::webview_version().is_ok() {
        return true;
    }
    let wide = |text: &str| text.encode_utf16().chain(Some(0)).collect::<Vec<u16>>();
    // Settings can't be read yet, so this follows the system's language.
    let lang = i18n::system();
    let text = wide(pick(
        lang,
        "fMess shows Messenger with Microsoft WebView2. Windows 10 and 11 come with it, \
         but it is missing on this PC.\n\nDownload Microsoft's installer now? Run it, then open fMess again.",
        "fMess dùng Microsoft WebView2 để hiện Messenger. Windows 10 và 11 có sẵn WebView2, \
         nhưng máy này lại không có.\n\nTải bộ cài của Microsoft về ngay? Chạy bộ cài xong thì mở lại fMess.",
    ));
    let title = wide(pick(lang, "fMess needs WebView2", "fMess cần WebView2"));
    unsafe {
        let answer = MessageBoxW(std::ptr::null_mut(), text.as_ptr(), title.as_ptr(), MB_YESNO | MB_ICONWARNING);
        if answer == IDYES {
            ShellExecuteW(
                std::ptr::null_mut(),
                wide("open").as_ptr(),
                wide(INSTALLER).as_ptr(),
                std::ptr::null(),
                std::ptr::null(),
                SW_SHOWNORMAL,
            );
        }
    }
    false
}
