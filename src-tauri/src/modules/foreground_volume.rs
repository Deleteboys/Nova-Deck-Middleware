use crate::action::actions::Action;
use crate::audio::adjust_volume_for_pids;
use crate::protocol::{HostToPico, VibrationPattern};
use std::sync::mpsc;
use log::error;

#[derive(Debug, Clone)]
pub struct ForegroundVolumeAction {
    pub step: i8,
    pub tx: mpsc::Sender<HostToPico>,
    pub snap: bool,
}

impl Action for ForegroundVolumeAction {
    fn execute(&self) {
        let step = self.step;
        let snap = self.snap;
        let tx = self.tx.clone();

        tauri::async_runtime::spawn(async move {
            unsafe {
                let pid = crate::platform::get_active_window_pid();

                if pid != 0 {
                    match adjust_volume_for_pids(&[pid], step, snap) {
                        Ok(true) => {
                            let _ = tx.send(HostToPico::Vibrate {
                                pattern: VibrationPattern::Medium,
                            });
                        }
                        Err(_e) => error!("Vordergrund-Lautstärke angepasst (PID: {})", pid),
                        _ => {} // Nichts tun, wenn das Limit nicht erreicht wurde
                    }
                }
            }
        });
    }
}
