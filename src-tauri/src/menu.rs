use tauri::{
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, LogicalPosition, Manager, Runtime, WebviewWindow,
};

use crate::{
    about,
    i18n::{self, pick, Lang},
    settings, show_main_window, update,
    webview::{self, messages_url},
    MAIN_WINDOW,
};

const TRAY: &str = "main";

const SHOW: &str = "show";
const MESSAGES: &str = "messages";
const RELOAD: &str = "reload";
const BACK: &str = "back";
const FORWARD: &str = "forward";
const SEARCH: &str = "search";
const NEW_MESSAGE: &str = "new-message";
const NEXT_CHAT: &str = "next-chat";
const PREVIOUS_CHAT: &str = "previous-chat";
const SETTINGS: &str = "settings";
const LOG_OUT: &str = "log-out";
const ZOOM_IN: &str = "zoom-in";
const ZOOM_OUT: &str = "zoom-out";
const ZOOM_RESET: &str = "zoom-reset";
const ABOUT: &str = "about";
const CHECK_FOR_UPDATES: &str = "check-for-updates";
const QUIT: &str = "quit";

pub fn install<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let lang = i18n::current(app);
    let mut tray = TrayIconBuilder::with_id(TRAY)
        .tooltip("fMess")
        .menu(&tray_menu(app, lang)?)
        // Left click shows the window; the menu stays on right click.
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main_window(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;

    #[cfg(target_os = "macos")]
    app.set_menu(app_menu(app, lang)?)?;
    Ok(())
}

/// Builds the menus again in another language (chosen in Settings).
pub fn apply_language<R: Runtime>(app: &AppHandle<R>, lang: Lang) -> tauri::Result<()> {
    if let Some(tray) = app.tray_by_id(TRAY) {
        tray.set_menu(Some(tray_menu(app, lang)?))?;
    }
    #[cfg(target_os = "macos")]
    app.set_menu(app_menu(app, lang)?)?;
    Ok(())
}

fn tray_menu<R: Runtime>(app: &AppHandle<R>, lang: Lang) -> tauri::Result<Menu<R>> {
    let t = |en, vi| pick(lang, en, vi);
    Menu::with_items(
        app,
        &[
            &MenuItem::with_id(app, SHOW, t("Show fMess", "Mở fMess"), true, None::<&str>)?,
            &MenuItem::with_id(app, MESSAGES, t("Open Chats", "Mở đoạn chat"), true, None::<&str>)?,
            &MenuItem::with_id(app, RELOAD, t("Reload", "Tải lại"), true, None::<&str>)?,
            &MenuItem::with_id(app, SETTINGS, t("Settings…", "Cài đặt…"), true, None::<&str>)?,
            &MenuItem::with_id(app, CHECK_FOR_UPDATES, t("Check for Updates…", "Kiểm tra cập nhật…"), true, None::<&str>)?,
            &MenuItem::with_id(app, ABOUT, t("About fMess", "Giới thiệu về fMess"), true, None::<&str>)?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, QUIT, t("Quit fMess", "Thoát fMess"), true, None::<&str>)?,
        ],
    )
}

