use crate::controller::{ControlMode, ControlModeMeta};
use crate::dial_device::DialHaptics;
use crate::error::{Error, Result};
use crate::fake_input;

use evdev_rs::enums::EV_KEY;

pub struct Seek {}

impl Seek {
    pub fn new() -> Seek {
        Seek {}
    }
}

impl ControlMode for Seek {
    fn meta(&self) -> ControlModeMeta {
        ControlModeMeta {
            name: "Seek",
            icon: "applications-multimedia",
            haptics: true,
            steps: 36,
        }
    }

    fn on_btn_press(&mut self, _: &DialHaptics) -> Result<()> {
        Ok(())
    }

    fn on_btn_release(&mut self, _: &DialHaptics) -> Result<()> {
        fake_input::key_click(&[EV_KEY::KEY_PLAYPAUSE]).map_err(Error::Evdev)?;
        Ok(())
    }

    fn on_dial(&mut self, _: &DialHaptics, delta: i32) -> Result<()> {
        if delta > 0 {
            eprintln!("fast forward");
            fake_input::key_click(&[EV_KEY::KEY_FASTFORWARD]).map_err(Error::Evdev)?;
        } else {
            eprintln!("rewind");
            fake_input::key_click(&[EV_KEY::KEY_REWIND]).map_err(Error::Evdev)?;
        }
        Ok(())
    }
}