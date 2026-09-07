//! Aktives Fenster über X11 – deckt die KDE-Plasma-X11-Session und beliebige
//! andere X11-Window-Manager ab.
//!
//! Gelesen werden die EWMH-Properties `_NET_ACTIVE_WINDOW` am Root-Fenster und
//! `_NET_WM_PID` bzw. `WM_CLASS` am aktiven Fenster.

use crate::platform::ActiveWindow;
use std::sync::OnceLock;
use x11rb::connection::Connection as _;
use x11rb::protocol::xproto::{AtomEnum, ConnectionExt as _, Window};
use x11rb::rust_connection::RustConnection;

struct Session {
    connection: RustConnection,
    root: Window,
    net_active_window: u32,
    net_wm_pid: u32,
}

impl Session {
    fn open() -> Option<Self> {
        let (connection, screen_number) = x11rb::connect(None).ok()?;
        let root = connection.setup().roots.get(screen_number)?.root;

        let net_active_window = intern(&connection, b"_NET_ACTIVE_WINDOW")?;
        let net_wm_pid = intern(&connection, b"_NET_WM_PID")?;

        Some(Self {
            connection,
            root,
            net_active_window,
            net_wm_pid,
        })
    }

    fn active_window_id(&self) -> Option<Window> {
        let reply = self
            .connection
            .get_property(
                false,
                self.root,
                self.net_active_window,
                AtomEnum::WINDOW,
                0,
                1,
            )
            .ok()?
            .reply()
            .ok()?;

        let window = reply.value32()?.next()?;
        (window != 0).then_some(window)
    }

    fn pid_of(&self, window: Window) -> Option<u32> {
        let reply = self
            .connection
            .get_property(false, window, self.net_wm_pid, AtomEnum::CARDINAL, 0, 1)
            .ok()?
            .reply()
            .ok()?;

        let pid = reply.value32()?.next()?;
        (pid != 0).then_some(pid)
    }

    /// `WM_CLASS` enthält zwei nullterminierte Strings: Instanz- und Klassenname.
    /// Der Klassenname (zweiter Eintrag) entspricht am ehesten der App-ID.
    fn class_of(&self, window: Window) -> Option<String> {
        let reply = self
            .connection
            .get_property(false, window, AtomEnum::WM_CLASS, AtomEnum::STRING, 0, 256)
            .ok()?
            .reply()
            .ok()?;

        let mut parts = reply
            .value
            .split(|byte| *byte == 0)
            .filter(|part| !part.is_empty())
            .filter_map(|part| std::str::from_utf8(part).ok());

        let instance = parts.next();
        parts.next().or(instance).map(str::to_owned)
    }
}

fn intern(connection: &RustConnection, name: &[u8]) -> Option<u32> {
    Some(connection.intern_atom(false, name).ok()?.reply().ok()?.atom)
}

fn session() -> Option<&'static Session> {
    static SESSION: OnceLock<Option<Session>> = OnceLock::new();
    SESSION.get_or_init(Session::open).as_ref()
}

/// Prüft nur, ob eine Verbindung samt EWMH-Atomen zustande kommt.
///
/// Ob gerade ein Fenster fokussiert ist, darf hier keine Rolle spielen: das
/// Ergebnis wird in [`super::window`] dauerhaft gecacht.
pub fn is_available() -> bool {
    session().is_some()
}

pub fn active_window() -> Option<ActiveWindow> {
    let session = session()?;
    let window = session.active_window_id()?;

    let active = ActiveWindow {
        pid: session.pid_of(window),
        app_id: session.class_of(window),
    };
    (!active.is_empty()).then_some(active)
}
