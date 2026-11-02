use std::{env, fs, path::Path};

fn main() {
    // The script fMess runs in the Messenger page is written in parts
    // (src/inject/), one per concern, sharing one function's scope; joined in
    // file-name order they make the script webview.rs puts in the page.
    let parts = Path::new("src/inject");
    println!("cargo:rerun-if-changed={}", parts.display());
    let mut files: Vec<_> = fs::read_dir(parts)
        .expect("src/inject holds the page script's parts")
        .filter_map(|entry| Some(entry.ok()?.path()))
        .filter(|path| path.extension().is_some_and(|ext| ext == "js"))
        .collect();
    files.sort();
    let script: String = files
        .iter()
        .map(|file| fs::read_to_string(file).unwrap_or_else(|err| panic!("{}: {err}", file.display())))
        .collect();
    let out = Path::new(&env::var("OUT_DIR").expect("set by Cargo")).join("inject.js");
    fs::write(out, script).expect("could not write the page script");

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
            ])),
    )
    .expect("failed to run tauri-build");
}
