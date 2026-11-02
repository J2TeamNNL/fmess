use std::{
    sync::atomic::{AtomicU64, AtomicUsize, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use tauri::{
    utils::config::BackgroundThrottlingPolicy,
    webview::{NewWindowFeatures, NewWindowResponse, PageLoadEvent},
    AppHandle, Manager, Runtime, Url, WebviewUrl, WebviewWindowBuilder,
};

use crate::{
    i18n::{self, pick, Lang},
    icon, links,
    settings::{self, Settings},
    AUTOSTART_ARG, MAIN_WINDOW,
};

pub const SETTINGS_WINDOW: &str = "settings";

/// Meta shut messenger.com down in April 2026; chats on the web live here now.
const MESSAGES_URL: &str = "https://www.facebook.com/messages/";

/// The page script, joined from src/inject/ by build.rs.
const INJECTED_SCRIPT: &str = include_str!(concat!(env!("OUT_DIR"), "/inject.js"));

/// The app icon, shown on fMess's loading screen in place of Facebook's.
const SPLASH_LOGO: &[u8] = include_bytes!("../icons/128x128@2x.png");
const SPLASH_DARK: tauri::window::Color = tauri::window::Color(28, 28, 29, 255);
const SPLASH_LIGHT: tauri::window::Color = tauri::window::Color(255, 255, 255, 255);

/// Height of the strip inject.js keeps free under the overlaid macOS title bar.
const TITLE_BAR_INSET: u32 = if cfg!(target_os = "macos") { 28 } else { 0 };

pub fn messages_url() -> Url {
    Url::parse(MESSAGES_URL).expect("MESSAGES_URL is valid")
}

pub fn create_main_window<R: Runtime>(app: &AppHandle<R>, settings: &Settings) -> tauri::Result<()> {
    let launched_at_login = std::env::args().any(|arg| arg == AUTOSTART_ARG);
    let builder = WebviewWindowBuilder::new(app, MAIN_WINDOW, WebviewUrl::External(messages_url()))
        .title("fMess")
        .inner_size(1100.0, 760.0)
        .min_inner_size(420.0, 520.0)
        .visible(!(launched_at_login && settings.start_hidden))
        .on_page_load(|window, payload| match payload.event() {
            PageLoadEvent::Started => {
                if should_leave_feed(payload.url()) {
                    let _ = window.navigate(messages_url());
                }
            }
            PageLoadEvent::Finished => {
                // The settings may have changed since the window opened.
                let settings = settings::current(window.app_handle());
                let _ = window.eval(settings::page_settings_script(window.app_handle(), &settings));
            }
        });
    // Messenger's colours from the first frame, before the page paints.
    let builder = match settings::messenger_theme(settings.messenger_dark_mode.as_deref()) {
        Some(tauri::Theme::Dark) => builder.background_color(SPLASH_DARK),
        Some(_) => builder.background_color(SPLASH_LIGHT),
        None => builder,
    };
    // Like the old desktop app: no separate title bar, window buttons over the page.
    #[cfg(target_os = "macos")]
    let builder = builder
        .title_bar_style(tauri::TitleBarStyle::Overlay)
        .hidden_title(true);
    let window = configure(builder, app, MAIN_WINDOW).build()?;
    if settings.zoom != 1.0 {
        let _ = window.set_zoom(settings.zoom);
    }
    Ok(())
}

/// Called by the Messenger page (a theme's Settings button).
#[tauri::command]
pub fn open_settings<R: Runtime>(app: AppHandle<R>) {
    open_settings_window(&app);
}

pub fn open_settings_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window(SETTINGS_WINDOW) {
        let _ = window.show();
        let _ = window.set_focus();
        return;
    }
    // Built away from the main thread: Settings opens from synchronous
    // commands (Ctrl+, from the page) and menu handlers, which run there, and
    // on Windows building a window there deadlocks (WebView2), leaving a
    // white window that can't be closed.
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let lang = i18n::current(&app);
        let result = WebviewWindowBuilder::new(&app, SETTINGS_WINDOW, WebviewUrl::App("settings.html".into()))
            .title(settings_title(lang))
            // So the page is in its language from the first frame, before it
            // has asked for the settings.
            .initialization_script(format!("window.__fmessLang = {};", serde_json::json!(lang)))
            .inner_size(640.0, 780.0)
            .min_inner_size(560.0, 560.0)
            .resizable(true)
            .minimizable(false)
            .maximizable(false)
            .build();
        match result {
            Ok(window) => icon::apply_to(&window),
            Err(err) => eprintln!("could not open settings: {err}"),
        }
    });
}

