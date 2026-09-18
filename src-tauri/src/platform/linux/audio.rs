//! Audio-API für Linux, aufgebaut auf [`super::pulse`].

use crate::audio::{calculate_next_position, is_master_slot, normalize_process_name, FOREGROUND_SLOT};
use crate::platform::linux::pulse::{self, SinkInputSnapshot};
use crate::platform::linux::window;
use crate::platform::{AudioDeviceInfo, AudioTarget, PlatformError, Result};
use std::collections::{HashMap, HashSet, VecDeque};
use sysinfo::{ProcessesToUpdate, System};

/// Auf Linux hält [`super::pulse`] eine dauerhafte Verbindung im eigenen
/// Worker-Thread – hier ist nur sicherzustellen, dass er läuft.
pub struct ThreadGuard;

impl ThreadGuard {
    pub fn acquire() -> Self {
        pulse::ensure_started();
        Self
    }
}

/// Ergänzt die übergebenen PIDs um alle Nachkommen.
///
/// Nötig für Chromium/Electron: dort gehört der Audio-Stream einem
/// Kindprozess, während das Fenster am Hauptprozess hängt. Vorfahren werden
/// bewusst *nicht* mitgenommen – über die Elternkette landet man schnell bei
/// der Login-Session und würde damit jeden Stream treffen.
fn expand_to_descendants(seeds: &[u32]) -> HashSet<u32> {
    let mut result: HashSet<u32> = seeds.iter().copied().collect();
    if result.is_empty() {
        return result;
    }

    let mut system = System::new();
    system.refresh_processes(ProcessesToUpdate::All, true);

    let mut children: HashMap<u32, Vec<u32>> = HashMap::new();
    for (pid, process) in system.processes() {
        if let Some(parent) = process.parent() {
            children.entry(parent.as_u32()).or_default().push(pid.as_u32());
        }
    }

    let mut queue: VecDeque<u32> = result.iter().copied().collect();
    while let Some(pid) = queue.pop_front() {
        for child in children.get(&pid).into_iter().flatten() {
            if result.insert(*child) {
                queue.push_back(*child);
            }
        }
    }

    result
}

/// Wählt die Streams aus, die zum Ziel gehören.
///
/// Die Suche läuft in Stufen, weil die letzte Stufe alle Prozesse einliest und
/// damit deutlich teurer ist als die ersten beiden.
fn select_inputs<'a>(
    inputs: &'a [SinkInputSnapshot],
    target: &AudioTarget,
) -> Vec<&'a SinkInputSnapshot> {
    let direct: Vec<&SinkInputSnapshot> = inputs
        .iter()
        .filter(|input| {
            input.pid.is_some_and(|pid| target.pids.contains(&pid))
                || target
                .name_hints
                .iter()
                .any(|hint| input.matches_hint(hint))
        })
        .collect();

    if !direct.is_empty() || target.pids.is_empty() {
        return direct;
    }

    let family = expand_to_descendants(&target.pids);
    inputs
        .iter()
        .filter(|input| input.pid.is_some_and(|pid| family.contains(&pid)))
        .collect()
}

fn sanitize_hint(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    let lower = trimmed.to_lowercase();
    let name = lower.strip_suffix(".exe").unwrap_or(&lower);

    // Generische Wrapper ignorieren, die keinen eigenen Audio-Sink besitzen
    const IGNORED: &[&str] = &["wine64-preloader", "wine-preloader", "wineserver"];
    if IGNORED.contains(&name) {
        return None;
    }

    Some(name.to_string())
}

