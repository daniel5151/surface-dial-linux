use crate::common::action_notification;
use crate::controller::ControlMode;
use crate::dial_device::DialHaptics;
use crate::fake_input::{FakeInput, ScrollStep};
use crate::DynResult;

use evdev_rs::enums::EV_KEY;

pub struct ScrollZoom {
    zoom: bool,

    fake_input: FakeInput,
}

impl ScrollZoom {
    pub fn new() -> ScrollZoom {
        ScrollZoom {
            zoom: false,

            fake_input: FakeInput::new(),
        }
    }
}

const ZOOM_SENSITIVITY: u16 = 36;
const SCROLL_SENSITIVITY: u16 = 90;

impl ControlMode for ScrollZoom {
    fn on_start(&mut self, haptics: &DialHaptics) -> DynResult<()> {
        haptics.set_mode(false, Some(SCROLL_SENSITIVITY))?;
        Ok(())
    }

    fn on_btn_press(&mut self, _: &DialHaptics) -> DynResult<()> {
        Ok(())
    }

    fn on_btn_release(&mut self, haptics: &DialHaptics) -> DynResult<()> {
        self.zoom = !self.zoom;
        haptics.buzz(1)?;

        if self.zoom {
            action_notification("Zoom Mode", "zoom-in")?;
            haptics.set_mode(false, Some(ZOOM_SENSITIVITY))?;
        } else {
            action_notification("ScrollZoom Mode", "input-mouse")?;
            haptics.set_mode(false, Some(SCROLL_SENSITIVITY))?;
        }

        Ok(())
    }

    fn on_dial(&mut self, _: &DialHaptics, delta: i32) -> DynResult<()> {
        if delta > 0 {
            if self.zoom {
                eprintln!("zoom in");
                self.fake_input
                    .key_click(&[EV_KEY::KEY_LEFTCTRL, EV_KEY::KEY_EQUAL])?;
            } else {
                eprintln!("scroll down");
                self.fake_input.scroll_step(ScrollStep::Down)?;
            }
        } else {
            if self.zoom {
                eprintln!("zoom out");
                self.fake_input
                    .key_click(&[EV_KEY::KEY_LEFTCTRL, EV_KEY::KEY_MINUS])?;
            } else {
                eprintln!("scroll up");
                self.fake_input.scroll_step(ScrollStep::Up)?;
            }
        }

        Ok(())
    }
}
