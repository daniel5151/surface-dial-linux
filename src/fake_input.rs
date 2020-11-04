use std::io;

use evdev_rs::enums::*;
use evdev_rs::{Device, InputEvent, TimeVal, UInputDevice};
use parking_lot::ReentrantMutex;

// this should be a fairly high number, as the axis is from 0..(MT_BASELINE*2)
const MT_BASELINE: i32 = std::i32::MAX / 4;
// higher = more sensitive
const MT_SENSITIVITY: i32 = 64;

lazy_static::lazy_static! {
    static ref FAKE_KEYBOARD: ReentrantMutex<UInputDevice> = {
        (|| -> io::Result<_> {
            let device = Device::new().unwrap();
            device.set_name("Surface Dial Virtual Keyboard/Mouse");

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

            Ok(ReentrantMutex::new(UInputDevice::create_from_device(&device)?))
        })().expect("failed to install virtual mouse/keyboard device")
    };

    static ref FAKE_TOUCHPAD: ReentrantMutex<UInputDevice> = {
        (|| -> io::Result<_> {
            let device = Device::new().unwrap();
            device.set_name("Surface Dial Virtual Touchpad");

            device.enable(&InputProp::INPUT_PROP_BUTTONPAD)?;
            device.enable(&InputProp::INPUT_PROP_POINTER)?;

            device.enable(&EventType::EV_SYN)?;
            device.enable(&EventCode::EV_SYN(EV_SYN::SYN_REPORT))?;

            device.enable(&EventType::EV_KEY)?;
            {
                device.enable(&EventCode::EV_KEY(EV_KEY::BTN_LEFT))?;
                device.enable(&EventCode::EV_KEY(EV_KEY::BTN_TOOL_FINGER))?;
                device.enable(&EventCode::EV_KEY(EV_KEY::BTN_TOUCH))?;
                device.enable(&EventCode::EV_KEY(EV_KEY::BTN_TOOL_DOUBLETAP))?;
                device.enable(&EventCode::EV_KEY(EV_KEY::BTN_TOOL_TRIPLETAP))?;
                device.enable(&EventCode::EV_KEY(EV_KEY::BTN_TOOL_QUADTAP))?;
            }

            // roughly copied from my laptop's trackpad (Aero 15x)
            device.enable(&EventType::EV_ABS)?;
            {
                let mut abs_info = evdev_rs::AbsInfo {
                    value: 0,
                    minimum: 0,
                    maximum: 0,
                    fuzz: 0,
                    flat: 0,
                    resolution: 0,
                };

                abs_info.minimum = 0;
                abs_info.maximum = 4;
                device.enable_event_code(&EventCode::EV_ABS(EV_ABS::ABS_MT_SLOT), Some(&abs_info))?;

                abs_info.minimum = 0;
                abs_info.maximum = 65535;
                device.enable_event_code(
                    &EventCode::EV_ABS(EV_ABS::ABS_MT_TRACKING_ID),
                    Some(&abs_info),
                )?;

                abs_info.resolution = MT_SENSITIVITY;
                abs_info.minimum = 0;
                abs_info.maximum = MT_BASELINE * 2;
                abs_info.value = MT_BASELINE;
                device.enable_event_code(
                    &EventCode::EV_ABS(EV_ABS::ABS_MT_POSITION_X),
                    Some(&abs_info),
                )?;

                abs_info.value = MT_BASELINE;
                abs_info.minimum = 0;
                abs_info.maximum = MT_BASELINE * 2;
                abs_info.resolution = MT_SENSITIVITY;
                device.enable_event_code(
                    &EventCode::EV_ABS(EV_ABS::ABS_MT_POSITION_Y),
                    Some(&abs_info),
                )?;
            }

            Ok(ReentrantMutex::new(UInputDevice::create_from_device(&device)?))
        })().expect("failed to install virtual touchpad device")
    };

}

