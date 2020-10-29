mod common;
pub mod controller;
mod dial_device;
mod error;
mod fake_input;

pub type DynResult<T> = Result<T, Box<dyn std::error::Error>>;

use crate::controller::DialController;
use crate::dial_device::DialDevice;
use crate::error::Error;

fn main() {
    if let Err(e) = true_main() {
        println!("{}", e);
    }
}

fn true_main() -> DynResult<()> {
    println!("Started.");

    let dial = DialDevice::new()?;
    println!("Found the dial.");

    common::action_notification("Active!", "input-mouse")?;

    let default_mode = Box::new(controller::controls::ScrollZoom::new(30));
    // let default_mode = Box::new(controller::controls::Volume::new(30));
    // let default_mode = Box::new(controller::controls::Media::new(50));
    // let default_mode = Box::new(controller::controls::DPad::new());

    let mut controller = DialController::new(dial, default_mode);

    controller.run()
}
