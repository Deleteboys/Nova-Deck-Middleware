//! Plattformabstraktion.
//!
//! Beide Backends (`windows/` und `linux/`) stellen dieselben Submodule mit
//! identischen Signaturen bereit:
//!
//! * `audio`      – Lautstärke/Mute pro Audio-Session, Master-Lautstärke, Ausgabegeräte
//! * `window`     – aktives Fenster (PID + App-ID)
//! * `media`      – Play/Pause für eine bestimmte Anwendung
//! * `shell`      – Desktop-Aktionen, die kein simulierter Tastendruck sind
//! * `diagnostics` – Speicher-/Handle-Zähler des eigenen Prozesses
//!
//! Das `unsafe` der jeweiligen System-API bleibt in den Backends eingekapselt;
//! nach außen sind alle Funktionen sicher aufrufbar.

use std::fmt;

#[cfg(not(any(windows, target_os = "linux")))]
compile_error!("NovaDeck unterstützt derzeit nur Windows und Linux");

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::*;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::*;

pub type Result<T> = std::result::Result<T, PlatformError>;

/// Fehler einer Plattform-API. Die Ursprungsfehler (HRESULT, PulseAudio-Errno,
/// D-Bus, …) werden auf eine lesbare Meldung reduziert, damit die aufrufenden
/// Module plattformunabhängig bleiben.
#[derive(Debug, Clone)]
pub struct PlatformError(String);

impl PlatformError {
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for PlatformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for PlatformError {}

impl From<String> for PlatformError {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for PlatformError {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

#[cfg(windows)]
impl From<::windows::core::Error> for PlatformError {
    fn from(value: ::windows::core::Error) -> Self {
        Self(value.to_string())
    }
}

/// Win32 gibt Strings als UTF-16 zurück; `PWSTR::to_string` kann daran scheitern.
#[cfg(windows)]
impl From<std::string::FromUtf16Error> for PlatformError {
    fn from(value: std::string::FromUtf16Error) -> Self {
        Self(value.to_string())
    }
}

#[cfg(target_os = "linux")]
impl From<libpulse_binding::error::PAErr> for PlatformError {
    fn from(value: libpulse_binding::error::PAErr) -> Self {
        // PAErr::to_string liefert Option; ohne Klartext bleibt der Errno-Code.
        Self(
            value
                .to_string()
                .unwrap_or_else(|| format!("PulseAudio-Fehler {}", value.0)),
        )
    }
}

/// Ein Audio-Ausgabegerät, wie es das Frontend in der Geräteauswahl anzeigt.
///
/// `id` ist der stabile, technische Bezeichner (Windows: Endpoint-ID,
/// Linux: PulseAudio-Sink-Name), `name` der anzeigbare Klartextname.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct AudioDeviceInfo {
    pub id: String,
    pub name: String,
}

/// Beschreibt, welche Audio-Streams eine Aktion treffen soll.
///
/// Windows kann rein über die PID auflösen, weil jede WASAPI-Session die PID des
/// Fensterprozesses trägt. Unter Linux hängt ein PulseAudio-Sink-Input dagegen
/// häufig an einem Kindprozess (Chromium/Electron starten einen eigenen
/// Audio-Service), deshalb gibt es zusätzlich Namens-Hinweise.
#[derive(Debug, Clone, Default)]
pub struct AudioTarget {
    pub pids: Vec<u32>,
    /// Klein geschrieben und ohne `.exe` – siehe [`crate::audio::normalize_process_name`].
    pub name_hints: Vec<String>,
}

impl AudioTarget {
    pub fn from_pids(pids: impl IntoIterator<Item = u32>) -> Self {
        Self {
            pids: pids.into_iter().collect(),
            name_hints: Vec::new(),
        }
    }

    /// Ergänzt einen Namens-Hinweis. Leere oder bereits vorhandene Hinweise
    /// werden verworfen.
    pub fn with_hint(mut self, hint: Option<&str>) -> Self {
        if let Some(hint) = hint {
            let normalized = crate::audio::normalize_process_name(hint);
            if !normalized.is_empty() && !self.name_hints.contains(&normalized) {
                self.name_hints.push(normalized);
            }
        }
        self
    }

    pub fn is_empty(&self) -> bool {
        self.pids.is_empty() && self.name_hints.is_empty()
    }
}

/// Das aktuell fokussierte Fenster.
///
/// `app_id` ist auf Windows immer `None` (dort genügt die PID); unter Linux
/// liefern die Compositor-Backends die Fensterklasse bzw. App-ID, die beim
/// Zuordnen von Audio-Streams hilft.
#[derive(Debug, Clone, Default)]
pub struct ActiveWindow {
    pub pid: Option<u32>,
    pub app_id: Option<String>,
}

impl ActiveWindow {
    pub fn is_empty(&self) -> bool {
        self.pid.is_none() && self.app_id.is_none()
    }
}

/// Speicher- und Handle-Zähler des eigenen Prozesses für die DevTools-Ansicht.
#[derive(serde::Serialize)]
pub struct ProcessDiagnostics {
    pub pid: u32,
    pub working_set_bytes: u64,
    pub peak_working_set_bytes: u64,
    pub private_usage_bytes: u64,
    pub peak_pagefile_usage_bytes: u64,
    pub pagefile_usage_bytes: u64,
    pub paged_pool_bytes: u64,
    pub nonpaged_pool_bytes: u64,
    pub page_faults: u64,
    /// Windows: offene Kernel-Handles. Linux: offene Dateideskriptoren.
    pub handle_count: u32,
}
