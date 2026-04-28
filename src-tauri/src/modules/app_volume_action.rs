use crate::action::actions::Action;
use crate::audio::adjust_volume_for_pids;
use crate::protocol::{HostToPico, VibrationPattern};
use std::sync::mpsc;
use sysinfo::{ProcessesToUpdate, System};

#[derive(Debug, Clone)]
pub struct AppVolumeAction {
    pub process_name: String,
    pub step: i8,
    pub tx: mpsc::Sender<HostToPico>,
}

impl Action for AppVolumeAction {
    fn execute(&self) {
        let name = self.process_name.clone();
        let step = self.step;
        let tx = self.tx.clone();

        tauri::async_runtime::spawn(async move {
            // FIX: Erstelle nur ein leeres System-Objekt (ohne Hardware-Sensoren, Disks, etc.)
            let mut sys = System::new();
            // Lade nur die Prozesse, das geht in wenigen Millisekunden
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
                    Err(e) => println!("Fehler bei {}: {}", name, e),
                    _ => {} // Nichts tun, wenn das Limit nicht erreicht wurde
                }
            }
        });
    }
}
