//! Plattformneutrale Audio-Facade.
//!
//! Die eigentlichen Implementierungen liegen in [`crate::platform`] (WASAPI
//! unter Windows, PulseAudio-Protokoll unter Linux). Hier bleiben nur die
//! Bezeichner und die reine Rechenlogik, die auf allen Plattformen gilt.

pub use crate::platform::audio::*;
pub use crate::platform::{AudioDeviceInfo, AudioTarget};

/// Slot-Bezeichner für die Master-Lautstärke.
pub const MASTER_SLOT: &str = "Master Volume";
/// Alter Bezeichner, den gespeicherte Konfigurationen noch enthalten.
pub const MASTER_SLOT_LEGACY: &str = "Windows Master Volume";
/// Slot-Bezeichner für „was gerade im Vordergrund läuft“.
pub const FOREGROUND_SLOT: &str = "Foreground Process";

pub fn is_master_slot(name: &str) -> bool {
    name == MASTER_SLOT || name == MASTER_SLOT_LEGACY
}

/// Vereinheitlicht Prozessnamen für den Vergleich.
///
/// Konfigurationen, die unter Windows angelegt wurden, enthalten Namen wie
/// `"Discord.exe"`; unter Linux heißt derselbe Prozess `discord`. Beide Seiten
/// laufen durch diese Funktion, damit ein Profil auf beiden Systemen passt.
pub fn normalize_process_name(name: &str) -> String {
    let lowercase = name.trim().to_lowercase();
    lowercase
        .strip_suffix(".exe")
        .unwrap_or(&lowercase)
        .to_owned()
}

/// Baut das Audio-Ziel für einen konfigurierten Prozessnamen.
///
/// Neben den PIDs aller passenden Prozesse wird der Name selbst als Hinweis
/// mitgegeben – unter Linux findet das auch Streams, die an einem Kindprozess
/// hängen (siehe [`AudioTarget`]).
pub fn target_for_process_name(name: &str) -> AudioTarget {
    let needle = normalize_process_name(name);

    let mut system = sysinfo::System::new();
    system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

    let pids = system
        .processes()
        .iter()
        .filter(|(_, process)| {
            normalize_process_name(&process.name().to_string_lossy()) == needle
        })
        .map(|(pid, _)| pid.as_u32());

    AudioTarget::from_pids(pids).with_hint(Some(name))
}

/// Bestimmt die nächste Lautstärke bei aktivem „Snap to grid“.
///
/// Ungerade Schrittweiten rasten auf 10er-, gerade auf 5er-Schritte ein. Wird
/// beim Weiterdrehen eine Rasterlinie überschritten, bleibt der Wert zunächst
/// auf ihr stehen.
pub fn calculate_next_position(start: i32, step: i32) -> i32 {
    if step == 0 {
        return start.clamp(0, 100);
    }

    let is_even = step % 2 == 0;
    let grid = if is_even { 5 } else { 10 };
    let org_next = start + step;

    if is_even && start % 10 == 5 {
        let delta = if step > 0 { 1 } else { -1 };
        return (start + delta).clamp(0, 100);
    }

    let crossed_boundary = (start / grid) != (org_next / grid);
    let is_on_boundary = (start % grid) == 0;

    let target = if crossed_boundary && !is_on_boundary {
        if step > 0 {
            ((start / grid) + 1) * grid
        } else {
            (start / grid) * grid
        }
    } else {
        org_next
    };

    target.clamp(0, 100)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_windows_and_linux_process_names() {
        assert_eq!(normalize_process_name("Discord.exe"), "discord");
        assert_eq!(normalize_process_name("discord"), "discord");
        assert_eq!(normalize_process_name("  Spotify.EXE "), "spotify");
        // Nur ein echtes Suffix wird entfernt, nicht ein Namensbestandteil
        assert_eq!(normalize_process_name("exe"), "exe");
        assert_eq!(normalize_process_name(""), "");
    }

    #[test]
    fn recognises_both_master_slot_labels() {
        assert!(is_master_slot(MASTER_SLOT));
        assert!(is_master_slot(MASTER_SLOT_LEGACY));
        assert!(!is_master_slot(FOREGROUND_SLOT));
        assert!(!is_master_slot("master volume"));
    }

    #[test]
    fn step_of_zero_only_clamps() {
        assert_eq!(calculate_next_position(42, 0), 42);
        assert_eq!(calculate_next_position(-5, 0), 0);
        assert_eq!(calculate_next_position(140, 0), 100);
    }

    #[test]
    fn stays_within_bounds() {
        assert_eq!(calculate_next_position(0, -3), 0);
        assert_eq!(calculate_next_position(100, 3), 100);
        assert_eq!(calculate_next_position(98, 4), 100);
    }

    #[test]
    fn odd_steps_snap_to_tens() {
        // Von einer Rasterlinie aus wird normal weitergezählt
        assert_eq!(calculate_next_position(30, 3), 33);
        // Beim Überschreiten der nächsten Linie bleibt der Wert auf ihr stehen
        assert_eq!(calculate_next_position(38, 3), 40);
        assert_eq!(calculate_next_position(42, -3), 40);
    }

    #[test]
    fn even_steps_snap_to_fives() {
        assert_eq!(calculate_next_position(30, 2), 32);
        assert_eq!(calculate_next_position(34, 2), 35);
        assert_eq!(calculate_next_position(36, -2), 35);
    }

    #[test]
    fn even_steps_leave_a_five_boundary_by_one() {
        // Sonderfall: auf einer 5er-Linie, die keine 10er-Linie ist, wird nur
        // um 1 verschoben, damit die nächste Linie wieder erreichbar ist.
        assert_eq!(calculate_next_position(35, 2), 36);
        assert_eq!(calculate_next_position(35, -2), 34);
    }
}
