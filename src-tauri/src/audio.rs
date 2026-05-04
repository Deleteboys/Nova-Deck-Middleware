use windows::core::Interface;
use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
use windows::Win32::Media::Audio::*;
use windows::Win32::System::Com::*;
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

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

pub unsafe fn get_master_volume() -> windows::core::Result<f32> {
    let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

    let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
    let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole)?;

    let endpoint_volume: IAudioEndpointVolume = device.Activate(CLSCTX_ALL, None)?;

    endpoint_volume.GetMasterVolumeLevelScalar()
}

pub unsafe fn get_volume_by_process_name(name: &str) -> windows::core::Result<Option<f32>> {
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
                if let Ok(process_path) = session2.GetSessionIdentifier() {
                    let id = process_path.to_string().unwrap_or_default().to_lowercase();

                    if id.contains(&name.to_lowercase()) {
                        if let Ok(simple_volume) = session.cast::<ISimpleAudioVolume>() {
                            return Ok(Some(simple_volume.GetMasterVolume()?));
                        }
                    }
                }
            }
        }
    }
    Ok(None)
}

pub unsafe fn get_master_status() -> windows::core::Result<(f32, bool)> {
    let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
    let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
    let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole)?;
    let endpoint_volume: IAudioEndpointVolume = device.Activate(CLSCTX_ALL, None)?;

    let vol = endpoint_volume.GetMasterVolumeLevelScalar()?;
    let mute = endpoint_volume.GetMute()?;

    Ok((vol, mute.as_bool()))
}

pub unsafe fn get_process_status(name: &str) -> windows::core::Result<Option<(f32, bool)>> {
    let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
    let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
    let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole)?;
    let manager: IAudioSessionManager2 =
        device.Activate::<IAudioSessionManager2>(CLSCTX_ALL, None)?;
    let session_enumerator = manager.GetSessionEnumerator()?;

    for i in 0..session_enumerator.GetCount()? {
        if let Ok(session) = session_enumerator.GetSession(i) {
            if let Ok(session2) = session.cast::<IAudioSessionControl2>() {
                if let Ok(process_path) = session2.GetSessionIdentifier() {
                    let id = process_path.to_string().unwrap_or_default().to_lowercase();
                    if id.contains(&name.to_lowercase()) {
                        if let Ok(simple_volume) = session.cast::<ISimpleAudioVolume>() {
                            let vol = simple_volume.GetMasterVolume()?;
                            let mute = simple_volume.GetMute()?;
                            return Ok(Some((vol, mute.as_bool())));
                        }
                    }
                }
            }
        }
    }
    Ok(None)
}

pub unsafe fn get_foreground_status() -> windows::core::Result<Option<(f32, bool)>> {
    let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

    // 1. Vordergrund-Fenster und dessen PID ermitteln
    let hwnd = GetForegroundWindow();
    if hwnd.is_invalid() {
        return Ok(None);
    }

    let mut pid: u32 = 0;
    GetWindowThreadProcessId(hwnd, Some(&mut pid));

    if pid == 0 {
        return Ok(None);
    }

    // 2. Audio-Session für diese spezifische PID suchen
    let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
    let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole)?;
    let manager: IAudioSessionManager2 =
        device.Activate::<IAudioSessionManager2>(CLSCTX_ALL, None)?;
    let session_enumerator = manager.GetSessionEnumerator()?;

    for i in 0..session_enumerator.GetCount()? {
        if let Ok(session) = session_enumerator.GetSession(i) {
            if let Ok(session2) = session.cast::<IAudioSessionControl2>() {
                if let Ok(session_pid) = session2.GetProcessId() {
                    if session_pid == pid {
                        if let Ok(simple_volume) = session.cast::<ISimpleAudioVolume>() {
                            let vol = simple_volume.GetMasterVolume()?;
                            let mute = simple_volume.GetMute()?;
                            return Ok(Some((vol, mute.as_bool())));
                        }
                    }
                }
            }
        }
    }
    Ok(None)
}
