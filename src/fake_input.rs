use std::io;
use std::sync::{LazyLock, Mutex};

use crate::kde;
use crate::xtest::{Keysym, XTest};
use evdev::uinput::{VirtualDevice, VirtualDeviceBuilder};
use evdev::{
    AbsInfo, AbsoluteAxisType, AttributeSet, EventType, InputEvent, Key, PropType,
    RelativeAxisType, UinputAbsSetup,
};

// this should be a fairly high number, as the axis is from 0..(MT_BASELINE*2)
const MT_BASELINE: i32 = i32::MAX / 8;
// higher = slower scrolling
const MT_SENSITIVITY: i32 = 48;

/// Keys the virtual keyboard is allowed to send.
const KEYBOARD_KEYS: &[Key] = &[
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
];

fn build_keyboard() -> io::Result<VirtualDevice> {
    let keys: AttributeSet<Key> = KEYBOARD_KEYS.iter().copied().collect();
    let rel: AttributeSet<RelativeAxisType> = [
        RelativeAxisType::REL_WHEEL,
        RelativeAxisType::REL_WHEEL_HI_RES,
    ]
    .into_iter()
    .collect();

    VirtualDeviceBuilder::new()?
        .name("Surface Dial Virtual Keyboard/Mouse")
        .with_keys(&keys)?
        .with_relative_axes(&rel)?
        .build()
}

fn build_touchpad() -> io::Result<VirtualDevice> {
    let keys: AttributeSet<Key> = [
        Key::BTN_LEFT,
        Key::BTN_TOOL_FINGER,
        Key::BTN_TOUCH,
        Key::BTN_TOOL_DOUBLETAP,
        Key::BTN_TOOL_TRIPLETAP,
        Key::BTN_TOOL_QUADTAP,
    ]
    .into_iter()
    .collect();
    let props: AttributeSet<PropType> = [PropType::BUTTONPAD, PropType::POINTER]
        .into_iter()
        .collect();

    let pos = AbsInfo::new(MT_BASELINE, 0, MT_BASELINE * 2, 0, 0, MT_SENSITIVITY);

    VirtualDeviceBuilder::new()?
        .name("Surface Dial Virtual Touchpad")
        .with_properties(&props)?
        .with_keys(&keys)?
        // ABS_X / ABS_Y are emitted too, so they must be declared (the original
        // code forgot them).
        .with_absolute_axis(&UinputAbsSetup::new(AbsoluteAxisType::ABS_X, pos))?
        .with_absolute_axis(&UinputAbsSetup::new(AbsoluteAxisType::ABS_Y, pos))?
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
        .build()
}

/// Creates a virtual device. A failure is logged instead of crashing the whole
/// daemon: only the modes that need this device become unusable.
fn create(what: &str, build: fn() -> io::Result<VirtualDevice>) -> Option<Mutex<VirtualDevice>> {
    match build() {
        Ok(dev) => {
            // give the kernel a chance to register the new device, otherwise
            // the first fake input is likely to be dropped.
            std::thread::sleep(std::time::Duration::from_millis(300));
            Some(Mutex::new(dev))
        }
        Err(e) => {
            eprintln!("could not create the virtual {}: {}", what, e);
            None
        }
    }
}

type Virtual = LazyLock<Option<Mutex<VirtualDevice>>>;

static KEYBOARD: Virtual = LazyLock::new(|| create("keyboard/mouse", build_keyboard));
static TOUCHPAD: Virtual = LazyLock::new(|| create("touchpad", build_touchpad));

/// X display to send keys and scroll clicks to, when there is one.
/// `SURFACE_DIAL_BACKEND=uinput` forces the uinput devices instead.
static XTEST: LazyLock<Option<XTest>> = LazyLock::new(|| {
    if std::env::var("SURFACE_DIAL_BACKEND").as_deref() == Ok("uinput") {
        return None;
    }
    XTest::connect()
});

/// The fake multitouch touchpad only exists with uinput: a X display (xrdp...)
/// would never see it.
pub fn touchpad_available() -> bool {
    XTEST.is_none() && TOUCHPAD.is_some()
}

/// Picks the output backend up-front, so the first dial event is not lost.
pub fn init() {
    match XTEST.as_ref() {
        Some(_) => eprintln!("sending input through XTEST (X display)"),
        None => {
            eprintln!("sending input through uinput");
            LazyLock::force(&KEYBOARD);
        }
    }
}

fn emit(dev: &Virtual, events: &[InputEvent]) -> io::Result<()> {
    match dev.as_ref() {
        Some(dev) => dev.lock().unwrap().emit(events),
        None => Err(io::Error::new(
            io::ErrorKind::NotFound,
            "virtual input device unavailable",
        )),
    }
}

fn key(k: Key, value: i32) -> InputEvent {
    InputEvent::new(EventType::KEY, k.code(), value)
}

fn abs(axis: AbsoluteAxisType, value: i32) -> InputEvent {
    InputEvent::new(EventType::ABSOLUTE, axis.0, value)
}

