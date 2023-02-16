use crate::controller::{ControlMode, ControlModeMeta};
use crate::dial_device::DialHaptics;
use crate::error::{Error, Result};
use crate::fake_input;

use evdev_rs::enums::EV_KEY;

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
            name: "Scroll (Fake Multitouch - EXPERIMENTAL)",
            icon: "input-mouse",
            haptics: false,
            steps: 3600,
        }
    }

    fn on_start(&mut self, _haptics: &DialHaptics) -> Result<()> {
        self.acc_delta = 0;

        // HACK: for some reason, if scroll mode is the startup mode, then just calling
        // `scroll_mt_start` doesn't work as expected.
        std::thread::spawn(move || {
            fake_input::scroll_mt_end().unwrap();
            std::thread::sleep(std::time::Duration::from_millis(200));
            fake_input::scroll_mt_start().unwrap();
        });

        Ok(())
    }

    fn on_end(&mut self, _haptics: &DialHaptics) -> Result<()> {
        fake_input::scroll_mt_end().map_err(Error::Evdev)?;
        Ok(())
    }

    // HACK: the button will reset the scroll event, which sometimes helps

    fn on_btn_press(&mut self, _: &DialHaptics) -> Result<()> {
       // fake_input::scroll_mt_end().map_err(Error::Evdev)?;
        Ok(())
    }

    fn on_btn_release(&mut self, _haptics: &DialHaptics) -> Result<()> {
        fake_input::key_click(&[EV_KEY::BTN_LEFT]).map_err(Error::Evdev)?;
        Ok(())
    }

    fn on_dial(&mut self, _: &DialHaptics, delta: i32) -> Result<()> {
        self.acc_delta += delta;
        fake_input::scroll_mt_step(self.acc_delta).map_err(Error::Evdev)?;

        Ok(())
    }
}
