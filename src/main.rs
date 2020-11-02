#![allow(clippy::collapsible_if, clippy::new_without_default)]

mod common;
mod config;
pub mod controller;
mod dial_device;
mod error;
mod fake_input;

pub type DynResult<T> = Result<T, Box<dyn std::error::Error>>;

use std::sync::mpsc;

use crate::controller::DialController;
use crate::dial_device::DialDevice;
use crate::error::Error;

use notify_rust::{Hint, Notification, Timeout};
use signal_hook::{iterator::Signals, SIGINT, SIGTERM};

fn main() {
    let (kill_notif_tx, kill_notif_rx) = mpsc::channel::<Option<(String, &'static str)>>();

    let handle = std::thread::spawn(move || {
        let active_notification = Notification::new()
            .hint(Hint::Resident(true))
            .hint(Hint::Category("device".into()))
            .timeout(Timeout::Never)
            .summary("Surface Dial")
            .body("Active!")
            .icon("media-optical") // it should be vaguely circular :P
            .show()
            .expect("failed to send notification");

        let kill_notif = kill_notif_rx.recv();

        active_notification.close();

        let (msg, icon) = match kill_notif {
            Ok(None) => {
                // shutdown immediately
                std::process::exit(1);
            }
            Ok(Some((msg, icon))) => (msg, icon),
            Err(_) => ("Unexpected Error".into(), "dialog-error"),
        };

        Notification::new()
            .hint(Hint::Transient(true))
            .hint(Hint::Category("device".into()))
            .timeout(Timeout::Milliseconds(100))
            .summary("Surface Dial")
            .body(&msg)
            .icon(icon)
            .show()
            .unwrap();

        std::process::exit(1);
    });

    if let Err(e) = true_main(kill_notif_tx.clone()) {
        println!("{}", e);
    }

    kill_notif_tx.send(None).unwrap(); // silently shut down
    handle.join().unwrap();
}

fn true_main(kill_notif_tx: mpsc::Sender<Option<(String, &'static str)>>) -> DynResult<()> {
    println!("Started");

    let cfg = config::Config::from_disk()?;

    let dial = DialDevice::new(std::time::Duration::from_millis(750))?;
    println!("Found the dial");

    std::thread::spawn(move || {
        let signals = Signals::new(&[SIGTERM, SIGINT]).unwrap();
        for sig in signals.forever() {
            eprintln!("received signal {:?}", sig);
            kill_notif_tx
                .send(Some(("Terminated!".into(), "dialog-warning")))
                .unwrap();
        }
    });

    let mut controller = DialController::new(
        dial,
        cfg.last_mode,
        vec![
            Box::new(controller::controls::ScrollZoom::new()),
            Box::new(controller::controls::Volume::new()),
            Box::new(controller::controls::Media::new()),
            Box::new(controller::controls::Paddle::new()),
        ],
    );

    controller.run()
}
