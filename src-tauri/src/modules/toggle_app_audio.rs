use crate::action::actions::Action;
use crate::audio::{target_for_process_name, toggle_mute};
use crate::modules::app_switcher::AppSwitcherRuntime;
use log::error;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct ToggleAppAudioAction {
    pub process_name: String,
    pub switcher_runtime: Option<Arc<Mutex<AppSwitcherRuntime>>>,
}

impl Action for ToggleAppAudioAction {
    fn execute(&self) {
        let name = if let Some(rt) = &self.switcher_runtime {
            let rt = rt.lock().unwrap();
            if rt.apps.is_empty() {
                return;
            }
            rt.apps[rt.current_index].process_name.clone()
        } else {
            self.process_name.clone()
        };

        tauri::async_runtime::spawn(async move {
            let target = target_for_process_name(&name);

            if let Err(e) = toggle_mute(&target) {
                error!("Fehler beim Toggeln von {}: {}", name, e);
            }
        });
    }
}
