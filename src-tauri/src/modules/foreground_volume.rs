use crate::action::actions::Action;
use crate::audio::{adjust_volume, AudioTarget};
use crate::platform::window::active_window;
use crate::protocol::{HostToPico, VibrationPattern};
use log::error;
use std::sync::mpsc;

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
            let Some(window) = active_window() else {
                return;
            };

            let target = AudioTarget::from_pids(window.pid).with_hint(window.app_id.as_deref());
            if target.is_empty() {
                return;
            }

            match adjust_volume(&target, step, snap) {
                Ok(true) => {
                    let _ = tx.send(HostToPico::Vibrate {
                        pattern: VibrationPattern::Medium,
                    });
                }
                Err(e) => error!(
                    "Vordergrund-Lautstärke konnte nicht angepasst werden (PID: {:?}): {}",
                    window.pid, e
                ),
                _ => {} // Nichts tun, wenn das Limit nicht erreicht wurde
            }
        });
    }
}
