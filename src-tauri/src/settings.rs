use std::{
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, Runtime, Theme, Webview};
use tauri_plugin_autostart::ManagerExt;

use crate::{
    badge,
    i18n::{self, pick, Lang},
    icon, menu, notify,
    services::{self, Service},
    themes,
    webview::{self, SETTINGS_WINDOW},
};

/// fMess's own preferences. Account settings (active status, sounds, dark
/// mode) stay in Facebook's Preferences, which sync with the phone app.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    /// When launched at login, stay in the tray instead of opening the window.
    pub start_hidden: bool,
    pub notifications: bool,
    /// Show the sender and text; otherwise just "New message".
    pub notification_preview: bool,
    pub badge: bool,
    /// Closing the window keeps fMess running; otherwise it quits.
    pub close_to_tray: bool,
    pub zoom: f64,
    /// Turn on Facebook's WebKit path for calls in encrypted chats (macOS).
    pub mac_calls: bool,
    /// Don't let people see that you're typing.
    pub block_typing: bool,
    /// Show initials in place of people's pictures.
    pub hide_avatars: bool,
    /// Fields check spelling. It knows only the system's languages: on a
    /// Windows set up in English nearly every Vietnamese word is underlined.
    pub spell_check: bool,
    /// Messenger's dark mode as last reported (ENABLED, DISABLED, USE_SYSTEM),
    /// so fMess starts in the right colours before the page has loaded.
    pub messenger_dark_mode: Option<String>,
    /// The chosen theme (see themes.rs); none shows Messenger as it is.
    pub theme: Option<String>,
    /// The app's icon shows the theme's (see icon.rs).
    pub theme_icon: bool,
    /// The name on fMess's windows, tray and notifications; none is "fMess".
    pub app_name: Option<String>,
    /// A picture chosen in Settings is the app's icon (see icon.rs).
    pub custom_icon: bool,
    /// Widths the panes were dragged to; none keeps Facebook's.
    pub pane_widths: PaneWidths,
    /// Instagram's chats run beside Messenger's. Off by default: its web
    /// view takes a few hundred MB.
    pub instagram: bool,
    /// The rail shows Instagram's icon; ⌃⌘2 switches to it either way.
    pub instagram_icon: bool,
    /// The service shown last, shown again at start. Set by the rail.
    #[serde(deserialize_with = "lenient_service")]
    pub last_service: Service,
    /// The theme's sidebar button folded its sidebar away (the Teams look's
    /// app bar, which is fMess's rail). Set by the page; a new theme unfolds it.
    pub sidebar_collapsed: bool,
    /// fMess's own language, "en" or "vi"; none follows the system (i18n.rs).
    pub language: Option<String>,
}

/// A service this version doesn't know (from a newer or older fMess) falls
/// back to Messenger instead of failing the whole file, which resets every setting.
fn lenient_service<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Service, D::Error> {
    Ok(Service::deserialize(serde_json::Value::deserialize(deserializer)?).unwrap_or_default())
}

/// In CSS pixels. Set by the page (set_pane_widths), not the Settings window.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct PaneWidths {
    /// The chat list.
    pub list: Option<u32>,
    /// The chat's details, beside the messages.
    pub details: Option<u32>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            start_hidden: false,
            notifications: true,
            notification_preview: true,
            badge: true,
            close_to_tray: true,
            zoom: 1.0,
            mac_calls: true,
            block_typing: false,
            hide_avatars: false,
            spell_check: true,
            messenger_dark_mode: None,
            theme: None,
            theme_icon: true,
            app_name: None,
            custom_icon: false,
            pane_widths: PaneWidths::default(),
            instagram: false,
            instagram_icon: true,
            last_service: Service::Messenger,
            sidebar_collapsed: false,
            language: None,
        }
    }
}

impl Settings {
    pub fn app_name(&self) -> &str {
        self.app_name.as_deref().map(str::trim).filter(|name| !name.is_empty()).unwrap_or("fMess")
    }
}

pub struct SettingsState(Mutex<Settings>);

/// Messenger's own Preferences, which belong to the account. The page reports
/// them (sync_messenger_prefs) so the Settings window can show them.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessengerPrefs {
    sounds_on: bool,
    active_status_on: Option<bool>,
    /// Seconds since 1970 when Do Not Disturb ends: 0 when off, -1 until turned off.
    mute_until: i64,
    /// ENABLED, DISABLED or USE_SYSTEM.
    dark_mode: String,
}

