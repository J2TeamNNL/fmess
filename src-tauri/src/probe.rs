// Debug builds only, for checking the live page while developing:
// FMESS_PROBE=<file.js> runs that file in the main window once the page has
// loaded (after FMESS_PROBE_DELAY seconds, 15 by default) and prints what it
// returns. The file is a function body: it may `await` and must `return`.
// FMESS_PROBE_WAIT=<seconds> waits that long for it (60 by default), for a
// probe that waits for something only a person does, like someone typing.
// FMESS_NO_INJECT=1 (webview.rs) leaves fMess's page script out, to measure
// what it costs.
// FMESS_WINDOW_SIZE=<width>x<height> sizes the window first, and
// FMESS_SNAPSHOT=<file.png> (macOS) then saves a picture of the page as the
// probe left it, through WebKit, which needs no screen-recording permission.
// For the rail: FMESS_THEME=<id> shows that theme in both web views without
// saving it, FMESS_RAIL_STATE=<json> overrides the rail's state (services,
// unread, account, dark...), FMESS_RAIL=1 shows the rail without a theme, and
// FMESS_SNAPSHOT_RAIL=<file.png> saves a picture of it too.
// FMESS_PROBE_SERVICE=instagram probes and pictures Instagram's web view
// instead, and selects it; with FMESS_SETTINGS='{"instagram":true}' (see
// settings.rs) it runs without touching the settings file.
use std::{thread, time::Duration};

use tauri::{AppHandle, LogicalSize, Manager, Runtime};

use crate::{
    services::{self, Service, RAIL},
    themes,
};

pub fn run<R: Runtime>(app: AppHandle<R>) {
    let Some(path) = std::env::var_os("FMESS_PROBE") else {
        return;
    };
    let delay = std::env::var("FMESS_PROBE_DELAY")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(15);
    let wait: u64 = std::env::var("FMESS_PROBE_WAIT")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(60);
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(delay));
        let script = match std::fs::read_to_string(&path) {
            Ok(script) => script,
            Err(err) => return eprintln!("[probe] cannot read {path:?}: {err}"),
        };
        let service = match std::env::var("FMESS_PROBE_SERVICE") {
            Ok(label) => match Service::from_label(&label) {
                Some(service) => service,
                None => return eprintln!("[probe] FMESS_PROBE_SERVICE: no service {label:?}"),
            },
            Err(_) => Service::Messenger,
        };
        let (Some(window), Some(webview)) = (services::main_window(&app), services::webview(&app, service)) else {
            return eprintln!("[probe] {} is not open", service.label());
        };
        if service != services::selected(&app) {
            let selector = app.clone();
            let _ = app.run_on_main_thread(move || services::select(&selector, service));
            thread::sleep(Duration::from_secs(1));
        }
        if let Some((width, height)) = std::env::var("FMESS_WINDOW_SIZE").ok().and_then(|size| {
            let (width, height) = size.split_once('x')?;
            Some((width.parse::<f64>().ok()?, height.parse::<f64>().ok()?))
        }) {
            let _ = window.set_size(LogicalSize::new(width, height));
            thread::sleep(Duration::from_secs(2));
        }
        override_rail(&app, &window);
        // Focused, as when someone uses it: WebKit matches :focus and fires
        // focus events only in the key window.
        let _ = window.set_focus();
        let _ = webview.set_focus();
        thread::sleep(Duration::from_millis(500));
        let _ = webview.eval(format!(
            "window.__fmessProbe = undefined; (async () => {{ {script}\n}})().then(\
             (result) => {{ window.__fmessProbe = JSON.stringify(result ?? null); }},\
             (error) => {{ window.__fmessProbe = JSON.stringify('error: ' + error); }});"
        ));
        for _ in 0..wait * 2 {
            thread::sleep(Duration::from_millis(500));
            let (tx, rx) = std::sync::mpsc::channel();
            let _ = webview.eval_with_callback("window.__fmessProbe ?? null", move |result| {
                let _ = tx.send(result);
            });
            match rx.recv_timeout(Duration::from_secs(5)) {
                Ok(result) if result != "null" => {
                    eprintln!("[probe] {result}");
                    #[cfg(target_os = "macos")]
                    if let Ok(path) = std::env::var("FMESS_SNAPSHOT") {
                        snapshot(&webview, path);
                    }
                    #[cfg(target_os = "macos")]
                    if let (Ok(path), Some(rail)) = (std::env::var("FMESS_SNAPSHOT_RAIL"), app.get_webview(RAIL)) {
                        snapshot(&rail, path);
                    }
                    return;
                }
                _ => {}
            }
        }
        eprintln!("[probe] no result after {wait} s");
    });
}