pub fn foreground_target() -> Option<AudioTarget> {
    let active = window::active_window()?;
    let mut target = AudioTarget::from_pids(active.pid);

    // 1. comm lesen
    if let Some(pid) = active.pid {
        if let Ok(comm) = std::fs::read_to_string(format!("/proc/{pid}/comm")) {
            if let Some(name) = sanitize_hint(&comm) {
                target = target.with_hint(Some(&name));
            }
        }
    }

    // 2. KWin Classes & Caption auswerten
    if let Some(ref app_id) = active.app_id {
        let (classes_str, caption) = match app_id.split_once("::") {
            Some((cls, cap)) => (cls, cap),
            None => (app_id.as_str(), ""),
        };

        let mut is_browser = false;

        for part in classes_str.split('|') {
            if let Some(name) = sanitize_hint(part) {
                let lower = name.to_lowercase();
                if lower.contains("firefox")
                    || lower.contains("chrome")
                    || lower.contains("chromium")
                    || lower.contains("brave")
                    || lower.contains("vivaldi")
                    || lower.contains("opera")
                    || lower.contains("edge")
                    || lower.contains("zen")
                    || lower.contains("floorp")
                    || lower.contains("librewolf")
                {
                    is_browser = true;
                }
                target = target.with_hint(Some(&name));
            }
        }

        // 3. Caption SOFORT als name_hint mitgeben – außer bei Browsern!
        if !is_browser && !caption.is_empty() {
            if let Some(name) = sanitize_hint(caption) {
                target = target.with_hint(Some(&name));
            }
        }
    }

    (!target.is_empty()).then_some(target)
}

fn percent_to_scalar(percent: i32) -> f32 {
    percent.clamp(0, 100) as f32 / 100.0
}

fn status_of(input: &SinkInputSnapshot) -> (f32, bool) {
    (
        percent_to_scalar(pulse::volume_to_percent(&input.volume)),
        input.muted,
    )
}

pub fn get_monitor_statuses(slots: &[Option<String>; 4]) -> Result<[Option<(f32, bool)>; 4]> {
    let mut results = [None; 4];

    if slots.iter().all(Option::is_none) {
        return Ok(results);
    }

    let snapshot = pulse::snapshot()?;
    crate::diagnostics::record_audio_snapshot(snapshot.inputs.len() as u64);

    let master = snapshot.default_sink().map(|sink| {
        (
            percent_to_scalar(pulse::volume_to_percent(&sink.volume)),
            sink.muted,
        )
    });

    // Nur ermitteln, wenn wirklich ein Slot das aktive Fenster beobachtet.
    let foreground = slots
        .iter()
        .any(|slot| matches!(slot.as_deref(), Some(FOREGROUND_SLOT)))
        .then(foreground_target)
        .flatten();

    for (index, slot) in slots.iter().enumerate() {
        let Some(name) = slot.as_deref() else {
            continue;
        };

        if is_master_slot(name) {
            results[index] = master;
            continue;
        }

        let matching = if name == FOREGROUND_SLOT {
            foreground.as_ref().and_then(|target| {
                select_inputs(&snapshot.inputs, target)
                    .into_iter()
                    .next()
                    .map(status_of)
            })
        } else {
            let needle = normalize_process_name(name);
            snapshot
                .inputs
                .iter()
                .find(|input| input.identifier().contains(&needle))
                .map(status_of)
        };

        results[index] = matching;
    }

    Ok(results)
}

/// Identifier aller Streams mit zugeordnetem Prozess – das Linux-Gegenstück zum
/// WASAPI-Session-Identifier.
pub fn list_open_session_identifiers() -> Result<Vec<String>> {
    Ok(pulse::snapshot()?
        .inputs
        .iter()
        .map(SinkInputSnapshot::identifier) // Hier darf kein PID-Filter mehr davor stehen!
        .filter(|identifier| !identifier.is_empty())
        .collect())
}

pub fn list_session_pids() -> Result<Vec<u32>> {
    Ok(pulse::snapshot()?
        .inputs
        .iter()
        .filter_map(|input| input.pid)
        .collect())
}

