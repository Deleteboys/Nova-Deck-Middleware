//! Ermittelt das aktive Fenster – je nach Desktop über ein anderes Backend.
//!
//! Anders als Windows hat Linux dafür keine einheitliche API: Wayland-Clients
//! dürfen fremde Fenster grundsätzlich nicht abfragen, jeder Compositor bietet
//! eigene Wege an. Das Backend wird einmalig beim ersten Zugriff ermittelt.

use crate::platform::linux::{window_hypr, window_plasma, window_x11};
use crate::platform::ActiveWindow;
use log::{info, warn};
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Backend {
    Hyprland,
    /// KDE Plasma auf Wayland
    Plasma,
    /// X11-Sessions, u. a. KDE Plasma auf X11
    X11,
    Unsupported,
}

fn backend() -> Backend {
    static BACKEND: OnceLock<Backend> = OnceLock::new();
    *BACKEND.get_or_init(detect)
}

fn detect() -> Backend {
    if window_hypr::is_available() {
        info!("Aktives Fenster wird über die Hyprland-IPC ermittelt");
        return Backend::Hyprland;
    }

    // In einer Wayland-Session ist X11 kein sinnvoller Rückfall: XWayland sieht
    // nur X11-Fenster und lässt `_NET_ACTIVE_WINDOW` auf dem letzten davon
    // stehen, während längst eine native Anwendung den Fokus hat. Ein falsches
    // Vordergrundprogramm ist schlechter als keines.
    if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        if window_plasma::is_available() {
            info!("Aktives Fenster wird über zwlr_foreign_toplevel_manager ermittelt");
            return Backend::Plasma;
        }
        return unsupported();
    }

    if std::env::var_os("DISPLAY").is_some() && window_x11::is_available() {
        info!("Aktives Fenster wird über X11 (_NET_ACTIVE_WINDOW) ermittelt");
        return Backend::X11;
    }

    unsupported()
}

fn unsupported() -> Backend {
    warn!(
        "Kein unterstützter Weg gefunden, das aktive Fenster zu ermitteln – \
         Aktionen und Slots für das Vordergrundprogramm bleiben wirkungslos"
    );
    Backend::Unsupported
}

pub fn active_window() -> Option<ActiveWindow> {
    match backend() {
        Backend::Hyprland => window_hypr::active_window(),
        Backend::Plasma => window_plasma::active_window(),
        Backend::X11 => window_x11::active_window(),
        Backend::Unsupported => None,
    }
}
