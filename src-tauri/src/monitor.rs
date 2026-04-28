use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::Duration;
use crate::protocol::HostToPico;
use crate::audio;

pub struct MonitorState {
    pub slots: Arc<Mutex<[Option<String>; 4]>>,
}

pub fn start_monitoring(state: Arc<Mutex<[Option<String>; 4]>>, tx: mpsc::Sender<HostToPico>) {
    thread::spawn(move || {
        println!("Audio thread is running in the background.");
        let mut last_volumes = [255u8; 4];
        let mut last_mutes = [false; 4]; // Speicher für Mute-Status

        loop {
            let current_slots = {
                let guard = state.lock().unwrap();
                guard.clone()
            };

            for (i, slot_opt) in current_slots.iter().enumerate() {
                let status_result = unsafe {
                    match slot_opt {
                        Some(name) if name == "Windows Master Volume" => audio::get_master_status().map(Some),
                        Some(name) if name == "Foreground Process" => audio::get_foreground_status(),
                        Some(name) => audio::get_process_status(name),
                        None => Ok(None),
                    }
                };

                if let Ok(Some((vol_float, is_muted))) = status_result {
                    let vol_u8 = (vol_float * 100.0) as u8;
                    let slot_idx = i as u8;

                    // 1. Check Lautstärke-Änderung
                    if vol_u8 != last_volumes[i] {
                        let _ = tx.send(HostToPico::SetVolume { slot: slot_idx, volume: vol_u8 });
                        last_volumes[i] = vol_u8;
                    }

                    // 2. Check Mute-Änderung
                    if is_muted != last_mutes[i] {
                        let _ = tx.send(HostToPico::SetMuteState { index: slot_idx, mute: is_muted });
                        last_mutes[i] = is_muted;
                    }
                }
            }
            thread::sleep(Duration::from_millis(100));
        }
    });
}