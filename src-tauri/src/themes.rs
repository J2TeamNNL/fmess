//! Themes change how Messenger looks: CSS that inject.js adds to the page,
//! picked in Settings. Zalo, WhatsApp and Teams come with fMess; other themes
//! are .css files the user adds, kept in the app data folder. A theme is CSS
//! and nothing more: no @import and no url() other than data: ones, so it
//! can't load anything.
//!
//! A theme may start with a comment naming it and asking for light colours:
//!
//! ```css
//! /* fMess theme
//!    name: Zalo
//!    mode: light
//!    preview: #005ae0 #ffffff #ebecf0 #e5f1ff #ffffff
//! */
//! ```
//!
//! The preview colours draw the theme's card in Settings: the bar at the
//! left, the chat list, the chat, your messages and theirs.

use std::{fs, path::PathBuf};

use cssparser::{ParseError, ParseErrorKind, Parser, ParserInput, Token};
use serde::Serialize;
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_dialog::DialogExt;

use crate::{
    i18n::{self, pick, Lang},
    settings,
};

const BUILT_IN: &[(&str, &str)] = &[
    ("zalo", include_str!("../themes/zalo.css")),
    ("whatsapp", include_str!("../themes/whatsapp.css")),
    ("teams", include_str!("../themes/teams.css")),
];
const MAX_SIZE: usize = 2 * 1024 * 1024;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeInfo {
    id: String,
    name: String,
    built_in: bool,
    preview: Option<Vec<String>>,
}

#[tauri::command]
pub fn list_themes<R: Runtime>(app: AppHandle<R>) -> Vec<ThemeInfo> {
    let mut themes: Vec<ThemeInfo> = BUILT_IN
        .iter()
        .map(|(id, css)| ThemeInfo {
            id: id.to_string(),
            name: header(css, "name").unwrap_or_else(|| id.to_string()),
            built_in: true,
            preview: preview(css),
        })
        .collect();
    let mut added: Vec<ThemeInfo> = folder(&app)
        .and_then(|dir| fs::read_dir(dir).ok())
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            if path.extension()? != "css" {
                return None;
            }
            let id = path.file_stem()?.to_str()?.to_string();
            let css = fs::read_to_string(&path).ok()?;
            Some(ThemeInfo {
                name: header(&css, "name").unwrap_or_else(|| id.clone()),
                id,
                built_in: false,
                preview: preview(&css),
            })
        })
        .collect();
    added.sort_by_key(|theme| theme.name.to_lowercase());
    themes.extend(added);
    themes
}

/// Asks for a .css file, checks it and keeps a copy. Returns the new theme's
/// id, or None when the user cancelled.
#[tauri::command]
pub async fn add_theme<R: Runtime>(app: AppHandle<R>) -> Result<Option<String>, String> {
    let lang = i18n::current(&app);
    let picker = app.clone();
    let picked = tauri::async_runtime::spawn_blocking(move || {
        picker
            .dialog()
            .file()
            .set_title(pick(lang, "Add a theme", "Thêm theme"))
            .add_filter("Theme", &["css"])
            .blocking_pick_file()
    })
    .await
    .map_err(|err| err.to_string())?;
    let Some(picked) = picked else {
        return Ok(None);
    };
    let path = picked.into_path().map_err(|err| err.to_string())?;
    let css = fs::read_to_string(&path).map_err(|err| match lang {
        Lang::En => format!("could not read {}: {err}", path.display()),
        Lang::Vi => format!("Không đọc được {}: {err}", path.display()),
    })?;
    check(&css).map_err(|problem| problem.text(lang))?;

    let mut id = slug(path.file_stem().and_then(|stem| stem.to_str()).unwrap_or_default());
    if BUILT_IN.iter().any(|(built_in, _)| *built_in == id) {
        id.push_str("-custom");
    }
    let dir = folder(&app).ok_or(no_folder(lang))?;
    let target = dir.join(format!("{id}.css"));
    fs::create_dir_all(&dir)
        .and_then(|()| fs::write(&target, css))
        .map_err(|err| match lang {
            Lang::En => format!("could not save the theme to {}: {err}", target.display()),
            Lang::Vi => format!("Không lưu được theme vào {}: {err}", target.display()),
        })?;
    Ok(Some(id))
}

#[tauri::command]
pub fn remove_theme<R: Runtime>(app: AppHandle<R>, id: String) -> Result<(), String> {
    let lang = i18n::current(&app);
    if BUILT_IN.iter().any(|(built_in, _)| *built_in == id) || !is_id(&id) {
        return Err(pick(lang, "This theme comes with fMess.", "Theme này có sẵn trong fMess.").into());
    }
    let path = folder(&app).ok_or(no_folder(lang))?.join(format!("{id}.css"));
    fs::remove_file(&path).map_err(|err| match lang {
        Lang::En => format!("could not remove {}: {err}", path.display()),
        Lang::Vi => format!("Không xoá được {}: {err}", path.display()),
    })?;
    if settings::current(&app).theme.as_deref() == Some(id.as_str()) {
        settings::clear_theme(&app)?;
    }
    Ok(())
}

