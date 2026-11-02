//! A picture chosen in Settings is fMess's icon. Otherwise, with the Zalo,
//! WhatsApp or Teams theme, it becomes that app's logo with fMess's own as a
//! badge (branding/icon.py --themes), unless turned off in Settings; otherwise
//! it is fMess's.
//! Only while fMess runs, in the Dock (macOS), on its windows and taskbar
//! button (Windows, Linux) and in the tray; Finder, Start and the app
//! launchers keep the icon it was installed with.

#[cfg(target_os = "macos")]
use std::borrow::Cow;
use std::{fs, path::PathBuf};

use tauri::{image::Image, AppHandle, Manager, Runtime, WebviewWindow};
use tauri_plugin_dialog::DialogExt;

use crate::{
    i18n::{self, pick, Lang},
    settings,
};

/// Larger than any icon needs to be. The picture also goes into the script of
/// every page load, for the loading screen, so it must stay small.
const MAX_CUSTOM_SIZE: usize = 1024 * 1024;

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

/// The chosen picture, as a copy kept beside the settings.
fn custom_path<R: Runtime>(app: &AppHandle<R>) -> Option<PathBuf> {
    Some(app.path().app_config_dir().ok()?.join("app-icon"))
}

/// The chosen picture, while one is chosen and can still be read.
pub fn custom<R: Runtime>(app: &AppHandle<R>) -> Option<Vec<u8>> {
    settings::current(app).custom_icon.then(|| fs::read(custom_path(app)?).ok()).flatten()
}

fn small<R: Runtime>(app: &AppHandle<R>) -> Option<Image<'static>> {
    custom(app)
        .and_then(|bytes| Image::from_bytes(&bytes).ok())
        .or_else(|| Image::from_bytes(icon(theme(app).as_deref()).0).ok())
}

/// The Dock's icon; none is the installed one.
#[cfg(target_os = "macos")]
fn large<R: Runtime>(app: &AppHandle<R>) -> Option<Cow<'static, [u8]>> {
    custom(app)
        .map(Cow::Owned)
        .or_else(|| icon(theme(app).as_deref()).1.map(Cow::Borrowed))
}

/// Asks for a .png or .ico picture, checks it and keeps a copy as the app's
/// icon. Returns false when the user cancelled.
#[tauri::command]
pub async fn choose_app_icon<R: Runtime>(app: AppHandle<R>) -> Result<bool, String> {
    let lang = i18n::current(&app);
    let picker = app.clone();
    let picked = tauri::async_runtime::spawn_blocking(move || {
        picker
            .dialog()
            .file()
            .set_title(pick(lang, "Choose an app icon", "Chọn biểu tượng ứng dụng"))
            .add_filter(pick(lang, "Picture", "Hình"), &["png", "ico"])
            .blocking_pick_file()
    })
    .await
    .map_err(|err| err.to_string())?;
    let Some(picked) = picked else {
        return Ok(false);
    };
    let path = picked.into_path().map_err(|err| err.to_string())?;
    let bytes = fs::read(&path).map_err(|err| match lang {
        Lang::En => format!("could not read {}: {err}", path.display()),
        Lang::Vi => format!("Không đọc được {}: {err}", path.display()),
    })?;
    if bytes.len() > MAX_CUSTOM_SIZE {
        return Err(pick(lang, "An icon can be at most 1 MB.", "Ảnh làm biểu tượng chỉ được nặng tối đa 1 MB.").into());
    }
    Image::from_bytes(&bytes).map_err(|_| {
        pick(lang, "This file isn't a .png or .ico picture fMess can use.", "fMess không dùng được file này làm biểu tượng (.png hoặc .ico).")
    })?;
    let target = custom_path(&app).ok_or(pick(lang, "no config directory", "Không tìm thấy thư mục cài đặt"))?;
    target
        .parent()
        .map_or(Ok(()), fs::create_dir_all)
        .and_then(|()| fs::write(&target, bytes))
        .map_err(|err| match lang {
            Lang::En => format!("could not save the icon to {}: {err}", target.display()),
            Lang::Vi => format!("Không lưu được biểu tượng vào {}: {err}", target.display()),
        })?;
    settings::set_custom_icon(&app, true)?;
    apply(&app);
    Ok(true)
}

/// Goes back to fMess's icon, or the theme's.
#[tauri::command]
pub fn reset_app_icon<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    settings::set_custom_icon(&app, false)?;
    if let Some(path) = custom_path(&app) {
        let _ = fs::remove_file(path);
    }
    apply(&app);
    Ok(())
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
        let picture = large(app);
        let _ = app.run_on_main_thread(move || set_dock_icon(picture));
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
fn set_dock_icon(picture: Option<Cow<'static, [u8]>>) {
    use objc2::{AllocAnyThread, MainThreadMarker};
    use objc2_app_kit::{NSApplication, NSImage};
    use objc2_foundation::NSData;

    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    let image = picture.and_then(|bytes| NSImage::initWithData(NSImage::alloc(), &NSData::with_bytes(&bytes)));
    #[cfg(debug_assertions)]
    eprintln!("[fmess] dock icon: {:?}", image.as_ref().map(|image| image.size()));
    // The Dock still draws the unread badge over it.
    unsafe { NSApplication::sharedApplication(mtm).setApplicationIconImage(image.as_deref()) };
}
