#![allow(clippy::collapsible_if, clippy::new_without_default)]

mod common;
pub mod controller;
mod dial_device;
mod error;
mod fake_input;

pub type DynResult<T> = Result<T, Box<dyn std::error::Error>>;

use crate::controller::DialController;
use crate::dial_device::DialDevice;
use crate::error::Error;

use notify_rust::{Hint, Notification, Timeout};
use signal_hook::{iterator::Signals, SIGINT, SIGTERM};

fn main() {
    if let Err(e) = true_main() {
        println!("{}", e);
    }
}

fn true_main() -> DynResult<()> {
    println!("Started.");

    let dial = DialDevice::new()?;
    println!("Found the dial.");

    std::thread::spawn(move || {
        let active_notification = Notification::new()
            .hint(Hint::Resident(true))
            .hint(Hint::Category("device".into()))
            .timeout(Timeout::Never)
            .summary("Surface Dial")
            .body("Active!")
            .icon("input-mouse")
            .show()
            .expect("failed to send notification");

        let signals = Signals::new(&[SIGTERM, SIGINT]).unwrap();
        for sig in signals.forever() {
            eprintln!("received signal {:?}", sig);
            active_notification.close();
            std::process::exit(1);
        }
    });

    // let default_mode = Box::new(controller::controls::Null::new());
    let default_mode = Box::new(controller::controls::ScrollZoom::new());
    // let default_mode = Box::new(controller::controls::Volume::new());
    // let default_mode = Box::new(controller::controls::Media::new());
    // let default_mode = Box::new(controller::controls::DPad::new());

    let mut controller = DialController::new(dial, default_mode);

    controller.run()
}
