use crate::action::actions::Action;
use std::fmt::Debug;
use sysinfo::{System, ProcessesToUpdate};
use windows::core::Interface;
use windows::Win32::Media::Audio::*;
use windows::Win32::System::Com::*;

#[derive(Debug, Clone)]
pub struct ToggleAppAudioAction {
    pub process_name: String,
}

unsafe fn get_audio_session_manager() -> windows::core::Result<IAudioSessionManager2> {
    let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
    let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;

    // eRender und eConsole sind durch "use windows::Win32::Media::Audio::*;" oben schon bekannt!
    let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole)?;
    let session_manager: IAudioSessionManager2 = device.Activate::<IAudioSessionManager2>(CLSCTX_ALL, None)?;
    Ok(session_manager)
}

pub unsafe fn toggle_app_mute(target_name: &str) -> windows::core::Result<()> {
    let mut sys = System::new_all();
    sys.refresh_processes(ProcessesToUpdate::All, true);

    let target_pids: Vec<u32> = sys.processes().iter()
        .filter(|(_, p)| p.name().to_string_lossy() == target_name)
        .map(|(pid, _)| pid.as_u32())
        .collect();

    if target_pids.is_empty() {
        return Ok(());
    }

    let manager = get_audio_session_manager()?;
    let session_enumerator = manager.GetSessionEnumerator()?;
    let session_count = session_enumerator.GetCount()?;

    for i in 0..session_count {
        if let Ok(session) = session_enumerator.GetSession(i) {
            if let Ok(session2) = session.cast::<IAudioSessionControl2>() {
                if let Ok(pid) = session2.GetProcessId() {

                    if target_pids.contains(&pid) {
                        if let Ok(simple_volume) = session.cast::<ISimpleAudioVolume>() {
                            let is_muted = simple_volume.GetMute()?;
                            let new_mute = !is_muted.as_bool();
                            simple_volume.SetMute(new_mute, std::ptr::null())?;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

impl Action for ToggleAppAudioAction {
    fn execute(&self) {
        let name = self.process_name.clone();

        tauri::async_runtime::spawn(async move {
            unsafe {
                if let Err(e) = toggle_app_mute(&name) {
                    println!("Fehler beim Toggeln von {}: {}", name, e);
                } else {
                    println!("Audio-Status für {} getoggelt.", name);
                }
            }
        });
    }
}