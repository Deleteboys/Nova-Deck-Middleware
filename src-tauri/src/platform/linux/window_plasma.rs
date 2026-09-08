use crate::platform::ActiveWindow;
use log::{debug, info, warn};
use std::fs;
use std::sync::{Mutex, OnceLock};
use std::thread;
use zbus::blocking::{connection::Builder, Connection};

fn cache() -> &'static Mutex<Option<ActiveWindow>> {
    static CACHE: OnceLock<Mutex<Option<ActiveWindow>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(None))
}

struct WindowTracker;

#[zbus::interface(name = "org.novadeck.Tracker")]
impl WindowTracker {
    // i32 statt u32 nutzen, da Qt/JS Zahlen als signed int32 überträgt
    fn window_changed(&self, app_id: String, pid: i32) {
        let active = ActiveWindow {
            pid: (pid > 0).then_some(pid as u32),
            app_id: (!app_id.is_empty()).then_some(app_id),
        };

        if let Ok(mut guard) = cache().lock() {
            *guard = (!active.is_empty()).then_some(active);
        }
    }
}

pub fn is_available() -> bool {
    static STARTED: OnceLock<bool> = OnceLock::new();
    *STARTED.get_or_init(start)
}

fn start() -> bool {
    let Ok(conn) = Connection::session() else {
        debug!("Keine D-Bus-Session-Verbindung möglich");
        return false;
    };

    // Prüfen, ob KWin auf dem Bus antwortet
    let kwin_exists: bool = conn
        .call_method(
            Some("org.freedesktop.DBus"),
            "/org/freedesktop/DBus",
            Some("org.freedesktop.DBus"),
            "NameHasOwner",
            &("org.kde.KWin",),
        )
        .and_then(|r| r.body().deserialize())
        .unwrap_or(false);

    if !kwin_exists {
        return false;
    }

    thread::Builder::new()
        .name("novadeck-plasma-dbus".to_owned())
        .spawn(move || {
            let script_path = std::env::temp_dir().join("novadeck_kwin_tracker.js");
            let script_content = r#"
            function send() {
                var w = workspace.activeWindow;
                if (w) {
                    var id = w.desktopFileName || w.resourceClass || w.resourceName || "";
                    var p = w.pid || 0;
                    console.warn("NOVADECK_TRIGGER: " + id + " (PID " + p + ")");
                    callDBus("org.novadeck.Tracker", "/Tracker", "org.novadeck.Tracker", "WindowChanged", id, p);
                } else {
                    callDBus("org.novadeck.Tracker", "/Tracker", "org.novadeck.Tracker", "WindowChanged", "", 0);
                }
            }
            workspace.windowActivated.connect(send);
            send();
            "#;

            if fs::write(&script_path, script_content).is_err() {
                return;
            }

            // Eigenen D-Bus Service anmelden
            let server_conn = match Builder::session()
                .and_then(|b| b.name("org.novadeck.Tracker"))
                .and_then(|b| b.serve_at("/Tracker", WindowTracker))
                .and_then(|b| b.build())
            {
                Ok(c) => c,
                Err(e) => {
                    warn!("D-Bus-Server konnte nicht gestartet werden: {e}");
                    return;
                }
            };

            // Altes Skript entladen, falls noch vorhanden
            let _ = server_conn.call_method(
                Some("org.kde.KWin"),
                "/Scripting",
                Some("org.kde.kwin.Scripting"),
                "unloadScript",
                &("novadeck_tracker",),
            );

            // Skript laden und ausführen
            let load_reply = server_conn.call_method(
                Some("org.kde.KWin"),
                "/Scripting",
                Some("org.kde.kwin.Scripting"),
                "loadScript",
                &(script_path.to_string_lossy(), "novadeck_tracker"),
            );

            if let Ok(reply) = load_reply {
                if let Ok(script_id) = reply.body().deserialize::<i32>() {
                    let path = format!("/Scripting/Script{script_id}");
                    let _ = server_conn.call_method(
                        Some("org.kde.KWin"),
                        path,
                        Some("org.kde.kwin.Script"),
                        "run",
                        &(),
                    );
                    info!("KWin-Fenster-Tracking erfolgreich aktiviert");
                }
            }

            // D-Bus-Thread am Leben halten
            loop {
                std::thread::park();
            }
        })
        .is_ok()
}

pub fn active_window() -> Option<ActiveWindow> {
    cache().lock().ok()?.clone()
}
