use std::sync::{Arc, Mutex};

use crate::dial_device::{DialDevice, DialEventKind, DialHaptics};
use crate::error::{Error, Result};

pub mod controls;

pub struct ControlModeMeta {
    name: &'static str,
    icon: &'static str,
}

pub trait ControlMode {
    fn meta(&self) -> ControlModeMeta;

    fn on_start(&mut self, haptics: &DialHaptics) -> Result<()>;
    fn on_end(&mut self, _haptics: &DialHaptics) -> Result<()> {
        Ok(())
    }

    fn on_btn_press(&mut self, haptics: &DialHaptics) -> Result<()>;
    fn on_btn_release(&mut self, haptics: &DialHaptics) -> Result<()>;
    fn on_dial(&mut self, haptics: &DialHaptics, delta: i32) -> Result<()>;
}

enum ActiveMode {
    Normal(usize),
    Meta,
}

pub struct DialController {
    device: DialDevice,

    modes: Vec<Box<dyn ControlMode>>,
    active_mode: ActiveMode,

    new_mode: Arc<Mutex<Option<usize>>>,
    meta_mode: Box<dyn ControlMode>, // always MetaMode
}

impl DialController {
    pub fn new(
        device: DialDevice,
        initial_mode: usize,
        modes: Vec<Box<dyn ControlMode>>,
    ) -> DialController {
        let metas = modes.iter().map(|m| m.meta()).collect();

        let new_mode = Arc::new(Mutex::new(None));

        DialController {
            device,

            modes,
            active_mode: ActiveMode::Normal(initial_mode),

            new_mode: new_mode.clone(),
            meta_mode: Box::new(MetaMode::new(new_mode, 0, metas)),
        }
    }

    pub fn run(&mut self) -> Result<()> {
        let initial_mode = match self.active_mode {
            ActiveMode::Normal(i) => i,
            ActiveMode::Meta => 0,
        };
        self.modes[initial_mode].on_start(self.device.haptics())?;

        loop {
            let evt = self.device.next_event()?;
            let haptics = self.device.haptics();

            if let Some(new_mode) = self.new_mode.lock().unwrap().take() {
                self.active_mode = ActiveMode::Normal(new_mode);
                self.modes[new_mode].on_start(haptics)?;
            }

            let mode = match self.active_mode {
                ActiveMode::Normal(idx) => &mut self.modes[idx],
                ActiveMode::Meta => &mut self.meta_mode,
            };

            // TODO: press and hold (+ rotate?) to switch between modes

            match evt.kind {
                DialEventKind::Ignored => {}
                DialEventKind::ButtonPress => mode.on_btn_press(haptics)?,
                DialEventKind::ButtonRelease => mode.on_btn_release(haptics)?,
                DialEventKind::Dial(delta) => mode.on_dial(haptics, delta)?,
                DialEventKind::ButtonLongPress => {
                    eprintln!("long press!");
                    if !matches!(self.active_mode, ActiveMode::Meta) {
                        mode.on_end(haptics)?;
                        self.active_mode = ActiveMode::Meta;
                        self.meta_mode.on_start(haptics)?;
                    }
                }
            }
        }
    }
}

/// A mode for switching between modes.
struct MetaMode {
    // constant
    metas: Vec<ControlModeMeta>,

    // stateful (across invocations)
    current_mode: usize,
    new_mode: Arc<Mutex<Option<usize>>>,

    // reset in on_start
    first_release: bool,
    notif: Option<notify_rust::NotificationHandle>,
}

impl MetaMode {
    fn new(
        new_mode: Arc<Mutex<Option<usize>>>,
        current_mode: usize,
        metas: Vec<ControlModeMeta>,
    ) -> MetaMode {
        MetaMode {
            metas,

            current_mode,
            new_mode,

            first_release: true,
            notif: None,
        }
    }
}

impl ControlMode for MetaMode {
    fn meta(&self) -> ControlModeMeta {
        unreachable!() // meta mode never queries itself
    }

    fn on_start(&mut self, haptics: &DialHaptics) -> Result<()> {
        use notify_rust::*;
        self.notif = Some(
            Notification::new()
                .hint(Hint::Resident(true))
                .hint(Hint::Category("device".into()))
                .timeout(Timeout::Never)
                .summary("Surface Dial")
                .body(&format!(
                    "Entered Meta Mode (From Mode: {})",
                    self.metas[self.current_mode].name
                ))
                .icon("emblem-system")
                .show()
                .map_err(Error::Notif)?,
        );

        haptics.buzz(1)?;

        self.first_release = true;

        haptics.set_mode(true, Some(36))?;
        Ok(())
    }

    fn on_btn_press(&mut self, _haptics: &DialHaptics) -> Result<()> {
        Ok(())
    }

    fn on_btn_release(&mut self, haptics: &DialHaptics) -> Result<()> {
        if self.first_release {
            self.first_release = false;
        } else {
            *self.new_mode.lock().unwrap() = Some(self.current_mode);

            crate::config::Config {
                last_mode: self.current_mode,
            }
            .to_disk()?;

            self.notif.take().unwrap().close();
            haptics.buzz(1)?;
        }
        Ok(())
    }

    fn on_dial(&mut self, _haptics: &DialHaptics, delta: i32) -> Result<()> {
        if delta > 0 {
            self.current_mode += 1;
        } else {
            if self.current_mode == 0 {
                self.current_mode = self.metas.len() - 1;
            } else {
                self.current_mode -= 1;
            }
        };

        self.current_mode %= self.metas.len();

        let mode_meta = &self.metas[self.current_mode];
        if let Some(ref mut notification) = self.notif {
            notification
                .body(&format!("New Mode: {}", mode_meta.name))
                .icon(mode_meta.icon);
            notification.update();
        }

        Ok(())
    }
}
