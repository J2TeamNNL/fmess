// fMess: an unofficial desktop shell for Messenger on the web.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod about;
mod badge;
mod i18n;
mod icon;
mod links;
mod menu;
mod notify;
#[cfg(debug_assertions)]
mod probe;
mod services;
mod settings;
mod themes;
mod update;
mod webview;
#[cfg(windows)]
mod webview2;
mod window_state;

use tauri::{AppHandle, Manager, Runtime, WindowEvent};
use tauri_plugin_window_state::StateFlags;

pub const MAIN_WINDOW: &str = "main";

/// Passed by the login item, so fMess can start hidden when the user asked.
pub const AUTOSTART_ARG: &str = "--autostart";

fn main() {
    // Without WebView2 no window can open (Windows).
    #[cfg(windows)]
    if !webview2::check() {
        return;
    }
    // Before any web view exists: WebKit asks AppKit once, when it starts.
    #[cfg(target_os = "macos")]
    webview::prefer_overlay_scrollbars();
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            show_main_window(app)
        }))
        .plugin(
            tauri_plugin_window_state::Builder::default()
                // Closing only hides the window, so never restore it hidden.
                .with_state_flags(StateFlags::all() & !StateFlags::VISIBLE)
                .build(),
        )
        .plugin(tauri_plugin_autostart::Builder::new().arg(AUTOSTART_ARG).build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            notify::notify,
            notify::clear_notifications,
            notify::test_notification,
            badge::set_unread_count,
            settings::get_settings,
            settings::update_settings,
            settings::sync_messenger_prefs,
            settings::set_pane_widths,
            settings::set_sidebar_collapsed,
            settings::set_messenger_pref,
            settings::open_messenger_preferences,
            webview::open_settings,
            themes::list_themes,
            themes::add_theme,
            themes::remove_theme,
            icon::choose_app_icon,
            icon::reset_app_icon,
            webview::set_viewer_open,
            menu::show_account_menu,
            menu::menu_shortcut,
            services::select_service,
            services::set_account,
        ])
        .on_menu_event(menu::handle)
        .setup(|app| {
            let handle = app.handle();
            app.manage(window_state::Closed::default());
            #[cfg(windows)]
            notify::register_app_id(handle);
            let settings = settings::init(handle);
            handle.set_theme(settings::messenger_theme(settings.messenger_dark_mode.as_deref()));
            webview::create_main_window(handle, &settings)?;
            menu::install(handle)?;
            icon::apply(handle);
            notify::route_clicks(handle)?;
            update::check_in_background(handle);
            #[cfg(debug_assertions)]
            probe::run(handle.clone());
            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() != MAIN_WINDOW {
                return;
            }
            match event {
                WindowEvent::CloseRequested { api, .. } => {
                    window_state::save_on_close(window);
                    if settings::current(window.app_handle()).close_to_tray {
                        // Keep running in the tray and Dock so messages keep arriving.
                        api.prevent_close();
                        let _ = window.hide();
                    } else {
                        window.app_handle().exit(0);
                    }
                }
                WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => services::relayout(window),
                _ => {}
            }
        })
        .build(tauri::generate_context!())
        .expect("failed to build fMess")
        .run(|app, event| match event {
            tauri::RunEvent::Exit => window_state::save_on_exit(app),
            #[cfg(target_os = "macos")]
            tauri::RunEvent::Reopen { .. } => show_main_window(app),
            _ => {}
        });
}

pub fn show_main_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = services::main_window(app) {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}
