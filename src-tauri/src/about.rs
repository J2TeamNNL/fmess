//! fMess's About panel: the standard macOS one, with credits that link to the
//! project and its author.

use tauri::{AppHandle, Runtime};

use crate::i18n::{self, Lang};

pub const REPOSITORY: &str = "https://github.com/toanbku/fmess";
const AUTHOR: (&str, &str) = ("Toan Ho", "https://github.com/toanbku");
const WEBSITE: &str = "https://fmess.app";

#[cfg(target_os = "macos")]
pub fn show<R: Runtime>(app: &AppHandle<R>) {
    use objc2::{runtime::AnyObject, MainThreadMarker};
    use objc2_app_kit::{
        NSAboutPanelOptionCredits, NSApplication, NSColor, NSFont, NSFontAttributeName, NSForegroundColorAttributeName,
        NSLinkAttributeName, NSMutableParagraphStyle, NSParagraphStyleAttributeName, NSTextAlignment,
    };
    use objc2_foundation::{NSDictionary, NSMutableAttributedString, NSRange, NSString, NSURL};

    // Menu items are handled on the main thread.
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    let (name, profile) = AUTHOR;
    let site = WEBSITE.trim_start_matches("https://");
    let repo = REPOSITORY.trim_start_matches("https://");
    let text = format!("{}\n{site} · {repo}", made_by(i18n::current(app), name));
    // NSString ranges count UTF-16 units.
    let range_of = |part: &str| {
        let start = text.find(part).expect("part of the text");
        NSRange::new(text[..start].encode_utf16().count(), part.encode_utf16().count())
    };
    let credits = NSMutableAttributedString::from_nsstring(&NSString::from_str(&text));
    let whole = NSRange::new(0, text.encode_utf16().count());
    let centred = NSMutableParagraphStyle::new();
    centred.setAlignment(NSTextAlignment::Center);
    unsafe {
        credits.addAttribute_value_range(NSFontAttributeName, &NSFont::systemFontOfSize(11.0), whole);
        // labelColor follows light and dark mode.
        credits.addAttribute_value_range(NSForegroundColorAttributeName, &NSColor::labelColor(), whole);
        credits.addAttribute_value_range(NSParagraphStyleAttributeName, &centred, whole);
        for (part, link) in [(name, profile), (site, WEBSITE), (repo, REPOSITORY)] {
            if let Some(url) = NSURL::URLWithString(&NSString::from_str(link)) {
                credits.addAttribute_value_range(NSLinkAttributeName, &url, range_of(part));
            }
        }
        let options: objc2::rc::Retained<NSDictionary<NSString, AnyObject>> =
            NSDictionary::from_slices(&[NSAboutPanelOptionCredits], &[credits.as_ref() as &AnyObject]);
        let app = NSApplication::sharedApplication(mtm);
        app.orderFrontStandardAboutPanelWithOptions(&options);
        app.activate();
    }
}

/// Elsewhere there is no About panel to fill in, so a dialog says the same.
#[cfg(not(target_os = "macos"))]
pub fn show<R: Runtime>(app: &AppHandle<R>) {
    use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};

    let lang = i18n::current(app);
    let version = app.package_info().version.to_string();
    let version = match lang {
        Lang::En => format!("Version {version}"),
        Lang::Vi => format!("Phiên bản {version}"),
    };
    let (name, _) = AUTHOR;
    let handle = app.clone();
    app.dialog()
        .message(format!("{version}\n\n{}\n{WEBSITE}", made_by(lang, name)))
        .title("fMess")
        .buttons(MessageDialogButtons::OkCancelCustom(
            i18n::pick(lang, "Open GitHub", "Mở GitHub").into(),
            i18n::pick(lang, "Close", "Đóng").into(),
        ))
        .show(move |open| {
            if open {
                if let Ok(url) = tauri::Url::parse(REPOSITORY) {
                    crate::links::open_external(&handle, &url);
                }
            }
        });
}

/// The credits' first line, naming the author.
fn made_by(lang: Lang, name: &str) -> String {
    match lang {
        Lang::En => format!("Made with ♥ by {name}"),
        Lang::Vi => format!("Được làm với ♥ bởi {name}"),
    }
}
