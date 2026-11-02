fn main() {
    // Declaring the app's commands turns on access control for them, which the
    // Messenger page needs before it may call them (see capabilities/).
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(tauri_build::AppManifest::new().commands(&[
                "notify",
                "clear_notifications",
                "set_unread_count",
                "set_viewer_open",
                "get_settings",
                "update_settings",
                "show_account_menu",
                "menu_shortcut",
                "sync_messenger_prefs",
                "set_pane_widths",
                "set_messenger_pref",
                "open_messenger_preferences",
                "test_notification",
                "open_settings",
                "list_themes",
                "add_theme",
                "remove_theme",
                "select_service",
                "set_account",
            ])),
    )
    .expect("failed to run tauri-build");
}
