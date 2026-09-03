use crate::action::actions::Action;
use crate::audio::{calculate_next_position, get_master_volume, set_master_volume};
use crate::protocol::{HostToPico, VibrationPattern};
use log::{debug, error};
use std::fmt::Debug;
use std::sync::mpsc;

#[derive(Debug, Clone)]
pub struct MasterVolumeAction {
    pub step: i8,
    pub tx: mpsc::Sender<HostToPico>,
    pub snap: bool,
}

impl Action for MasterVolumeAction {
    fn execute(&self) {
        let step = self.step as i32;
        let tx = self.tx.clone();
        let snap = self.snap;

        tauri::async_runtime::spawn(async move {
            unsafe {
                if let Ok(current_vol) = get_master_volume() {
                    let current_vol_pct = (current_vol * 100.0).round() as i32;
                    let new_vol_pct = if snap {
                        calculate_next_position(current_vol_pct, step)
                    } else {
                        (current_vol_pct + step).clamp(0, 100)
                    };
                    let new_vol = new_vol_pct as f32 / 100.0;

                    if new_vol_pct == 100 || new_vol_pct == 0 {
                        let _ = tx.send(HostToPico::Vibrate {
                            pattern: VibrationPattern::Medium,
                        });
                    }

                    if let Err(e) = set_master_volume(new_vol) {
                        error!("Fehler beim Setzen der Windows-Lautstärke: {}", e);
                    } else {
                        debug!(
                            "Windows Volume von {:.0}% auf {:.0}% gesetzt",
                            current_vol * 100.0,
                            new_vol * 100.0
                        );
                    }
                } else {
                    debug!("Konnte aktuelle Windows-Lautstärke nicht auslesen.");
                }
            }
        });
    }
}
