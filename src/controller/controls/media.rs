use crate::controller::{ControlMode, ControlModeMeta};
use crate::dial_device::DialHaptics;
use crate::error::{Error, Result};
use crate::fake_input::FakeInput;

use evdev_rs::enums::EV_KEY;

pub struct Media {
    fake_input: FakeInput,
}

impl Media {
    pub fn new() -> Media {
        Media {
            fake_input: FakeInput::new(),
        }
    }
}

impl ControlMode for Media {
    fn meta(&self) -> ControlModeMeta {
        ControlModeMeta {
            name: "Media",
            icon: "applications-multimedia",
        }
    }

    fn on_start(&mut self, haptics: &DialHaptics) -> Result<()> {
        haptics.set_mode(true, Some(36))?;
        Ok(())
    }

    fn on_btn_press(&mut self, _: &DialHaptics) -> Result<()> {
        Ok(())
    }

    fn on_btn_release(&mut self, _: &DialHaptics) -> Result<()> {
        self.fake_input
            .key_click(&[EV_KEY::KEY_PLAYPAUSE])
            .map_err(Error::Evdev)?;
        Ok(())
    }

    fn on_dial(&mut self, _: &DialHaptics, delta: i32) -> Result<()> {
        if delta > 0 {
            eprintln!("next song");
            self.fake_input
                .key_click(&[EV_KEY::KEY_NEXTSONG])
                .map_err(Error::Evdev)?;
        } else {
            eprintln!("last song");
            self.fake_input
                .key_click(&[EV_KEY::KEY_PREVIOUSSONG])
                .map_err(Error::Evdev)?;
        }
        Ok(())
    }
}
