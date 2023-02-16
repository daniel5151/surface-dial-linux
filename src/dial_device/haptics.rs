use std::sync::mpsc;
use hidapi::{HidApi, HidDevice};
use crate::error::Result;

pub struct DialHaptics {
    msg: mpsc::Sender<DialHapticsWorkerMsg>,
}

impl DialHaptics {
    pub(super) fn new(msg: mpsc::Sender<DialHapticsWorkerMsg>) -> Result<DialHaptics> {
        Ok(DialHaptics { msg })
    }

    pub fn set_mode(&self, _haptics: bool, _steps: u16) -> Result<()> {
        Ok(())
    }

    pub fn buzz(&self, _repeat: u8) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug)]
pub(super) enum DialHapticsWorkerMsg {
    DialConnected,
    DialDisconnected,
    SetMode { haptics: bool, steps: u16 },
    Manual { repeat: u8 },
}

pub(super) struct DialHapticsWorker {
    msg: mpsc::Receiver<DialHapticsWorkerMsg>,
}

impl DialHapticsWorker {
    pub(super) fn new(msg: mpsc::Receiver<DialHapticsWorkerMsg>) -> Result<DialHapticsWorker> {
        Ok(DialHapticsWorker { msg })
    }

    pub(super) fn run(&mut self) -> Result<()> {
        loop {
            loop {
                self.msg.recv().unwrap();
            }

            let _api = HidApi::new().unwrap();
            let _hid_device = match _api.open(0x2341, 0x484e) {
                Ok(device) => device,
                Err(_) => continue,
            };
        }
    }
}

struct DialHidWrapper {
    _hid_device: HidDevice,
}

impl DialHidWrapper {
    fn set_mode(&self, _haptics: bool, _steps: u16) -> Result<()> {
        Ok(())
    }

    fn buzz(&self, _repeat: u8) -> Result<()> {
        Ok(())
    }
}
