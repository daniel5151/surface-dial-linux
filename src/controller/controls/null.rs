use crate::controller::{ControlMode, ControlModeMeta};
use crate::dial_device::DialHaptics;
use crate::DynResult;

impl ControlMode for () {
    fn meta(&self) -> ControlModeMeta {
        ControlModeMeta {
            name: "null",
            icon: "",
        }
    }

    fn on_start(&mut self, haptics: &DialHaptics) -> DynResult<()> {
        haptics.set_mode(false, Some(0))?;
        Ok(())
    }

    fn on_btn_press(&mut self, _haptics: &DialHaptics) -> DynResult<()> {
        Ok(())
    }

    fn on_btn_release(&mut self, _haptics: &DialHaptics) -> DynResult<()> {
        Ok(())
    }

    fn on_dial(&mut self, _haptics: &DialHaptics, _delta: i32) -> DynResult<()> {
        Ok(())
    }
}
