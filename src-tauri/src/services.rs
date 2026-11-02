use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, Rect, Runtime, Url, Webview, Window};

use crate::{badge, notify, settings, themes, webview as pages, MAIN_WINDOW};

/// The web view of the bar down the left (ui/rail.html), which switches
/// between the services and is drawn by the theme.
pub const RAIL: &str = "rail";
/// In logical pixels, under the macOS window buttons.
pub const RAIL_WIDTH: f64 = 68.0;

/// A site fMess shows in its window, each in a web view of its own.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Service {
    #[default]
    Messenger,
    Instagram,
}

impl Service {
    pub const ALL: [Service; 2] = [Service::Messenger, Service::Instagram];

    pub fn label(self) -> &'static str {
        match self {
            Service::Messenger => "messenger",
            Service::Instagram => "instagram",
        }
    }

    pub fn from_label(label: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|service| service.label() == label)
    }
}

/// Your account as the Messenger page last reported it, for the rail.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Account {
    picture: Option<String>,
    name: String,
}

static ACCOUNT: Mutex<Option<Account>> = Mutex::new(None);

pub fn webview<R: Runtime>(app: &AppHandle<R>, service: Service) -> Option<Webview<R>> {
    app.get_webview(service.label())
}

/// The web views of the services that are on.
pub fn webviews<R: Runtime>(app: &AppHandle<R>) -> Vec<Webview<R>> {
    Service::ALL.into_iter().filter_map(|service| webview(app, service)).collect()
}

pub fn main_window<R: Runtime>(app: &AppHandle<R>) -> Option<Window<R>> {
    app.get_window(MAIN_WINDOW)
}

/// The services with a web view whose icon the rail shows.
fn on_rail<R: Runtime>(app: &AppHandle<R>, settings: &settings::Settings) -> Vec<Service> {
    Service::ALL
        .into_iter()
        .filter(|&service| webview(app, service).is_some())
        .filter(|&service| service != Service::Instagram || settings.instagram_icon)
        .collect()
}

/// A theme with a rail section draws the rail, unless its sidebar button
/// folded it away; otherwise it shows only when there is more than one
/// service to switch between.
fn rail_visible<R: Runtime>(app: &AppHandle<R>, settings: &settings::Settings) -> bool {
    if themes::draws_rail(app, settings.theme.as_deref()) {
        !settings.sidebar_collapsed
    } else {
        on_rail(app, settings).len() > 1
    }
}

/// Whether layout last showed the rail: resizing reuses it rather than
/// reading the theme again.
static RAIL_SHOWN: AtomicBool = AtomicBool::new(false);

/// Places the web views in fMess's window: the rail at the left when it is
/// shown, the services beside it.
pub fn layout<R: Runtime>(window: &Window<R>) {
    let app = window.app_handle();
    let shown = rail_visible(app, &settings::current(app));
    RAIL_SHOWN.store(shown, Ordering::Relaxed);
    place(window, shown);
}

/// The same after the window changed size.
pub fn relayout<R: Runtime>(window: &Window<R>) {
    place(window, RAIL_SHOWN.load(Ordering::Relaxed));
}

pub fn place<R: Runtime>(window: &Window<R>, rail: bool) {
    let (Ok(size), Ok(scale)) = (window.inner_size(), window.scale_factor()) else {
        return;
    };
    let size = size.to_logical::<f64>(scale);
    let left = if rail { RAIL_WIDTH.min(size.width) } else { 0.0 };
    for webview in window.webviews() {
        let bounds = if webview.label() == RAIL {
            let _ = if rail { webview.show() } else { webview.hide() };
            Rect {
                position: LogicalPosition::new(0.0, 0.0).into(),
                size: LogicalSize::new(left, size.height).into(),
            }
        } else if Service::from_label(webview.label()).is_some() {
            Rect {
                position: LogicalPosition::new(left, 0.0).into(),
                size: LogicalSize::new(size.width - left, size.height).into(),
            }
        } else {
            continue;
        };
        let _ = webview.set_bounds(bounds);
    }
}

/// Shows `service` (when it has a web view) and hides the others.
pub fn select<R: Runtime>(app: &AppHandle<R>, service: Service) {
    let Some(selected) = webview(app, service) else {
        return;
    };
    for other in Service::ALL.into_iter().filter(|&other| other != service) {
        if let Some(other) = webview(app, other) {
            let _ = other.hide();
        }
    }
    let _ = selected.show();
    let _ = selected.set_focus();
    if settings::current(app).last_service != service {
        if let Err(err) = settings::set_last_service(app, service) {
            eprintln!("could not save the service: {err}");
        }
    }
    let _ = app.emit_to(RAIL, "service-selected", service);
}

