use crate::action::actions::Action;
use crate::audio::toggle_master_mute;
use log::{debug, error};
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct ToggleMasterMuteAction {}

impl Action for ToggleMasterMuteAction {
    fn execute(&self) {
        tauri::async_runtime::spawn(async move {
            if let Err(e) = toggle_master_mute() {
                error!("Fehler beim Toggeln des globalen Sounds: {}", e);
            } else {
                debug!("Globaler Sound getoggelt.");
            }
        });
    }
}
