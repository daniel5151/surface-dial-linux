use std::fmt;

use evdev_rs::InputEvent;

#[derive(Debug)]
pub enum Error {
    MissingDial,
    MultipleDials,
    UnexpectedEvt(InputEvent),
    Evdev(std::io::Error),
    Io(std::io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::MissingDial => write!(f, "Could not find the Surface Dial"),
            Error::MultipleDials => write!(f, "Found multiple dials"),
            Error::UnexpectedEvt(evt) => write!(f, "Unexpected event: {:?}", evt),
            Error::Evdev(e) => write!(f, "Evdev error: {:?}", e),
            Error::Io(e) => write!(f, "Io error: {:?}", e),
        }
    }
}

impl std::error::Error for Error {}
