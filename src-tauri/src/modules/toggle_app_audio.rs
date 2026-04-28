use crate::action::actions::Action;
use crate::audio::toggle_mute_for_pids;
use std::fmt::Debug;
use sysinfo::{ProcessesToUpdate, System}; // <-- Import der zentralen Funktion

#[derive(Debug, Clone)]
pub struct ToggleAppAudioAction {
    pub process_name: String,
}

impl Action for ToggleAppAudioAction {
    fn execute(&self) {
        let name = self.process_name.clone();

        tauri::async_runtime::spawn(async move {
            let mut sys = System::new();
            sys.refresh_processes(ProcessesToUpdate::All, true);

            let target_pids: Vec<u32> = sys
                .processes()
                .iter()
                .filter(|(_, p)| p.name().to_string_lossy() == name)
                .map(|(pid, _)| pid.as_u32())
                .collect();

            unsafe {
                if let Err(e) = toggle_mute_for_pids(&target_pids) {
                    println!("Fehler beim Toggeln von {}: {}", name, e);
                } else {
                    println!("Audio-Status für {} getoggelt.", name);
                }
            }
        });
    }
}
