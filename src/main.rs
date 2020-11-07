#![deny(unsafe_code)]
#![allow(clippy::collapsible_if, clippy::new_without_default)]

pub mod common;
mod config;
pub mod controller;
mod dial_device;
mod error;
mod fake_input;

use std::sync::mpsc;

use crate::controller::DialController;
use crate::dial_device::DialDevice;
use crate::error::{Error, Result};

use notify_rust::{Hint, Notification, Timeout};
use signal_hook::{iterator::Signals, SIGINT, SIGTERM};

fn main() {
    let (terminate_tx, terminate_rx) = mpsc::channel::<Result<()>>();

    std::thread::spawn({
        let terminate_tx = terminate_tx.clone();
        move || {
            let signals = Signals::new(&[SIGTERM, SIGINT]).unwrap();
            for sig in signals.forever() {
                eprintln!("received signal {:?}", sig);
                let _ = terminate_tx.send(Err(Error::TermSig));
            }
        }
    });

    std::thread::spawn({
        let terminate_tx = terminate_tx;
        move || {
            let _ = terminate_tx.send(controller_main());
        }
    });

    let (silent, msg, icon) = match terminate_rx.recv() {
        Ok(Ok(())) => (true, "".into(), ""),
        Ok(Err(e)) => {
            println!("Error: {}", e);
            match e {
                Error::TermSig => (false, "Terminated!".into(), "dialog-warning"),
                // HACK: silently exit if the dial disconnects
                Error::Evdev(e) if e.raw_os_error() == Some(19) => (true, "".into(), ""),
                other => (false, format!("Error: {}", other), "dialog-error"),
            }
        }
        Err(_) => {
            println!("Error: Unexpected Error");
            (false, "Unexpected Error".into(), "dialog-error")
        }
    };

    if !silent {
        Notification::new()
            .hint(Hint::Transient(true))
            .hint(Hint::Category("device".into()))
            .timeout(Timeout::Milliseconds(100))
            .summary("Surface Dial")
            .body(&msg)
            .icon(icon)
            .show()
            .unwrap();
    }

    // cleaning up threads is hard...
    std::process::exit(1);
}

fn controller_main() -> Result<()> {
    println!("Started");

    let cfg = config::Config::from_disk()?;

    let dial = DialDevice::new(std::time::Duration::from_millis(750))?;

    let mut controller = DialController::new(
        dial,
        cfg.last_mode,
        vec![
            Box::new(controller::controls::Scroll::new()),
            Box::new(controller::controls::ScrollMT::new()),
            Box::new(controller::controls::Zoom::new()),
            Box::new(controller::controls::Volume::new()),
            Box::new(controller::controls::Media::new()),
            Box::new(controller::controls::MediaWithVolume::new()),
            Box::new(controller::controls::Paddle::new()),
        ],
    );

    controller.run()
}
