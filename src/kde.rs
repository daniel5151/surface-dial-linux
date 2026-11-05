//! KDE Plasma global shortcuts, triggered over D-Bus.
//!
//! Going through kglobalaccel instead of faking media keys works in every
//! session type (local, xrdp...) and shows Plasma's own volume OSD.

use std::sync::LazyLock;

use zbus::blocking::Connection;

static SESSION_BUS: LazyLock<Option<Connection>> = LazyLock::new(|| Connection::session().ok());

/// Invokes a global shortcut. Returns `false` if KDE is not available, so the
/// caller can fall back to a fake key press.
pub fn invoke(component: &str, shortcut: &str) -> bool {
    let Some(conn) = SESSION_BUS.as_ref() else {
        return false;
    };
    let path = format!("/component/{}", component);
    match conn.call_method(
        Some("org.kde.kglobalaccel"),
        path.as_str(),
        Some("org.kde.kglobalaccel.Component"),
        "invokeShortcut",
        &(shortcut,),
    ) {
        Ok(_) => true,
        Err(e) => {
            eprintln!("KDE shortcut {}/{} failed: {}", component, shortcut, e);
            false
        }
    }
}