#[derive(Default)]
pub struct MessengerPrefsState(Mutex<Option<MessengerPrefs>>);

/// What the Settings window shows: the saved settings plus live system state.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsView {
    #[serde(flatten)]
    settings: Settings,
    launch_at_login: bool,
    platform: &'static str,
    /// The language the window speaks: the chosen one, or the system's.
    lang: Lang,
    /// None until the page has read them, or when logged out.
    messenger: Option<MessengerPrefs>,
}

pub fn init<R: Runtime>(app: &AppHandle<R>) -> Settings {
    let settings: Settings = settings_path(app)
        .and_then(|path| fs::read(path).ok())
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default();
    #[cfg(debug_assertions)]
    let settings = debug_override(settings);
    app.manage(SettingsState(Mutex::new(Settings::clone(&settings))));
    app.manage(MessengerPrefsState::default());
    settings
}

/// Debug builds only: FMESS_SETTINGS=<json> sets some settings for this run,
/// such as `{"instagram": true, "lastService": "instagram"}`, and keeps every
/// change in memory, so the settings file stays as it was.
#[cfg(debug_assertions)]
fn debug_override(settings: Settings) -> Settings {
    let Ok(json) = std::env::var("FMESS_SETTINGS") else {
        return settings;
    };
    let overridden = serde_json::from_str::<serde_json::Value>(&json)
        .and_then(|value| merge(&settings, value))
        .map_err(|err| eprintln!("[fmess] FMESS_SETTINGS: {err}"));
    match overridden {
        Ok(overridden) => {
            IN_MEMORY.store(true, std::sync::atomic::Ordering::Relaxed);
            overridden
        }
        Err(()) => settings,
    }
}

#[cfg(debug_assertions)]
static IN_MEMORY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// `settings` with the fields `changes` gives.
#[cfg(debug_assertions)]
fn merge(settings: &Settings, changes: serde_json::Value) -> serde_json::Result<Settings> {
    let mut value = serde_json::to_value(settings)?;
    if let (Some(fields), serde_json::Value::Object(changes)) = (value.as_object_mut(), changes) {
        fields.extend(changes);
    }
    serde_json::from_value(value)
}

pub fn current<R: Runtime>(app: &AppHandle<R>) -> Settings {
    app.state::<SettingsState>().0.lock().unwrap().clone()
}

#[tauri::command]
pub fn get_settings<R: Runtime>(app: AppHandle<R>) -> SettingsView {
    view(&app, current(&app))
}

#[tauri::command]
pub fn update_settings<R: Runtime>(
    app: AppHandle<R>,
    settings: Settings,
    launch_at_login: bool,
) -> Result<SettingsView, String> {
    let previous = current(&app);
    let autostart = app.autolaunch();
    if autostart.is_enabled().unwrap_or(false) != launch_at_login {
        let result = if launch_at_login { autostart.enable() } else { autostart.disable() };
        result.map_err(|err| match Lang::from_setting(settings.language.as_deref()) {
            Lang::En => format!("could not change Open at Login: {err}"),
            Lang::Vi => format!("Không đổi được mục Mở khi đăng nhập: {err}"),
        })?;
    }

    let settings = change(&app, |current| {
        // Kept by the page, the rail and the icon commands, not edited in the Settings window.
        let kept = (
            current.messenger_dark_mode.take(),
            std::mem::take(&mut current.pane_widths),
            current.last_service,
            current.custom_icon,
        );
        let sidebar_collapsed = current.sidebar_collapsed && current.theme == settings.theme;
        *current = settings;
        (current.messenger_dark_mode, current.pane_widths, current.last_service, current.custom_icon) = kept;
        current.sidebar_collapsed = sidebar_collapsed;
    })?;

    if settings.theme != previous.theme || settings.theme_icon != previous.theme_icon {
        icon::apply(&app);
    }
    if settings.app_name() != previous.app_name() {
        apply_app_name(&app, settings.app_name());
    }
    let lang = Lang::from_setting(settings.language.as_deref());
    let language_changed = lang != Lang::from_setting(previous.language.as_deref());
    if language_changed {
        if let Err(err) = menu::apply_language(&app, lang) {
            eprintln!("could not change the menus' language: {err}");
        }
        notify::apply_language(lang);
        if let Some(window) = app.get_webview_window(SETTINGS_WINDOW) {
            let _ = window.set_title(webview::settings_title(lang));
        }
    }
    if settings.instagram != previous.instagram {
        services::set_instagram(&app, settings.instagram);
    }
    if settings.theme != previous.theme
        || settings.instagram_icon != previous.instagram_icon
        || settings.hide_avatars != previous.hide_avatars
        || language_changed
    {
        services::update_rail(&app);
    }
    if settings.badge != previous.badge {
        if let Some(window) = services::main_window(&app) {
            badge::refresh(&window);
        }
    }
    for webview in services::webviews(&app) {
        if settings.zoom != previous.zoom {
            let _ = webview.set_zoom(settings.zoom);
        }
        if settings.block_typing != previous.block_typing
            || settings.hide_avatars != previous.hide_avatars
            || settings.spell_check != previous.spell_check
            || settings.theme != previous.theme
            || language_changed
        {
            let _ = webview.eval(page_settings_script(&webview, &settings));
        }
    }
    if let Some(webview) = services::webview(&app, Service::Messenger) {
        if settings.mac_calls != previous.mac_calls {
            // The call gate is patched as Facebook loads, so reload to apply
            // it; the page's settings from when the window opened are older,
            // so this one value goes to the reload through localStorage.
            let _ = webview.eval(format!(
                "try {{ localStorage.setItem('fmess.macCalls', '{}'); }} catch {{}} location.reload();",
                if settings.mac_calls { "1" } else { "0" }
            ));
        }
    }
    Ok(view(&app, settings))
}

