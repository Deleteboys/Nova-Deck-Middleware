use crate::action::actions::Action;
use crate::audio::toggle_mute_for_pids;
use std::fmt::Debug;
use log::{debug, error};

#[derive(Debug, Clone)]
pub struct ToggleForegroundAudioAction {}

impl Action for ToggleForegroundAudioAction {
    fn execute(&self) {
        tauri::async_runtime::spawn(async move {
            unsafe {
                let pid = crate::platform::get_active_window_pid();

                if pid != 0 {
                    // Wir übergeben das Array mit einer einzigen PID an audio.rs
                    if let Err(e) = toggle_mute_for_pids(&[pid]) {
                        error!("Fehler beim Toggeln des Vordergrund-Programms: {}", e);
                    } else {
                        debug!("Vordergrund-Audio getoggelt (PID: {})", pid);
                    }
                }
            }
        });
    }
}
