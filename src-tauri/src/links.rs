use tauri::{AppHandle, Runtime, Url};
use tauri_plugin_opener::OpenerExt;

use crate::services::Service;

/// Hosts that stay inside fMess's Messenger. Everything else opens in the default browser.
const MESSENGER_HOSTS: &[&str] = &[
    "messenger.com",
    "facebook.com",
    "facebook.net",
    "fbcdn.net",
    "fbsbx.com",
];

/// Facebook and Instagram route outgoing links through a tracking redirect:
/// `l.php?u=<target>` on Facebook, `/?u=<target>` on Instagram.
const LINK_SHIM_HOSTS: &[&str] = &["l.messenger.com", "l.facebook.com", "lm.facebook.com", "l.instagram.com"];

/// Pages a popup may show inside fMess: chats, their photo/video viewer and
/// the call windows they start. The rest of Facebook (reels, posts, profiles)
/// opens in the browser. Keep in step with IN_APP_PATHS in inject/06-routes.js.
const MESSENGER_PATHS: &[&str] = &["/messages", "/messenger_media", "/groupcall", "/videocall"];

/// Instagram's pages that stay inside fMess: the chats, and logging in with
/// the checks Instagram may ask for then. Posts, reels and profiles open in
/// the browser. The home page stays too: logging in lands there, and fMess
/// moves on to the chats (see webview.rs).
const INSTAGRAM_PATHS: &[&str] = &["/direct", "/accounts", "/challenge", "/auth_platform", "/two_factor", "/consent"];

/// Facebook's pages Instagram's "Log in with Facebook" passes through,
/// besides versioned dialogs (/v19.0/dialog/oauth).
const INSTAGRAM_FACEBOOK_PATHS: &[&str] =
    &["/dialog", "/login", "/login.php", "/checkpoint", "/recover", "/two_step_verification", "/instagram", "/x/oauth"];

fn on_host(host: &str, domain: &str) -> bool {
    host == domain || host.ends_with(&format!(".{domain}"))
}

/// The site's own pages, as opposed to its APIs, frames and media, which other subdomains serve.
fn is_page_host(host: &str, domain: &str) -> bool {
    matches!(host.strip_suffix(domain), Some("" | "www." | "m." | "web."))
}

fn starts_with_any(path: &str, prefixes: &[&str]) -> bool {
    prefixes.iter().any(|prefix| {
        path.strip_prefix(prefix)
            .is_some_and(|rest| rest.is_empty() || rest.starts_with('/'))
    })
}

fn is_instagram_facebook_page(path: &str) -> bool {
    // /v19.0/dialog/oauth: drop the API version.
    let unversioned = path
        .strip_prefix("/v")
        .and_then(|rest| rest.split_once('/'))
        .filter(|(version, _)| !version.is_empty() && version.chars().all(|c| c.is_ascii_digit() || c == '.'))
        .map_or(path.to_owned(), |(_, rest)| format!("/{rest}"));
    starts_with_any(&unversioned, INSTAGRAM_FACEBOOK_PATHS)
}

/// Whether `url` is one of `service`'s pages that fMess shows itself.
fn is_in_app_page(service: Service, url: &Url) -> bool {
    if unwrap_link_shim(url).is_some() || !matches!(url.scheme(), "http" | "https") {
        return false;
    }
    let (Some(host), path) = (url.host_str(), url.path()) else {
        return false;
    };
    match service {
        Service::Messenger => {
            MESSENGER_HOSTS.iter().any(|app_host| on_host(host, app_host))
                && MESSENGER_PATHS.iter().any(|prefix| path.starts_with(prefix))
        }
        Service::Instagram if on_host(host, "instagram.com") => {
            !is_page_host(host, "instagram.com") || path == "/" || starts_with_any(path, INSTAGRAM_PATHS)
        }
        Service::Instagram if on_host(host, "facebook.com") => {
            !is_page_host(host, "facebook.com") || is_instagram_facebook_page(path)
        }
        Service::Instagram => on_host(host, "cdninstagram.com") || on_host(host, "fbcdn.net"),
    }
}

/// Whether a `window.open()` popup from `service` stays inside fMess.
pub fn opens_in_app(service: Service, url: &Url) -> bool {
    // window.open() with no URL, filled in by the page afterwards.
    url.scheme() == "about" || is_in_app_page(service, url)
}

/// The news feed, where Facebook sends people after they log in.
pub fn is_facebook_feed(url: &Url) -> bool {
    matches!(
        url.host_str(),
        Some("www.facebook.com" | "facebook.com" | "web.facebook.com")
    ) && matches!(url.path(), "/" | "/home.php")
}

/// Instagram's home page, where logging in lands.
pub fn is_instagram_home(url: &Url) -> bool {
    matches!(url.host_str(), Some("www.instagram.com" | "instagram.com")) && url.path() == "/"
}

