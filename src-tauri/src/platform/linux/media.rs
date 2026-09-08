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
        // Trennt Suffixe wie ".instance1234" (Firefox) oder ".profile_xyz" (Jellyfin) sauber ab
        let base = suffix.split('.').next().unwrap_or(suffix);
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
    // Hier speichern wir nur noch (Bus-Name, Status) als reine Strings, keine Proxys mehr
    let mut matched_players: Vec<(String, String)> = Vec::new();

    // 1. Alle passenden Player sammeln und deren Status auslesen
    for name in names {
        let bus_name = name.as_str();
        if !bus_name.starts_with(MPRIS_PREFIX) {
            continue;
        }

        let aliases = player_aliases(&connection, bus_name);

        // Entfernt Sonderzeichen und Leerzeichen für einen robusteren Abgleich
        let matches = aliases
            .iter()
            .any(|alias| {
                let clean_alias = alias.replace('-', "").replace(' ', "").replace('_', "");
                let clean_needle = needle.replace('-', "").replace(' ', "").replace('_', "");

                clean_alias.contains(&clean_needle) || clean_needle.contains(&clean_alias)
            });

        if matches {
            if let Ok(player) = Proxy::new(&connection, bus_name, MPRIS_PATH, MPRIS_PLAYER_INTERFACE) {
                let status = player.get_property::<String>("PlaybackStatus")
                    .unwrap_or_else(|_| "Stopped".to_string());
                matched_players.push((bus_name.to_owned(), status));
            }
        }
    }

    // 2. Ziel auswählen (Priorität: Playing -> Paused)
    let target = matched_players.iter().find(|(_, status)| status == "Playing")
        .or_else(|| matched_players.iter().find(|(_, status)| status == "Paused"));

    // 3. PlayPause an das ausgewählte Ziel senden
    if let Some((bus_name, status)) = target {
        debug!("Sende PlayPause exakt an {} (Status: {})", bus_name, status);

        // Proxy für den EINEN ausgewählten Player neu aufbauen
        if let Ok(player) = Proxy::new(&connection, bus_name.as_str(), MPRIS_PATH, MPRIS_PLAYER_INTERFACE) {
            if player.call_method("PlayPause", &()).is_ok() {
                toggled.push(bus_name.clone());
            }
        }
    }

    Ok(toggled)
}