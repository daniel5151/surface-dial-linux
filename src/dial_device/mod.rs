use std::sync::mpsc;
use std::time::Duration;

use crate::error::{Error, Result};

mod events;
mod haptics;

use haptics::{DialHapticsWorker, DialHapticsWorkerMsg};

pub use haptics::DialHaptics;

/// Encapsulates all the the nitty-gritty (and pretty gnarly) device handling
/// code, exposing a simple interface to wait for incoming [`DialEvent`]s.
pub struct DialDevice {
    // configurable constants
    long_press_timeout: Duration,

    // handles
    haptics: DialHaptics,
    events: mpsc::Receiver<events::RawInputEvent>,

    // mutable state
    possible_long_press: bool,
}

#[derive(Debug)]
pub struct DialEvent {
    pub time: Duration,
    pub kind: DialEventKind,
}

#[derive(Debug)]
pub enum DialEventKind {
    Connect,
    Disconnect,

    Ignored,
    ButtonPress,
    ButtonRelease,
    Dial(i32),

    /// NOTE: this is a synthetic event, and is _not_ directly provided by the
    /// dial itself.
    ButtonLongPress,
}

impl DialDevice {
    pub fn new(long_press_timeout: Duration) -> Result<DialDevice> {
        let (events_tx, events_rx) = mpsc::channel();
        let (haptics_msg_tx, haptics_msg_rx) = mpsc::channel();

        // TODO: interleave control events with regular events
        // (once we figure out what control events actually do...)

        std::thread::spawn({
            let haptics_msg_tx = haptics_msg_tx.clone();
            let mut worker = events::EventsWorker::new(
                events::DialInputKind::MultiAxis,
                events_tx,
                haptics_msg_tx,
            );
            move || {
                worker.run().unwrap();
                eprintln!("the events worker died!");
            }
        });

        std::thread::spawn({
            let mut worker = DialHapticsWorker::new(haptics_msg_rx)?;
            move || {
                if let Err(err) = worker.run() {
                    eprintln!("Unexpected haptics worker error! {}", err);
                }
                eprintln!("the haptics worker died!");
                // there's no coming back from this.
                std::process::exit(0);
            }
        });

        Ok(DialDevice {
            long_press_timeout,
            events: events_rx,
            haptics: DialHaptics::new(haptics_msg_tx)?,

            possible_long_press: false,
        })
    }

    /// Blocks until a new dial event comes occurs.
    // TODO?: rewrite code using async/await?
    // TODO?: "cheat" by exposing an async interface to the current next_event impl
    pub fn next_event(&mut self) -> Result<DialEvent> {
        let evt = if self.possible_long_press {
            self.events.recv_timeout(self.long_press_timeout)
        } else {
            self.events
                .recv()
                .map_err(|_| mpsc::RecvTimeoutError::Disconnected)
        };

        let event = match evt {
            Ok(events::RawInputEvent::Event(_event_status, event)) => {
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
            Ok(events::RawInputEvent::Connect) => {
                DialEvent {
                    time: Duration::from_secs(0), // this could be improved...
                    kind: DialEventKind::Connect,
                }
            }
            Ok(events::RawInputEvent::Disconnect) => {
                DialEvent {
                    time: Duration::from_secs(0), // this could be improved...
                    kind: DialEventKind::Disconnect,
                }
            }
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
    fn from_raw_evt(evt: evdev_rs::InputEvent) -> Option<DialEvent> {
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
