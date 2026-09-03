use crate::audio;
use crate::protocol::HostToPico;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;
use log::info;
use tauri::Emitter;

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

        #[cfg(target_os = "windows")]
        let _com = unsafe { crate::com::ComGuard::init_multithreaded().ok() };

        let mut last_volumes = [255u8; 4];
        let mut last_mutes = [false; 4];

        loop {
            crate::diagnostics::record_audio_tick();

            let current_slots = {
                let guard = state.lock().unwrap();
                guard.clone()
            };

            let statuses = unsafe { audio::get_monitor_statuses(&current_slots) };
            if statuses.is_err() {
                crate::diagnostics::record_audio_status_error();
            }

            for (i, slot_opt) in current_slots.iter().enumerate() {
                if slot_opt.is_some() {
                    crate::diagnostics::record_audio_slot_poll();
                }

                let status_result = statuses.as_ref().map(|values| values[i]);

                if let Ok(Some((vol_float, is_muted))) = status_result {
                    let vol_u8 = (vol_float * 100.0) as u8;
                    let slot_idx = i as u8;
                    let mut changed = false;

                    if vol_u8 != last_volumes[i] {
                        let _ = tx.send(HostToPico::SetVolume {
                            slot: slot_idx,
                            volume: vol_u8,
                        });
                        crate::diagnostics::record_audio_pico_command_sent();
                        last_volumes[i] = vol_u8;
                        changed = true;
                    }

                    if is_muted != last_mutes[i] {
                        let _ = tx.send(HostToPico::SetMuteState {
                            index: slot_idx,
                            mute: is_muted,
                        });
                        crate::diagnostics::record_audio_pico_command_sent();
                        last_mutes[i] = is_muted;
                        changed = true;
                    }

                    if changed {
                        let _ = app_handle.emit(
                            "audio-update",
                            AudioUpdatePayload {
                                slot: slot_idx,
                                volume: last_volumes[i],
                                muted: last_mutes[i],
                            },
                        );
                        crate::diagnostics::record_audio_update_emit();
                    }
                } else {
                    if let Ok(None) = status_result {
                        crate::diagnostics::record_audio_empty_result();
                    }

                    if 255 != last_volumes[i] {
                        let slot_index = i as u8;
                        let _ = tx.send(HostToPico::SetVolume {
                            slot: slot_index,
                            volume: 255,
                        });
                        crate::diagnostics::record_audio_pico_command_sent();
                        last_volumes[i] = 255;

                        let _ = app_handle.emit(
                            "audio-update",
                            AudioUpdatePayload {
                                slot: slot_index,
                                volume: last_volumes[i],
                                muted: last_mutes[i],
                            },
                        );
                        crate::diagnostics::record_audio_update_emit();
                    }
                }
            }
            thread::sleep(Duration::from_millis(200));
        }
    });
}