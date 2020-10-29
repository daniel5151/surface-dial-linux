mod common;
pub mod controller;
mod dial_device;
mod error;
mod fake_input;

pub type DynResult<T> = Result<T, Box<dyn std::error::Error>>;

use crate::controller::DialController;
use crate::dial_device::DialDevice;
use crate::error::Error;

fn main() -> DynResult<()> {
    let dial = DialDevice::new()?;
    println!("Found the dial.");

    let default_mode = Box::new(controller::controls::Volume::new(30));
    // let default_mode = Box::new(controls::Media::new(50));
    // let default_mode = Box::new(controls::DPad::new());

    let mut controller = DialController::new(dial, default_mode);

    controller.run()
}
