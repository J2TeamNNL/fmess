use serde::Deserialize;
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_notification::NotificationExt;

use crate::{
    i18n::{self, pick, Lang},
    settings, show_main_window, MAIN_WINDOW,
};

/// Who wrote a message and in which chat, as inject.js reads them from
/// Messenger. Pictures are base64. The sender and the group only go into
/// macOS's communication notifications.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    thread: String,
    #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
    sender: String,
    #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
    sender_picture: Option<String>,
    group: Option<Group>,
}

#[derive(Deserialize)]
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub struct Group {
    name: Option<String>,
    picture: Option<String>,
}

/// Called by inject.js whenever the page creates a web `Notification`. `icon`
/// is its picture and `image` a photo in the message (base64); `message`
/// says who wrote in which chat, when the page knows.
#[tauri::command]
pub async fn notify<R: Runtime>(
    app: AppHandle<R>,
    id: i32,
    title: String,
    body: String,
    icon: Option<String>,
    image: Option<String>,
    message: Option<Message>,
) -> Result<(), String> {
    #[cfg(debug_assertions)]
    eprintln!(
        "[fmess] notify #{id}: {title} (thread {:?}, group {})",
        message.as_ref().map(|message| &message.thread),
        message.as_ref().is_some_and(|message| message.group.is_some())
    );
    let settings = settings::current(&app);
    if !settings.notifications {
        return Ok(());
    }
    // What a notification says when it may not show the sender and message.
    let new_message = pick(Lang::from_setting(settings.language.as_deref()), "New message", "Tin nhắn mới");
    #[cfg(target_os = "macos")]
    if communication::available() {
        use base64::Engine;
        let decode = |data: Option<String>| data.and_then(|data| base64::engine::general_purpose::STANDARD.decode(data).ok());
        let notification = if settings.notification_preview {
            communication::Notification {
                id,
                title,
                body,
                image: decode(image),
                message: message.map(|message| communication::Sender {
                    thread: message.thread,
                    name: message.sender,
                    picture: decode(message.sender_picture.or(icon)),
                    group: message.group.map(|group| (group.name, decode(group.picture))),
                }),
                attachments_dir: app.path().app_cache_dir().ok().map(|dir| dir.join("notifications")),
            }
        } else {
            communication::Notification::plain(id, "fMess", new_message)
        };
        communication::show(notification);
        return Ok(());
    }
    let _ = (icon, image, message);
    let (title, body) = if settings.notification_preview {
        (title, body)
    } else {
        ("fMess".to_owned(), new_message.to_owned())
    };
    app.notification()
        .builder()
        .id(id)
        .title(title)
        .body(body)
        .show()
        .map_err(|err| err.to_string())
}

/// The Settings window's Send Test Notification: shows one through the same
/// path messages use, and says what went wrong when it can.
#[tauri::command]
pub fn test_notification<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    let (title, body) = ("fMess", pick(i18n::current(&app), "Notifications are working.", "Thông báo đã hoạt động."));
    #[cfg(target_os = "macos")]
    if communication::available() {
        communication::show(communication::Notification::plain(0, title, body));
        return Ok(());
    }
    app.notification()
        .builder()
        .title(title)
        .body(body)
        .show()
        .map_err(|err| err.to_string())
}

/// Windows shows notifications only from apps it knows by their app ID
/// (AppUserModelID): the installer's Start menu shortcut carries it, but the
/// MSI's doesn't, and a copy run from elsewhere has none. So fMess registers
/// the ID itself, with its name and icon, and runs under it.
#[cfg(windows)]
pub fn register_app_id<R: Runtime>(app: &AppHandle<R>) {
    use windows_sys::Win32::{
        System::Registry::{RegCloseKey, RegCreateKeyExW, RegSetValueExW, HKEY, HKEY_CURRENT_USER, KEY_WRITE, REG_OPTION_NON_VOLATILE, REG_SZ},
        UI::Shell::SetCurrentProcessExplicitAppUserModelID,
    };

    let wide = |text: &str| text.encode_utf16().chain(Some(0)).collect::<Vec<u16>>();
    let id = app.config().identifier.clone();
    let icon = app.path().app_data_dir().ok().map(|dir| dir.join("notification-icon.png"));
    if let Some(icon) = &icon {
        let _ = icon.parent().map(std::fs::create_dir_all);
        let _ = std::fs::write(icon, include_bytes!("../icons/128x128@2x.png"));
    }
    let id_wide = wide(&id);
    unsafe {
        SetCurrentProcessExplicitAppUserModelID(id_wide.as_ptr());
        let mut key: HKEY = std::ptr::null_mut();
        let path = wide(&format!(r"Software\Classes\AppUserModelId\{id}"));
        let opened = RegCreateKeyExW(
            HKEY_CURRENT_USER,
            path.as_ptr(),
            0,
            std::ptr::null(),
            REG_OPTION_NON_VOLATILE,
            KEY_WRITE,
            std::ptr::null(),
            &mut key,
            std::ptr::null_mut(),
        );
        if opened != 0 {
            return;
        }
        let set = |name: &str, value: &str| {
            let name = wide(name);
            let value = wide(value);
            RegSetValueExW(key, name.as_ptr(), 0, REG_SZ, value.as_ptr().cast(), (value.len() * 2) as u32);
        };
        set("DisplayName", &app.package_info().name);
        if let Some(icon) = icon {
            set("IconUri", &icon.to_string_lossy());
        }
        RegCloseKey(key);
    }
}

