// tauri-plugin-window-state writes its file at exit, but refreshes from the
// live window only for windows that are a single web view: "main" holds
// several, so its maximized and full screen state would be lost. fMess writes
// that window's entry itself, in the plugin's format, after the plugin.
use std::{fs, path::PathBuf, sync::Mutex};

use serde_json::{json, Map, Value};
use tauri::{AppHandle, Manager, Runtime, Window};
use tauri_plugin_window_state::AppHandleExt;

use crate::MAIN_WINDOW;

/// The main window's state when it was last closed into the tray.
#[derive(Default)]
pub struct Closed(Mutex<Option<Map<String, Value>>>);

fn file<R: Runtime>(app: &AppHandle<R>) -> Option<PathBuf> {
    Some(app.path().app_config_dir().ok()?.join(app.filename()))
}

fn read<R: Runtime>(app: &AppHandle<R>) -> Map<String, Value> {
    file(app)
        .and_then(|path| fs::read(path).ok())
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default()
}

/// Mirrors the plugin's update: size and position only while the window is
/// neither maximized nor minimized; visibility is never restored.
fn capture<R: Runtime>(window: &Window<R>, entry: &mut Map<String, Value>) -> tauri::Result<()> {
    let maximized = window.is_maximized()?;
    entry.insert("maximized".into(), json!(maximized));
    entry.insert("fullscreen".into(), json!(window.is_fullscreen()?));
    if !maximized && !window.is_minimized()? {
        let size = window.inner_size()?;
        if size.width > 0 && size.height > 0 {
            entry.insert("width".into(), json!(size.width));
            entry.insert("height".into(), json!(size.height));
        }
        let position = window.outer_position()?;
        entry.insert("x".into(), json!(position.x));
        entry.insert("y".into(), json!(position.y));
    }
    Ok(())
}

fn write<R: Runtime>(app: &AppHandle<R>, entry: Map<String, Value>) {
    let Some(path) = file(app) else { return };
    let mut states = read(app);
    let mut merged = match states.remove(MAIN_WINDOW) {
        Some(Value::Object(old)) => old,
        // The plugin reads every field back, so a new entry needs them all.
        _ => json!({ "width": 0, "height": 0, "x": 0, "y": 0, "prev_x": 0, "prev_y": 0, "maximized": false, "visible": true, "decorated": true, "fullscreen": false })
            .as_object()
            .cloned()
            .unwrap_or_default(),
    };
    merged.extend(entry);
    states.insert(MAIN_WINDOW.into(), Value::Object(merged));
    let result = path
        .parent()
        .map_or(Ok(()), fs::create_dir_all)
        .and_then(|()| fs::write(&path, serde_json::to_vec_pretty(&states).unwrap_or_default()));
    if let Err(err) = result {
        eprintln!("[fmess] could not save the window state: {err}");
    }
}

/// Called before the window hides into the tray, while it still shows its state.
pub fn save_on_close<R: Runtime>(window: &Window<R>) {
    let mut entry = Map::new();
    if capture(window, &mut entry).is_err() {
        return;
    }
    let app = window.app_handle();
    if let Some(closed) = app.try_state::<Closed>() {
        *closed.0.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(entry.clone());
    }
    write(app, entry);
}

/// Called at exit, after the plugin wrote its file.
pub fn save_on_exit<R: Runtime>(app: &AppHandle<R>) {
    let Some(window) = app.get_window(MAIN_WINDOW) else { return };
    let mut entry = Map::new();
    let live = window.is_visible().unwrap_or(false) && capture(&window, &mut entry).is_ok();
    if !live {
        let closed = app.try_state::<Closed>().and_then(|closed| closed.0.lock().ok()?.clone());
        match closed {
            Some(closed) => entry = closed,
            None => return,
        }
    }
    write(app, entry);
}