/// Called by inject.js with Messenger's Preferences, which the phone app or
/// Messenger's own dialog may have changed.
#[tauri::command]
pub fn sync_messenger_prefs<R: Runtime>(app: AppHandle<R>, prefs: MessengerPrefs) -> Result<(), String> {
    let previous = app.state::<MessengerPrefsState>().0.lock().unwrap().replace(prefs.clone());
    if !matches!(previous, Some(previous) if previous.dark_mode == prefs.dark_mode) {
        // fMess's windows (Settings, title bars, menus) look like Messenger.
        app.set_theme(messenger_theme(Some(&prefs.dark_mode)));
    }
    if current(&app).messenger_dark_mode.as_deref() == Some(prefs.dark_mode.as_str()) {
        return Ok(());
    }
    change(&app, |settings| settings.messenger_dark_mode = Some(prefs.dark_mode))?;
    services::update_rail(&app);
    Ok(())
}

/// Called by the page when the theme's sidebar button folds its sidebar away
/// or back; with a theme that draws the rail, that folds the rail.
#[tauri::command]
pub fn set_sidebar_collapsed<R: Runtime>(app: AppHandle<R>, collapsed: bool) -> Result<(), String> {
    if current(&app).sidebar_collapsed == collapsed {
        return Ok(());
    }
    change(&app, |settings| settings.sidebar_collapsed = collapsed)?;
    if let Some(window) = services::main_window(&app) {
        services::layout(&window);
    }
    Ok(())
}

/// Called by inject.js when a pane has been dragged to a new width.
#[tauri::command]
pub fn set_pane_widths<R: Runtime>(app: AppHandle<R>, widths: PaneWidths) -> Result<(), String> {
    if current(&app).pane_widths == widths {
        return Ok(());
    }
    change(&app, |settings| settings.pane_widths = widths).map(drop)
}

/// The window theme for Messenger's dark mode setting; None follows the system.
pub fn messenger_theme(dark_mode: Option<&str>) -> Option<Theme> {
    match dark_mode? {
        "ENABLED" => Some(Theme::Dark),
        "USE_SYSTEM" => None,
        _ => Some(Theme::Light),
    }
}

/// Changes one of Messenger's Preferences from the Settings window: soundsOn,
/// darkMode or muteFor (milliseconds, -1 until turned off, 0 to turn it off).
#[tauri::command]
pub fn set_messenger_pref<R: Runtime>(app: AppHandle<R>, name: String, value: serde_json::Value) -> Result<(), String> {
    let webview = services::webview(&app, Service::Messenger)
        .ok_or_else(|| pick(i18n::current(&app), "fMess's window is closed", "Cửa sổ fMess đang đóng"))?;
    if let ("darkMode", Some(mode)) = (name.as_str(), value.as_str()) {
        // Recolour fMess's windows together with the page, not after it reports.
        app.set_theme(messenger_theme(Some(mode)));
    }
    let call = serde_json::to_string(&(name, value)).expect("JSON serializes");
    webview
        .eval(format!("window.__fmessSetMessengerPref?.(...{call});"))
        .map_err(|err| err.to_string())
}

