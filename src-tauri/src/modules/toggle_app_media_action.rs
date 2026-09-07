use crate::action::actions::Action;
use crate::platform::media::toggle_play_pause;
use log::{error, info};
use std::fmt::Debug;
use std::thread;

#[derive(Debug, Clone)]
pub struct ToggleAppMediaAction {
    pub process_name: String,
}

impl Action for ToggleAppMediaAction {
    fn execute(&self) {
        let name = self.process_name.clone();

        // Eigener Thread statt async-Runtime: beide Backends (WinRT bzw. D-Bus)
        // blockieren, bis die Medien-Session geantwortet hat.
        thread::spawn(move || match toggle_play_pause(&name) {
            Ok(toggled) if toggled.is_empty() => {
                info!("Keine Medien-Session für {} gefunden.", name)
            }
            Ok(toggled) => info!("Media-Status (Play/Pause) getoggelt für {}.", toggled.join(", ")),
            Err(e) => error!("Fehler beim Toggeln der Medien für {}: {}", name, e),
        });
    }
}