fn override_rail<R: Runtime>(app: &AppHandle<R>, window: &tauri::Window<R>) {
    let theme = std::env::var("FMESS_THEME").ok().map(|id| themes::for_page(app, Some(&id)));
    let mut state = std::env::var("FMESS_RAIL_STATE")
        .ok()
        .and_then(|json| serde_json::from_str::<serde_json::Value>(&json).map_err(|err| eprintln!("[probe] FMESS_RAIL_STATE: {err}")).ok())
        .unwrap_or_else(|| serde_json::json!({}));
    if let Some(theme) = &theme {
        state["theme"] = theme.clone();
        for webview in services::webviews(app) {
            let _ = webview.eval(format!("window.__fmessApplySettings?.({});", serde_json::json!({ "theme": theme })));
        }
    }
    if let Some(rail) = app.get_webview(RAIL) {
        let _ = rail.eval(format!("window.__fmessRail?.({state});"));
    }
    if theme.is_some() || std::env::var("FMESS_RAIL").is_ok_and(|value| value == "1") {
        services::place(window, true);
    }
    thread::sleep(Duration::from_secs(1));
}

/// Saves a picture of the page as a PNG, at the screen's scale.
#[cfg(target_os = "macos")]
fn snapshot<R: Runtime>(webview: &tauri::Webview<R>, path: String) {
    use block2::RcBlock;
    use objc2::{class, msg_send, rc::Retained, runtime::AnyObject};
    use objc2_foundation::NSString;

    let _ = webview.with_webview(move |webview| {
        let view = webview.inner().cast::<AnyObject>();
        let done = RcBlock::new(move |image: *mut AnyObject, _error: *mut AnyObject| {
            if image.is_null() {
                return eprintln!("[snapshot] failed");
            }
            // SAFETY: an NSImage from WebKit; NSBitmapImageFileTypePNG is 4.
            // An empty image (a hidden rail) has no bitmap, hence the checks.
            let saved: bool = unsafe {
                let tiff: *mut AnyObject = msg_send![image, TIFFRepresentation];
                let bitmap: *mut AnyObject = if tiff.is_null() {
                    std::ptr::null_mut()
                } else {
                    msg_send![class!(NSBitmapImageRep), imageRepWithData: tiff]
                };
                if bitmap.is_null() {
                    false
                } else {
                    let options: *mut AnyObject = msg_send![class!(NSDictionary), dictionary];
                    let png: *mut AnyObject = msg_send![bitmap, representationUsingType: 4usize, properties: options];
                    !png.is_null() && msg_send![png, writeToFile: &*NSString::from_str(&path), atomically: true]
                }
            };
            eprintln!("[snapshot] {} {path}", if saved { "saved" } else { "not saved" });
        });
        // SAFETY: the platform web view is a WKWebView, on the main thread.
        unsafe {
            let config: Retained<AnyObject> = msg_send![class!(WKSnapshotConfiguration), new];
            // With the selection drawn (left out by default), to check how it looks.
            let yes: *mut AnyObject = msg_send![class!(NSNumber), numberWithBool: true];
            let _: () = msg_send![&*config, setValue: yes, forKey: &*NSString::from_str("includesSelectionHighlighting")];
            let _: () = msg_send![view, takeSnapshotWithConfiguration: &*config, completionHandler: &*done];
        }
    });
}
