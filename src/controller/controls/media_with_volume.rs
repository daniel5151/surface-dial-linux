use std::sync::mpsc;
use std::time::Duration;

use crate::controller::{ControlMode, ControlModeMeta};
use crate::dial_device::DialHaptics;
use crate::error::{Error, Result};
use crate::fake_input;

use evdev_rs::enums::EV_KEY;

fn double_click_worker(click: mpsc::Receiver<()>, release: mpsc::Receiver<()>) -> Result<()> {
    loop {
        // drain any spurious clicks/releases
        for _ in click.try_iter() {}
        for _ in release.try_iter() {}

        click.recv().unwrap();
        // recv with timeout, in case this is a long-press
        match release.recv_timeout(Duration::from_secs(1)) {
            Ok(()) => {}
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(mpsc::RecvTimeoutError::Disconnected) => panic!(),
        }

        match click.recv_timeout(Duration::from_millis(250)) {
            Ok(()) => {
                // double click
                release.recv().unwrap(); // should only fire after button is released
                eprintln!("next track");
                fake_input::key_click(&[EV_KEY::KEY_NEXTSONG]).map_err(Error::Evdev)?;
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                // single click
                eprintln!("play/pause");
                fake_input::key_click(&[EV_KEY::KEY_PLAYPAUSE]).map_err(Error::Evdev)?;
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => panic!(),
        }
    }
}

pub struct MediaWithVolume {
    click_tx: mpsc::Sender<()>,
    release_tx: mpsc::Sender<()>,
    worker_handle: Option<std::thread::JoinHandle<Result<()>>>,
}

impl MediaWithVolume {
    pub fn new() -> MediaWithVolume {
        let (click_tx, click_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();

        let worker_handle = std::thread::spawn(move || double_click_worker(click_rx, release_rx));

        MediaWithVolume {
            click_tx,
            release_tx,
            worker_handle: Some(worker_handle),
        }
    }
}

impl ControlMode for MediaWithVolume {
    fn meta(&self) -> ControlModeMeta {
        ControlModeMeta {
            name: "Media + Volume",
            icon: "applications-multimedia",
            haptics: true,
            steps: 36 * 2,
        }
    }

    fn on_btn_press(&mut self, _: &DialHaptics) -> Result<()> {
        if self.click_tx.send(()).is_err() {
            self.worker_handle
                .take()
                .unwrap()
                .join()
                .expect("panic on thread join")?;
        }
        Ok(())
    }

    fn on_btn_release(&mut self, _: &DialHaptics) -> Result<()> {
        if self.release_tx.send(()).is_err() {
            self.worker_handle
                .take()
                .unwrap()
                .join()
                .expect("panic on thread join")?;
        }
        Ok(())
    }

    fn on_dial(&mut self, _: &DialHaptics, delta: i32) -> Result<()> {
        if delta > 0 {
            eprintln!("volume up");
            fake_input::key_click(&[EV_KEY::KEY_VOLUMEUP])
                .map_err(Error::Evdev)?;
        } else {
            eprintln!("volume down");
            fake_input::key_click(&[EV_KEY::KEY_VOLUMEDOWN])
                .map_err(Error::Evdev)?;
        }

        Ok(())
    }
}
