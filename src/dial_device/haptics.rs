use std::sync::mpsc;

use hidapi::{HidApi, HidDevice};

use crate::error::{Error, Result};

/// Proxy object - forwards requests to the DialHapticsWorker task
pub struct DialHaptics {
    msg: mpsc::Sender<DialHapticsWorkerMsg>,
}

impl DialHaptics {
    pub(super) fn new(msg: mpsc::Sender<DialHapticsWorkerMsg>) -> Result<DialHaptics> {
        Ok(DialHaptics { msg })
    }

    /// `steps` should be a value between 0 and 3600, which corresponds to the
    /// number of subdivisions the dial should use.
    pub fn set_mode(&self, haptics: bool, steps: u16) -> Result<()> {
        let _ = (self.msg).send(DialHapticsWorkerMsg::SetMode { haptics, steps });
        Ok(())
    }

    pub fn buzz(&self, repeat: u8) -> Result<()> {
        let _ = (self.msg).send(DialHapticsWorkerMsg::Manual { repeat });
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
            eprintln!("haptics worker is waiting...");

            loop {
                match self.msg.recv().unwrap() {
                    DialHapticsWorkerMsg::DialConnected => break,
                    other => eprintln!("haptics worker dropped an event: {:?}", other),
                }
            }

            eprintln!("haptics worker is ready");

            let api = HidApi::new().map_err(Error::HidError)?;
            let hid_device = api.open(0x045e, 0x091b).map_err(|_| Error::MissingDial)?;
            let wrapper = DialHidWrapper { hid_device };

            loop {
                match self.msg.recv().unwrap() {
                    DialHapticsWorkerMsg::DialConnected => {
                        eprintln!("Unexpected haptics worker ready event.");
                        // should be fine though?
                    }
                    DialHapticsWorkerMsg::DialDisconnected => break,
                    DialHapticsWorkerMsg::SetMode { haptics, steps } => {
                        wrapper.set_mode(haptics, steps)?
                    }
                    DialHapticsWorkerMsg::Manual { repeat } => wrapper.buzz(repeat)?,
                }
            }
        }
    }
}

struct DialHidWrapper {
    hid_device: HidDevice,
}

impl DialHidWrapper {
    /// `steps` should be a value between 0 and 3600, which corresponds to the
    /// number of subdivisions the dial should use. If left unspecified, this
    /// defaults to 36 (an arbitrary choice that "feels good" most of the time)
    fn set_mode(&self, haptics: bool, steps: u16) -> Result<()> {
        assert!(steps <= 3600);

        let steps_lo = steps & 0xff;
        let steps_hi = (steps >> 8) & 0xff;

        let mut buf = [0; 8];
        buf[0] = 1;
        buf[1] = steps_lo as u8; // steps
        buf[2] = steps_hi as u8; // steps
        buf[3] = 0x00; // Repeat Count
        buf[4] = if haptics { 0x03 } else { 0x02 }; // auto trigger
        buf[5] = 0x00; // Waveform Cutoff Time
        buf[6] = 0x00; // retrigger period
        buf[7] = 0x00; // retrigger period
        self.hid_device
            .send_feature_report(&buf[..8])
            .map_err(Error::HidError)?;

        Ok(())
    }

    fn buzz(&self, repeat: u8) -> Result<()> {
        let mut buf = [0; 5];
        buf[0] = 0x01; // Report ID
        buf[1] = repeat; // RepeatCount
        buf[2] = 0x03; // ManualTrigger
        buf[3] = 0x00; // RetriggerPeriod (lo)
        buf[4] = 0x00; // RetriggerPeriod (hi)
        self.hid_device.write(&buf).map_err(Error::HidError)?;
        Ok(())
    }
}
