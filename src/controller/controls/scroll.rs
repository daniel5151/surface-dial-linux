use crate::controller::{ControlMode, ControlModeMeta};
use crate::dial_device::DialHaptics;
use crate::error::{Error, Result};
use crate::fake_input::{self, ScrollStep};

use evdev_rs::enums::EV_KEY;

pub struct Scroll {}

impl Scroll {
    pub fn new() -> Scroll {
        Scroll {}
    }
}

impl ControlMode for Scroll {
    fn meta(&self) -> ControlModeMeta {
        ControlModeMeta {
            name: "Scroll",
            icon: "input-mouse",
            haptics: false,
            steps: 90,
        }
    }

    fn on_btn_press(&mut self, _: &DialHaptics) -> Result<()> {
        //fake_input::key_click(&[EV_KEY::BTN_LEFT]).map_err(Error::Evdev)?;
        Ok(())
    }

    fn on_btn_release(&mut self, _haptics: &DialHaptics) -> Result<()> {
        fake_input::key_click(&[EV_KEY::BTN_LEFT]).map_err(Error::Evdev)?;
        Ok(())
    }

    fn on_dial(&mut self, _: &DialHaptics, delta: i32) -> Result<()> {
        if delta > 0 {
            eprintln!("scroll down");
            fake_input::scroll_step(ScrollStep::Down).map_err(Error::Evdev)?;
        } else {
            eprintln!("scroll up");
            fake_input::scroll_step(ScrollStep::Up).map_err(Error::Evdev)?;
        }

        Ok(())
    }
}
