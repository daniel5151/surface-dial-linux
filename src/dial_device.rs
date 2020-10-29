use std::fs;
use std::time::Duration;

use evdev_rs::{Device, InputEvent};

use crate::error::Error;

pub struct DialDevice {
    // TODO: explore what the control channel can be used for...
    _control: Device,
    axis: Device,
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

        for e in fs::read_dir("/dev/input/").map_err(Error::Io)? {
            let e = e.map_err(Error::Io)?;
            if !e.file_name().to_str().unwrap().starts_with("event") {
                continue;
            }

            let file = fs::File::open(e.path()).map_err(Error::Io)?;
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
