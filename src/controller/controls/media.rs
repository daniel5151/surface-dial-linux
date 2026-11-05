use crate::controller::{ControlMode, ControlModeMeta};
use crate::dial_device::DialHaptics;
use crate::error::{Error, Result};
use crate::fake_input;

use crate::fake_input::Action;

pub struct Media {}

impl Media {
    pub fn new() -> Media {
        Media {}
    }
}

impl ControlMode for Media {
    fn meta(&self) -> ControlModeMeta {
        ControlModeMeta {
            name: "Media",
            icon: "applications-multimedia",
            haptics: true,
            steps: 36,
        }
    }

    fn on_btn_press(&mut self, _: &DialHaptics) -> Result<()> {
        Ok(())
    }

    fn on_btn_release(&mut self, _: &DialHaptics) -> Result<()> {
        fake_input::action(Action::PlayPause).map_err(Error::Evdev)?;
        Ok(())
    }

    fn on_dial(&mut self, _: &DialHaptics, delta: i32) -> Result<()> {
        if delta > 0 {
            eprintln!("next song");
            fake_input::action(Action::Next).map_err(Error::Evdev)?;
        } else {
            eprintln!("last song");
            fake_input::action(Action::Previous).map_err(Error::Evdev)?;
        }
        Ok(())
    }
}
