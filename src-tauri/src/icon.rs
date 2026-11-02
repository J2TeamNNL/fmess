//! With the Zalo, WhatsApp or Teams theme, fMess's icon becomes that app's
//! logo with fMess's own as a badge (branding/icon.py --themes), unless turned
//! off in Settings; otherwise it is fMess's.
//! Only while fMess runs, in the Dock (macOS), on its windows and taskbar
//! button (Windows, Linux) and in the tray; Finder, Start and the app
//! launchers keep the icon it was installed with.

use tauri::{image::Image, AppHandle, Runtime, WebviewWindow};

use crate::settings;

/// The icon at 256 (windows, tray) and 512 (Dock; none is the installed one).
/// Themes the user adds, and Messenger's own look, get fMess's.
fn icon(theme: Option<&str>) -> (&'static [u8], Option<&'static [u8]>) {
    match theme {
        Some("zalo") => (
            include_bytes!("../icons/themes/zalo-256.png"),
            Some(include_bytes!("../icons/themes/zalo-512.png")),
        ),
        Some("whatsapp") => (
            include_bytes!("../icons/themes/whatsapp-256.png"),
            Some(include_bytes!("../icons/themes/whatsapp-512.png")),
        ),
        Some("teams") => (
            include_bytes!("../icons/themes/teams-256.png"),
            Some(include_bytes!("../icons/themes/teams-512.png")),
        ),
        _ => (include_bytes!("../icons/128x128@2x.png"), None),
    }
}

/// The theme whose icon to show, if the icon follows the theme.
fn theme<R: Runtime>(app: &AppHandle<R>) -> Option<String> {
    let settings = settings::current(app);
    settings.theme.filter(|_| settings.theme_icon)
}

fn small<R: Runtime>(app: &AppHandle<R>) -> Option<Image<'static>> {
    Image::from_bytes(icon(theme(app).as_deref()).0).ok()
}

/// Shows the chosen theme's icon everywhere fMess has one.
pub fn apply<R: Runtime>(app: &AppHandle<R>) {
    if let Some(image) = small(app) {
        #[cfg(not(target_os = "macos"))]
        for window in tauri::Manager::windows(app).values() {
            let _ = window.set_icon(image.clone());
        }
        if let Some(tray) = app.tray_by_id("main") {
            let _ = tray.set_icon(Some(image));
        }
    }
    #[cfg(target_os = "macos")]
    {
        let png = icon(theme(app).as_deref()).1;
        let _ = app.run_on_main_thread(move || set_dock_icon(png));
    }
}

/// For a window opened after the theme was applied (Settings, calls).
pub fn apply_to<R: Runtime>(_window: &WebviewWindow<R>) {
    #[cfg(not(target_os = "macos"))]
    if let Some(image) = small(tauri::Manager::app_handle(_window)) {
        let _ = _window.set_icon(image);
    }
}

#[cfg(target_os = "macos")]
fn set_dock_icon(png: Option<&'static [u8]>) {
    use objc2::{AllocAnyThread, MainThreadMarker};
    use objc2_app_kit::{NSApplication, NSImage};
    use objc2_foundation::NSData;

    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    let image = png.and_then(|png| NSImage::initWithData(NSImage::alloc(), &NSData::with_bytes(png)));
    #[cfg(debug_assertions)]
    eprintln!("[fmess] dock icon: {:?}", image.as_ref().map(|image| image.size()));
    // The Dock still draws the unread badge over it.
    unsafe { NSApplication::sharedApplication(mtm).setApplicationIconImage(image.as_deref()) };
}
