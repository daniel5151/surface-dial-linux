use crate::controller::{ControlMode, ControlModeMeta};
use crate::dial_device::DialHaptics;
use crate::error::{Error, Result};
use crate::fake_input::{FakeInput, ScrollStep};

pub struct Scroll {
    fake_input: FakeInput,
}

impl Scroll {
    pub fn new() -> Scroll {
        Scroll {
            fake_input: FakeInput::new(),
        }
    }
}

impl ControlMode for Scroll {
    fn meta(&self) -> ControlModeMeta {
        ControlModeMeta {
            name: "Scroll",
            icon: "input-mouse",
        }
    }

    fn on_start(&mut self, haptics: &DialHaptics) -> Result<()> {
        haptics.set_mode(false, Some(90))?;
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
            eprintln!("scroll down");
            self.fake_input
                .scroll_step(ScrollStep::Down)
                .map_err(Error::Evdev)?;
        } else {
            eprintln!("scroll up");
            self.fake_input
                .scroll_step(ScrollStep::Up)
                .map_err(Error::Evdev)?;
        }

        Ok(())
    }
}
