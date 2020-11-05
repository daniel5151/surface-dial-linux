use crate::controller::{ControlMode, ControlModeMeta};
use crate::dial_device::DialHaptics;
use crate::error::{Error, Result};
use crate::fake_input;

pub struct ScrollMT {
    acc_delta: i32,
}

impl ScrollMT {
    pub fn new() -> ScrollMT {
        ScrollMT { acc_delta: 0 }
    }
}

impl ControlMode for ScrollMT {
    fn meta(&self) -> ControlModeMeta {
        ControlModeMeta {
            name: "Scroll",
            icon: "input-mouse",
        }
    }

    fn on_start(&mut self, haptics: &DialHaptics) -> Result<()> {
        haptics.set_mode(false, Some(3600))?;
        self.acc_delta = 0;

        fake_input::scroll_mt_start().map_err(Error::Evdev)?;

        Ok(())
    }

    fn on_end(&mut self, _haptics: &DialHaptics) -> Result<()> {
        fake_input::scroll_mt_end().map_err(Error::Evdev)?;
        Ok(())
    }

    // HACK: the button will reset the scroll event, which sometimes helps

    fn on_btn_press(&mut self, _: &DialHaptics) -> Result<()> {
        fake_input::scroll_mt_end().map_err(Error::Evdev)?;
        Ok(())
    }

    fn on_btn_release(&mut self, _haptics: &DialHaptics) -> Result<()> {
        fake_input::scroll_mt_start().map_err(Error::Evdev)?;
        Ok(())
    }

    fn on_dial(&mut self, _: &DialHaptics, delta: i32) -> Result<()> {
        self.acc_delta += delta;
        fake_input::scroll_mt_step(self.acc_delta).map_err(Error::Evdev)?;

        Ok(())
    }
}