/// Decides whether a webview may load `url`. WebKit asks this for iframes too,
/// so only links the user clearly meant to leave with are sent to the browser:
/// link shims, and on Instagram its own pages that aren't chats or login (its
/// links elsewhere all go through a link shim).
pub fn allow_navigation<R: Runtime>(app: &AppHandle<R>, service: Service, url: &Url) -> bool {
    match url.scheme() {
        "http" | "https" if unwrap_link_shim(url).is_some() || leaves_service(service, url) => {
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

fn leaves_service(service: Service, url: &Url) -> bool {
    let Some(host) = url.host_str() else {
        return false;
    };
    service == Service::Instagram
        && (is_page_host(host, "instagram.com") || is_page_host(host, "facebook.com"))
        && !is_in_app_page(service, url)
}

pub fn open_external<R: Runtime>(app: &AppHandle<R>, url: &Url) {
    let target = unwrap_link_shim(url).map_or_else(|| url.clone(), strip_tracking);
    if let Err(err) = app.opener().open_url(target.as_str(), None::<&str>) {
        eprintln!("could not open {target}: {err}");
    }
}

fn unwrap_link_shim(url: &Url) -> Option<Url> {
    let host = url.host_str()?;
    let shim_path = if host == "l.instagram.com" { "/" } else { "/l.php" };
    if !LINK_SHIM_HOSTS.contains(&host) || url.path() != shim_path {
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
        let messenger = |s| opens_in_app(Service::Messenger, &url(s));
        assert!(messenger("https://www.facebook.com/messages/t/123"));
        assert!(messenger("https://www.facebook.com/groupcall/ROOM:1/?has_video=true"));
        assert!(messenger("https://www.facebook.com/messenger_media/?attachment_id=1&message_id=mid.2&thread_id=3"));
        assert!(messenger("about:blank"));
        assert!(!messenger("https://www.facebook.com/reel/123"));
        assert!(!messenger("https://www.facebook.com/"));
        assert!(!messenger("https://example.com/messages"));
        assert!(!messenger("https://notfacebook.com/messages"));
        assert!(!messenger("https://www.instagram.com/direct/inbox/"));
    }

    #[test]
    fn instagram_keeps_chats_and_login_in_app() {
        let instagram = |s| opens_in_app(Service::Instagram, &url(s));
        assert!(instagram("https://www.instagram.com/direct/inbox/"));
        assert!(instagram("https://www.instagram.com/direct/t/1234567890/"));
        assert!(instagram("https://www.instagram.com/accounts/login/?next=%2Fdirect%2Finbox%2F"));
        assert!(instagram("https://www.instagram.com/accounts/onetap/"));
        assert!(instagram("https://www.instagram.com/challenge/action/AXG/"));
        assert!(instagram("https://www.instagram.com/"));
        assert!(instagram("https://instagram.com/direct/inbox/"));
        assert!(instagram("https://scontent-sin6-1.cdninstagram.com/v/t51.2885-19/1.jpg"));
        assert!(instagram("https://scontent.xx.fbcdn.net/v/1.jpg"));
        assert!(instagram("https://www.facebook.com/v19.0/dialog/oauth?client_id=1"));
        assert!(instagram("https://www.facebook.com/dialog/oauth?client_id=1"));
        assert!(instagram("https://m.facebook.com/login.php?next=x"));
        assert!(instagram("https://m.facebook.com/login/?next=x"));
        assert!(instagram("https://www.facebook.com/checkpoint/1501092823525282/"));
        assert!(instagram("https://www.facebook.com/instagram/login_sync/"));
        assert!(instagram("about:blank"));

        assert!(!instagram("https://www.instagram.com/someone/"));
        assert!(!instagram("https://www.instagram.com/p/C0abc/"));
        assert!(!instagram("https://www.instagram.com/reel/C0abc/"));
        assert!(!instagram("https://www.instagram.com/directory/"));
        assert!(!instagram("https://www.facebook.com/"));
        assert!(!instagram("https://www.facebook.com/messages/t/1"));
        assert!(!instagram("https://www.facebook.com/videos/1"));
        assert!(!instagram("https://www.facebook.com/vanity.name/posts/1"));
        assert!(!instagram("https://example.com/direct/inbox/"));
        assert!(!instagram("https://notinstagram.com/direct/inbox/"));
        assert!(!instagram("https://l.instagram.com/?u=https%3A%2F%2Fexample.com%2F&e=AT0"));
    }

    #[test]
    fn instagram_sends_its_other_pages_to_the_browser() {
        let leaves = |s| leaves_service(Service::Instagram, &url(s));
        assert!(leaves("https://www.instagram.com/someone/"));
        assert!(leaves("https://www.instagram.com/p/C0abc/"));
        assert!(leaves("https://www.facebook.com/someone"));
        assert!(!leaves("https://www.instagram.com/direct/t/1/"));
        assert!(!leaves("https://www.instagram.com/"));
        // Frames and APIs on other hosts load as on Messenger.
        assert!(!leaves("https://i.instagram.com/api/v1/x"));
        assert!(!leaves("https://www.google.com/recaptcha/api2/anchor"));
        assert!(!leaves("https://static.cdninstagram.com/rsrc.php/x.js"));
        // Messenger's rules are unchanged: only link shims leave.
        assert!(!leaves_service(Service::Messenger, &url("https://www.facebook.com/reel/1")));
    }

    #[test]
    fn recognizes_instagram_home() {
        assert!(is_instagram_home(&url("https://www.instagram.com/")));
        assert!(is_instagram_home(&url("https://instagram.com/?hl=vi")));
        assert!(!is_instagram_home(&url("https://www.instagram.com/direct/inbox/")));
        assert!(!is_instagram_home(&url("https://www.facebook.com/")));
    }

    #[test]
    fn instagram_link_shim_leaves_the_app() {
        let shim = url("https://l.instagram.com/?u=https%3A%2F%2Fexample.com%2Fa%3Fx%3D1%26fbclid%3Dabc&e=AT0");
        let target = unwrap_link_shim(&shim).map(strip_tracking).unwrap();
        assert_eq!(target.as_str(), "https://example.com/a?x=1");
        assert!(unwrap_link_shim(&url("https://l.instagram.com/l.php?u=https%3A%2F%2Fexample.com")).is_none());
    }

    #[test]
    fn link_shims_leave_the_app_without_tracking() {
        let shim = url("https://l.messenger.com/l.php?u=https%3A%2F%2Fexample.com%2Fa%3Fx%3D1%26fbclid%3Dabc&h=AT0");
        assert!(!opens_in_app(Service::Messenger, &shim));
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
