use crate::action::actions::Action;
use std::fmt::Debug;
use std::sync::mpsc;
use windows::Win32::Media::Audio::*;
use windows::Win32::Media::Audio::Endpoints::*;
use windows::Win32::System::Com::*;
use crate::protocol::{HostToPico, VibrationPattern};

#[derive(Debug, Clone)]
pub struct MasterVolumeAction {
    pub step: i8,
    pub tx: mpsc::Sender<HostToPico>,
}

// Helper-Funktionen für die Windows Core Audio API
unsafe fn get_master_volume_interface() -> windows::core::Result<IAudioEndpointVolume> {
    // WICHTIG: COM für diesen Thread initialisieren
    let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
    let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
    let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole)?;
    let endpoint_volume: IAudioEndpointVolume = device.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None)?;
    Ok(endpoint_volume)
}

pub unsafe fn set_master_volume(level: f32) -> windows::core::Result<()> {
    let interface = get_master_volume_interface()?;
    interface.SetMasterVolumeLevelScalar(level, std::ptr::null())
}

pub unsafe fn get_master_volume() -> windows::core::Result<f32> {
    let interface = get_master_volume_interface()?;
    interface.GetMasterVolumeLevelScalar()
}

impl Action for MasterVolumeAction {
    fn execute(&self) {
        // Windows nutzt f32 von 0.0 bis 1.0 (also z.B. 5 / 100 = 0.05)
        let step_float = self.step as f32 / 100.0;
        let tx = self.tx.clone();

        // Auslagern in einen Hintergrund-Task, damit das Streamdeck sofort weiterläuft
        tauri::async_runtime::spawn(async move {
            unsafe {
                if let Ok(current_vol) = get_master_volume() {
                    // Neue Lautstärke berechnen und zwischen 0.0 und 1.0 limitieren
                    let new_vol = (current_vol + step_float).clamp(0.0, 1.0);
                    if new_vol == 1.0 || new_vol == 0.0 {
                        let _ = tx.send(HostToPico::Vibrate { pattern: VibrationPattern::Medium });
                    }

                    if let Err(e) = set_master_volume(new_vol) {
                        println!("Fehler beim Setzen der Windows-Lautstärke: {}", e);
                    } else {
                        println!("Windows Volume von {:.0}% auf {:.0}% gesetzt", current_vol * 100.0, new_vol * 100.0);
                    }
                } else {
                    println!("Konnte aktuelle Windows-Lautstärke nicht auslesen.");
                }
            }
        });
    }
}