/// X keysym for each key the daemon may send.
fn keysym(k: Key) -> Keysym {
    match k {
        Key::KEY_LEFTSHIFT => 0xffe1,
        Key::KEY_LEFTCTRL => 0xffe3,
        Key::KEY_LEFT => 0xff51,
        Key::KEY_RIGHT => 0xff53,
        Key::KEY_SPACE => 0x0020,
        Key::KEY_EQUAL => 0x003d,
        Key::KEY_MINUS => 0x002d,
        Key::KEY_MUTE => 0x1008ff12,
        Key::KEY_VOLUMEDOWN => 0x1008ff11,
        Key::KEY_VOLUMEUP => 0x1008ff13,
        Key::KEY_PLAYPAUSE => 0x1008ff14,
        Key::KEY_PREVIOUSSONG => 0x1008ff16,
        Key::KEY_NEXTSONG => 0x1008ff17,
        other => unreachable!("no keysym for {:?}", other),
    }
}

fn keys(keys: &[Key], press: bool) -> io::Result<()> {
    if let Some(x) = XTEST.as_ref() {
        let syms: Vec<_> = keys.iter().map(|&k| keysym(k)).collect();
        return x.keys(&syms, press);
    }
    let events: Vec<_> = keys.iter().map(|&k| key(k, press as i32)).collect();
    emit(&KEYBOARD, &events)
}

pub fn key_click(k: &[Key]) -> io::Result<()> {
    key_press(k)?;
    key_release(k)
}

pub fn key_press(k: &[Key]) -> io::Result<()> {
    keys(k, true)
}

pub fn key_release(k: &[Key]) -> io::Result<()> {
    keys(k, false)
}

pub fn scroll_step(dir: ScrollStep) -> io::Result<()> {
    if let Some(x) = XTEST.as_ref() {
        return x.click(match dir {
            ScrollStep::Up => 4,
            ScrollStep::Down => 5,
        });
    }
    let (lo, hi) = match dir {
        ScrollStep::Down => (-1, -120),
        ScrollStep::Up => (1, 120),
    };
    emit(
        &KEYBOARD,
        &[
            InputEvent::new(EventType::RELATIVE, RelativeAxisType::REL_WHEEL.0, lo),
            InputEvent::new(EventType::RELATIVE, RelativeAxisType::REL_WHEEL_HI_RES.0, hi),
        ],
    )
}

/// Volume / media actions: KDE's global shortcuts when available (works in any
/// session and shows Plasma's OSD), fake media keys otherwise.
pub enum Action {
    VolumeUp,
    VolumeDown,
    Mute,
    PlayPause,
    Next,
    Previous,
}

pub fn action(a: Action) -> io::Result<()> {
    let (component, shortcut, fallback): (_, _, &[Key]) = match a {
        Action::VolumeUp => (
            "kmix",
            "increase_volume_small",
            &[Key::KEY_LEFTSHIFT, Key::KEY_VOLUMEUP],
        ),
        Action::VolumeDown => (
            "kmix",
            "decrease_volume_small",
            &[Key::KEY_LEFTSHIFT, Key::KEY_VOLUMEDOWN],
        ),
        Action::Mute => ("kmix", "mute", &[Key::KEY_MUTE]),
        Action::PlayPause => ("mediacontrol", "playpausemedia", &[Key::KEY_PLAYPAUSE]),
        Action::Next => ("mediacontrol", "nextmedia", &[Key::KEY_NEXTSONG]),
        Action::Previous => ("mediacontrol", "previousmedia", &[Key::KEY_PREVIOUSSONG]),
    };
    if kde::invoke(component, shortcut) {
        return Ok(());
    }
    key_click(fallback)
}

pub fn scroll_mt_start() -> io::Result<()> {
    use AbsoluteAxisType as A;

    emit(
        &TOUCHPAD,
        &[
            abs(A::ABS_MT_SLOT, 0),
            abs(A::ABS_MT_TRACKING_ID, 1),
            abs(A::ABS_MT_POSITION_X, MT_BASELINE),
            abs(A::ABS_MT_POSITION_Y, MT_BASELINE),
            key(Key::BTN_TOUCH, 1),
            key(Key::BTN_TOOL_FINGER, 1),
            abs(A::ABS_X, MT_BASELINE),
            abs(A::ABS_Y, MT_BASELINE),
        ],
    )?;
    emit(
        &TOUCHPAD,
        &[
            abs(A::ABS_MT_SLOT, 1),
            abs(A::ABS_MT_TRACKING_ID, 2),
            abs(A::ABS_MT_POSITION_X, MT_BASELINE / 2),
            abs(A::ABS_MT_POSITION_Y, MT_BASELINE),
            key(Key::BTN_TOOL_FINGER, 0),
            key(Key::BTN_TOOL_DOUBLETAP, 1),
        ],
    )
}

pub fn scroll_mt_step(delta: i32) -> io::Result<()> {
    use AbsoluteAxisType as A;

    emit(
        &TOUCHPAD,
        &[
            abs(A::ABS_MT_SLOT, 0),
            abs(A::ABS_MT_POSITION_Y, MT_BASELINE + delta),
            abs(A::ABS_MT_SLOT, 1),
            abs(A::ABS_MT_POSITION_Y, MT_BASELINE + delta),
            abs(A::ABS_Y, MT_BASELINE + delta),
        ],
    )
}

pub fn scroll_mt_end() -> io::Result<()> {
    use AbsoluteAxisType as A;

    emit(
        &TOUCHPAD,
        &[
            abs(A::ABS_MT_SLOT, 0),
            abs(A::ABS_MT_TRACKING_ID, -1),
            abs(A::ABS_MT_SLOT, 1),
            abs(A::ABS_MT_TRACKING_ID, -1),
            key(Key::BTN_TOUCH, 0),
            key(Key::BTN_TOOL_DOUBLETAP, 0),
        ],
    )
}

pub enum ScrollStep {
    Up,
    Down,
}