/// Called by inject.js when a chat is read in fMess: its notifications go, as
/// in the old desktop app.
#[tauri::command]
pub fn clear_notifications(thread: String) {
    #[cfg(target_os = "macos")]
    if communication::available() {
        communication::clear(thread);
    }
    #[cfg(not(target_os = "macos"))]
    let _ = thread;
}

/// A click on a notification brings fMess forward and replays the click in
/// the page, so Messenger's own handler opens the right conversation.
pub fn route_clicks<R: Runtime>(app: &AppHandle<R>) -> tauri_plugin_notification::Result<()> {
    #[cfg(target_os = "macos")]
    if communication::available() {
        let clicks = app.clone();
        let replies = app.clone();
        communication::init(
            i18n::current(app),
            move |id, thread| open_clicked(&clicks, id, thread),
            move |id, thread, text| send_reply(&replies, id, thread, text),
        );
    }
    let handle = app.clone();
    app.notification().on_action(move |action| {
        if action.action_id() != "tap" {
            return;
        }
        if let Some(notification) = action.notification() {
            open_clicked(&handle, notification.id(), None);
        }
    })
}

/// Notifications' Reply action in another language (chosen in Settings).
pub fn apply_language(lang: Lang) {
    #[cfg(target_os = "macos")]
    if communication::available() {
        communication::set_categories(lang);
    }
    #[cfg(not(target_os = "macos"))]
    let _ = lang;
}

/// A reply typed in a notification: the page sends it in that chat, without
/// bringing fMess forward.
#[cfg(target_os = "macos")]
fn send_reply<R: Runtime>(app: &AppHandle<R>, id: i32, thread: String, text: String) {
    // Answering a chat reads it, as opening it does.
    communication::clear(thread.clone());
    let app_for_main = app.clone();
    let _ = app.run_on_main_thread(move || {
        if let Some(window) = app_for_main.get_webview_window(MAIN_WINDOW) {
            let args = serde_json::to_string(&(id, thread, text)).expect("JSON serializes");
            let _ = window.eval(format!("window.__fmessReply?.(...{args})"));
        }
    });
}

fn open_clicked<R: Runtime>(app: &AppHandle<R>, id: i32, thread: Option<String>) {
    let app_for_main = app.clone();
    // Clicks arrive on a background thread.
    let _ = app.run_on_main_thread(move || {
        show_main_window(&app_for_main);
        if let Some(window) = app_for_main.get_webview_window(MAIN_WINDOW) {
            let thread = serde_json::to_string(&thread).expect("JSON serializes");
            let _ = window.eval(format!("window.__fmessNotificationClicked?.({id}, {thread})"));
        }
    });
}

/// Communication Notifications, as the old Messenger app showed them: the
/// sender's picture in place of the app icon, a group's name and picture, and
/// each chat's notifications kept together. They take an entitlement only a
/// signed build carries (see tauri.signed.conf.json); other builds show plain
/// notifications through the notification plugin.
#[cfg(target_os = "macos")]
mod communication {
    use std::{path::PathBuf, sync::OnceLock};

