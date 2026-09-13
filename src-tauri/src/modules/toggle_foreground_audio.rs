use crate::action::actions::Action;
use crate::audio::{toggle_mute, AudioTarget};
use crate::platform::window::active_window;
use log::{debug, error};
use std::fmt::Debug;
use crate::platform::audio;

#[derive(Debug, Clone)]
pub struct ToggleForegroundAudioAction {}

impl Action for ToggleForegroundAudioAction {
    fn execute(&self) {
        tauri::async_runtime::spawn(async move {
            let Some(window) = active_window() else {
                return;
            };

            let Some(target) = audio::foreground_target() else {
                return;
            };

            if let Err(e) = toggle_mute(&target) {
                error!("Fehler beim Toggeln des Vordergrund-Programms: {}", e);
            } else {
                debug!("Vordergrund-Audio getoggelt (PID: {:?})", window.pid);
            }
        });
    }
}