/// The selected service, or Messenger when that one has no web view.
pub fn selected<R: Runtime>(app: &AppHandle<R>) -> Service {
    let last = settings::current(app).last_service;
    if webview(app, last).is_some() {
        last
    } else {
        Service::Messenger
    }
}

/// Opens Instagram's web view or closes it, after the Settings window turned
/// the service on or off. Closing forgets its unread chats and notifications.
pub fn set_instagram<R: Runtime>(app: &AppHandle<R>, on: bool) {
    match (on, webview(app, Service::Instagram)) {
        (true, None) => {
            // Built away from the main thread, as the Settings window is: on
            // Windows, building a web view there from a command deadlocks.
            let app = app.clone();
            tauri::async_runtime::spawn(async move {
                // Settings may have turned it off (or on again) before this ran.
                let settings = settings::current(&app);
                let Some(window) = main_window(&app).filter(|_| settings.instagram && webview(&app, Service::Instagram).is_none()) else {
                    return;
                };
                match pages::add_instagram(&app, &window, &settings) {
                    Ok(instagram) if !settings::current(&app).instagram => {
                        let _ = instagram.close();
                        return;
                    }
                    Ok(_) => {}
                    Err(err) => eprintln!("could not open Instagram: {err}"),
                }
                update_rail(&app);
            });
        }
        (false, Some(instagram)) => {
            if selected(app) == Service::Instagram {
                select(app, Service::Messenger);
            }
            let _ = instagram.close();
            badge::forget(app, Service::Instagram);
            notify::clear_service(Service::Instagram);
            update_rail(app);
        }
        _ => {}
    }
}

/// Called by the rail.
#[tauri::command]
pub fn select_service<R: Runtime>(app: AppHandle<R>, service: Service) {
    select(&app, service);
}

/// Called by inject.js with your name and profile picture, which the rail shows.
#[tauri::command]
pub fn set_account<R: Runtime>(app: AppHandle<R>, picture: Option<String>, name: String) -> Result<(), String> {
    // The rail puts the link in CSS: only a parsed https URL, which has its quotes escaped.
    let picture = match picture {
        Some(picture) => {
            let url = Url::parse(&picture).map_err(|err| format!("bad picture link: {err}"))?;
            if url.scheme() != "https" {
                return Err("the picture must be an https link".into());
            }
            Some(url.to_string())
        }
        None => None,
    };
    let account = Account {
        picture,
        name: name.chars().take(200).collect(),
    };
    let mut current = ACCOUNT.lock().unwrap();
    if current.as_ref() != Some(&account) {
        let _ = app.emit_to(RAIL, "account", &account);
        *current = Some(account);
    }
    Ok(())
}

/// Everything the rail shows. Its page takes it at start and when it loads
/// (rail.html); single changes arrive as events.
pub fn rail_state<R: Runtime>(app: &AppHandle<R>) -> serde_json::Value {
    let settings = settings::current(app);
    let dark = settings::messenger_theme(settings.messenger_dark_mode.as_deref()).map(|theme| theme == tauri::Theme::Dark);
    serde_json::json!({
        "titleBar": crate::webview::TITLE_BAR_INSET,
        "theme": themes::for_scoped_page(app, settings.theme.as_deref()),
        "dark": dark,
        "services": on_rail(app, &settings),
        "selected": selected(app),
        "unread": badge::counts(),
        "account": *ACCOUNT.lock().unwrap(),
        "hideAvatars": settings.hide_avatars,
        "lang": crate::i18n::Lang::from_setting(settings.language.as_deref()),
    })
}

/// Hands the rail its state again and shows or hides it, after a change to
/// the theme, the dark mode or the services.
pub fn update_rail<R: Runtime>(app: &AppHandle<R>) {
    if let Some(rail) = app.get_webview(RAIL) {
        let _ = rail.eval(rail_script(app));
    }
    if let Some(window) = main_window(app) {
        layout(&window);
    }
}

pub fn rail_script<R: Runtime>(app: &AppHandle<R>) -> String {
    format!("window.__fmessRail?.({});", rail_state(app))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_round_trip() {
        for service in Service::ALL {
            assert_eq!(Service::from_label(service.label()), Some(service));
        }
        assert_eq!(Service::from_label("main"), None);
        assert_eq!(Service::from_label(RAIL), None);
    }

    #[test]
    fn serializes_as_its_label() {
        for service in Service::ALL {
            let json = serde_json::to_string(&service).unwrap();
            assert_eq!(json, format!("\"{}\"", service.label()));
            assert_eq!(serde_json::from_str::<Service>(&json).unwrap(), service);
        }
    }
}