/// The chosen theme as the page takes it: its CSS, and whether it wants
/// Messenger's light colours in dark mode too.
pub fn for_page<R: Runtime>(app: &AppHandle<R>, id: Option<&str>) -> serde_json::Value {
    let Some(css) = id.and_then(|id| css(app, id)) else {
        return serde_json::Value::Null;
    };
    let light = header(&css, "mode").is_some_and(|mode| mode.eq_ignore_ascii_case("light"));
    serde_json::json!({ "css": css, "light": light })
}

fn css<R: Runtime>(app: &AppHandle<R>, id: &str) -> Option<String> {
    if let Some((_, css)) = BUILT_IN.iter().find(|(built_in, _)| *built_in == id) {
        return Some(css.to_string());
    }
    if !is_id(id) {
        return None;
    }
    let css = fs::read_to_string(folder(app)?.join(format!("{id}.css"))).ok()?;
    check(&css).ok()?;
    Some(css)
}

fn no_folder(lang: Lang) -> &'static str {
    pick(lang, "no data folder", "Không tìm thấy thư mục dữ liệu")
}

/// Why a file can't be a theme.
#[derive(Clone, Copy, Debug)]
enum Problem {
    TooBig,
    Link,
    Import,
    NotATheme,
}

impl Problem {
    fn text(self, lang: Lang) -> &'static str {
        match self {
            Problem::TooBig => pick(lang, "A theme can be at most 2 MB.", "Theme chỉ được nặng tối đa 2 MB."),
            Problem::Link => pick(
                lang,
                "A theme can only use pictures inside the file (data: URLs), not links.",
                "Theme chỉ được dùng hình nằm sẵn trong file (data: URL), không dùng link.",
            ),
            Problem::Import => pick(lang, "A theme can't load other files (@import).", "Theme không được tải file khác (@import)."),
            Problem::NotATheme => pick(lang, "This file isn't a theme fMess can use.", "fMess không dùng được file này làm theme."),
        }
    }
}

/// A theme may only style the page: nothing it could load or run.
fn check(css: &str) -> Result<(), Problem> {
    if css.len() > MAX_SIZE {
        return Err(Problem::TooBig);
    }
    // The CSS is read the way the browser reads it, escapes, comments and
    // line breaks included: looking for "url(" in the text missed `\\75rl(`.
    let mut input = ParserInput::new(css);
    walk(&mut Parser::new(&mut input)).map_err(|err| match err.kind {
        ParseErrorKind::Custom(problem) => problem,
        ParseErrorKind::Basic(_) => Problem::NotATheme,
    })
}

/// Functions that load something or run script, besides url() and src().
const LOADING: [&str; 6] = ["image-set", "-webkit-image-set", "image", "cross-fade", "element", "expression"];

/// Every token of the CSS, inside blocks and functions too.
fn walk<'i>(parser: &mut Parser<'i, '_>) -> Result<(), ParseError<'i, Problem>> {
    while let Ok(token) = parser.next_including_whitespace_and_comments() {
        match token.clone() {
            Token::UnquotedUrl(url) if !is_data(&url) => return Err(parser.new_custom_error(Problem::Link)),
            Token::BadUrl(_) => return Err(parser.new_custom_error(Problem::Link)),
            Token::AtKeyword(name) if name.eq_ignore_ascii_case("import") => return Err(parser.new_custom_error(Problem::Import)),
            Token::Function(name) if name.eq_ignore_ascii_case("url") || name.eq_ignore_ascii_case("src") => {
                // url("data:...") is a function with a string in it.
                parser.parse_nested_block(|args| match args.expect_string().cloned() {
                    Ok(url) if is_data(&url) => {
                        while args.next().is_ok() {}
                        Ok(())
                    }
                    _ => Err(args.new_custom_error(Problem::Link)),
                })?;
            }
            Token::Function(name) if LOADING.iter().any(|loading| name.eq_ignore_ascii_case(loading)) => {
                return Err(parser.new_custom_error(Problem::NotATheme));
            }
            Token::Function(_) | Token::ParenthesisBlock | Token::SquareBracketBlock | Token::CurlyBracketBlock => {
                parser.parse_nested_block(walk)?;
            }
            _ => {}
        }
    }
    Ok(())
}

fn is_data(url: &str) -> bool {
    url.trim_start().get(..5).is_some_and(|scheme| scheme.eq_ignore_ascii_case("data:"))
}