/// The Settings window's title.
pub fn settings_title(lang: Lang) -> &'static str {
    pick(lang, "fMess Settings", "Cài đặt fMess")
}

/// Called by inject.js when the photo/video viewer opens or closes. The viewer
/// fills the page and places its buttons at the very top, so on macOS it gets
/// a regular title bar while open, and the chats go back to the overlaid one.
#[tauri::command]
pub fn set_viewer_open<R: Runtime>(app: AppHandle<R>, open: bool) {
    #[cfg(target_os = "macos")]
    if let Some(window) = app.get_webview_window(MAIN_WINDOW) {
        let style = if open {
            tauri::TitleBarStyle::Visible
        } else {
            tauri::TitleBarStyle::Overlay
        };
        let _ = window.set_title_bar_style(style);
        set_title_visible(&window, open);
    }
    #[cfg(not(target_os = "macos"))]
    let _ = (app, open);
}

/// Shows or hides "fMess" in the title bar; Tauri only sets this at creation.
#[cfg(target_os = "macos")]
fn set_title_visible<R: Runtime>(window: &tauri::WebviewWindow<R>, visible: bool) {
    use objc2_app_kit::{NSWindow, NSWindowTitleVisibility};

    let Ok(ns_window) = window.ns_window() else {
        return;
    };
    // Raw pointers aren't Send; AppKit must be called on the main thread.
    let ns_window = ns_window as usize;
    let _ = window.run_on_main_thread(move || {
        // SAFETY: Tauri returns the live NSWindow of a window that still exists.
        let ns_window = unsafe { &*(ns_window as *const NSWindow) };
        ns_window.setTitleVisibility(if visible {
            NSWindowTitleVisibility::Visible
        } else {
            NSWindowTitleVisibility::Hidden
        });
    });
}

/// Facebook lands on the news feed after logging in, but fMess is for chats.
/// Redirect at most once every few seconds so a server-side bounce can't loop.
fn should_leave_feed(url: &Url) -> bool {
    static LAST_REDIRECT: AtomicU64 = AtomicU64::new(0);
    if !links::is_facebook_feed(url) {
        return false;
    }
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs());
    let last = LAST_REDIRECT.swap(now, Ordering::Relaxed);
    now.saturating_sub(last) >= 5
}

/// Settings shared by the main window and the popups it opens (calls, media).
fn configure<'a, R: Runtime, M: Manager<R>>(
    builder: WebviewWindowBuilder<'a, R, M>,
    app: &AppHandle<R>,
    label: &str,
) -> WebviewWindowBuilder<'a, R, M> {
    let navigation_app = app.clone();
    let popup_app = app.clone();
    let label = label.to_owned();
    let builder = builder
        // Call popups need the script too: Facebook checks call support again
        // there. (On macOS a popup shares its opener's scripts and skips these.)
        .initialization_script(initialization_script(app))
        .on_navigation(move |url| {
            if is_close_window_url(url) {
                close_popup(&navigation_app, &label);
                return false;
            }
            links::allow_navigation(&navigation_app, url)
        })
        .on_new_window(move |url, features| open_popup(&popup_app, url, features))
        // Without a download handler, macOS cancels every download.
        .on_download(|_, _| true)
        // Tauri takes files dropped on the window for itself (as its own
        // events) and keeps them from the page, so a file or screenshot
        // dragged into a chat did nothing; Messenger takes them as in a browser.
        .disable_drag_drop_handler()
        // Hidden windows must keep receiving messages (macOS 14+ only).
        .background_throttling(BackgroundThrottlingPolicy::Disabled);
    match user_agent() {
        Some(user_agent) => builder.user_agent(&user_agent),
        None => builder,
    }
}

/// WebKit draws overlay scroll bars (thin, shown only while scrolling, moved
/// by its scrolling thread together with the content) only when AppKit
/// prefers them, which follows System Settings and whether a mouse is
/// connected; otherwise it draws classic always-on ones that take room from
/// the page. fMess always wants overlay ones: inject.js otherwise hides the
/// classic ones and draws an indicator of its own. A value in fMess's own
/// defaults domain wins over the system-wide one, for fMess only.
#[cfg(target_os = "macos")]
pub fn prefer_overlay_scrollbars() {
    use objc2::runtime::AnyObject;
    use objc2_foundation::{NSString, NSUserDefaults};

    let defaults = NSUserDefaults::standardUserDefaults();
    let value = NSString::from_str("WhenScrolling");
    let value: &AnyObject = &value;
    // SAFETY: a string is a valid property-list value.
    unsafe { defaults.setObject_forKey(Some(value), &NSString::from_str("AppleShowScrollBars")) };
}

