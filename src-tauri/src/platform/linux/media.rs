//! Play/Pause pro Anwendung über MPRIS – das Linux-Gegenstück zu den
//! Global System Media Transport Controls.
//!
//! Jeder Player registriert einen D-Bus-Namen `org.mpris.MediaPlayer2.<name>`
//! und implementiert darauf `org.mpris.MediaPlayer2.Player`.

use crate::audio::normalize_process_name;
use crate::platform::{PlatformError, Result};
use log::debug;
use zbus::blocking::{fdo::DBusProxy, Connection, Proxy};

const MPRIS_PREFIX: &str = "org.mpris.MediaPlayer2.";
const MPRIS_PATH: &str = "/org/mpris/MediaPlayer2";
const MPRIS_ROOT_INTERFACE: &str = "org.mpris.MediaPlayer2";
const MPRIS_PLAYER_INTERFACE: &str = "org.mpris.MediaPlayer2.Player";

fn dbus_error(context: &str, error: zbus::Error) -> PlatformError {
    PlatformError::new(format!("{}: {}", context, error))
}

/// Namen, unter denen ein Player erkannt werden kann: der Bus-Name ohne Präfix
/// und Instanz-Suffix, die `Identity` und der `DesktopEntry`.
fn player_aliases(connection: &Connection, bus_name: &str) -> Vec<String> {
    let mut aliases = Vec::with_capacity(3);

    if let Some(suffix) = bus_name.strip_prefix(MPRIS_PREFIX) {
        // Mehrere Instanzen hängen ".instance1234" an
        let base = suffix.split(".instance").next().unwrap_or(suffix);
        aliases.push(base.to_lowercase());
    }

    if let Ok(root) = Proxy::new(connection, bus_name, MPRIS_PATH, MPRIS_ROOT_INTERFACE) {
        for property in ["Identity", "DesktopEntry"] {
            if let Ok(value) = root.get_property::<String>(property) {
                if !value.is_empty() {
                    aliases.push(value.to_lowercase());
                }
            }
        }
    }

    aliases
}

/// Schaltet Play/Pause bei allen Playern, deren Name zum Prozessnamen passt.
/// Gibt die Bus-Namen der umgeschalteten Player zurück.
pub fn toggle_play_pause(process_name: &str) -> Result<Vec<String>> {
    let needle = normalize_process_name(process_name);
    if needle.is_empty() {
        return Ok(Vec::new());
    }

    let connection =
        Connection::session().map_err(|e| dbus_error("keine D-Bus-Session-Verbindung", e))?;
    let dbus = DBusProxy::new(&connection).map_err(|e| dbus_error("D-Bus nicht erreichbar", e))?;
    let names = dbus
        .list_names()
        .map_err(|e| dbus_error("Bus-Namen konnten nicht gelesen werden", e.into()))?;

    let mut toggled = Vec::new();

    for name in names {
        let bus_name = name.as_str();
        if !bus_name.starts_with(MPRIS_PREFIX) {
            continue;
        }

        let aliases = player_aliases(&connection, bus_name);
        debug!("MPRIS-Player gefunden: {} ({:?})", bus_name, aliases);

        let matches = aliases
            .iter()
            .any(|alias| alias.contains(&needle) || needle.contains(alias));
        if !matches {
            continue;
        }

        let player = Proxy::new(&connection, bus_name, MPRIS_PATH, MPRIS_PLAYER_INTERFACE)
            .map_err(|e| dbus_error("Player-Proxy fehlgeschlagen", e))?;

        player
            .call_method("PlayPause", &())
            .map_err(|e| dbus_error(&format!("PlayPause für {} fehlgeschlagen", bus_name), e))?;

        toggled.push(bus_name.to_owned());
    }

    Ok(toggled)
}
