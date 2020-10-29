use crate::common::{action_notification, DialDir, ThresholdHelper};
use crate::controller::ControlMode;
use crate::fake_input::{FakeInput, ScrollStep};
use crate::DynResult;

use evdev_rs::enums::EV_KEY;

pub struct ScrollZoom {
    thresh: ThresholdHelper,
    zoom: bool,

    fake_input: FakeInput,
}

impl ScrollZoom {
    pub fn new(sensitivity: i32) -> ScrollZoom {
        ScrollZoom {
            thresh: ThresholdHelper::new(sensitivity),
            zoom: false,

            fake_input: FakeInput::new(),
        }
    }
}

impl ControlMode for ScrollZoom {
    fn on_btn_press(&mut self) -> DynResult<()> {
        Ok(())
    }

    fn on_btn_release(&mut self) -> DynResult<()> {
        self.zoom = !self.zoom;
        if self.zoom {
            action_notification("Zoom Mode", "zoom-in")?;
        } else {
            action_notification("ScrollZoom Mode", "input-mouse")?;
        }

        Ok(())
    }

    fn on_dial(&mut self, delta: i32) -> DynResult<()> {
        match self.thresh.update(delta) {
            None => {}
            Some(DialDir::Left) => {
                if self.zoom {
                    eprintln!("zoom out");
                    self.fake_input
                        .key_click(&[EV_KEY::KEY_LEFTCTRL, EV_KEY::KEY_MINUS])?;
                } else {
                    eprintln!("scroll up");
                    self.fake_input.scroll_step(ScrollStep::Up)?;
                }
            }
            Some(DialDir::Right) => {
                if self.zoom {
                    eprintln!("zoom in");
                    self.fake_input
                        .key_click(&[EV_KEY::KEY_LEFTCTRL, EV_KEY::KEY_EQUAL])?;
                } else {
                    eprintln!("scroll down");
                    self.fake_input.scroll_step(ScrollStep::Down)?;
                }
            }
        }
        Ok(())
    }
}
