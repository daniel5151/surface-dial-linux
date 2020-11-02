use std::fs;
use std::sync::mpsc;
use std::time::Duration;

use evdev_rs::{Device, InputEvent, ReadStatus};
use hidapi::{HidApi, HidDevice};

use crate::error::Error;

pub struct DialDevice {
    long_press_timeout: Duration,
    haptics: DialHaptics,
    events: mpsc::Receiver<std::io::Result<(ReadStatus, InputEvent)>>,

    possible_long_press: bool,
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
    ButtonLongPress,
}

impl DialDevice {
    pub fn new(long_press_timeout: Duration) -> Result<DialDevice, crate::Error> {
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

        // TODO: explore what the control channel can be used for...
        let _control = control.ok_or(Error::MissingDial)?;
        let axis = axis.ok_or(Error::MissingDial)?;

        let (events_tx, events_rx) = mpsc::channel();

        // TODO: interleave control events with regular events

        std::thread::spawn({
            let events = events_tx;
            move || loop {
                events
                    .send(axis.next_event(evdev_rs::ReadFlag::NORMAL))
                    .expect("failed to send axis event");
            }
        });

        Ok(DialDevice {
            long_press_timeout,
            events: events_rx,
            haptics: DialHaptics::new()?,

            possible_long_press: false,
        })
    }

    pub fn next_event(&mut self) -> Result<DialEvent, Error> {
        let evt = if self.possible_long_press {
            self.events.recv_timeout(self.long_press_timeout)
        } else {
            self.events
                .recv()
                .map_err(|_| mpsc::RecvTimeoutError::Disconnected)
        };

        let event = match evt {
            Ok(Ok((_event_status, event))) => {
                // assert!(matches!(axis_status, ReadStatus::Success));
                let event =
                    DialEvent::from_raw_evt(event.clone()).ok_or(Error::UnexpectedEvt(event))?;
                match event.kind {
                    DialEventKind::ButtonPress => self.possible_long_press = true,
                    DialEventKind::ButtonRelease => self.possible_long_press = false,
                    _ => {}
                }
                event
            }
            Ok(Err(e)) => return Err(Error::Evdev(e)),
            Err(mpsc::RecvTimeoutError::Timeout) => {
                self.possible_long_press = false;
                DialEvent {
                    time: Duration::from_secs(0), // this could be improved...
                    kind: DialEventKind::ButtonLongPress,
                }
            }
            Err(_e) => panic!("Could not recv event"),
        };

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