/// A value from the theme's opening comment ("name: Zalo").
fn header(css: &str, key: &str) -> Option<String> {
    let comment = css.trim_start().strip_prefix("/*")?.split("*/").next()?;
    comment.lines().find_map(|line| {
        let (name, value) = line.trim().split_once(':')?;
        name.trim().eq_ignore_ascii_case(key).then(|| value.trim().to_string()).filter(|value| !value.is_empty())
    })
}

/// The five preview colours, when the theme gives all of them as hex.
fn preview(css: &str) -> Option<Vec<String>> {
    let colours: Vec<String> = header(css, "preview")?.split_whitespace().map(str::to_string).collect();
    let hex = |colour: &String| {
        colour.len() >= 4 && colour.len() <= 9 && colour.starts_with('#') && colour[1..].chars().all(|c| c.is_ascii_hexdigit())
    };
    (colours.len() == 5 && colours.iter().all(hex)).then_some(colours)
}

fn slug(name: &str) -> String {
    let slug: String = name
        .to_lowercase()
        .chars()
        .filter_map(|c| match c {
            c if c.is_ascii_alphanumeric() => Some(c),
            c if c.is_whitespace() || c.is_ascii_punctuation() => Some('-'),
            _ => None,
        })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    if slug.is_empty() { "theme".into() } else { slug }
}

fn is_id(id: &str) -> bool {
    !id.is_empty() && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}

fn folder<R: Runtime>(app: &AppHandle<R>) -> Option<PathBuf> {
    app.path().app_data_dir().ok().map(|dir| dir.join("themes"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_header() {
        let css = "/* fMess theme\n   name: Zalo\n   mode: light\n*/\nbody {}";
        assert_eq!(header(css, "name").as_deref(), Some("Zalo"));
        assert_eq!(header(css, "mode").as_deref(), Some("light"));
        assert_eq!(header("body {}", "name"), None);
        let card = "/* fMess theme\n   preview: #005ae0 #fff #ebecf0 #e5f1ff #ffffff\n*/";
        assert_eq!(preview(card).map(|colours| colours.len()), Some(5));
        assert_eq!(preview("/* fMess theme\n   preview: red #fff #000 #111 #222\n*/"), None);
    }

    #[test]
    fn accepts_only_css_that_loads_nothing() {
        assert!(check("a { background: url(\"data:image/svg+xml,x\") }").is_ok());
        assert!(check("a { background: url( 'data:image/png;base64,x') }").is_ok());
        assert!(check("@import 'x.css';").is_err());
        assert!(check("a { background: url(https://example.com/x.png) }").is_err());
        assert!(check("a { background: url(x.png) }").is_err());
        assert!(check("a { background: image-set('x.png' 1x) }").is_err());
        // The same, written with CSS escapes.
        assert!(check("a { background: \\75rl(https://example.com/x.png) }").is_err());
        assert!(check("a { background: \\75 rl(https://example.com/x.png) }").is_err());
        assert!(check("a { background: u\\rl(https://example.com/x.png) }").is_err());
        assert!(check("a { background: \\55\\52\\4c(https://example.com/x.png) }").is_err());
        assert!(check("@\\69mport 'x.css';").is_err());
        assert!(check("@\\000069mport 'x.css';").is_err());
        assert!(check("a { background: src('x.png') }").is_err());
        // Line breaks after an escape count as one, CRLF included.
        assert!(check("a { background: \\75\r\nrl(https://example.com/x.png) }").is_err());
        assert!(check("@\\69\r\nmport 'x.css';").is_err());
        assert!(check("a { background: \\75\x0crl(https://example.com/x.png) }").is_err());
        // Inside blocks and other functions too, and quoted.
        assert!(check("@media screen { a { b: var(--x, url('https://example.com/x.png')) } }").is_err());
        assert!(check("a { background: url('https://example.com/x.png') }").is_err());
        assert!(check("a { background: -webkit-image-set(\"x.png\" 1x) }").is_err());
        // Pictures inside the file, however they are written.
        assert!(check("a { background: URL( 'DATA:image/png;base64,x' ) }").is_ok());
        assert!(check("a { background: \\75rl(data:image/png;base64,x) }").is_ok());
        // An escape that isn't one of those is fine.
        assert!(check("a::before { content: \"\\201C\" }").is_ok());
    }

    #[test]
    fn built_in_themes_load_nothing_and_have_cards() {
        for (id, css) in BUILT_IN {
            assert!(check(css).is_ok(), "{id} loads something");
            assert!(header(css, "name").is_some(), "{id} has no name");
            assert!(preview(css).is_some(), "{id} has no preview colours");
        }
    }

    #[test]
    fn makes_ids_from_file_names() {
        assert_eq!(slug("Zalo Look (2)"), "zalo-look-2");
        assert_eq!(slug("Lục"), "lc");
        assert_eq!(slug("..."), "theme");
    }
}
