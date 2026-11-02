//! fMess's own words, in English or Vietnamese: its menus, dialogs, Settings
//! window and the notifications it writes itself. Messenger's page follows
//! the Facebook account's language instead.
//!
//! Each place keeps both texts side by side (`pick`, or a `match` on the
//! language for formatted ones), so a text can't go missing.

use serde::Serialize;
use tauri::{AppHandle, Runtime};

use crate::settings;

/// As the Settings window and the page script take it: "en" or "vi".
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Lang {
    En,
    Vi,
}

impl Lang {
    /// The language chosen in Settings ("en" or "vi"); none follows the system.
    pub fn from_setting(setting: Option<&str>) -> Self {
        match setting {
            Some("en") => Lang::En,
            Some("vi") => Lang::Vi,
            _ => system(),
        }
    }
}

/// The language fMess speaks now.
pub fn current<R: Runtime>(app: &AppHandle<R>) -> Lang {
    Lang::from_setting(settings::current(app).language.as_deref())
}

/// The system's preferred language: Vietnamese when it is, English otherwise.
pub fn system() -> Lang {
    from_locale(sys_locale::get_locale().as_deref())
}

fn from_locale(locale: Option<&str>) -> Lang {
    match locale.and_then(|locale| locale.get(..2)) {
        Some(code) if code.eq_ignore_ascii_case("vi") => Lang::Vi,
        _ => Lang::En,
    }
}

/// The text in `lang`.
pub fn pick(lang: Lang, en: &'static str, vi: &'static str) -> &'static str {
    match lang {
        Lang::En => en,
        Lang::Vi => vi,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_system_language() {
        assert_eq!(from_locale(Some("vi-VN")), Lang::Vi);
        assert_eq!(from_locale(Some("VI")), Lang::Vi);
        assert_eq!(from_locale(Some("en-US")), Lang::En);
        assert_eq!(from_locale(Some("fr")), Lang::En);
        assert_eq!(from_locale(Some("v")), Lang::En);
        assert_eq!(from_locale(None), Lang::En);
    }
}