    use block2::{DynBlock, RcBlock};
    use objc2::{
        define_class, msg_send,
        rc::Retained,
        runtime::{Bool, NSObject, NSObjectProtocol, ProtocolObject},
        AllocAnyThread, ClassType,
    };
    use objc2_foundation::{NSArray, NSData, NSError, NSSet, NSString, NSURL};
    use objc2_intents::{
        INImage, INInteraction, INInteractionDirection, INOutgoingMessageType, INPerson, INPersonHandle,
        INPersonHandleType, INSendMessageIntent, INSpeakableString,
    };
    use objc2_user_notifications::{
        UNAuthorizationOptions, UNMutableNotificationContent, UNNotification,
        UNNotificationActionOptions, UNNotificationAttachment, UNNotificationCategory, UNNotificationCategoryOptions,
        UNNotificationContent, UNNotificationPresentationOptions, UNNotificationRequest, UNNotificationResponse,
        UNTextInputNotificationAction, UNTextInputNotificationResponse, UNUserNotificationCenter,
        UNUserNotificationCenterDelegate,
    };

    use crate::i18n::{pick, Lang};

    pub struct Notification {
        pub id: i32,
        pub title: String,
        pub body: String,
        /// A photo in the message, shown in the notification.
        pub image: Option<Vec<u8>>,
        pub message: Option<Sender>,
        /// Where photos wait until macOS takes them.
        pub attachments_dir: Option<PathBuf>,
    }

    pub struct Sender {
        pub thread: String,
        pub name: String,
        pub picture: Option<Vec<u8>>,
        /// A group's name, when it has one, and picture.
        pub group: Option<(Option<String>, Option<Vec<u8>>)>,
    }

    impl Notification {
        /// Just a title and text, with nothing about who wrote what.
        pub fn plain(id: i32, title: &str, body: &str) -> Self {
            Self {
                id,
                title: title.to_owned(),
                body: body.to_owned(),
                image: None,
                message: None,
                attachments_dir: None,
            }
        }
    }

    type ClickHandler = Box<dyn Fn(i32, Option<String>) + Send + Sync>;
    type ReplyHandler = Box<dyn Fn(i32, String, String) + Send + Sync>;
    static ON_CLICK: OnceLock<ClickHandler> = OnceLock::new();
    static ON_REPLY: OnceLock<ReplyHandler> = OnceLock::new();

    /// Message notifications offer a reply field, as the old app's did.
    const MESSAGE_CATEGORY: &str = "message";
    const REPLY_ACTION: &str = "reply";

    define_class!(
        #[unsafe(super(NSObject))]
        #[name = "FMessNotificationDelegate"]
        struct Delegate;

        unsafe impl NSObjectProtocol for Delegate {}

        unsafe impl UNUserNotificationCenterDelegate for Delegate {
            // fMess notifies only while its window isn't the active one, which
            // may be while fMess itself is frontmost.
            #[unsafe(method(userNotificationCenter:willPresentNotification:withCompletionHandler:))]
            fn will_present(
                &self,
                _center: &UNUserNotificationCenter,
                _notification: &UNNotification,
                handler: &DynBlock<dyn Fn(UNNotificationPresentationOptions)>,
            ) {
                handler.call((UNNotificationPresentationOptions::Banner | UNNotificationPresentationOptions::List,));
            }

            #[unsafe(method(userNotificationCenter:didReceiveNotificationResponse:withCompletionHandler:))]
            fn did_receive(
                &self,
                _center: &UNUserNotificationCenter,
                response: &UNNotificationResponse,
                handler: &DynBlock<dyn Fn()>,
            ) {
                let request = response.notification().request();
                let thread = request.content().threadIdentifier().to_string();
                let reply = response.downcast_ref::<UNTextInputNotificationResponse>().map(|reply| reply.userText().to_string());
                match (request.identifier().to_string().parse(), reply) {
                    (Ok(id), Some(text)) if response.actionIdentifier().to_string() == REPLY_ACTION => {
                        if let (false, false, Some(on_reply)) = (thread.is_empty(), text.trim().is_empty(), ON_REPLY.get()) {
                            on_reply(id, thread, text);
                        }
                    }
                    (Ok(id), _) => {
                        if let Some(on_click) = ON_CLICK.get() {
                            on_click(id, (!thread.is_empty()).then_some(thread));
                        }
                    }
                    _ => {}
                }
                handler.call(());
            }
        }
    );

