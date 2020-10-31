use std::cmp::Ordering;
use std::sync::mpsc;
use std::thread::JoinHandle;
use std::time::Duration;

use crate::controller::{ControlMode, ControlModeMeta};
use crate::dial_device::DialHaptics;
use crate::fake_input::FakeInput;
use crate::DynResult;

use evdev_rs::enums::EV_KEY;

enum Msg {
    Kill,
    Delta(i32),
}

struct Worker {
    msg: mpsc::Receiver<Msg>,
    fake_input: FakeInput,

    timeout: u64,
    falloff: i32,
    cap: i32,
    deadzone: i32,

    last_delta: i32,
    velocity: i32,
}

impl Worker {
    pub fn new(msg: mpsc::Receiver<Msg>) -> Worker {
        Worker {
            msg,
            fake_input: FakeInput::new(),

            // tweak these for "feel"
            timeout: 5,
            falloff: 10,
            cap: 250,
            deadzone: 10,

            last_delta: 0,
            velocity: 0,
        }
    }

    pub fn run(&mut self) {
        loop {
            let falloff = self.velocity.abs() / self.falloff + 1;

            match self.msg.recv_timeout(Duration::from_millis(self.timeout)) {
                Ok(Msg::Kill) => return,
                Ok(Msg::Delta(delta)) => {
                    // abrupt direction change!
                    if (delta < 0) != (self.last_delta < 0) {
                        self.velocity = 0
                    }
                    self.last_delta = delta;

                    self.velocity += delta
                }
                Err(mpsc::RecvTimeoutError::Timeout) => match self.velocity.cmp(&0) {
                    Ordering::Equal => {}
                    Ordering::Less => self.velocity += falloff,
                    Ordering::Greater => self.velocity -= falloff,
                },
                Err(other) => panic!("{}", other),
            }

            // clamp velocity within the cap bounds
            if self.velocity > self.cap {
                self.velocity = self.cap;
            } else if self.velocity < -self.cap {
                self.velocity = -self.cap;
            }

            if self.velocity.abs() < self.deadzone {
                self.fake_input
                    .key_release(&[EV_KEY::KEY_LEFT, EV_KEY::KEY_RIGHT])
                    .unwrap();
                continue;
            }

            match self.velocity.cmp(&0) {
                Ordering::Equal => {}
                Ordering::Less => self.fake_input.key_press(&[EV_KEY::KEY_LEFT]).unwrap(),
                Ordering::Greater => self.fake_input.key_press(&[EV_KEY::KEY_RIGHT]).unwrap(),
            }

            // eprintln!("{:?}", self.velocity);
        }
    }
}

/// A bit of a misnomer, since it's only left-right.
pub struct DPad {
    _worker: JoinHandle<()>,
    msg: mpsc::Sender<Msg>,

    fake_input: FakeInput,
}

impl Drop for DPad {
    fn drop(&mut self) {
        let _ = self.msg.send(Msg::Kill);
    }
}

impl DPad {
    pub fn new() -> DPad {
        let (msg_tx, msg_rx) = mpsc::channel();

        let worker = std::thread::spawn(move || Worker::new(msg_rx).run());

        DPad {
            _worker: worker,
            msg: msg_tx,

            fake_input: FakeInput::new(),
        }
    }
}

impl ControlMode for DPad {
    fn meta(&self) -> ControlModeMeta {
        ControlModeMeta {
            name: "Paddle",
            icon: "input-gaming",
        }
    }

    fn on_start(&mut self, haptics: &DialHaptics) -> DynResult<()> {
        haptics.set_mode(false, Some(3600))?;
        Ok(())
    }

    fn on_btn_press(&mut self, _: &DialHaptics) -> DynResult<()> {
        eprintln!("space");
        self.fake_input.key_click(&[EV_KEY::KEY_SPACE])?;
        Ok(())
    }

    fn on_btn_release(&mut self, _: &DialHaptics) -> DynResult<()> {
        Ok(())
    }

    fn on_dial(&mut self, _: &DialHaptics, delta: i32) -> DynResult<()> {
        self.msg.send(Msg::Delta(delta))?;
        Ok(())
    }
}
