use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, PhysicalPosition, Rect, Runtime, Webview, Window};

use crate::MAIN_WINDOW;

/// A site fMess shows in its window, each in a web view of its own.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Service {
    Messenger,
    Instagram,
}

impl Service {
    pub fn label(self) -> &'static str {
        match self {
            Service::Messenger => "messenger",
            Service::Instagram => "instagram",
        }
    }

    pub fn from_label(label: &str) -> Option<Self> {
        [Service::Messenger, Service::Instagram]
            .into_iter()
            .find(|service| service.label() == label)
    }
}

pub fn webview<R: Runtime>(app: &AppHandle<R>, service: Service) -> Option<Webview<R>> {
    app.get_webview(service.label())
}

pub fn main_window<R: Runtime>(app: &AppHandle<R>) -> Option<Window<R>> {
    app.get_window(MAIN_WINDOW)
}

/// Places the web views in fMess's window: Messenger fills it.
pub fn layout<R: Runtime>(window: &Window<R>) {
    let Ok(size) = window.inner_size() else {
        return;
    };
    for webview in window.webviews() {
        if Service::from_label(webview.label()) == Some(Service::Messenger) {
            let _ = webview.set_bounds(Rect {
                position: PhysicalPosition::new(0, 0).into(),
                size: size.into(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_round_trip() {
        for service in [Service::Messenger, Service::Instagram] {
            assert_eq!(Service::from_label(service.label()), Some(service));
        }
        assert_eq!(Service::from_label("main"), None);
    }

    #[test]
    fn serializes_as_its_label() {
        for service in [Service::Messenger, Service::Instagram] {
            let json = serde_json::to_string(&service).unwrap();
            assert_eq!(json, format!("\"{}\"", service.label()));
            assert_eq!(serde_json::from_str::<Service>(&json).unwrap(), service);
        }
    }
}