    /// Signed builds carry the provisioning profile that grants the entitlement.
    pub fn available() -> bool {
        static AVAILABLE: OnceLock<bool> = OnceLock::new();
        *AVAILABLE.get_or_init(|| {
            std::env::current_exe()
                .ok()
                .and_then(|exe| Some(exe.parent()?.parent()?.join("embedded.provisionprofile")))
                .is_some_and(|profile| profile.is_file())
        })
    }

    pub fn init(
        lang: Lang,
        on_click: impl Fn(i32, Option<String>) + Send + Sync + 'static,
        on_reply: impl Fn(i32, String, String) + Send + Sync + 'static,
    ) {
        let _ = ON_CLICK.set(Box::new(on_click));
        let _ = ON_REPLY.set(Box::new(on_reply));
        let center = UNUserNotificationCenter::currentNotificationCenter();
        set_categories(lang);
        let delegate: Retained<Delegate> = unsafe { msg_send![Delegate::class(), new] };
        center.setDelegate(Some(ProtocolObject::from_ref(&*delegate)));
        // The center holds its delegate weakly, and fMess needs it until it quits.
        std::mem::forget(delegate);
        let done = RcBlock::new(|_granted: Bool, error: *mut NSError| {
            if let Some(error) = unsafe { error.as_ref() } {
                eprintln!("notifications not allowed: {error}");
            }
        });
        center.requestAuthorizationWithOptions_completionHandler(
            UNAuthorizationOptions::Alert | UNAuthorizationOptions::Sound | UNAuthorizationOptions::Badge,
            &done,
        );
    }

    /// The message category, with its Reply action named in `lang`.
    pub fn set_categories(lang: Lang) {
        let reply = UNTextInputNotificationAction::actionWithIdentifier_title_options_textInputButtonTitle_textInputPlaceholder(
            &NSString::from_str(REPLY_ACTION),
            &NSString::from_str(pick(lang, "Reply", "Trả lời")),
            UNNotificationActionOptions::empty(),
            &NSString::from_str(pick(lang, "Send", "Gửi")),
            &NSString::from_str("Aa"),
        );
        let category = UNNotificationCategory::categoryWithIdentifier_actions_intentIdentifiers_options(
            &NSString::from_str(MESSAGE_CATEGORY),
            &NSArray::from_retained_slice(&[reply.into_super()]),
            &NSArray::from_retained_slice(&[NSString::from_str("INSendMessageIntent")]),
            UNNotificationCategoryOptions::empty(),
        );
        UNUserNotificationCenter::currentNotificationCenter()
            .setNotificationCategories(&NSSet::from_retained_slice(&[category]));
    }

    pub fn show(notification: Notification) {
        let content = UNMutableNotificationContent::new();
        content.setTitle(&NSString::from_str(&notification.title));
        content.setBody(&NSString::from_str(&notification.body));
        if let Some(sender) = &notification.message {
            content.setThreadIdentifier(&NSString::from_str(&sender.thread));
            content.setCategoryIdentifier(&NSString::from_str(MESSAGE_CATEGORY));
        }
        // Messenger plays its own sound in the page.
        if let (Some(image), Some(dir)) = (&notification.image, &notification.attachments_dir) {
            if let Some(attachment) = photo_attachment(notification.id, image, dir) {
                content.setAttachments(&NSArray::from_retained_slice(&[attachment]));
            }
        }
        let content: Retained<UNNotificationContent> = notification
            .message
            .as_ref()
            .and_then(|sender| from_sender(&content, sender, &notification.body))
            .unwrap_or_else(|| content.into_super());
        let request = UNNotificationRequest::requestWithIdentifier_content_trigger(
            &NSString::from_str(&notification.id.to_string()),
            &content,
            None,
        );
        let id = notification.id;
        let done = RcBlock::new(move |error: *mut NSError| match unsafe { error.as_ref() } {
            Some(error) => eprintln!("could not show a notification: {error}"),
            None if cfg!(debug_assertions) => eprintln!("[fmess] notification #{id} delivered"),
            None => {}
        });
        UNUserNotificationCenter::currentNotificationCenter().addNotificationRequest_withCompletionHandler(&request, Some(&done));
    }

