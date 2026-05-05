use crate::action::actions::Action;
use crate::audio::toggle_mute_for_pids;
use std::fmt::Debug;
use log::{debug, error};
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId}; // <-- Import der zentralen Funktion

#[derive(Debug, Clone)]
pub struct ToggleForegroundAudioAction {}

impl Action for ToggleForegroundAudioAction {
    fn execute(&self) {
        tauri::async_runtime::spawn(async move {
            unsafe {
                let hwnd = GetForegroundWindow();
                if hwnd.is_invalid() {
                    return;
                }

                let mut pid: u32 = 0;
                GetWindowThreadProcessId(hwnd, Some(&mut pid));

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