/// macOS needs a menu bar for copy/paste shortcuts; it has navigation too.
/// Tauri's default menu (App, File, Edit, View, Window, Help) with Go before
/// Window, built here so every item can be named in fMess's language; the
/// Vietnamese names are the ones macOS itself uses.
#[cfg(target_os = "macos")]
fn app_menu<R: Runtime>(app: &AppHandle<R>, lang: Lang) -> tauri::Result<Menu<R>> {
    use tauri::menu::{Submenu, HELP_SUBMENU_ID, WINDOW_SUBMENU_ID};

    let t = |en, vi| pick(lang, en, vi);
    // Its About fMess is fMess's own (with credits), followed by Check for
    // Updates…, a separator and Settings.
    let app_submenu = Submenu::with_items(
        app,
        &app.package_info().name,
        true,
        &[
            &MenuItem::with_id(app, ABOUT, t("About fMess", "Giới thiệu về fMess"), true, None::<&str>)?,
            &MenuItem::with_id(app, CHECK_FOR_UPDATES, t("Check for Updates…", "Kiểm tra cập nhật…"), true, None::<&str>)?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, SETTINGS, t("Settings…", "Cài đặt…"), true, Some("CmdOrCtrl+,"))?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::services(app, Some(t("Services", "Dịch vụ")))?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::hide(app, Some(t("Hide fMess", "Ẩn fMess")))?,
            &PredefinedMenuItem::hide_others(app, Some(t("Hide Others", "Ẩn các ứng dụng khác")))?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::quit(app, Some(t("Quit fMess", "Thoát fMess")))?,
        ],
    )?;
    let file = Submenu::with_items(
        app,
        t("File", "Tệp"),
        true,
        &[&PredefinedMenuItem::close_window(app, Some(t("Close Window", "Đóng cửa sổ")))?],
    )?;
    let edit = Submenu::with_items(
        app,
        t("Edit", "Sửa"),
        true,
        &[
            &PredefinedMenuItem::undo(app, Some(t("Undo", "Hoàn tác")))?,
            &PredefinedMenuItem::redo(app, Some(t("Redo", "Làm lại")))?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::cut(app, Some(t("Cut", "Cắt")))?,
            &PredefinedMenuItem::copy(app, Some(t("Copy", "Sao chép")))?,
            &PredefinedMenuItem::paste(app, Some(t("Paste", "Dán")))?,
            &PredefinedMenuItem::select_all(app, Some(t("Select All", "Chọn tất cả")))?,
        ],
    )?;
    let view = Submenu::with_items(
        app,
        t("View", "Xem"),
        true,
        &[
            &MenuItem::with_id(app, ZOOM_RESET, t("Actual Size", "Kích thước thực"), true, Some("CmdOrCtrl+0"))?,
            &MenuItem::with_id(app, ZOOM_IN, t("Zoom In", "Phóng to"), true, Some("CmdOrCtrl+="))?,
            &MenuItem::with_id(app, ZOOM_OUT, t("Zoom Out", "Thu nhỏ"), true, Some("CmdOrCtrl+-"))?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::fullscreen(app, Some(t("Toggle Full Screen", "Bật/tắt toàn màn hình")))?,
        ],
    )?;
    let go = Submenu::with_items(
        app,
        t("Go", "Đi"),
        true,
        &[
            &MenuItem::with_id(app, MESSAGES, t("Chats", "Đoạn chat"), true, Some("CmdOrCtrl+Shift+H"))?,
            &MenuItem::with_id(app, SEARCH, t("Search", "Tìm kiếm"), true, Some("CmdOrCtrl+K"))?,
            &MenuItem::with_id(app, NEW_MESSAGE, t("New Message", "Tin nhắn mới"), true, Some("CmdOrCtrl+N"))?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(
                app,
                NEXT_CHAT,
                t("Next Conversation", "Đoạn chat tiếp theo"),
                true,
                Some("CmdOrCtrl+Shift+]"),
            )?,
            &MenuItem::with_id(
                app,
                PREVIOUS_CHAT,
                t("Previous Conversation", "Đoạn chat trước"),
                true,
                Some("CmdOrCtrl+Shift+["),
            )?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, BACK, t("Back", "Quay lại"), true, Some("CmdOrCtrl+["))?,
            &MenuItem::with_id(app, FORWARD, t("Forward", "Tiến tới"), true, Some("CmdOrCtrl+]"))?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, RELOAD, t("Reload", "Tải lại"), true, Some("CmdOrCtrl+R"))?,
        ],
    )?;
    // With these ids macOS fills the Window menu with the open windows and
    // gives Help its search field.
    let window = Submenu::with_id_and_items(
        app,
        WINDOW_SUBMENU_ID,
        t("Window", "Cửa sổ"),
        true,
        &[
            &PredefinedMenuItem::minimize(app, Some(t("Minimize", "Thu nhỏ")))?,
            &PredefinedMenuItem::maximize(app, Some(t("Zoom", "Thu phóng")))?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::close_window(app, Some(t("Close Window", "Đóng cửa sổ")))?,
        ],
    )?;
    let help = Submenu::with_id_and_items(app, HELP_SUBMENU_ID, t("Help", "Trợ giúp"), true, &[])?;
    Menu::with_items(app, &[&app_submenu, &file, &edit, &view, &go, &window, &help])
}

