use std::fs;
use std::sync::mpsc;
use std::time::Duration;

use evdev_rs::{InputEvent, ReadStatus};
use std::os::unix::io::AsRawFd;

use super::DialHapticsWorkerMsg;

pub enum RawInputEvent {
    Event(ReadStatus, InputEvent),
    Connect,
    Disconnect,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DialInputKind {
    Control,
    MultiAxis,
}

pub struct EventsWorker {
    events: mpsc::Sender<RawInputEvent>,
    haptics_msg: mpsc::Sender<DialHapticsWorkerMsg>,
    input_kind: DialInputKind,
}

impl EventsWorker {
    pub(super) fn new(
        input_kind: DialInputKind,
        events: mpsc::Sender<RawInputEvent>,
        haptics_msg: mpsc::Sender<DialHapticsWorkerMsg>,
    ) -> EventsWorker {
        EventsWorker {
            input_kind,
            events,
            haptics_msg,
        }
    }

    fn udev_to_evdev(&self, device: &udev::Device) -> std::io::Result<Option<evdev_rs::Device>> {
        let devnode = match device.devnode() {
            Some(path) => path,
            None => return Ok(None),
        };

        // we care about the `/dev/input/eventXX` device, which is a child of the
        // actual input device (that has a nice name we can match against)
        match device.parent() {
            None => return Ok(None),
            Some(parent) => {
                let name = parent
                    .property_value("NAME")
                    .unwrap_or_else(|| std::ffi::OsStr::new(""))
                    .to_string_lossy();

                match (self.input_kind, name.as_ref()) {
                    (DialInputKind::Control, r#""Surface Dial System Control""#) => {}
                    (DialInputKind::MultiAxis, r#""Surface Dial System Multi Axis""#) => {}
                    _ => return Ok(None),
                }
            }
        }

        let file = fs::File::open(devnode)?;
        evdev_rs::Device::new_from_fd(file).map(Some)
    }

    fn event_loop(&mut self, device: evdev_rs::Device) -> std::io::Result<()> {
        // HACK: don't want to double-send these events
        if self.input_kind != DialInputKind::Control {
            self.haptics_msg
                .send(DialHapticsWorkerMsg::DialConnected)
                .unwrap();
            self.events.send(RawInputEvent::Connect).unwrap();
        }

        loop {
            let _ = self
                .events
                .send(match device.next_event(evdev_rs::ReadFlag::BLOCKING) {
                    Ok((read_status, event)) => RawInputEvent::Event(read_status, event),
                    // this error corresponds to the device disconnecting, which is fine
                    Err(e) if e.raw_os_error() == Some(19) => break,
                    Err(e) => return Err(e),
                });
        }

        // HACK: don't want to double-send these events
        if self.input_kind != DialInputKind::Control {
            self.haptics_msg
                .send(DialHapticsWorkerMsg::DialDisconnected)
                .unwrap();
            self.events.send(RawInputEvent::Disconnect).unwrap();
        }

        Ok(())
    }

    pub fn run(&mut self) -> std::io::Result<()> {
        // eagerly check if the device already exists

        let mut enumerator = {
            let mut e = udev::Enumerator::new()?;
            e.match_subsystem("input")?;
            e
        };
        for device in enumerator.scan_devices()? {
            let dev = match self.udev_to_evdev(&device)? {
                None => continue,
                Some(dev) => dev,
            };

            self.event_loop(dev)?;
        }

        // enter udev event loop to gracefully handle disconnect/reconnect

        let mut socket = udev::MonitorBuilder::new()?
            .match_subsystem("input")?
            .listen()?;

        loop {
            nix::poll::ppoll(
                &mut [nix::poll::PollFd::new(
                    socket.as_raw_fd(),
                    nix::poll::PollFlags::POLLIN,
                )],
                None,
                nix::sys::signal::SigSet::empty(),
            )
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

            let event = match socket.next() {
                Some(evt) => evt,
                None => {
                    std::thread::sleep(Duration::from_millis(10));
                    continue;
                }
            };

            if !matches!(event.event_type(), udev::EventType::Add) {
                continue;
            }

            let dev = match self.udev_to_evdev(&event.device())? {
                None => continue,
                Some(dev) => dev,
            };

            self.event_loop(dev)?;
        }
    }
}
