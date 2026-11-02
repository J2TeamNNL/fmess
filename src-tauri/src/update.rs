//! New versions come from the GitHub releases (see plugins.updater in
//! tauri.conf.json); the release workflow signs them, and fMess installs only
//! what carries that signature.

use std::{sync::Mutex, time::Duration};

use tauri::{AppHandle, Runtime};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use tauri_plugin_updater::{Update, UpdaterExt};

use crate::i18n::{self, pick, Lang};

/// fMess often runs for days from the tray, so it looks again now and then.
const CHECK_EVERY: Duration = Duration::from_secs(12 * 60 * 60);

/// A version the user chose to install later, so it isn't offered again
/// until they check for updates themselves.
static POSTPONED: Mutex<Option<String>> = Mutex::new(None);

/// Looks for updates in the background: shortly after launch, then every
/// CHECK_EVERY. Development builds don't, so they never replace themselves.
pub fn check_in_background<R: Runtime>(app: &AppHandle<R>) {
    if cfg!(debug_assertions) {
        return;
    }
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        // Let the window load first.
        tokio::time::sleep(Duration::from_secs(20)).await;
        loop {
            check(&app, false).await;
            tokio::time::sleep(CHECK_EVERY).await;
        }
    });
}

/// The menu's Check for Updates…: answers even when there is nothing new.
pub fn check_now<R: Runtime>(app: &AppHandle<R>) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move { check(&app, true).await });
}

async fn check<R: Runtime>(app: &AppHandle<R>, asked: bool) {
    let found = match app.updater() {
        Ok(updater) => updater.check().await,
        Err(err) => Err(err),
    };
    let lang = i18n::current(app);
    match found {
        Ok(Some(update)) => {
            let postponed = POSTPONED.lock().unwrap().as_deref() == Some(update.version.as_str());
            if asked || !postponed {
                offer(app, update);
            }
        }
        Ok(None) if asked => {
            let version = app.package_info().version.to_string();
            app.dialog()
                .message(match lang {
                    Lang::En => format!("fMess {version} is the newest version."),
                    Lang::Vi => format!("fMess {version} là phiên bản mới nhất."),
                })
                .title(pick(lang, "You're up to date", "Bạn đang dùng bản mới nhất"))
                .show(|_| {});
        }
        Err(err) if asked => {
            app.dialog()
                .message(match lang {
                    Lang::En => format!("fMess couldn't check for updates.\n\n{err}"),
                    Lang::Vi => format!("fMess không kiểm tra được bản cập nhật.\n\n{err}"),
                })
                .title(pick(lang, "Check for Updates", "Kiểm tra cập nhật"))
                .kind(MessageDialogKind::Warning)
                .show(|_| {});
        }
        _ => {
            #[cfg(debug_assertions)]
            if let Err(err) = found {
                eprintln!("[fmess] update check failed: {err}");
            }
        }
    }
}

fn offer<R: Runtime>(app: &AppHandle<R>, update: Update) {
    let lang = i18n::current(app);
    let current = app.package_info().version.to_string();
    let available = match lang {
        Lang::En => format!("fMess {} is available. You have {current}.", update.version),
        Lang::Vi => format!("Đã có fMess {}. Bạn đang dùng bản {current}.", update.version),
    };
    let notes = update.body.as_deref().and_then(|notes| notes_in(notes, lang));
    let message = match notes {
        Some(notes) => format!("{available}\n\n{notes}"),
        None => available,
    };
    let handle = app.clone();
    app.dialog()
        .message(message)
        .title(pick(lang, "A new version of fMess is available", "Đã có phiên bản fMess mới"))
        .buttons(MessageDialogButtons::OkCancelCustom(
            pick(lang, "Install and Restart", "Cài và khởi động lại").into(),
            pick(lang, "Later", "Để sau").into(),
        ))
        .show(move |install| {
            if !install {
                *POSTPONED.lock().unwrap() = Some(update.version.clone());
                return;
            }
            tauri::async_runtime::spawn(async move {
                match update.download_and_install(|_, _| {}, || {}).await {
                    Ok(()) => handle.restart(),
                    Err(err) => {
                        handle
                            .dialog()
                            .message(match lang {
                                Lang::En => format!("fMess couldn't install the update.\n\n{err}"),
                                Lang::Vi => format!("fMess không cài được bản cập nhật.\n\n{err}"),
                            })
                            .title(pick(lang, "Update failed", "Cập nhật không thành công"))
                            .kind(MessageDialogKind::Error)
                            .show(|_| {});
                    }
                }
            });
        });
}

/// The release notes in `lang`: they may hold the English, then a line
/// "---", then the Vietnamese (older fMess versions show both).
fn notes_in(notes: &str, lang: Lang) -> Option<&str> {
    let mut parts = notes.splitn(2, "\n---\n");
    let english = parts.next().unwrap_or_default();
    let notes = match (lang, parts.next()) {
        (Lang::Vi, Some(vietnamese)) => vietnamese,
        _ => english,
    };
    Some(notes.trim()).filter(|notes| !notes.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notes_in_each_language() {
        let both = "What's new:\n• A\n---\nCó gì mới:\n• A";
        assert_eq!(notes_in(both, Lang::En), Some("What's new:\n• A"));
        assert_eq!(notes_in(both, Lang::Vi), Some("Có gì mới:\n• A"));
        assert_eq!(notes_in("Only English", Lang::Vi), Some("Only English"));
        assert_eq!(notes_in("  ", Lang::En), None);
    }
}