#[non_exhaustive]
pub struct FakeInput {}

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
        FakeInput {}
    }

    fn kbd_syn_report(&self) -> io::Result<()> {
        (FAKE_KEYBOARD.lock()).write_event(&input_event!(EV_SYN, SYN_REPORT, 0))
    }

    pub fn key_click(&self, keys: &[EV_KEY]) -> io::Result<()> {
        self.key_press(keys)?;
        self.key_release(keys)?;
        Ok(())
    }

    pub fn key_press(&self, keys: &[EV_KEY]) -> io::Result<()> {
        let keyboard = FAKE_KEYBOARD.lock();

        for key in keys {
            keyboard.write_event(&InputEvent {
                time: TimeVal::new(0, 0),
                event_code: EventCode::EV_KEY(*key),
                event_type: EventType::EV_KEY,
                value: 1,
            })?;
        }
        self.kbd_syn_report()?;
        Ok(())
    }

    pub fn key_release(&self, keys: &[EV_KEY]) -> io::Result<()> {
        let keyboard = FAKE_KEYBOARD.lock();

        for key in keys.iter().clone() {
            keyboard.write_event(&InputEvent {
                time: TimeVal::new(0, 0),
                event_code: EventCode::EV_KEY(*key),
                event_type: EventType::EV_KEY,
                value: 0,
            })?;
        }
        self.kbd_syn_report()?;
        Ok(())
    }

    pub fn scroll_step(&self, dir: ScrollStep) -> io::Result<()> {
        let keyboard = FAKE_KEYBOARD.lock();

        // copied from my razer blackwidow chroma mouse
        keyboard.write_event(&InputEvent {
            time: TimeVal::new(0, 0),
            event_code: EventCode::EV_REL(EV_REL::REL_WHEEL),
            event_type: EventType::EV_REL,
            value: match dir {
                ScrollStep::Down => -1,
                ScrollStep::Up => 1,
            },
        })?;
        keyboard.write_event(&InputEvent {
            time: TimeVal::new(0, 0),
            event_code: EventCode::EV_REL(EV_REL::REL_WHEEL_HI_RES),
            event_type: EventType::EV_REL,
            value: match dir {
                ScrollStep::Down => -120,
                ScrollStep::Up => 120,
            },
        })?;
        self.kbd_syn_report()?;
        Ok(())
    }

    fn touch_syn_report(&self) -> io::Result<()> {
        (FAKE_TOUCHPAD.lock()).write_event(&input_event!(EV_SYN, SYN_REPORT, 0))
    }

    pub fn scroll_mt_start(&self) -> io::Result<()> {
        let touchpad = FAKE_TOUCHPAD.lock();

        {
            touchpad.write_event(&input_event!(EV_ABS, ABS_MT_SLOT, 0))?;
            touchpad.write_event(&input_event!(EV_ABS, ABS_MT_TRACKING_ID, 1))?;
            touchpad.write_event(&input_event!(EV_ABS, ABS_MT_POSITION_X, MT_BASELINE))?;
            touchpad.write_event(&input_event!(EV_ABS, ABS_MT_POSITION_Y, MT_BASELINE))?;

            touchpad.write_event(&input_event!(EV_KEY, BTN_TOUCH, 1))?;
            touchpad.write_event(&input_event!(EV_KEY, BTN_TOOL_FINGER, 1))?;

            touchpad.write_event(&input_event!(EV_ABS, ABS_X, MT_BASELINE))?;
            touchpad.write_event(&input_event!(EV_ABS, ABS_Y, MT_BASELINE))?;
        }

        self.touch_syn_report()?;

        {
            touchpad.write_event(&input_event!(EV_ABS, ABS_MT_SLOT, 1))?;
            touchpad.write_event(&input_event!(EV_ABS, ABS_MT_TRACKING_ID, 2))?;
            touchpad.write_event(&input_event!(EV_ABS, ABS_MT_POSITION_X, MT_BASELINE / 2))?;
            touchpad.write_event(&input_event!(EV_ABS, ABS_MT_POSITION_Y, MT_BASELINE))?;

            touchpad.write_event(&input_event!(EV_KEY, BTN_TOOL_FINGER, 0))?;
            touchpad.write_event(&input_event!(EV_KEY, BTN_TOOL_DOUBLETAP, 1))?;
        }

        self.touch_syn_report()?;

        Ok(())
    }

    pub fn scroll_mt_step(&self, delta: i32) -> io::Result<()> {
        let touchpad = FAKE_TOUCHPAD.lock();

        touchpad.write_event(&input_event!(EV_ABS, ABS_MT_SLOT, 0))?;
        touchpad.write_event(&input_event!(
            EV_ABS,
            ABS_MT_POSITION_Y,
            MT_BASELINE + delta
        ))?;
        touchpad.write_event(&input_event!(EV_ABS, ABS_MT_SLOT, 1))?;
        touchpad.write_event(&input_event!(
            EV_ABS,
            ABS_MT_POSITION_Y,
            MT_BASELINE + delta
        ))?;

        touchpad.write_event(&input_event!(EV_ABS, ABS_Y, MT_BASELINE + delta))?;

        self.touch_syn_report()?;

        Ok(())
    }

    pub fn scroll_mt_end(&self) -> io::Result<()> {
        let touchpad = FAKE_TOUCHPAD.lock();

        touchpad.write_event(&input_event!(EV_ABS, ABS_MT_SLOT, 0))?;
        touchpad.write_event(&input_event!(EV_ABS, ABS_MT_TRACKING_ID, -1))?;
        touchpad.write_event(&input_event!(EV_ABS, ABS_MT_SLOT, 1))?;
        touchpad.write_event(&input_event!(EV_ABS, ABS_MT_TRACKING_ID, -1))?;

        touchpad.write_event(&input_event!(EV_KEY, BTN_TOUCH, 0))?;
        touchpad.write_event(&input_event!(EV_KEY, BTN_TOOL_DOUBLETAP, 0))?;

        self.touch_syn_report()?;

        Ok(())
    }
}

pub enum ScrollStep {
    Up,
    Down,
}
