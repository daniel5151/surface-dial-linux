use crate::common::{DialDir, ThresholdHelper};
use crate::controller::ControlMode;
use crate::fake_input::FakeInput;
use crate::DynResult;

use evdev_rs::enums::EV_KEY;

pub struct Volume {
    thresh: ThresholdHelper,

    fake_input: FakeInput,
}

impl Volume {
    pub fn new(sensitivity: i32) -> Volume {
        Volume {
            thresh: ThresholdHelper::new(sensitivity),

            fake_input: FakeInput::new(),
        }
    }
}

impl ControlMode for Volume {
    fn on_btn_press(&mut self) -> DynResult<()> {
        // TODO: support double-click to mute
        Ok(())
    }

    fn on_btn_release(&mut self) -> DynResult<()> {
        eprintln!("play/pause");
        // self.fake_input.mute()?
        self.fake_input.key_click(&[EV_KEY::KEY_PLAYPAUSE])?;
        Ok(())
    }

    fn on_dial(&mut self, delta: i32) -> DynResult<()> {
        match self.thresh.update(delta) {
            Some(DialDir::Left) => {
                eprintln!("volume down");
                self.fake_input
                    .key_click(&[EV_KEY::KEY_LEFTSHIFT, EV_KEY::KEY_VOLUMEDOWN])?
            }
            Some(DialDir::Right) => {
                eprintln!("volume up");
                self.fake_input
                    .key_click(&[EV_KEY::KEY_LEFTSHIFT, EV_KEY::KEY_VOLUMEUP])?
            }
            None => {}
        }
        Ok(())
    }
}
