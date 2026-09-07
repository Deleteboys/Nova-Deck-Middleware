//! Desktop-Aktionen, die sich nicht als Tastendruck simulieren lassen.
//!
//! Unter Windows gibt es dafür feste Programme (`LockWorkStation`, `taskmgr`).
//! Unter Linux hängt es vom Desktop ab, deshalb wird eine Liste von Kandidaten
//! durchprobiert; der erste startbare gewinnt.

use log::debug;
use std::process::Command;

/// Führt bekannte Makros nativ aus, statt sie über enigo zu tippen.
/// Gibt `true` zurück, wenn das Makro behandelt wurde.
pub fn run_special_shortcut(keys: &str) -> bool {
    match keys {
        "Win + L" => lock_session(),
        "Ctrl + Shift + Esc" => open_system_monitor(),
        _ => false,
    }
}

fn is_hyprland() -> bool {
    std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some()
}

fn spawn(program: &str, args: &[&str]) -> bool {
    match Command::new(program).args(args).spawn() {
        Ok(_) => {
            debug!("Nativ ausgeführt: {} {}", program, args.join(" "));
            true
        }
        Err(_) => false,
    }
}

fn lock_session() -> bool {
    let mut candidates: Vec<(&str, &[&str])> = Vec::new();

    // Auf Hyprland lauscht standardmäßig nichts auf das logind-Lock-Signal,
    // deshalb dort zuerst den Locker direkt starten.
    if is_hyprland() {
        candidates.push(("hyprlock", &[]));
    }
    // Unter KDE (und allem mit logind-Integration) ist das der saubere Weg.
    candidates.push(("loginctl", &["lock-session"]));
    candidates.push(("swaylock", &["-f"]));
    candidates.push(("xdg-screensaver", &["lock"]));

    candidates
        .into_iter()
        .any(|(program, args)| spawn(program, args))
}

fn open_system_monitor() -> bool {
    const CANDIDATES: [&str; 5] = [
        "plasma-systemmonitor",
        "ksysguard",
        "mission-center",
        "gnome-system-monitor",
        "xfce4-taskmanager",
    ];

    CANDIDATES
        .iter()
        .any(|program| spawn(program, &[]))
}
