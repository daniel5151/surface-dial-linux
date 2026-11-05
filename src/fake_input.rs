use std::io;

use evdev::uinput::{VirtualDevice, VirtualDeviceBuilder};
use evdev::{
    AbsInfo, AbsoluteAxisType, AttributeSet, EventType, InputEvent, Key, PropType,
    RelativeAxisType, UinputAbsSetup,
};
use parking_lot::Mutex;

// this should be a fairly high number, as the axis is from 0..(MT_BASELINE*2)
const MT_BASELINE: i32 = std::i32::MAX / 8;
// higher = slower scrolling
const MT_SENSITIVITY: i32 = 48;

pub struct FakeInputs {
    keyboard: Mutex<VirtualDevice>,
    touchpad: Mutex<VirtualDevice>,
}

lazy_static::lazy_static! {
    pub static ref FAKE_INPUTS: FakeInputs = {
        let keyboard = (|| -> io::Result<_> {
            let keys: AttributeSet<Key> = [
                Key::KEY_LEFTSHIFT,
                Key::KEY_LEFTCTRL,

                Key::KEY_MUTE,
                Key::KEY_VOLUMEDOWN,
                Key::KEY_VOLUMEUP,
                Key::KEY_NEXTSONG,
                Key::KEY_PLAYPAUSE,
                Key::KEY_PREVIOUSSONG,

                Key::KEY_LEFT,
                Key::KEY_RIGHT,

                Key::KEY_SPACE,
                Key::KEY_EQUAL,
                Key::KEY_MINUS,
            ]
            .iter()
            .copied()
            .collect();

            let rel: AttributeSet<RelativeAxisType> = [
                RelativeAxisType::REL_WHEEL,
                RelativeAxisType::REL_WHEEL_HI_RES,
            ]
            .iter()
            .copied()
            .collect();

            let device = VirtualDeviceBuilder::new()?
                .name("Surface Dial Virtual Keyboard/Mouse")
                .with_keys(&keys)?
                .with_relative_axes(&rel)?
                .build()?;

            Ok(Mutex::new(device))
        })().expect("failed to install virtual mouse/keyboard device");

        let touchpad = (|| -> io::Result<_> {
            let props: AttributeSet<PropType> = [PropType::BUTTONPAD, PropType::POINTER]
                .iter()
                .copied()
                .collect();

            let keys: AttributeSet<Key> = [
                Key::BTN_LEFT,
                Key::BTN_TOOL_FINGER,
                Key::BTN_TOUCH,
                Key::BTN_TOOL_DOUBLETAP,
                Key::BTN_TOOL_TRIPLETAP,
                Key::BTN_TOOL_QUADTAP,
            ]
            .iter()
            .copied()
            .collect();

            // roughly copied from my laptop's trackpad (Aero 15x)
            let pos = AbsInfo::new(MT_BASELINE, 0, MT_BASELINE * 2, 0, 0, MT_SENSITIVITY);

            let device = VirtualDeviceBuilder::new()?
                .name("Surface Dial Virtual Touchpad")
                .with_properties(&props)?
                .with_keys(&keys)?
                .with_absolute_axis(&UinputAbsSetup::new(
                    AbsoluteAxisType::ABS_MT_SLOT,
                    AbsInfo::new(0, 0, 4, 0, 0, 0),
                ))?
                .with_absolute_axis(&UinputAbsSetup::new(
                    AbsoluteAxisType::ABS_MT_TRACKING_ID,
                    AbsInfo::new(0, 0, 65535, 0, 0, 0),
                ))?
                .with_absolute_axis(&UinputAbsSetup::new(
                    AbsoluteAxisType::ABS_MT_POSITION_X,
                    pos,
                ))?
                .with_absolute_axis(&UinputAbsSetup::new(
                    AbsoluteAxisType::ABS_MT_POSITION_Y,
                    pos,
                ))?
                .build()?;

            Ok(Mutex::new(device))
        })().expect("failed to install virtual touchpad device");

        // HACK: give the kernel a chance to register the new devices. If this
        // line is omitted, the first fake input is likely to be dropped.
        std::thread::sleep(std::time::Duration::from_millis(500));

        FakeInputs {
            keyboard,
            touchpad
        }
    };
}

