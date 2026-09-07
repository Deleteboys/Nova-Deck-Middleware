//! Aktives Fenster über die Hyprland-IPC.
//!
//! Hyprland liefert auf `j/activewindow` ein JSON-Objekt mit `pid` und `class`.
//! Ist kein Fenster fokussiert, kommt ein leeres Objekt zurück.

use crate::platform::ActiveWindow;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::time::Duration;

const IO_TIMEOUT: Duration = Duration::from_millis(200);

pub fn is_available() -> bool {
    socket_paths().iter().any(|path| path.exists())
}

/// Hyprland legt den Socket seit 0.40 unter `$XDG_RUNTIME_DIR` ab, ältere
/// Versionen unter `/tmp`.
fn socket_paths() -> Vec<PathBuf> {
    let Some(signature) = std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE") else {
        return Vec::new();
    };

    let mut paths = Vec::with_capacity(2);
    if let Some(runtime_dir) = std::env::var_os("XDG_RUNTIME_DIR") {
        paths.push(
            PathBuf::from(runtime_dir)
                .join("hypr")
                .join(&signature)
                .join(".socket.sock"),
        );
    }
    paths.push(
        PathBuf::from("/tmp/hypr")
            .join(&signature)
            .join(".socket.sock"),
    );
    paths
}

fn query(command: &str) -> Option<String> {
    for path in socket_paths() {
        let Ok(mut stream) = UnixStream::connect(&path) else {
            continue;
        };
        let _ = stream.set_read_timeout(Some(IO_TIMEOUT));
        let _ = stream.set_write_timeout(Some(IO_TIMEOUT));

        if stream.write_all(command.as_bytes()).is_err() {
            continue;
        }

        let mut response = String::new();
        if stream.read_to_string(&mut response).is_err() {
            continue;
        }
        return Some(response);
    }
    None
}

pub fn active_window() -> Option<ActiveWindow> {
    let response = query("j/activewindow")?;
    let value: serde_json::Value = serde_json::from_str(&response).ok()?;

    let pid = value
        .get("pid")
        .and_then(serde_json::Value::as_i64)
        .filter(|pid| *pid > 0)
        .map(|pid| pid as u32);

    let app_id = ["class", "initialClass"]
        .iter()
        .filter_map(|key| value.get(*key).and_then(serde_json::Value::as_str))
        .find(|class| !class.is_empty())
        .map(str::to_owned);

    let active = ActiveWindow { pid, app_id };
    (!active.is_empty()).then_some(active)
}