    /// The content as a message from `sender`: macOS then shows their
    /// picture (or the group's) and name in place of fMess's icon and title.
    fn from_sender(
        content: &UNNotificationContent,
        sender: &Sender,
        body: &str,
    ) -> Option<Retained<UNNotificationContent>> {
        let thread = NSString::from_str(&sender.thread);
        let from = person(&sender.name, sender.picture.as_deref(), &format!("sender:{}", sender.name));
        let group_name = sender.group.as_ref().and_then(|(name, _)| name.as_deref()).map(|name| unsafe {
            INSpeakableString::initWithSpokenPhrase(INSpeakableString::alloc(), &NSString::from_str(name))
        });
        // More than one recipient makes it a group conversation.
        let recipients = sender.group.is_some().then(|| NSArray::from_retained_slice(&[person("Me", None, "me")]));
        unsafe {
            let intent = INSendMessageIntent::initWithRecipients_outgoingMessageType_content_speakableGroupName_conversationIdentifier_serviceName_sender_attachments(
                INSendMessageIntent::alloc(),
                recipients.as_deref(),
                INOutgoingMessageType::OutgoingMessageText,
                Some(&NSString::from_str(body)),
                group_name.as_deref(),
                Some(&thread),
                None,
                Some(&from),
                None,
            );
            if let Some((_, Some(picture))) = &sender.group {
                intent.setImage_forParameterNamed(
                    Some(&INImage::imageWithImageData(&NSData::with_bytes(picture))),
                    &NSString::from_str("speakableGroupName"),
                );
            }
            let interaction = INInteraction::initWithIntent_response(INInteraction::alloc(), &intent, None);
            interaction.setDirection(INInteractionDirection::Incoming);
            interaction.setGroupIdentifier(Some(&thread));
            interaction.donateInteractionWithCompletion(None);
            // INSendMessageIntent provides notification content, though the
            // bindings don't say so.
            let updated: Result<Retained<UNNotificationContent>, Retained<NSError>> =
                msg_send![content, contentByUpdatingWithProvider: &*intent, error: _];
            match updated {
                Ok(updated) => Some(updated),
                Err(error) => {
                    eprintln!("could not show the sender in a notification: {error}");
                    None
                }
            }
        }
    }

    fn person(name: &str, picture: Option<&[u8]>, id: &str) -> Retained<INPerson> {
        let name = NSString::from_str(name);
        let id = NSString::from_str(id);
        unsafe {
            let handle = INPersonHandle::initWithValue_type(INPersonHandle::alloc(), Some(&id), INPersonHandleType::Unknown);
            let image = picture.map(|bytes| INImage::imageWithImageData(&NSData::with_bytes(bytes)));
            INPerson::initWithPersonHandle_nameComponents_displayName_image_contactIdentifier_customIdentifier(
                INPerson::alloc(),
                &handle,
                None,
                Some(&name),
                image.as_deref(),
                None,
                Some(&id),
            )
        }
    }

    /// A photo for a notification. macOS moves the file into its own store.
    fn photo_attachment(id: i32, image: &[u8], dir: &std::path::Path) -> Option<Retained<UNNotificationAttachment>> {
        let extension = match image {
            [0xFF, 0xD8, ..] => "jpg",
            [0x89, b'P', b'N', b'G', ..] => "png",
            [b'G', b'I', b'F', ..] => "gif",
            _ => return None,
        };
        std::fs::create_dir_all(dir).ok()?;
        let path = dir.join(format!("{id}.{extension}"));
        std::fs::write(&path, image).ok()?;
        let url = NSURL::fileURLWithPath(&NSString::from_str(&path.to_string_lossy()));
        let attachment = unsafe {
            UNNotificationAttachment::attachmentWithIdentifier_URL_options_error(&NSString::from_str(&id.to_string()), &url, None)
        };
        if attachment.is_err() {
            let _ = std::fs::remove_file(&path);
        }
        attachment.ok()
    }

    /// Removes the notifications of a chat that has been read.
    pub fn clear(thread: String) {
        let center = UNUserNotificationCenter::currentNotificationCenter();
        let remove = RcBlock::new(move |delivered: std::ptr::NonNull<NSArray<UNNotification>>| {
            let delivered = unsafe { delivered.as_ref() };
            let ids: Vec<Retained<NSString>> = delivered
                .iter()
                .map(|notification| notification.request())
                .filter(|request| request.content().threadIdentifier().to_string() == thread)
                .map(|request| request.identifier())
                .collect();
            if !ids.is_empty() {
                let ids = NSArray::from_retained_slice(&ids);
                UNUserNotificationCenter::currentNotificationCenter().removeDeliveredNotificationsWithIdentifiers(&ids);
            }
        });
        center.getDeliveredNotificationsWithCompletionHandler(&remove);
    }
}
