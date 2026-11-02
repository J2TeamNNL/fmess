use std::sync::atomic::{AtomicI64, Ordering};

use tauri::{Manager, Runtime, Window};

use crate::{services, settings};

/// The unread chat count inject.js last reported.
static UNREAD: AtomicI64 = AtomicI64::new(0);

/// Called by inject.js with the number of unread chats in the list. (The page
/// title counts Facebook's own notifications, so it can't be used.)
#[tauri::command]
pub fn set_unread_count<R: Runtime>(app: tauri::AppHandle<R>, count: i64) {
    #[cfg(debug_assertions)]
    eprintln!("[fmess] unread chats: {count}");
    UNREAD.store(count, Ordering::Relaxed);
    if let Some(window) = services::main_window(&app) {
        refresh(&window);
    }
}

/// Shows the last reported count, or nothing when the badge is turned off.
pub fn refresh<R: Runtime>(window: &Window<R>) {
    let count = if settings::current(window.app_handle()).badge {
        UNREAD.load(Ordering::Relaxed)
    } else {
        0
    };
    show(window, count);
}

#[cfg(not(target_os = "windows"))]
fn show<R: Runtime>(window: &Window<R>, count: i64) {
    let _ = window.set_badge_count((count > 0).then_some(count));
}

/// Windows has no numeric taskbar badge; show a red dot overlay instead.
#[cfg(target_os = "windows")]
fn show<R: Runtime>(window: &Window<R>, count: i64) {
    let _ = window.set_overlay_icon((count > 0).then(red_dot));
}

#[cfg(target_os = "windows")]
fn red_dot() -> tauri::image::Image<'static> {
    const SIZE: u32 = 16;
    let center = (SIZE as f32 - 1.0) / 2.0;
    let mut rgba = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    for y in 0..SIZE {
        for x in 0..SIZE {
            let distance = ((x as f32 - center).powi(2) + (y as f32 - center).powi(2)).sqrt();
            let alpha = ((center + 0.5 - distance).clamp(0.0, 1.0) * 255.0) as u8;
            rgba.extend_from_slice(&[0xE5, 0x39, 0x35, alpha]);
        }
    }
    tauri::image::Image::new_owned(rgba, SIZE, SIZE)
}