/// inject.js, preceded by what it needs from fMess: the title bar inset, its
/// settings, and the logo and colours of the loading screen.
fn initialization_script<R: Runtime>(app: &AppHandle<R>) -> String {
    use base64::Engine;

    let settings = settings::current(app);
    let dark = match settings::messenger_theme(settings.messenger_dark_mode.as_deref()) {
        Some(tauri::Theme::Dark) => "true",
        Some(_) => "false",
        None => "null",
    };
    let logo = base64::engine::general_purpose::STANDARD.encode(SPLASH_LOGO);
    let debug = cfg!(debug_assertions);
    format!(
        "window.__fmessTitleBarInset = {TITLE_BAR_INSET};\n\
         window.__fmessDebug = {debug};\n\
         window.__fmessSettings = {};\n\
         window.__fmessDark = {dark};\n\
         window.__fmessSplashLogo = 'data:image/png;base64,{logo}';\n\
         {INJECTED_SCRIPT}",
        settings::page_settings(app, &settings)
    )
}

fn open_popup<R: Runtime>(
    app: &AppHandle<R>,
    url: Url,
    features: NewWindowFeatures,
) -> NewWindowResponse<R> {
    if !links::opens_in_app(&url) {
        links::open_external(app, &url);
        return NewWindowResponse::Deny;
    }
    static NEXT_POPUP: AtomicUsize = AtomicUsize::new(1);
    let label = format!("popup-{}", NEXT_POPUP.fetch_add(1, Ordering::Relaxed));
    let size = features.size().map(|size| (size.width, size.height));
    let mut builder = WebviewWindowBuilder::new(app, &label, WebviewUrl::External(url))
        .title("fMess")
        .window_features(features);
    if let Some((x, y)) = centred_on_main_window(app, size) {
        builder = builder.position(x, y);
    }
    match configure(builder, app, &label).build() {
        Ok(window) => {
            icon::apply_to(&window);
            NewWindowResponse::Create { window }
        }
        Err(err) => {
            eprintln!("could not open popup window: {err}");
            NewWindowResponse::Deny
        }
    }
}

/// Where a popup of `size` sits centred on fMess's window. Pages centre their
/// popups on their own window, but WKWebView reports it as having no size at
/// the bottom of the screen, so Facebook's call window opened off the bottom
/// of a screen fMess might not even be on.
fn centred_on_main_window<R: Runtime>(app: &AppHandle<R>, size: Option<(f64, f64)>) -> Option<(f64, f64)> {
    let main = app.get_webview_window(MAIN_WINDOW)?;
    let scale = main.scale_factor().ok()?;
    let position = main.outer_position().ok()?.to_logical::<f64>(scale);
    let main_size = main.outer_size().ok()?.to_logical::<f64>(scale);
    // Tauri's default window size, for popups that don't ask for one.
    let (width, height) = size.unwrap_or((800.0, 600.0));
    Some((
        position.x + (main_size.width - width) / 2.0,
        position.y + (main_size.height - height) / 2.0,
    ))
}

/// inject.js sends `window.close()` here, because WKWebView ignores it.
fn is_close_window_url(url: &Url) -> bool {
    url.scheme() == "fmess" && url.host_str() == Some("close-window")
}

fn close_popup<R: Runtime>(app: &AppHandle<R>, label: &str) {
    if label == MAIN_WINDOW {
        return;
    }
    let handle = app.clone();
    let label = label.to_owned();
    // Close after WebKit's navigation callback returns, not inside it.
    let _ = app.run_on_main_thread(move || {
        if let Some(window) = handle.get_webview_window(&label) {
            let _ = window.close();
        }
    });
}

/// WKWebView's default user agent carries no Safari version, so Facebook may
/// treat it as an unknown browser. Claim the Safari that ships with this macOS.
#[cfg(target_os = "macos")]
fn user_agent() -> Option<String> {
    let plist = std::fs::read_to_string("/System/Library/CoreServices/SystemVersion.plist").ok()?;
    let version = plist
        .split("<key>ProductVersion</key>")
        .nth(1)?
        .split("<string>")
        .nth(1)?
        .split('<')
        .next()?;
    let macos_major: u32 = version.split('.').next()?.parse().ok()?;
    // Safari's major version was macOS + 3 until macOS 26 aligned the two.
    let safari_major = if macos_major >= 26 { macos_major } else { macos_major + 3 };
    Some(format!(
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 \
         (KHTML, like Gecko) Version/{safari_major}.0 Safari/605.1.15"
    ))
}

/// WebView2 (Edge) and WebKitGTK already send a user agent Facebook supports.
#[cfg(not(target_os = "macos"))]
fn user_agent() -> Option<String> {
    None
}
