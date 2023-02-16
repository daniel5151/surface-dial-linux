use std::cmp::Ordering;
use std::sync::mpsc;
use std::thread::JoinHandle;
use std::time::Duration;

use crate::controller::{ControlMode, ControlModeMeta};
use crate::dial_device::DialHaptics;
use crate::error::Result;
use crate::fake_input;

use evdev_rs::enums::EV_KEY;

// everything is done in a worker, as we need use `recv_timeout` as a (very)
// poor man's `select!`.

enum Msg {
    Kill,
    ButtonDown,
    ButtonUp,
    Delta(i32),
    Enabled(bool),
}

struct Worker {
    msg: mpsc::Receiver<Msg>,

    timeout: u64,
    falloff: i32,
    cap: i32,
    deadzone: i32,

    enabled: bool,
    last_delta: i32,
    velocity: i32,
}

impl Worker {
    pub fn new(msg: mpsc::Receiver<Msg>) -> Worker {
        Worker {
            msg,

            // tweak these for "feel"
            timeout:0,
            falloff: 10,
            cap: 250,
            deadzone: 10,

            enabled: false,
            last_delta: 0,
            velocity: 0,
        }
    }

    pub fn run(&mut self) {
        loop {
            let falloff = self.velocity.abs() / self.falloff + 1;

            let msg = if self.enabled {
                self.msg.recv_timeout(Duration::from_millis(self.timeout))
            } else {
                self.msg
                    .recv()
                    .map_err(|_| mpsc::RecvTimeoutError::Disconnected)
            };

            match msg {
                Ok(Msg::Kill) => return,
                Ok(Msg::Enabled(enabled)) => {
                    self.enabled = enabled;
                    if !enabled {
                        fake_input::key_release(&[
                            EV_KEY::KEY_SPACE,
                            EV_KEY::KEY_LEFT,
                            EV_KEY::KEY_RIGHT,
                        ])
                        .unwrap()
                    }
                }
                Ok(Msg::ButtonDown) => fake_input::key_press(&[EV_KEY::KEY_SPACE]).unwrap(),
                Ok(Msg::ButtonUp) => fake_input::key_release(&[EV_KEY::KEY_SPACE]).unwrap(),
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
                fake_input::key_release(&[EV_KEY::KEY_LEFT, EV_KEY::KEY_RIGHT]).unwrap();
                continue;
            }

            match self.velocity.cmp(&0) {
                Ordering::Equal => {}
                Ordering::Less => fake_input::key_press(&[EV_KEY::KEY_LEFT]).unwrap(),
                Ordering::Greater => fake_input::key_press(&[EV_KEY::KEY_RIGHT]).unwrap(),
            }

            // eprintln!("{:?}", self.velocity);
        }
    }
}

/// A bit of a misnomer, since it's only left-right.
pub struct Paddle {
    _worker: JoinHandle<()>,
    msg: mpsc::Sender<Msg>,
}

impl Drop for Paddle {
    fn drop(&mut self) {
        let _ = self.msg.send(Msg::Kill);
    }
}

impl Paddle {
    pub fn new() -> Paddle {
        let (msg_tx, msg_rx) = mpsc::channel();

        let worker = std::thread::spawn(move || Worker::new(msg_rx).run());

        Paddle {
            _worker: worker,
            msg: msg_tx,
        }
    }
}

impl ControlMode for Paddle {
    fn meta(&self) -> ControlModeMeta {
        ControlModeMeta {
            name: "Paddle",
            icon: "input-gaming",
            haptics: false,
            steps: 3600,
        }
    }

    fn on_start(&mut self, _haptics: &DialHaptics) -> Result<()> {
        let _ = self.msg.send(Msg::Enabled(true));
        Ok(())
    }

    fn on_end(&mut self, _haptics: &DialHaptics) -> Result<()> {
        let _ = self.msg.send(Msg::Enabled(false));
        Ok(())
    }

    fn on_btn_press(&mut self, _: &DialHaptics) -> Result<()> {
        let _ = self.msg.send(Msg::ButtonDown);
        Ok(())
    }

    fn on_btn_release(&mut self, _: &DialHaptics) -> Result<()> {
        let _ = self.msg.send(Msg::ButtonUp);
        Ok(())
    }

    fn on_dial(&mut self, _: &DialHaptics, delta: i32) -> Result<()> {
        let _ = self.msg.send(Msg::Delta(delta));
        Ok(())
    }
}
