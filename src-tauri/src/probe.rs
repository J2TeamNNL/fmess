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
// Always, on stderr: each navigation of every window ([nav]), each popup
// ([popup]) and the page's JS errors ([page]), URLs without query values.
use std::{thread, time::Duration};

use tauri::{AppHandle, LogicalSize, Manager, Runtime, Url};

/// A URL as it may be printed: query keys only (values can be tokens), no fragment.
pub fn loggable(url: &Url) -> String {
    let mut out = format!("{}:", url.scheme());
    if let Some(host) = url.host_str() {
        out += &format!("//{host}");
    }
    out += url.path();
    let keys: Vec<_> = url.query_pairs().map(|(key, _)| key.into_owned()).collect();
    if !keys.is_empty() {
        let keys = keys.join(",");
        let keys: String = keys.chars().take(80).collect();
        out += &format!("?{keys}");
    }
    out
}

pub fn log_navigation(label: &str, url: &Url, allowed: bool) {
    let verdict = if allowed {
        "allowed"
    } else if matches!(url.scheme(), "http" | "https" | "mailto" | "tel") {
        "opened externally"
    } else {
        "blocked"
    };
    eprintln!("[nav] {label} {verdict} {}", loggable(url));
}

/// Prints what inject.js collected in window.__fmessErrors, from every window.
fn watch_page_errors<R: Runtime>(app: AppHandle<R>) {
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(2));
        for (label, window) in app.webview_windows() {
            let _ = window.eval_with_callback(
                "JSON.stringify((window.__fmessErrors || []).splice(0))",
                move |result| {
                    let entries = serde_json::from_str::<String>(&result)
                        .ok()
                        .and_then(|json| serde_json::from_str::<Vec<String>>(&json).ok())
                        .unwrap_or_default();
                    for entry in entries {
                        eprintln!("[page] {label} {entry}");
                    }
                },
            );
        }
    });
}

pub fn run<R: Runtime>(app: AppHandle<R>) {
    watch_page_errors(app.clone());
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
        let Some(window) = app.get_webview_window(crate::MAIN_WINDOW) else {
            return;
        };
        if let Some((width, height)) = std::env::var("FMESS_WINDOW_SIZE").ok().and_then(|size| {
            let (width, height) = size.split_once('x')?;
            Some((width.parse::<f64>().ok()?, height.parse::<f64>().ok()?))
        }) {
            let _ = window.set_size(LogicalSize::new(width, height));
            thread::sleep(Duration::from_secs(2));
        }
        // Focused, as when someone uses it: WebKit matches :focus and fires
        // focus events only in the key window.
        let _ = window.set_focus();
        thread::sleep(Duration::from_millis(500));
        let _ = window.eval(format!(
            "window.__fmessProbe = undefined; (async () => {{ {script}\n}})().then(\
             (result) => {{ window.__fmessProbe = JSON.stringify(result ?? null); }},\
             (error) => {{ window.__fmessProbe = JSON.stringify('error: ' + error); }});"
        ));
        for _ in 0..wait * 2 {
            thread::sleep(Duration::from_millis(500));
            let (tx, rx) = std::sync::mpsc::channel();
            let _ = window.eval_with_callback("window.__fmessProbe ?? null", move |result| {
                let _ = tx.send(result);
            });
            match rx.recv_timeout(Duration::from_secs(5)) {
                Ok(result) if result != "null" => {
                    eprintln!("[probe] {result}");
                    #[cfg(target_os = "macos")]
                    if let Ok(path) = std::env::var("FMESS_SNAPSHOT") {
                        snapshot(&window, path);
                    }
                    return;
                }
                _ => {}
            }
        }
        eprintln!("[probe] no result after {wait} s");
    });
}

/// Saves a picture of the page as a PNG, at the screen's scale.
#[cfg(target_os = "macos")]
fn snapshot<R: Runtime>(window: &tauri::WebviewWindow<R>, path: String) {
    use block2::RcBlock;
    use objc2::{class, msg_send, rc::Retained, runtime::AnyObject};
    use objc2_foundation::NSString;

    let _ = window.with_webview(move |webview| {
        let view = webview.inner().cast::<AnyObject>();
        let done = RcBlock::new(move |image: *mut AnyObject, _error: *mut AnyObject| {
            if image.is_null() {
                return eprintln!("[snapshot] failed");
            }
            // SAFETY: an NSImage from WebKit; NSBitmapImageFileTypePNG is 4.
            let saved: bool = unsafe {
                let tiff: *mut AnyObject = msg_send![image, TIFFRepresentation];
                let bitmap: *mut AnyObject = msg_send![class!(NSBitmapImageRep), imageRepWithData: tiff];
                let options: *mut AnyObject = msg_send![class!(NSDictionary), dictionary];
                let png: *mut AnyObject = msg_send![bitmap, representationUsingType: 4usize, properties: options];
                msg_send![png, writeToFile: &*NSString::from_str(&path), atomically: true]
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
