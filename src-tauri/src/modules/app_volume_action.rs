use crate::action::actions::Action;
use crate::audio::adjust_volume_for_pids;
use crate::modules::app_switcher::AppSwitcherRuntime;
use crate::protocol::{HostToPico, VibrationPattern};
use log::error;
use std::sync::{mpsc, Arc, Mutex};
use sysinfo::{ProcessesToUpdate, System};

#[derive(Debug)]
pub struct AppVolumeAction {
    pub process_name: String,
    pub step: i8,
    pub tx: mpsc::Sender<HostToPico>,
    pub switcher_runtime: Option<Arc<Mutex<AppSwitcherRuntime>>>,
}

impl Action for AppVolumeAction {
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

        let step = self.step;
        let tx = self.tx.clone();

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
                match adjust_volume_for_pids(&target_pids, step) {
                    Ok(true) => {
                        let _ = tx.send(HostToPico::Vibrate {
                            pattern: VibrationPattern::Medium,
                        });
                    }
                    Err(e) => error!("Fehler bei {}: {}", name, e),
                    _ => {}
                }
            }
        });
    }
}
