use crate::action::actions::Action;
use crate::audio::toggle_mute_for_pids;
use crate::modules::app_switcher::AppSwitcherRuntime;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use log::error;
use sysinfo::{ProcessesToUpdate, System};

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
                    error!("Fehler beim Toggeln von {}: {}", name, e);
                }
            }
        });
    }
}