/// Opens Messenger's own Preferences in fMess's window, or its Active Status
/// page, for what the Settings window leaves to it.
#[tauri::command]
pub fn open_messenger_preferences<R: Runtime>(app: AppHandle<R>, section: Option<String>) {
    crate::show_main_window(&app);
    if let Some(webview) = services::webview(&app, Service::Messenger) {
        let section = serde_json::to_string(&section).expect("JSON serializes");
        let _ = webview.eval(format!("window.__fmessOpenMessengerPreferences?.({section});"));
    }
}

/// Text sizes, as the Settings window lists them and View → Zoom steps through.
const ZOOM_STEPS: [f64; 11] = [0.5, 0.67, 0.75, 0.8, 0.9, 1.0, 1.1, 1.25, 1.5, 1.75, 2.0];

/// The View menu's Zoom In (1), Zoom Out (-1) and Actual Size (0).
pub fn zoom_by<R: Runtime>(app: &AppHandle<R>, step: i32) {
    let now = current(app).zoom;
    let nearest = (0..ZOOM_STEPS.len())
        .min_by(|&a, &b| (ZOOM_STEPS[a] - now).abs().total_cmp(&(ZOOM_STEPS[b] - now).abs()))
        .unwrap_or(0);
    let zoom = match step {
        0 => 1.0,
        _ => ZOOM_STEPS[nearest.saturating_add_signed(step as isize).min(ZOOM_STEPS.len() - 1)],
    };
    if zoom == now {
        return;
    }
    if let Err(err) = change(app, |settings| settings.zoom = zoom) {
        eprintln!("could not save the text size: {err}");
    }
    for webview in services::webviews(app) {
        let _ = webview.set_zoom(zoom);
    }
}

/// The settings inject.js works with. Facebook clears localStorage keys it
/// doesn't know within seconds, so they reach the page from here: in the
/// window's initialization script, and again after every page load.
pub fn page_settings<R: Runtime>(app: &AppHandle<R>, settings: &Settings, service: Service) -> String {
    let theme = match service {
        Service::Messenger => themes::for_page(app, settings.theme.as_deref()),
        Service::Instagram => themes::for_scoped_page(app, settings.theme.as_deref()),
    };
    serde_json::json!({
        "macCalls": settings.mac_calls,
        "blockTyping": settings.block_typing,
        "hideAvatars": settings.hide_avatars,
        "spellCheck": settings.spell_check,
        "theme": theme,
        "paneWidths": settings.pane_widths,
        "sidebarCollapsed": settings.sidebar_collapsed,
        // For the notifications the page writes itself.
        "lang": Lang::from_setting(settings.language.as_deref()),
    })
    .to_string()
}

/// Hands the current settings to the loaded page of `webview`.
pub fn page_settings_script<R: Runtime>(webview: &Webview<R>, settings: &Settings) -> String {
    let service = Service::from_label(webview.label()).unwrap_or_default();
    format!("window.__fmessApplySettings?.({});", page_settings(webview.app_handle(), settings, service))
}

/// Goes back to Messenger's own look (the theme was removed).
pub fn clear_theme<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    change(app, |settings| settings.theme = None)?;
    icon::apply(app);
    services::update_rail(app);
    for webview in services::webviews(app) {
        let _ = webview.eval(page_settings_script(&webview, &current(app)));
    }
    Ok(())
}

pub fn set_last_service<R: Runtime>(app: &AppHandle<R>, service: Service) -> Result<(), String> {
    change(app, |settings| settings.last_service = service).map(drop)
}

/// Called by the icon commands (icon.rs), which keep the picture itself.
pub fn set_custom_icon<R: Runtime>(app: &AppHandle<R>, on: bool) -> Result<(), String> {
    change(app, |settings| settings.custom_icon = on).map(drop)
}

/// Renames fMess's open windows and tray icon; the Dock, the menu bar and the
/// Start menu keep the name it was installed with.
fn apply_app_name<R: Runtime>(app: &AppHandle<R>, name: &str) {
    if let Some(window) = services::main_window(app) {
        let _ = window.set_title(name);
    }
    for (label, window) in app.webview_windows() {
        if label != SETTINGS_WINDOW {
            let _ = window.set_title(name);
        }
    }
    menu::set_tooltip(app, name);
}

