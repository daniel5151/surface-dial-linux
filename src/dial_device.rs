use std::fs;
use std::time::Duration;

use evdev_rs::{Device, InputEvent};
use hidapi::{HidApi, HidDevice};

use crate::error::Error;

pub struct DialDevice {
    // TODO: explore what the control channel can be used for...
    _control: Device,
    axis: Device,
    haptics: DialHaptics,
}

#[derive(Debug)]
pub struct DialEvent {
    pub time: Duration,
    pub kind: DialEventKind,
}

#[derive(Debug)]
pub enum DialEventKind {
    Ignored,
    ButtonPress,
    ButtonRelease,
    Dial(i32),
}

impl DialDevice {
    pub fn new() -> Result<DialDevice, crate::Error> {
        let mut control = None;
        let mut axis = None;

        // discover the evdev devices
        for e in fs::read_dir("/dev/input/").map_err(Error::OpenDevInputDir)? {
            let e = e.map_err(Error::OpenDevInputDir)?;
            if !e.file_name().to_str().unwrap().starts_with("event") {
                continue;
            }

            let file =
                fs::File::open(e.path()).map_err(|err| Error::OpenEventFile(e.path(), err))?;
            let dev = Device::new_from_fd(file).map_err(Error::Evdev)?;

            match dev.name() {
                Some("Surface Dial System Control") => match control {
                    None => control = Some(dev),
                    Some(_) => return Err(Error::MultipleDials),
                },
                Some("Surface Dial System Multi Axis") => match axis {
                    None => axis = Some(dev),
                    Some(_) => return Err(Error::MultipleDials),
                },
                // Some(other) => println!("{:?}", other),
                _ => {}
            }

            // early return once both were found
            if control.is_some() && axis.is_some() {
                break;
            }
        }

        Ok(DialDevice {
            _control: control.ok_or(Error::MissingDial)?,
            axis: axis.ok_or(Error::MissingDial)?,
            haptics: DialHaptics::new()?,
        })
    }

    pub fn next_event(&self) -> Result<DialEvent, Error> {
        // TODO: figure out how to interleave control events into the same event stream.

        let (_axis_status, axis_evt) = self
            .axis
            .next_event(evdev_rs::ReadFlag::NORMAL)
            .map_err(Error::Evdev)?;
        // assert!(matches!(axis_status, ReadStatus::Success));

        let event =
            DialEvent::from_raw_evt(axis_evt.clone()).ok_or(Error::UnexpectedEvt(axis_evt))?;

        Ok(event)
    }

    pub fn haptics(&self) -> &DialHaptics {
        &self.haptics
    }
}

impl DialEvent {
    fn from_raw_evt(evt: InputEvent) -> Option<DialEvent> {
        use evdev_rs::enums::*;

        let evt_kind = match evt.event_type {
            EventType::EV_SYN | EventType::EV_MSC => DialEventKind::Ignored,
            EventType::EV_KEY => match evt.event_code {
                EventCode::EV_KEY(EV_KEY::BTN_0) => match evt.value {
                    0 => DialEventKind::ButtonRelease,
                    1 => DialEventKind::ButtonPress,
                    _ => return None,
                },
                _ => return None,
            },
            EventType::EV_REL => match evt.event_code {
                EventCode::EV_REL(EV_REL::REL_DIAL) => DialEventKind::Dial(evt.value),
                _ => return None,
            },
            _ => return None,
        };

        let evt = DialEvent {
            time: Duration::new(evt.time.tv_sec as u64, (evt.time.tv_usec * 1000) as u32),
            kind: evt_kind,
        };

        Some(evt)
    }
}

pub struct DialHaptics {
    hid_device: HidDevice,
}

impl DialHaptics {
    fn new() -> Result<DialHaptics, Error> {
        let api = HidApi::new().map_err(Error::HidError)?;
        let hid_device = api.open(0x045e, 0x091b).map_err(|_| Error::MissingDial)?;

        // let mut buf = [0; 256];

        // buf[0] = 1;
        // let len = device
        //     .get_feature_report(&mut buf)
        //     .map_err(Error::HidError)?;
        // eprintln!("1: {:02x?}", &buf[..len]);

        // buf[0] = 2;
        // let len = device
        //     .get_feature_report(&mut buf)
        //     .map_err(Error::HidError)?;
        // eprintln!("2: {:02x?}", &buf[..len]);

        Ok(DialHaptics { hid_device })
    }

    /// `steps` should be a value between 0 and 3600, which corresponds to the
    /// number of subdivisions the dial should use. If left unspecified, this
    /// defaults to 36 (an arbitrary choice that "feels good" most of the time)
    pub fn set_mode(&self, haptics: bool, steps: Option<u16>) -> Result<(), Error> {
        let steps = steps.unwrap_or(36);
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

    pub fn buzz(&self, repeat: u8) -> Result<(), Error> {
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
