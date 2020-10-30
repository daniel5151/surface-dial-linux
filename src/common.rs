use notify_rust::error::Result as NotifyResult;
use notify_rust::{Hint, Notification, NotificationHandle, Timeout};

pub fn action_notification(msg: &str, icon: &str) -> NotifyResult<NotificationHandle> {
    Notification::new()
        .hint(Hint::Transient(true))
        .hint(Hint::Category("device".into()))
        .timeout(Timeout::Milliseconds(100))
        .summary("Surface Dial")
        .body(msg)
        .icon(icon)
        .show()
}