pub fn adjust_volume(target: &AudioTarget, step: i8, snap: bool) -> Result<bool> {
    if target.is_empty() {
        return Ok(false);
    }

    let snapshot = pulse::snapshot()?;
    let mut boundary_hit = false;

    // NEU: Wir merken uns die berechnete Ziel-Lautstärke für den Fall, dass es Spotify ist
    let mut next_volume_percent = None;

    for input in select_inputs(&snapshot.inputs, target) {
        if !input.volume_writable {
            continue;
        }

        let current = pulse::volume_to_percent(&input.volume);
        let next = if snap {
            calculate_next_position(current, step as i32)
        } else {
            (current + step as i32).clamp(0, 100)
        };

        if next == 100 || next == 0 {
            boundary_hit = true;
        }

        // Wert für unseren Hybrid-Ansatz speichern
        next_volume_percent = Some(next);

        pulse::set_sink_input_volume(input.index, pulse::volume_with_percent(&input.volume, next))?;
    }

    // --- HYBRID ANSATZ FÜR SPOTIFY ---
    if let Some(vol) = next_volume_percent {
        // Prüfen, ob "spotify" in den name_hints des AudioTargets vorkommt
        let is_spotify = target
            .name_hints
            .iter()
            .any(|hint| hint.to_lowercase().contains("spotify"));

        if is_spotify {
            // MPRIS erwartet die Lautstärke als Dezimalwert (f64) zwischen 0.0 und 1.0
            let mpris_vol = vol as f64 / 100.0;

            // Da du in mod.rs bereits "pub mod media;" hast, könntest du hier deine eigene MPRIS-Logik aufrufen.
            // Falls du dort (noch) keine Funktion zum Setzen der Lautstärke hast,
            // feuern wir hier einfach out-of-the-box den direkten D-Bus Befehl im Hintergrund ab:
            std::thread::spawn(move || {
                let _ = std::process::Command::new("dbus-send")
                    .args([
                        "--print-reply",
                        "--dest=org.mpris.MediaPlayer2.spotify",
                        "/org/mpris/MediaPlayer2",
                        "org.freedesktop.DBus.Properties.Set",
                        "string:org.mpris.MediaPlayer2.Player",
                        "string:Volume",
                        &format!("variant:double:{}", mpris_vol),
                    ])
                    .output();
            });
        }
    }

    Ok(boundary_hit)
}


pub fn toggle_mute(target: &AudioTarget) -> Result<()> {
    if target.is_empty() {
        return Ok(());
    }

    let snapshot = pulse::snapshot()?;
    for input in select_inputs(&snapshot.inputs, target) {
        pulse::set_sink_input_mute(input.index, !input.muted)?;
    }

    Ok(())
}

fn default_sink() -> Result<pulse::SinkSnapshot> {
    pulse::snapshot()?
        .default_sink()
        .cloned()
        .ok_or_else(|| PlatformError::new("kein Standard-Ausgabegerät vorhanden"))
}

pub fn get_master_volume() -> Result<f32> {
    Ok(percent_to_scalar(pulse::volume_to_percent(
        &default_sink()?.volume,
    )))
}

pub fn set_master_volume(level: f32) -> Result<()> {
    let sink = default_sink()?;
    let percent = (level.clamp(0.0, 1.0) * 100.0).round() as i32;
    pulse::set_sink_volume(&sink.name, pulse::volume_with_percent(&sink.volume, percent))
}

pub fn toggle_master_mute() -> Result<()> {
    let sink = default_sink()?;
    pulse::set_sink_mute(&sink.name, !sink.muted)
}

pub fn list_audio_devices() -> Result<Vec<AudioDeviceInfo>> {
    // Nach Namen entdoppeln: PipeWire kann denselben Sink-Namen mehrfach
    // führen (etwa nach HDMI-Hotplug). Da der Name die Geräte-ID ist, sind
    // solche Einträge funktional identisch und im Frontend nur verwirrend.
    let mut seen = HashSet::new();

    Ok(pulse::snapshot()?
        .sinks
        .into_iter()
        .filter(|sink| !sink.name.is_empty() && seen.insert(sink.name.clone()))
        .map(|sink| AudioDeviceInfo {
            id: sink.name,
            name: sink.description,
        })
        .collect())
}

/// Sink-Name des aktuellen Standard-Wiedergabegeräts.
pub fn default_device_id() -> Result<Option<String>> {
    Ok(pulse::snapshot()?.default_sink_name)
}

/// `id` ist ein Sink-Name aus [`list_audio_devices`].
pub fn set_default_device(id: &str) -> Result<()> {
    pulse::set_default_sink(id)
}
