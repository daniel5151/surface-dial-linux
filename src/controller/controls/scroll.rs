use crate::controller::{ControlMode, ControlModeMeta};
use crate::dial_device::DialHaptics;
use crate::fake_input::{FakeInput, ScrollStep};
use crate::DynResult;

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

    fn on_start(&mut self, haptics: &DialHaptics) -> DynResult<()> {
        haptics.set_mode(false, Some(90))?;
        Ok(())
    }

    fn on_btn_press(&mut self, _: &DialHaptics) -> DynResult<()> {
        Ok(())
    }

    fn on_btn_release(&mut self, _haptics: &DialHaptics) -> DynResult<()> {
        Ok(())
    }

    fn on_dial(&mut self, _: &DialHaptics, delta: i32) -> DynResult<()> {
        if delta > 0 {
            eprintln!("scroll down");
            self.fake_input.scroll_step(ScrollStep::Down)?;
        } else {
            eprintln!("scroll up");
            self.fake_input.scroll_step(ScrollStep::Up)?;
        }

        Ok(())
    }
}
