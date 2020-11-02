use crate::controller::{ControlMode, ControlModeMeta};
use crate::dial_device::DialHaptics;
use crate::fake_input::FakeInput;
use crate::DynResult;

pub struct ScrollMT {
    acc_delta: i32,

    fake_input: FakeInput,
}

impl ScrollMT {
    pub fn new() -> ScrollMT {
        ScrollMT {
            acc_delta: 0,
            fake_input: FakeInput::new(),
        }
    }
}

impl ControlMode for ScrollMT {
    fn meta(&self) -> ControlModeMeta {
        ControlModeMeta {
            name: "Scroll",
            icon: "input-mouse",
        }
    }

    fn on_start(&mut self, haptics: &DialHaptics) -> DynResult<()> {
        haptics.set_mode(false, Some(3600))?;
        self.acc_delta = 0;

        self.fake_input.scroll_mt_start()?;

        Ok(())
    }

    fn on_end(&mut self, _haptics: &DialHaptics) -> DynResult<()> {
        self.fake_input.scroll_mt_end()?;
        Ok(())
    }

    // HACK: the button will reset the scroll event, which sometimes helps

    fn on_btn_press(&mut self, _: &DialHaptics) -> DynResult<()> {
        self.fake_input.scroll_mt_end()?;
        Ok(())
    }

    fn on_btn_release(&mut self, _haptics: &DialHaptics) -> DynResult<()> {
        self.fake_input.scroll_mt_start()?;
        Ok(())
    }

    fn on_dial(&mut self, _: &DialHaptics, delta: i32) -> DynResult<()> {
        self.acc_delta += delta;
        self.fake_input.scroll_mt_step(self.acc_delta)?;

        Ok(())
    }
}
