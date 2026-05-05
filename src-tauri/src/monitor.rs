use crate::audio;
use crate::protocol::HostToPico;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;
use log::info;
use tauri::Emitter; // WICHTIG: Erlaubt das Senden von Events ans Frontend

#[derive(serde::Serialize, Clone)]
pub struct AudioUpdatePayload {
    pub slot: u8,
    pub volume: u8,
    pub muted: bool,
}

pub fn start_monitoring(
    app_handle: tauri::AppHandle,
    state: Arc<Mutex<[Option<String>; 4]>>,
    tx: mpsc::Sender<HostToPico>,
) {
    thread::spawn(move || {
        info!("Audio thread is running in the background.");
        let mut last_volumes = [255u8; 4];
        let mut last_mutes = [false; 4];

        loop {
            let current_slots = {
                let guard = state.lock().unwrap();
                guard.clone()
            };

            for (i, slot_opt) in current_slots.iter().enumerate() {
                let status_result = unsafe {
                    match slot_opt {
                        Some(name) if name == "Windows Master Volume" => {
                            audio::get_master_status().map(Some)
                        }
                        Some(name) if name == "Foreground Process" => {
                            audio::get_foreground_status()
                        }
                        Some(name) => audio::get_process_status(name),
                        None => Ok(None),
                    }
                };
                if let Ok(Some((vol_float, is_muted))) = status_result {
                    let vol_u8 = (vol_float * 100.0) as u8;
                    let slot_idx = i as u8;
                    let mut changed = false;

                    if vol_u8 != last_volumes[i] {
                        let _ = tx.send(HostToPico::SetVolume {
                            slot: slot_idx,
                            volume: vol_u8,
                        });
                        last_volumes[i] = vol_u8;
                        changed = true;
                    }

                    if is_muted != last_mutes[i] {
                        let _ = tx.send(HostToPico::SetMuteState {
                            index: slot_idx,
                            mute: is_muted,
                        });
                        last_mutes[i] = is_muted;
                        changed = true;
                    }

                    // HIER NEU: Sende die Änderung auch an das Frontend
                    if changed {
                        let _ = app_handle.emit(
                            "audio-update",
                            AudioUpdatePayload {
                                slot: slot_idx,
                                volume: last_volumes[i],
                                muted: last_mutes[i],
                            },
                        );
                    }
                } else {
                    if 255 != last_volumes[i] {
                        let slot_index = i as u8;
                        let _ = tx.send(HostToPico::SetVolume {
                            slot: slot_index,
                            volume: 255,
                        });
                        last_volumes[i] = 255;

                        let _ = app_handle.emit(
                            "audio-update",
                            AudioUpdatePayload {
                                slot: slot_index,
                                volume: last_volumes[i],
                                muted: last_mutes[i],
                            },
                        );
                    }
                }
            }
            thread::sleep(Duration::from_millis(250));
        }
    });
}
