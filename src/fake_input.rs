use std::io;

use evdev_rs::enums::*;
use evdev_rs::{Device, InputEvent, TimeVal, UInputDevice};

static mut FAKE_INPUT: Option<UInputDevice> = None;
fn get_fake_input() -> io::Result<&'static UInputDevice> {
    if unsafe { FAKE_INPUT.is_none() } {
        let device = Device::new().unwrap();
        device.set_name("Surface Dial Virtual Input");

        device.enable(&EventType::EV_SYN)?;
        device.enable(&EventCode::EV_SYN(EV_SYN::SYN_REPORT))?;

        device.enable(&EventType::EV_MSC)?;
        device.enable(&EventCode::EV_MSC(EV_MSC::MSC_SCAN))?;

        device.enable(&EventType::EV_KEY)?;
        {
            device.enable(&EventCode::EV_KEY(EV_KEY::KEY_LEFTSHIFT))?;
            device.enable(&EventCode::EV_KEY(EV_KEY::KEY_LEFTCTRL))?;

            device.enable(&EventCode::EV_KEY(EV_KEY::KEY_MUTE))?;
            device.enable(&EventCode::EV_KEY(EV_KEY::KEY_VOLUMEDOWN))?;
            device.enable(&EventCode::EV_KEY(EV_KEY::KEY_VOLUMEUP))?;
            device.enable(&EventCode::EV_KEY(EV_KEY::KEY_NEXTSONG))?;
            device.enable(&EventCode::EV_KEY(EV_KEY::KEY_PLAYPAUSE))?;
            device.enable(&EventCode::EV_KEY(EV_KEY::KEY_PREVIOUSSONG))?;

            device.enable(&EventCode::EV_KEY(EV_KEY::KEY_LEFT))?;
            device.enable(&EventCode::EV_KEY(EV_KEY::KEY_RIGHT))?;

            device.enable(&EventCode::EV_KEY(EV_KEY::KEY_SPACE))?;
            device.enable(&EventCode::EV_KEY(EV_KEY::KEY_EQUAL))?;
            device.enable(&EventCode::EV_KEY(EV_KEY::KEY_MINUS))?;
        }

        device.enable(&EventType::EV_REL)?;
        {
            device.enable(&EventCode::EV_REL(EV_REL::REL_WHEEL))?;
            device.enable(&EventCode::EV_REL(EV_REL::REL_WHEEL_HI_RES))?;
        }

        unsafe { FAKE_INPUT = Some(UInputDevice::create_from_device(&device)?) }
    }
    unsafe { Ok(FAKE_INPUT.as_ref().unwrap()) }
}

#[non_exhaustive]
pub struct FakeInput {
    uinput: &'static UInputDevice,
}

macro_rules! input_event {
    ($type:ident, $code:ident, $value:expr) => {
        InputEvent {
            time: TimeVal::new(0, 0),
            event_code: EventCode::$type($type::$code),
            event_type: EventType::$type,
            value: $value,
        }
    };
}

impl Default for FakeInput {
    fn default() -> Self {
        Self::new()
    }
}

impl FakeInput {
    pub fn new() -> FakeInput {
        FakeInput {
            uinput: get_fake_input().expect("could not install fake input device"),
        }
    }

    fn syn_report(&self) -> io::Result<()> {
        self.uinput
            .write_event(&input_event!(EV_SYN, SYN_REPORT, 0))
    }

    pub fn key_click(&self, keys: &[EV_KEY]) -> io::Result<()> {
        self.key_press(keys)?;
        self.key_release(keys)?;
        Ok(())
    }

    pub fn key_press(&self, keys: &[EV_KEY]) -> io::Result<()> {
        for key in keys {
            self.uinput.write_event(&InputEvent {
                time: TimeVal::new(0, 0),
                event_code: EventCode::EV_KEY(*key),
                event_type: EventType::EV_KEY,
                value: 1,
            })?;
        }
        self.syn_report()?;
        Ok(())
    }

    pub fn key_release(&self, keys: &[EV_KEY]) -> io::Result<()> {
        for key in keys.iter().clone() {
            self.uinput.write_event(&InputEvent {
                time: TimeVal::new(0, 0),
                event_code: EventCode::EV_KEY(*key),
                event_type: EventType::EV_KEY,
                value: 0,
            })?;
        }
        self.syn_report()?;
        Ok(())
    }

    pub fn scroll_step(&self, dir: ScrollStep) -> io::Result<()> {
        // copied from my razer blackwidow chroma mouse
        self.uinput.write_event(&InputEvent {
            time: TimeVal::new(0, 0),
            event_code: EventCode::EV_REL(EV_REL::REL_WHEEL),
            event_type: EventType::EV_REL,
            value: match dir {
                ScrollStep::Down => -1,
                ScrollStep::Up => 1,
            },
        })?;
        self.uinput.write_event(&InputEvent {
            time: TimeVal::new(0, 0),
            event_code: EventCode::EV_REL(EV_REL::REL_WHEEL_HI_RES),
            event_type: EventType::EV_REL,
            value: match dir {
                ScrollStep::Down => -120,
                ScrollStep::Up => 120,
            },
        })?;
        self.syn_report()?;
        Ok(())
    }
}

pub enum ScrollStep {
    Up,
    Down,
}
