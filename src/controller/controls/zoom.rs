use crate::controller::{ControlMode, ControlModeMeta};
use crate::dial_device::DialHaptics;
use crate::error::{Error, Result};
use crate::fake_input::FakeInput;

use evdev_rs::enums::EV_KEY;

pub struct Zoom {
    fake_input: FakeInput,
}

impl Zoom {
    pub fn new() -> Zoom {
        Zoom {
            fake_input: FakeInput::new(),
        }
    }
}

impl ControlMode for Zoom {
    fn meta(&self) -> ControlModeMeta {
        ControlModeMeta {
            name: "Zoom",
            icon: "zoom-in",
        }
    }

    fn on_start(&mut self, haptics: &DialHaptics) -> Result<()> {
        haptics.set_mode(true, Some(36))?;
        Ok(())
    }

    fn on_btn_press(&mut self, _: &DialHaptics) -> Result<()> {
        Ok(())
    }

    fn on_btn_release(&mut self, _haptics: &DialHaptics) -> Result<()> {
        Ok(())
    }

    fn on_dial(&mut self, _: &DialHaptics, delta: i32) -> Result<()> {
        if delta > 0 {
            eprintln!("zoom in");
            self.fake_input
                .key_click(&[EV_KEY::KEY_LEFTCTRL, EV_KEY::KEY_EQUAL])
                .map_err(Error::Evdev)?;
        } else {
            eprintln!("zoom out");
            self.fake_input
                .key_click(&[EV_KEY::KEY_LEFTCTRL, EV_KEY::KEY_MINUS])
                .map_err(Error::Evdev)?;
        }

        Ok(())
    }
}
