pub enum DialDir {
    Left,
    Right,
}

pub struct ThresholdHelper {
    sensitivity: i32,
    pos: i32,
}

impl ThresholdHelper {
    pub fn new(sensitivity: i32) -> ThresholdHelper {
        ThresholdHelper {
            sensitivity,
            pos: 0,
        }
    }

    pub fn update(&mut self, delta: i32) -> Option<DialDir> {
        self.pos += delta;

        if self.pos > self.sensitivity {
            self.pos -= self.sensitivity;
            return Some(DialDir::Right);
        }

        if self.pos < -self.sensitivity {
            self.pos += self.sensitivity;
            return Some(DialDir::Left);
        }

        None
    }
}

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
