use windows::core::Interface;
use windows::Win32::Devices::FunctionDiscovery::PKEY_Device_FriendlyName;
use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
use windows::Win32::Media::Audio::*;
use windows::Win32::System::Com::*;
use windows::Win32::UI::Shell::PropertiesSystem::IPropertyStore;
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

struct MonitorSessionStatus {
    pid: u32,
    identifier: String,
    volume: f32,
    muted: bool,
}

pub unsafe fn get_monitor_statuses(
    slots: &[Option<String>; 4],
) -> windows::core::Result<[Option<(f32, bool)>; 4]> {
    let mut results = [None; 4];

    if slots.iter().all(Option::is_none) {
        return Ok(results);
    }

    let _com = crate::com::ComGuard::init_multithreaded()?;
    let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
    let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole)?;

    if slots
        .iter()
        .any(|slot| matches!(slot.as_deref(), Some("Windows Master Volume")))
    {
        let endpoint_volume: IAudioEndpointVolume = device.Activate(CLSCTX_ALL, None)?;
        let status = (
            endpoint_volume.GetMasterVolumeLevelScalar()?,
            endpoint_volume.GetMute()?.as_bool(),
        );

        for (index, slot) in slots.iter().enumerate() {
            if matches!(slot.as_deref(), Some("Windows Master Volume")) {
                results[index] = Some(status);
            }
        }
    }

    let needs_sessions = slots.iter().any(|slot| {
        matches!(slot.as_deref(), Some("Foreground Process"))
            || matches!(slot.as_deref(), Some(name) if name != "Windows Master Volume")
    });

    if !needs_sessions {
        crate::diagnostics::record_audio_snapshot(0);
        return Ok(results);
    }

    let foreground_pid = if slots
        .iter()
        .any(|slot| matches!(slot.as_deref(), Some("Foreground Process")))
    {
        foreground_process_id()
    } else {
        0
    };

    let manager: IAudioSessionManager2 =
        device.Activate::<IAudioSessionManager2>(CLSCTX_ALL, None)?;
    let session_enumerator = manager.GetSessionEnumerator()?;
    let session_count = session_enumerator.GetCount()?;
    let mut sessions = Vec::with_capacity(session_count as usize);

    for index in 0..session_count {
        let Ok(session) = session_enumerator.GetSession(index) else {
            continue;
        };
        let Ok(session2) = session.cast::<IAudioSessionControl2>() else {
            continue;
        };
        let Ok(simple_volume) = session.cast::<ISimpleAudioVolume>() else {
            continue;
        };

        let pid = session2.GetProcessId().unwrap_or(0);
        use windows::Win32::System::Com::CoTaskMemFree;
        use std::ffi::c_void;

        let identifier = if let Ok(pwstr) = session2.GetSessionIdentifier() {
            // 1. Rust-String erstellen
            let text = pwstr.to_string().unwrap_or_default().to_lowercase();

            // 2. Windows-Speicher freigeben
            // Hinweis: Je nach Version des windows-Crates muss der Pointer ggf. anders extrahiert werden (z.B. pwstr.0).
            CoTaskMemFree(Some(pwstr.as_ptr() as *const c_void));

            text
        } else {
            String::new()
        };
        let volume = simple_volume.GetMasterVolume()?;
        let muted = simple_volume.GetMute()?.as_bool();

        sessions.push(MonitorSessionStatus {
            pid,
            identifier,
            volume,
            muted,
        });
    }
    crate::diagnostics::record_audio_snapshot(sessions.len() as u64);

    for (slot_index, slot) in slots.iter().enumerate() {
        let Some(name) = slot.as_deref() else {
            continue;
        };

        if name == "Windows Master Volume" {
            continue;
        }

        let matching_session = if name == "Foreground Process" {
            sessions.iter().find(|session| session.pid == foreground_pid)
        } else {
            let needle = name.to_lowercase();
            sessions
                .iter()
                .find(|session| session.identifier.contains(&needle))
        };

        if let Some(session) = matching_session {
            results[slot_index] = Some((session.volume, session.muted));
        }
    }

    Ok(results)
}

unsafe fn foreground_process_id() -> u32 {
    let hwnd = GetForegroundWindow();
    if hwnd.is_invalid() {
        return 0;
    }

    let mut pid: u32 = 0;
    GetWindowThreadProcessId(hwnd, Some(&mut pid));
    pid
}

pub unsafe fn adjust_volume_for_pids(target_pids: &[u32], step: i8) -> windows::core::Result<bool> {
    if target_pids.is_empty() {
        return Ok(false);
    }

    let _com = crate::com::ComGuard::init_multithreaded()?;
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
                            let current_vol_pct = (current_vol * 100.0).round() as i16;
                            let new_vol_pct = (current_vol_pct + step as i16).clamp(0, 100);
                            if new_vol_pct == 100 || new_vol_pct == 0 {
                                boundary_hit = true;
                            }

                            let new_vol = new_vol_pct as f32 / 100.0;
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

    let _com = crate::com::ComGuard::init_multithreaded()?;
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
    let _com = crate::com::ComGuard::init_multithreaded()?;

    let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
    let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole)?;

    let endpoint_volume: IAudioEndpointVolume = device.Activate(CLSCTX_ALL, None)?;

    endpoint_volume.GetMasterVolumeLevelScalar()
}

pub unsafe fn get_volume_by_process_name(name: &str) -> windows::core::Result<Option<f32>> {
    let _com = crate::com::ComGuard::init_multithreaded()?;
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
    let _com = crate::com::ComGuard::init_multithreaded()?;
    let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
    let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole)?;
    let endpoint_volume: IAudioEndpointVolume = device.Activate(CLSCTX_ALL, None)?;

    let vol = endpoint_volume.GetMasterVolumeLevelScalar()?;
    let mute = endpoint_volume.GetMute()?;

    Ok((vol, mute.as_bool()))
}

pub unsafe fn get_process_status(name: &str) -> windows::core::Result<Option<(f32, bool)>> {
    let _com = crate::com::ComGuard::init_multithreaded()?;
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
    let _com = crate::com::ComGuard::init_multithreaded()?;

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

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct AudioDeviceInfo {
    pub id: String,
    pub name: String,
}

pub unsafe fn list_audio_devices() -> windows::core::Result<Vec<AudioDeviceInfo>> {
    let _com = crate::com::ComGuard::init_multithreaded()?;
    let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;

    let collection = enumerator.EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)?;
    let count = collection.GetCount()?;

    let mut devices = Vec::with_capacity(count as usize);

    for i in 0..count {
        let device = collection.Item(i)?;
        let id = device.GetId()?.to_string()?;

        let store: IPropertyStore = device.OpenPropertyStore(STGM_READ)?;
        let prop = store.GetValue(&PKEY_Device_FriendlyName)?;
        let name = prop.Anonymous.Anonymous.Anonymous.pwszVal.to_string()?;

        devices.push(AudioDeviceInfo { id, name });
    }

    Ok(devices)
}
