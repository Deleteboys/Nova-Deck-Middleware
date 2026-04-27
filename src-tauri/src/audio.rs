use crate::send_to_pico;
use windows::core::Interface;
use windows::Win32::Media::Audio::*;
use windows::Win32::System::Com::*;

pub unsafe fn adjust_volume_for_pids(target_pids: &[u32], step: i8) -> windows::core::Result<bool> {
    if target_pids.is_empty() {
        return Ok(false);
    }

    let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
    let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
    let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole)?;
    let manager: IAudioSessionManager2 =
        device.Activate::<IAudioSessionManager2>(CLSCTX_ALL, None)?;

    let session_enumerator = manager.GetSessionEnumerator()?;
    let session_count = session_enumerator.GetCount()?;
    let step_float = step as f32 / 100.0;

    let mut boundary_hit = false;

    for i in 0..session_count {
        if let Ok(session) = session_enumerator.GetSession(i) {
            if let Ok(session2) = session.cast::<IAudioSessionControl2>() {
                if let Ok(pid) = session2.GetProcessId() {
                    if target_pids.contains(&pid) {
                        if let Ok(simple_volume) = session.cast::<ISimpleAudioVolume>() {
                            let current_vol = simple_volume.GetMasterVolume()?;
                            let new_vol = (current_vol + step_float).clamp(0.0, 1.0);
                            if new_vol == 1.0 || new_vol == 0.0 {
                                boundary_hit = true;
                            }
                            simple_volume.SetMasterVolume(new_vol, std::ptr::null())?;
                        }
                    }
                }
            }
        }
    }
    Ok(boundary_hit)
}

pub unsafe fn toggle_mute_for_pids(target_pids: &[u32]) -> windows::core::Result<()> {
    if target_pids.is_empty() {
        return Ok(());
    }

    let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
    let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
    let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole)?;
    let manager: IAudioSessionManager2 =
        device.Activate::<IAudioSessionManager2>(CLSCTX_ALL, None)?;

    let session_enumerator = manager.GetSessionEnumerator()?;
    let session_count = session_enumerator.GetCount()?;

    for i in 0..session_count {
        if let Ok(session) = session_enumerator.GetSession(i) {
            if let Ok(session2) = session.cast::<IAudioSessionControl2>() {
                if let Ok(pid) = session2.GetProcessId() {
                    // Prüfen, ob die Session in unserer Liste ist
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
