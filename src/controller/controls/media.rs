use crate::common::{DialDir, ThresholdHelper};
use crate::controller::ControlMode;
use crate::fake_input::FakeInput;
use crate::DynResult;

use evdev_rs::enums::EV_KEY;

pub struct Media {
    thresh: ThresholdHelper,

    fake_input: FakeInput,
}

impl Media {
    pub fn new(sensitivity: i32) -> Media {
        Media {
            thresh: ThresholdHelper::new(sensitivity),

            fake_input: FakeInput::new(),
        }
    }
}

impl ControlMode for Media {
    fn on_btn_press(&mut self) -> DynResult<()> {
        Ok(())
    }

    fn on_btn_release(&mut self) -> DynResult<()> {
        self.fake_input.key_click(&[EV_KEY::KEY_PLAYPAUSE])?;
        Ok(())
    }

    fn on_dial(&mut self, delta: i32) -> DynResult<()> {
        match self.thresh.update(delta) {
            Some(DialDir::Left) => {
                eprintln!("next song");
                self.fake_input.key_click(&[EV_KEY::KEY_NEXTSONG])?;
            }
            Some(DialDir::Right) => {
                eprintln!("last song");
                self.fake_input.key_click(&[EV_KEY::KEY_PREVIOUSSONG])?;
            }
            None => {}
        }
        Ok(())
    }
}
