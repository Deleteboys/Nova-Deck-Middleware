use crate::action::actions::Action;
use std::fmt::Debug;
use windows::Win32::Media::Audio::*;
use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
use windows::Win32::System::Com::*;

#[derive(Debug, Clone)]
pub struct ToggleMasterMuteAction {} // Braucht keine Parameter!

unsafe fn get_master_volume_interface() -> windows::core::Result<IAudioEndpointVolume> {
    let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
    let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
    let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole)?;
    let endpoint_volume: IAudioEndpointVolume = device.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None)?;
    Ok(endpoint_volume)
}

pub unsafe fn toggle_master_mute() -> windows::core::Result<()> {
    let interface = get_master_volume_interface()?;

    // Aktuellen Master-Mute-Status abfragen
    let is_muted = interface.GetMute()?;

    // Status umkehren und setzen
    let new_mute = !is_muted.as_bool();
    interface.SetMute(new_mute, std::ptr::null())?;

    Ok(())
}

impl Action for ToggleMasterMuteAction {
    fn execute(&self) {
        tauri::async_runtime::spawn(async move {
            unsafe {
                if let Err(e) = toggle_master_mute() {
                    println!("Fehler beim Toggeln des globalen Sounds: {}", e);
                } else {
                    println!("Globaler Sound getoggelt.");
                }
            }
        });
    }
}