use tauri::{AppHandle, Runtime, Url};
use tauri_plugin_opener::OpenerExt;

/// Hosts that stay inside fMess. Everything else opens in the default browser.
const APP_HOSTS: &[&str] = &[
    "messenger.com",
    "facebook.com",
    "facebook.net",
    "fbcdn.net",
    "fbsbx.com",
];

/// Facebook routes outgoing links through a tracking redirect: `l.php?u=<target>`.
const LINK_SHIM_HOSTS: &[&str] = &["l.messenger.com", "l.facebook.com", "lm.facebook.com"];

/// Pages a popup may show inside fMess: chats, their photo/video viewer and
/// the call windows they start. The rest of Facebook (reels, posts, profiles)
/// opens in the browser. Keep in step with IN_APP_PATHS in inject/06-routes.js.
const IN_APP_PATHS: &[&str] = &["/messages", "/messenger_media", "/groupcall", "/videocall"];

/// Whether a `window.open()` popup stays inside fMess.
pub fn opens_in_app(url: &Url) -> bool {
    if url.scheme() == "about" {
        // window.open() with no URL, filled in by the page afterwards.
        return true;
    }
    if unwrap_link_shim(url).is_some() || !matches!(url.scheme(), "http" | "https") {
        return false;
    }
    let on_app_host = url.host_str().is_some_and(|host| {
        APP_HOSTS
            .iter()
            .any(|app_host| host == *app_host || host.ends_with(&format!(".{app_host}")))
    });
    on_app_host && IN_APP_PATHS.iter().any(|path| url.path().starts_with(path))
}

/// The news feed, where Facebook sends people after they log in.
pub fn is_facebook_feed(url: &Url) -> bool {
    matches!(
        url.host_str(),
        Some("www.facebook.com" | "facebook.com" | "web.facebook.com")
    ) && matches!(url.path(), "/" | "/home.php")
}

/// Decides whether a webview may load `url`. WebKit asks this for iframes too,
/// so only links the user clearly meant to leave with are sent to the browser.
pub fn allow_navigation<R: Runtime>(app: &AppHandle<R>, url: &Url) -> bool {
    match url.scheme() {
        "http" | "https" if unwrap_link_shim(url).is_some() => {
            open_external(app, url);
            false
        }
        "http" | "https" | "about" | "blob" | "data" => true,
        "mailto" | "tel" => {
            open_external(app, url);
            false
        }
        // App deep links (fb://, itms-apps://, ...) would only launch other apps.
        _ => false,
    }
}

pub fn open_external<R: Runtime>(app: &AppHandle<R>, url: &Url) {
    let target = unwrap_link_shim(url).map_or_else(|| url.clone(), strip_tracking);
    if let Err(err) = app.opener().open_url(target.as_str(), None::<&str>) {
        eprintln!("could not open {target}: {err}");
    }
}

fn unwrap_link_shim(url: &Url) -> Option<Url> {
    let host = url.host_str()?;
    if !LINK_SHIM_HOSTS.contains(&host) || url.path() != "/l.php" {
        return None;
    }
    let (_, target) = url.query_pairs().find(|(key, _)| key == "u")?;
    Url::parse(&target)
        .ok()
        .filter(|target| matches!(target.scheme(), "http" | "https"))
}

/// Drops the `fbclid` parameter Facebook appends to outgoing links.
fn strip_tracking(mut url: Url) -> Url {
    if !url.query_pairs().any(|(key, _)| key == "fbclid") {
        return url;
    }
    let kept: Vec<(String, String)> = url
        .query_pairs()
        .filter(|(key, _)| key != "fbclid")
        .map(|(key, value)| (key.into_owned(), value.into_owned()))
        .collect();
    if kept.is_empty() {
        url.set_query(None);
    } else {
        url.query_pairs_mut().clear().extend_pairs(kept);
    }
    url
}

#[cfg(test)]
mod tests {
    use super::*;

    fn url(s: &str) -> Url {
        Url::parse(s).unwrap()
    }

    #[test]
    fn only_chats_and_calls_open_in_app() {
        assert!(opens_in_app(&url("https://www.facebook.com/messages/t/123")));
        assert!(opens_in_app(&url("https://www.facebook.com/groupcall/ROOM:1/?has_video=true")));
        assert!(opens_in_app(&url(
            "https://www.facebook.com/messenger_media/?attachment_id=1&message_id=mid.2&thread_id=3"
        )));
        assert!(opens_in_app(&url("about:blank")));
        assert!(!opens_in_app(&url("https://www.facebook.com/reel/123")));
        assert!(!opens_in_app(&url("https://www.facebook.com/")));
        assert!(!opens_in_app(&url("https://example.com/messages")));
        assert!(!opens_in_app(&url("https://notfacebook.com/messages")));
    }

    #[test]
    fn link_shims_leave_the_app_without_tracking() {
        let shim = url("https://l.messenger.com/l.php?u=https%3A%2F%2Fexample.com%2Fa%3Fx%3D1%26fbclid%3Dabc&h=AT0");
        assert!(!opens_in_app(&shim));
        let target = unwrap_link_shim(&shim).map(strip_tracking).unwrap();
        assert_eq!(target.as_str(), "https://example.com/a?x=1");
    }

    #[test]
    fn recognizes_the_news_feed() {
        assert!(is_facebook_feed(&url("https://www.facebook.com/")));
        assert!(is_facebook_feed(&url("https://www.facebook.com/?sk=h_chr")));
        assert!(is_facebook_feed(&url("https://web.facebook.com/home.php")));
        assert!(!is_facebook_feed(&url("https://www.facebook.com/messages/")));
        assert!(!is_facebook_feed(&url("https://www.facebook.com/messages/t/123")));
        assert!(!is_facebook_feed(&url("https://www.messenger.com/")));
    }

    #[test]
    fn link_shim_must_point_to_the_web() {
        let shim = url("https://l.facebook.com/l.php?u=javascript%3Aalert(1)");
        assert!(unwrap_link_shim(&shim).is_none());
    }
}