/// The old desktop app's account menu: inject.js puts the account's picture
/// above the chats, and clicking it opens this under the picture.
#[tauri::command]
pub fn show_account_menu<R: Runtime>(
    window: WebviewWindow<R>,
    name: String,
    x: f64,
    y: f64,
) -> tauri::Result<()> {
    let app = window.app_handle();
    let lang = i18n::current(app);
    let t = |en, vi| pick(lang, en, vi);
    let menu = Menu::with_items(
        app,
        &[
            &MenuItem::new(app, name, false, None::<&str>)?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, SETTINGS, t("Settings…", "Cài đặt…"), true, None::<&str>)?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, LOG_OUT, t("Log Out", "Đăng xuất"), true, None::<&str>)?,
        ],
    )?;
    // The page measures in CSS pixels, which fMess's zoom setting scales.
    let zoom = settings::current(app).zoom;
    window.popup_menu_at(&menu, LogicalPosition::new(x * zoom, y * zoom))
}

/// Handles clicks from the tray menu, the macOS menu bar and the account menu.
pub fn handle<R: Runtime>(app: &AppHandle<R>, event: MenuEvent) {
    run(app, event.id().as_ref());
}

/// The menu bar's shortcuts, which only macOS has: on Windows and Linux the
/// page catches the same keys (Ctrl+, for Settings...) and asks for them here.
#[tauri::command]
pub fn menu_shortcut<R: Runtime>(app: AppHandle<R>, id: String) {
    if [MESSAGES, SETTINGS, RELOAD, BACK, FORWARD, ZOOM_IN, ZOOM_OUT, ZOOM_RESET].contains(&id.as_str()) {
        run(&app, &id);
    }
}

fn run<R: Runtime>(app: &AppHandle<R>, id: &str) {
    let window = app.get_webview_window(MAIN_WINDOW);
    match id {
        SHOW => show_main_window(app),
        MESSAGES => {
            show_main_window(app);
            if let Some(window) = window {
                let _ = window.navigate(messages_url());
            }
        }
        RELOAD => {
            if let Some(window) = window {
                let _ = window.reload();
            }
        }
        BACK => {
            if let Some(window) = window {
                let _ = window.eval("history.back()");
            }
        }
        FORWARD => {
            if let Some(window) = window {
                let _ = window.eval("history.forward()");
            }
        }
        SETTINGS => webview::open_settings_window(app),
        ABOUT => about::show(app),
        CHECK_FOR_UPDATES => update::check_now(app),
        ZOOM_IN => settings::zoom_by(app, 1),
        ZOOM_OUT => settings::zoom_by(app, -1),
        ZOOM_RESET => settings::zoom_by(app, 0),
        LOG_OUT => {
            if let Some(window) = window {
                let _ = window.eval("window.__fmessLogOut?.()");
            }
        }
        // The shortcuts live in inject.js; the menu shows them and forwards to them.
        SEARCH | NEW_MESSAGE | NEXT_CHAT | PREVIOUS_CHAT => {
            let call = match id {
                SEARCH => "'search'",
                NEW_MESSAGE => "'newMessage'",
                NEXT_CHAT => "'moveChat', 1",
                _ => "'moveChat', -1",
            };
            if let Some(window) = window {
                let _ = window.eval(format!("window.__fmessShortcut?.({call})"));
            }
        }
        QUIT => app.exit(0),
        _ => {}
    }
}
