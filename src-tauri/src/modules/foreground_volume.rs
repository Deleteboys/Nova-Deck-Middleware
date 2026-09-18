use crate::action::actions::Action;
use crate::audio::{adjust_volume, AudioTarget};
use crate::platform::window::active_window;
use crate::protocol::{HostToPico, VibrationPattern};
use log::{debug, error};
use std::sync::mpsc;
use crate::platform::audio;

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

            let Some(target) = audio::foreground_target() else {
                return;
            };

            debug!("Foreground app: {:?}", target.name_hints);

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
