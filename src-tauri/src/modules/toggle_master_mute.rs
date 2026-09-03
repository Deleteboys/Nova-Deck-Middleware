use crate::action::actions::Action;
use log::{debug, error};
use std::fmt::Debug;
use crate::audio::toggle_master_mute;

#[derive(Debug, Clone)]
pub struct ToggleMasterMuteAction {}

impl Action for ToggleMasterMuteAction {
    fn execute(&self) {
        tauri::async_runtime::spawn(async move {
            unsafe {
                if let Err(e) = toggle_master_mute() {
                    error!("Fehler beim Toggeln des globalen Sounds: {}", e);
                } else {
                    debug!("Globaler Sound getoggelt.");
                }
            }
        });
    }
}
