use std::{
    sync::atomic::{AtomicU64, AtomicUsize, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use tauri::{
    utils::config::BackgroundThrottlingPolicy,
    webview::{NewWindowFeatures, NewWindowResponse, PageLoadEvent, WebviewBuilder},
    window::WindowBuilder,
    AppHandle, Manager, PhysicalPosition, PhysicalSize, Runtime, Url, Webview, WebviewUrl, WebviewWindowBuilder, Window,
};

use crate::{
    icon, links,
    services::{self, Service},
    settings::{self, Settings},
    AUTOSTART_ARG, MAIN_WINDOW,
};

/// Settings shared by the services' web views and the popups they open
/// (calls, media), which are built by different builders with the same methods.
macro_rules! configure {
    ($builder:expr, $app:expr, $label:expr, $service:expr) => {{
        let app: &AppHandle<_> = $app;
        let service: Service = $service;
        let navigation_app = app.clone();
        let popup_app = app.clone();
        let label: String = $label.to_owned();
        let builder = $builder
            // Call popups need the script too: Facebook checks call support again
            // there. (On macOS a popup shares its opener's scripts and skips these.)
            .initialization_script(initialization_script(app, service))
            .on_navigation(move |url| {
                if is_close_window_url(url) {
                    close_popup(&navigation_app, &label);
                    return false;
                }
                links::allow_navigation(&navigation_app, service, url)
            })
            .on_new_window(move |url, features| open_popup(&popup_app, service, url, features))
            // Without a download handler, macOS cancels every download.
            .on_download(|_, _| true)
            // Hidden windows must keep receiving messages (macOS 14+ only).
            .background_throttling(BackgroundThrottlingPolicy::Disabled);
        match user_agent() {
            Some(user_agent) => builder.user_agent(&user_agent),
            None => builder,
        }
    }};
}

pub const SETTINGS_WINDOW: &str = "settings";

/// Meta shut messenger.com down in April 2026; chats on the web live here now.
const MESSAGES_URL: &str = "https://www.facebook.com/messages/";

/// Instagram's chats, opened when the Instagram service is turned on.
const INSTAGRAM_URL: &str = "https://www.instagram.com/direct/inbox/";

const INJECTED_SCRIPT: &str = include_str!("inject.js");
const INSTAGRAM_SCRIPT: &str = include_str!("inject-instagram.js");

/// The app icon, shown on fMess's loading screen in place of Facebook's.
const SPLASH_LOGO: &[u8] = include_bytes!("../icons/128x128@2x.png");
const SPLASH_DARK: tauri::window::Color = tauri::window::Color(28, 28, 29, 255);
const SPLASH_LIGHT: tauri::window::Color = tauri::window::Color(255, 255, 255, 255);

/// Height of the strip inject.js keeps free under the overlaid macOS title bar.
pub const TITLE_BAR_INSET: u32 = if cfg!(target_os = "macos") { 28 } else { 0 };

pub fn messages_url() -> Url {
    Url::parse(MESSAGES_URL).expect("MESSAGES_URL is valid")
}

pub fn instagram_url() -> Url {
    Url::parse(INSTAGRAM_URL).expect("INSTAGRAM_URL is valid")
}

pub fn create_main_window<R: Runtime>(app: &AppHandle<R>, settings: &Settings) -> tauri::Result<()> {
    let launched_at_login = std::env::args().any(|arg| arg == AUTOSTART_ARG);
    let builder = WindowBuilder::new(app, MAIN_WINDOW)
        .title("fMess")
        .inner_size(1100.0, 760.0)
        .min_inner_size(420.0, 520.0)
        .visible(!(launched_at_login && settings.start_hidden));
    let messenger = WebviewBuilder::new(Service::Messenger.label(), WebviewUrl::External(messages_url()))
        .on_page_load(|webview, payload| match payload.event() {
            PageLoadEvent::Started => {
                static LAST_REDIRECT: AtomicU64 = AtomicU64::new(0);
                if links::is_facebook_feed(payload.url()) && may_redirect(&LAST_REDIRECT) {
                    let _ = webview.navigate(messages_url());
                }
            }
            PageLoadEvent::Finished => apply_page_settings(&webview),
        });
    // Messenger's colours from the first frame, before the page paints.
    let background = match settings::messenger_theme(settings.messenger_dark_mode.as_deref()) {
        Some(tauri::Theme::Dark) => Some(SPLASH_DARK),
        Some(_) => Some(SPLASH_LIGHT),
        None => None,
    };
    let (builder, messenger) = match background {
        Some(colour) => (builder.background_color(colour), messenger.background_color(colour)),
        None => (builder, messenger),
    };
    // Like the old desktop app: no separate title bar, window buttons over the page.
    #[cfg(target_os = "macos")]
    let builder = builder
        .title_bar_style(tauri::TitleBarStyle::Overlay)
        .hidden_title(true);
    let window = builder.build()?;
    let size = window.inner_size()?;
    let messenger = configure!(messenger, app, Service::Messenger.label(), Service::Messenger);
    let webview = window.add_child(messenger, PhysicalPosition::new(0, 0), size)?;
    if settings.zoom != 1.0 {
        let _ = webview.set_zoom(settings.zoom);
    }
    if settings.instagram {
        add_instagram(app, &window, settings)?;
    }
    // After the services, whose icons it shows.
    let rail = match background {
        Some(colour) => rail(app).background_color(colour),
        None => rail(app),
    };
    window.add_child(rail, PhysicalPosition::new(0, 0), PhysicalSize::new(0, size.height))?;
    services::layout(&window);
    services::select(app, services::selected(app));
    Ok(())
}

/// Instagram's chats, beside Messenger's and hidden until selected; it keeps
/// running so its messages arrive. services::layout places it.
pub fn add_instagram<R: Runtime>(app: &AppHandle<R>, window: &Window<R>, settings: &Settings) -> tauri::Result<Webview<R>> {
    let label = Service::Instagram.label();
    let builder = WebviewBuilder::new(label, WebviewUrl::External(instagram_url())).on_page_load(|webview, payload| {
        match payload.event() {
            PageLoadEvent::Started => {
                static LAST_REDIRECT: AtomicU64 = AtomicU64::new(0);
                if links::is_instagram_home(payload.url()) && may_redirect(&LAST_REDIRECT) {
                    let _ = webview.navigate(instagram_url());
                }
            }
            PageLoadEvent::Finished => apply_page_settings(&webview),
        }
    });
    let builder = configure!(builder, app, label, Service::Instagram);
    let webview = window.add_child(builder, PhysicalPosition::new(0, 0), PhysicalSize::new(0, 0))?;
    let _ = webview.hide();
    if settings.zoom != 1.0 {
        let _ = webview.set_zoom(settings.zoom);
    }
    Ok(webview)
}

/// The settings may have changed since the web view opened.
fn apply_page_settings<R: Runtime>(webview: &Webview<R>) {
    let app = webview.app_handle();
    let _ = webview.eval(settings::page_settings_script(app, &settings::current(app)));
}

/// The bar down the left (ui/rail.html), which services::layout places.
fn rail<R: Runtime>(app: &AppHandle<R>) -> WebviewBuilder<R> {
    WebviewBuilder::new(services::RAIL, WebviewUrl::App("rail.html".into()))
        // Its state from the first frame, so a theme's rail shows at once.
        .initialization_script(format!("window.__fmessRailInit = {};", services::rail_state(app)))
        .on_page_load(|webview, payload| {
            if let PageLoadEvent::Finished = payload.event() {
                // Events sent while it was loading were missed.
                let _ = webview.eval(services::rail_script(webview.app_handle()));
            }
        })
        // A local page with nothing to link to.
        .on_navigation(|url| url.scheme() == "tauri" || url.host_str() == Some("tauri.localhost"))
}

/// Called by the rail and the Messenger page (a theme's Settings button).
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
        let result = WebviewWindowBuilder::new(&app, SETTINGS_WINDOW, WebviewUrl::App("settings.html".into()))
            .title("fMess Settings")
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

/// Called by inject.js when the photo/video viewer opens or closes. The viewer
/// fills the page and places its buttons at the very top, so on macOS it gets
/// a regular title bar while open, and the chats go back to the overlaid one.
#[tauri::command]
pub fn set_viewer_open<R: Runtime>(app: AppHandle<R>, open: bool) {
    #[cfg(target_os = "macos")]
    if let Some(window) = services::main_window(&app) {
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
fn set_title_visible<R: Runtime>(window: &tauri::Window<R>, visible: bool) {
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

/// Facebook lands on the news feed after logging in, and Instagram on its
/// home page, but fMess is for chats. Redirect at most once every few seconds
/// so a server-side bounce can't loop.
fn may_redirect(last_redirect: &AtomicU64) -> bool {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs());
    let last = last_redirect.swap(now, Ordering::Relaxed);
    now.saturating_sub(last) >= 5
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

/// The service's page script (inject.js or inject-instagram.js), preceded by
/// what it needs from fMess: the title bar inset, its settings, and the logo
/// and colours of Messenger's loading screen.
fn initialization_script<R: Runtime>(app: &AppHandle<R>, service: Service) -> String {
    use base64::Engine;

    let settings = settings::current(app);
    let dark = match settings::messenger_theme(settings.messenger_dark_mode.as_deref()) {
        Some(tauri::Theme::Dark) => "true",
        Some(_) => "false",
        None => "null",
    };
    let debug = cfg!(debug_assertions);
    if service == Service::Instagram {
        return format!(
            "window.__fmessTitleBarInset = {TITLE_BAR_INSET};\n\
             window.__fmessDebug = {debug};\n\
             window.__fmessSettings = {};\n\
             {INSTAGRAM_SCRIPT}",
            settings::page_settings(app, &settings)
        );
    }
    let logo = base64::engine::general_purpose::STANDARD.encode(SPLASH_LOGO);
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
    service: Service,
    url: Url,
    features: NewWindowFeatures,
) -> NewWindowResponse<R> {
    if !links::opens_in_app(service, &url) {
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
    match configure!(builder, app, &label, service).build() {
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
    let main = services::main_window(app)?;
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
    if Service::from_label(label).is_some() {
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
