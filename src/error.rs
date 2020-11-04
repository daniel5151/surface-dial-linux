use std::fmt;
use std::io;

use evdev_rs::InputEvent;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ConfigFile(String),
    OpenDevInputDir(io::Error),
    OpenEventFile(std::path::PathBuf, io::Error),
    HidError(hidapi::HidError),
    MissingDial,
    MultipleDials,
    UnexpectedEvt(InputEvent),
    Evdev(io::Error),
    Notif(notify_rust::error::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ConfigFile(e) => write!(f, "Could not open config file: {}", e),
            Error::OpenDevInputDir(e) => write!(f, "Could not open /dev/input directory: {}", e),
            Error::OpenEventFile(path, e) => write!(f, "Could not open {:?}: {}", path, e),
            Error::HidError(e) => write!(f, "HID API Error: {}", e),
            Error::MissingDial => write!(f, "Could not find the Surface Dial"),
            Error::MultipleDials => write!(f, "Found multiple dials"),
            Error::UnexpectedEvt(evt) => write!(f, "Unexpected event: {:?}", evt),
            Error::Evdev(e) => write!(f, "Evdev error: {}", e),
            Error::Notif(e) => write!(f, "Notification error: {}", e),
        }
    }
}

impl std::error::Error for Error {}