/// Changes the settings and saves them. The lock is held from reading to
/// saving, so changes made at the same time (by the page and the Settings
/// window) don't undo each other, and the settings stay as they were when
/// they can't be saved.
fn change<R: Runtime>(app: &AppHandle<R>, edit: impl FnOnce(&mut Settings)) -> Result<Settings, String> {
    let state = app.state::<SettingsState>();
    let mut settings = state.0.lock().unwrap();
    let mut next = settings.clone();
    edit(&mut next);
    save(app, &next)?;
    *settings = next.clone();
    Ok(next)
}

fn view<R: Runtime>(app: &AppHandle<R>, settings: Settings) -> SettingsView {
    SettingsView {
        lang: Lang::from_setting(settings.language.as_deref()),
        settings,
        launch_at_login: app.autolaunch().is_enabled().unwrap_or(false),
        platform: std::env::consts::OS,
        messenger: app.state::<MessengerPrefsState>().0.lock().unwrap().clone(),
    }
}

/// Errors are in the language of the settings being saved: `change` holds
/// the settings' lock, so `i18n::current` can't be asked here.
fn save<R: Runtime>(app: &AppHandle<R>, settings: &Settings) -> Result<(), String> {
    #[cfg(debug_assertions)]
    if IN_MEMORY.load(std::sync::atomic::Ordering::Relaxed) {
        return Ok(());
    }
    let lang = Lang::from_setting(settings.language.as_deref());
    let path = settings_path(app).ok_or(pick(lang, "no config directory", "Không tìm thấy thư mục cài đặt"))?;
    let json = serde_json::to_vec_pretty(settings).expect("settings serialize to JSON");
    write_whole(&path, &json).map_err(|err| match lang {
        Lang::En => format!("could not save settings to {}: {err}", path.display()),
        Lang::Vi => format!("Không lưu được cài đặt vào {}: {err}", path.display()),
    })
}

/// Writes the file beside itself, then moves it over the old one: if fMess
/// stops halfway, the old file is still whole (moving replaces the file on
/// Windows too).
fn write_whole(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let temporary = path.with_extension("json.tmp");
    path.parent().map_or(Ok(()), fs::create_dir_all)?;
    fs::write(&temporary, bytes)?;
    fs::rename(&temporary, path)
}

fn settings_path<R: Runtime>(app: &AppHandle<R>) -> Option<PathBuf> {
    let dir = app.path().app_config_dir().ok()?;
    Some(dir.join("settings.json"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instagram_is_opt_in() {
        let settings: Settings = serde_json::from_str("{}").unwrap();
        assert!(!settings.instagram);
        assert!(settings.instagram_icon);
    }

    #[test]
    fn an_unknown_service_keeps_the_other_settings() {
        let settings: Settings = serde_json::from_str(r#"{"zoom": 1.25, "instagram": true, "lastService": "whatsapp"}"#).unwrap();
        assert_eq!(settings.last_service, Service::Messenger);
        assert_eq!(settings.zoom, 1.25);
        assert!(settings.instagram);
        let settings: Settings = serde_json::from_str(r#"{"lastService": 2}"#).unwrap();
        assert_eq!(settings.last_service, Service::Messenger);
        let settings: Settings = serde_json::from_str(r#"{"lastService": "instagram"}"#).unwrap();
        assert_eq!(settings.last_service, Service::Instagram);
    }

    #[cfg(debug_assertions)]
    #[test]
    fn overrides_only_the_given_settings() {
        let settings = Settings {
            zoom: 1.25,
            ..Settings::default()
        };
        let merged = merge(&settings, serde_json::json!({ "instagram": true, "lastService": "instagram" })).unwrap();
        assert!(merged.instagram);
        assert_eq!(merged.last_service, Service::Instagram);
        assert_eq!(merged.zoom, 1.25);
        assert!(merge(&settings, serde_json::json!({ "instagram": "yes" })).is_err());
    }

    #[test]
    fn writes_settings_whole() {
        let dir = std::env::temp_dir().join(format!("fmess-settings-test-{}", std::process::id()));
        let path = dir.join("settings.json");
        write_whole(&path, b"{\"zoom\": 1.0}").unwrap();
        write_whole(&path, b"{\"zoom\": 1.25}").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "{\"zoom\": 1.25}");
        assert!(!path.with_extension("json.tmp").exists(), "the temporary file is left behind");
        fs::remove_dir_all(dir).unwrap();
    }
}
