use crate::DynResult;

use crate::dial_device::{DialDevice, DialEventKind, DialHaptics};

pub mod controls;

pub trait ControlMode {
    fn on_start(&mut self, haptics: &DialHaptics) -> DynResult<()>;
    fn on_btn_press(&mut self, haptics: &DialHaptics) -> DynResult<()>;
    fn on_btn_release(&mut self, haptics: &DialHaptics) -> DynResult<()>;
    fn on_dial(&mut self, haptics: &DialHaptics, delta: i32) -> DynResult<()>;
}

pub struct DialController {
    device: DialDevice,

    mode: Box<dyn ControlMode>,
}

impl DialController {
    pub fn new(device: DialDevice, default_mode: Box<dyn ControlMode>) -> DialController {
        DialController {
            mode: default_mode,

            device,
        }
    }

    pub fn run(&mut self) -> DynResult<()> {
        let haptics = self.device.haptics();

        self.mode.on_start(haptics)?;

        loop {
            let evt = self.device.next_event()?;

            // TODO: press and hold (+ rotate?) to switch between modes

            match evt.kind {
                DialEventKind::Ignored => {}
                DialEventKind::ButtonPress => self.mode.on_btn_press(haptics)?,
                DialEventKind::ButtonRelease => self.mode.on_btn_release(haptics)?,
                DialEventKind::Dial(delta) => self.mode.on_dial(haptics, delta)?,
            }
        }
    }
}
