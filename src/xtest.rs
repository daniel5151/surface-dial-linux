//! Fake input through the X server's XTEST extension.
//!
//! uinput devices only reach the local seat. A remote X session (xrdp, VNC...)
//! never sees them, so when a X display is available the daemon sends keys and
//! scroll clicks straight to it instead.

use std::collections::HashMap;
use std::io;
use std::sync::Mutex;

use x11rb::connection::Connection;
use x11rb::protocol::xproto::{
    ConnectionExt as _, Keycode, Window, BUTTON_PRESS_EVENT, BUTTON_RELEASE_EVENT,
    KEY_PRESS_EVENT, KEY_RELEASE_EVENT,
};
use x11rb::protocol::xtest::ConnectionExt as _;
use x11rb::rust_connection::RustConnection;

pub type Keysym = u32;

pub struct XTest {
    conn: RustConnection,
    root: Window,
    keymap: Mutex<HashMap<Keysym, Keycode>>,
}

fn x_err(e: impl std::fmt::Display) -> io::Error {
    io::Error::other(format!("X11: {}", e))
}

impl XTest {
    /// Connects to `$DISPLAY`. Returns `None` if there is no usable X server.
    pub fn connect() -> Option<XTest> {
        std::env::var_os("DISPLAY")?;

        let (conn, screen) = match x11rb::connect(None) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("no X server: {}", e);
                return None;
            }
        };
        if let Err(e) = conn.xtest_get_version(2, 2).map(|c| c.reply()) {
            eprintln!("X server without XTEST: {}", e);
            return None;
        }

        let root = conn.setup().roots[screen].root;
        let xtest = XTest {
            conn,
            root,
            keymap: Mutex::new(HashMap::new()),
        };
        xtest.reload_keymap().ok()?;
        Some(xtest)
    }

    fn reload_keymap(&self) -> io::Result<()> {
        let setup = self.conn.setup();
        let (min, max) = (setup.min_keycode, setup.max_keycode);
        let reply = self
            .conn
            .get_keyboard_mapping(min, max - min + 1)
            .map_err(x_err)?
            .reply()
            .map_err(x_err)?;

        let per = reply.keysyms_per_keycode as usize;
        let mut map = HashMap::new();
        for (i, syms) in reply.keysyms.chunks(per).enumerate() {
            // only the unshifted keysym: we never want an implicit Shift
            if let Some(&sym) = syms.first() {
                if sym != 0 {
                    map.entry(sym).or_insert(min + i as u8);
                }
            }
        }
        *self.keymap.lock().unwrap() = map;
        Ok(())
    }

    fn keycode(&self, sym: Keysym) -> io::Result<Keycode> {
        if let Some(&code) = self.keymap.lock().unwrap().get(&sym) {
            return Ok(code);
        }
        // the layout may have changed since we started
        self.reload_keymap()?;
        self.keymap
            .lock()
            .unwrap()
            .get(&sym)
            .copied()
            .ok_or_else(|| x_err(format!("keysym {:#x} not in the keyboard map", sym)))
    }

    fn fake(&self, kind: u8, detail: u8) -> io::Result<()> {
        self.conn
            .xtest_fake_input(kind, detail, 0, self.root, 0, 0, 0)
            .map_err(x_err)?;
        Ok(())
    }

    pub fn keys(&self, syms: &[Keysym], press: bool) -> io::Result<()> {
        let kind = if press { KEY_PRESS_EVENT } else { KEY_RELEASE_EVENT };
        for &sym in syms {
            self.fake(kind, self.keycode(sym)?)?;
        }
        self.conn.flush().map_err(x_err)
    }

    /// Clicks a mouse button (4 = wheel up, 5 = wheel down).
    pub fn click(&self, button: u8) -> io::Result<()> {
        self.fake(BUTTON_PRESS_EVENT, button)?;
        self.fake(BUTTON_RELEASE_EVENT, button)?;
        self.conn.flush().map_err(x_err)
    }
}
