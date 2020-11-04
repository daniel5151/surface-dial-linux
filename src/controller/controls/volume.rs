use crate::controller::{ControlMode, ControlModeMeta};
use crate::dial_device::DialHaptics;
use crate::error::{Error, Result};
use crate::fake_input::FakeInput;

use evdev_rs::enums::EV_KEY;

pub struct Volume {
    fake_input: FakeInput,
}

impl Volume {
    pub fn new() -> Volume {
        Volume {
            fake_input: FakeInput::new(),
        }
    }
}

impl ControlMode for Volume {
    fn meta(&self) -> ControlModeMeta {
        ControlModeMeta {
            name: "Volume",
            icon: "audio-volume-high",
        }
    }

    fn on_start(&mut self, haptics: &DialHaptics) -> Result<()> {
        haptics.set_mode(true, Some(36 * 2))?;
        Ok(())
    }

    fn on_btn_press(&mut self, _: &DialHaptics) -> Result<()> {
        // TODO: support double-click to play/pause
        Ok(())
    }

    fn on_btn_release(&mut self, _: &DialHaptics) -> Result<()> {
        eprintln!("play/pause");
        // self.fake_input.mute()?
        self.fake_input
            .key_click(&[EV_KEY::KEY_PLAYPAUSE])
            .map_err(Error::Evdev)?;
        Ok(())
    }

    fn on_dial(&mut self, _: &DialHaptics, delta: i32) -> Result<()> {
        if delta > 0 {
            eprintln!("volume up");
            self.fake_input
                .key_click(&[EV_KEY::KEY_LEFTSHIFT, EV_KEY::KEY_VOLUMEUP])
                .map_err(Error::Evdev)?;
        } else {
            eprintln!("volume down");
            self.fake_input
                .key_click(&[EV_KEY::KEY_LEFTSHIFT, EV_KEY::KEY_VOLUMEDOWN])
                .map_err(Error::Evdev)?;
        }

        Ok(())
    }
}