// NOTE: `VirtualDevice::emit` appends the SYN_REPORT itself.

fn key(key: Key, value: i32) -> InputEvent {
    InputEvent::new(EventType::KEY, key.code(), value)
}

fn abs(axis: AbsoluteAxisType, value: i32) -> InputEvent {
    InputEvent::new(EventType::ABSOLUTE, axis.0, value)
}

fn rel(axis: RelativeAxisType, value: i32) -> InputEvent {
    InputEvent::new(EventType::RELATIVE, axis.0, value)
}

pub fn key_click(keys: &[Key]) -> io::Result<()> {
    key_press(keys)?;
    key_release(keys)?;
    Ok(())
}

pub fn key_press(keys: &[Key]) -> io::Result<()> {
    let events: Vec<_> = keys.iter().map(|&k| key(k, 1)).collect();
    FAKE_INPUTS.keyboard.lock().emit(&events)
}

pub fn key_release(keys: &[Key]) -> io::Result<()> {
    let events: Vec<_> = keys.iter().map(|&k| key(k, 0)).collect();
    FAKE_INPUTS.keyboard.lock().emit(&events)
}

pub fn scroll_step(dir: ScrollStep) -> io::Result<()> {
    // copied from my razer blackwidow chroma mouse
    let (lo_res, hi_res) = match dir {
        ScrollStep::Down => (-1, -120),
        ScrollStep::Up => (1, 120),
    };

    FAKE_INPUTS.keyboard.lock().emit(&[
        rel(RelativeAxisType::REL_WHEEL, lo_res),
        rel(RelativeAxisType::REL_WHEEL_HI_RES, hi_res),
    ])
}

pub fn scroll_mt_start() -> io::Result<()> {
    use AbsoluteAxisType as A;

    let mut touchpad = FAKE_INPUTS.touchpad.lock();

    touchpad.emit(&[
        abs(A::ABS_MT_SLOT, 0),
        abs(A::ABS_MT_TRACKING_ID, 1),
        abs(A::ABS_MT_POSITION_X, MT_BASELINE),
        abs(A::ABS_MT_POSITION_Y, MT_BASELINE),
        key(Key::BTN_TOUCH, 1),
        key(Key::BTN_TOOL_FINGER, 1),
        abs(A::ABS_X, MT_BASELINE),
        abs(A::ABS_Y, MT_BASELINE),
    ])?;

    touchpad.emit(&[
        abs(A::ABS_MT_SLOT, 1),
        abs(A::ABS_MT_TRACKING_ID, 2),
        abs(A::ABS_MT_POSITION_X, MT_BASELINE / 2),
        abs(A::ABS_MT_POSITION_Y, MT_BASELINE),
        key(Key::BTN_TOOL_FINGER, 0),
        key(Key::BTN_TOOL_DOUBLETAP, 1),
    ])
}

pub fn scroll_mt_step(delta: i32) -> io::Result<()> {
    use AbsoluteAxisType as A;

    FAKE_INPUTS.touchpad.lock().emit(&[
        abs(A::ABS_MT_SLOT, 0),
        abs(A::ABS_MT_POSITION_Y, MT_BASELINE + delta),
        abs(A::ABS_MT_SLOT, 1),
        abs(A::ABS_MT_POSITION_Y, MT_BASELINE + delta),
        abs(A::ABS_Y, MT_BASELINE + delta),
    ])
}

pub fn scroll_mt_end() -> io::Result<()> {
    use AbsoluteAxisType as A;

    FAKE_INPUTS.touchpad.lock().emit(&[
        abs(A::ABS_MT_SLOT, 0),
        abs(A::ABS_MT_TRACKING_ID, -1),
        abs(A::ABS_MT_SLOT, 1),
        abs(A::ABS_MT_TRACKING_ID, -1),
        key(Key::BTN_TOUCH, 0),
        key(Key::BTN_TOOL_DOUBLETAP, 0),
    ])
}

pub enum ScrollStep {
    Up,
    Down,
}
