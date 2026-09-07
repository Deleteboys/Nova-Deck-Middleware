use crate::action::actions::Action;
use crate::audio::{adjust_volume, target_for_process_name};
use crate::modules::app_switcher::AppSwitcherRuntime;
use crate::protocol::{HostToPico, VibrationPattern};
use log::error;
use std::sync::{mpsc, Arc, Mutex};

#[derive(Debug)]
pub struct AppVolumeAction {
    pub process_name: String,
    pub step: i8,
    pub tx: mpsc::Sender<HostToPico>,
    pub switcher_runtime: Option<Arc<Mutex<AppSwitcherRuntime>>>,
    pub snap: bool,
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
        let snap = self.snap;

        tauri::async_runtime::spawn(async move {
            let target = target_for_process_name(&name);

            match adjust_volume(&target, step, snap) {
                Ok(true) => {
                    let _ = tx.send(HostToPico::Vibrate {
                        pattern: VibrationPattern::Medium,
                    });
                }
                Err(e) => error!("Fehler bei {}: {}", name, e),
                _ => {}
            }
        });
    }
}
