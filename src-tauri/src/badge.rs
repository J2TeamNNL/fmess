use std::{collections::HashMap, sync::Mutex};

use tauri::{AppHandle, Emitter, Manager, Runtime, Webview, Window};

use crate::{
    services::{self, Service},
    settings,
};

/// The unread chat counts the services last reported.
static UNREAD: Mutex<Option<HashMap<Service, i64>>> = Mutex::new(None);

/// Called by each service's page with the number of unread chats in its
/// list. (Messenger's page title counts Facebook's own notifications, so it
/// can't be used.)
#[tauri::command]
pub fn set_unread_count<R: Runtime>(webview: Webview<R>, count: i64) {
    let Some(service) = Service::from_label(webview.label()) else {
        return;
    };
    // A page closing after Instagram was turned off may still report.
    if service == Service::Instagram && !settings::current(webview.app_handle()).instagram {
        return;
    }
    let count = count.max(0);
    #[cfg(debug_assertions)]
    eprintln!("[fmess] unread chats in {}: {count}", service.label());
    UNREAD.lock().unwrap().get_or_insert_with(HashMap::new).insert(service, count);
    changed(webview.app_handle(), service, count);
}

/// A service that was turned off has no unread chats.
pub fn forget<R: Runtime>(app: &AppHandle<R>, service: Service) {
    if let Some(unread) = UNREAD.lock().unwrap().as_mut() {
        unread.remove(&service);
    }
    changed(app, service, 0);
}

fn changed<R: Runtime>(app: &AppHandle<R>, service: Service, count: i64) {
    let _ = app.emit_to(services::RAIL, "unread", serde_json::json!({ "service": service, "count": count }));
    if let Some(window) = services::main_window(app) {
        refresh(&window);
    }
}

pub fn counts() -> HashMap<Service, i64> {
    UNREAD.lock().unwrap().clone().unwrap_or_default()
}

/// Shows the services' total, or nothing when the badge is turned off.
pub fn refresh<R: Runtime>(window: &Window<R>) {
    let count = if settings::current(window.app_handle()).badge {
        counts().values().sum()
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
