//! Desktop-Aktionen, die sich nicht als Tastendruck simulieren lassen.

use log::debug;
use std::process::Command;

/// Führt bekannte Makros nativ aus, statt sie über enigo zu tippen.
/// Gibt `true` zurück, wenn das Makro behandelt wurde.
pub fn run_special_shortcut(keys: &str) -> bool {
    match keys {
        "Win + L" => {
            let _ = Command::new("rundll32.exe")
                .args(["user32.dll,LockWorkStation"])
                .spawn();
            debug!("PC nativ gesperrt: {}", keys);
            true
        }
        "Ctrl + Shift + Esc" => {
            let _ = Command::new("taskmgr.exe").spawn();
            debug!("Task-Manager nativ geöffnet: {}", keys);
            true
        }
        _ => false,
    }
}
