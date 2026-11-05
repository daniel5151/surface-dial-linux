use std::sync::mpsc;
use std::time::{Duration, Instant};

use evdev::{InputEvent, InputEventKind};

use super::DialHapticsWorkerMsg;

const MULTI_AXIS_NAME: &str = "Surface Dial System Multi Axis";

/// How often to look for the dial while it is disconnected.
const RESCAN_PERIOD: Duration = Duration::from_secs(1);

/// When it wakes up, the dial replays every rotation made while it was asleep
/// (see upstream issue #15). Rotations received this soon after connecting are
/// dropped.
const CONNECT_GRACE: Duration = Duration::from_millis(750);

pub enum RawInputEvent {
    Event(InputEvent),
    Connect,
    Disconnect,
}

pub struct EventsWorker {
    events: mpsc::Sender<RawInputEvent>,
    haptics_msg: mpsc::Sender<DialHapticsWorkerMsg>,
}

impl EventsWorker {
    pub(super) fn new(
        events: mpsc::Sender<RawInputEvent>,
        haptics_msg: mpsc::Sender<DialHapticsWorkerMsg>,
    ) -> EventsWorker {
        EventsWorker {
            events,
            haptics_msg,
        }
    }

    fn find_dial() -> Option<evdev::Device> {
        evdev::enumerate()
            .map(|(_path, dev)| dev)
            .find(|dev| dev.name() == Some(MULTI_AXIS_NAME))
    }

    fn event_loop(&mut self, mut device: evdev::Device) -> std::io::Result<()> {
        let _ = self.haptics_msg.send(DialHapticsWorkerMsg::DialConnected);
        let _ = self.events.send(RawInputEvent::Connect);

        let connected_at = Instant::now();

        let res = loop {
            let events = match device.fetch_events() {
                Ok(events) => events,
                // ENODEV: the dial disconnected, which is fine
                Err(e) if e.raw_os_error() == Some(19) => break Ok(()),
                Err(e) => break Err(e),
            };

            for event in events {
                let is_rotation = matches!(event.kind(), InputEventKind::RelAxis(_));
                if is_rotation && connected_at.elapsed() < CONNECT_GRACE {
                    continue;
                }
                let _ = self.events.send(RawInputEvent::Event(event));
            }
        };

        let _ = self.haptics_msg.send(DialHapticsWorkerMsg::DialDisconnected);
        let _ = self.events.send(RawInputEvent::Disconnect);

        res
    }

    pub fn run(&mut self) -> std::io::Result<()> {
        let mut announced = false;
        loop {
            match Self::find_dial() {
                Some(dev) => {
                    eprintln!("found the Surface Dial");
                    announced = false;
                    self.event_loop(dev)?;
                }
                None => {
                    if !announced {
                        eprintln!("waiting for the Surface Dial...");
                        announced = true;
                    }
                    std::thread::sleep(RESCAN_PERIOD);
                }
            }
        }
    }
}
