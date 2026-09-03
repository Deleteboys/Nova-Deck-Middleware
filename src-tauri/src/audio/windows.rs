use super::AudioDeviceInfo;
use super::calculate_next_position;
use windows::core::Interface;
use windows::Win32::Devices::FunctionDiscovery::PKEY_Device_FriendlyName;
use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
use windows::Win32::Media::Audio::*;
use windows::Win32::System::Com::*;
use windows::Win32::UI::Shell::PropertiesSystem::IPropertyStore;
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

// Deine internen Structs und privaten Funktionen bleiben identisch...
struct MonitorSessionStatus {
    pid: u32,
    identifier: String,
    volume: f32,
    muted: bool,
}

// Öffentliche Funktionen mappen das windows::core::Result in ein generisches Result
pub unsafe fn get_master_volume() -> Result<f32, String> {
    get_master_volume_internal().map_err(|e| e.to_string())
}

unsafe fn get_master_volume_internal() -> windows::core::Result<f32> {
    let _com = crate::com::ComGuard::init_multithreaded()?;
    let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
    let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole)?;
    let endpoint_volume: IAudioEndpointVolume = device.Activate(CLSCTX_ALL, None)?;
    endpoint_volume.GetMasterVolumeLevelScalar()
}


pub fn set_master_volume(level: f32) -> Result<(), String> {
    unsafe {
        let _com = crate::com::ComGuard::init_multithreaded().map_err(|e| e.to_string())?;
        let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
            .map_err(|e| e.to_string())?;
        let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole)
            .map_err(|e| e.to_string())?;
        let endpoint_volume: IAudioEndpointVolume = device.Activate(CLSCTX_ALL, None)
            .map_err(|e| e.to_string())?;

        endpoint_volume
            .SetMasterVolumeLevelScalar(level.clamp(0.0, 1.0), std::ptr::null())
            .map_err(|e| e.to_string())
    }
}

pub fn toggle_master_mute() -> Result<(), String> {
    unsafe {
        let _com = crate::com::ComGuard::init_multithreaded().map_err(|e| e.to_string())?;
        let enumerator: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).map_err(|e| e.to_string())?;
        let device = enumerator
            .GetDefaultAudioEndpoint(eRender, eConsole)
            .map_err(|e| e.to_string())?;
        let endpoint_volume: IAudioEndpointVolume =
            device.Activate(CLSCTX_ALL, None).map_err(|e| e.to_string())?;

        let is_muted = endpoint_volume.GetMute().map_err(|e| e.to_string())?;
        let new_mute = !is_muted.as_bool();

        endpoint_volume
            .SetMute(new_mute, std::ptr::null())
            .map_err(|e| e.to_string())
    }
}

pub fn get_active_audio_pids() -> std::collections::HashSet<u32> {
    let mut audio_pids = std::collections::HashSet::new();
    unsafe {
        if let Ok(_com) = crate::com::ComGuard::init_apartment_threaded() {
            let enumerator_result: windows::core::Result<IMMDeviceEnumerator> =
                CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL);
            if let Ok(enumerator) = enumerator_result {
                if let Ok(device) = enumerator.GetDefaultAudioEndpoint(eRender, eConsole) {
                    if let Ok(manager) = device.Activate::<IAudioSessionManager2>(CLSCTX_ALL, None) {
                        if let Ok(session_enumerator) = manager.GetSessionEnumerator() {
                            let count = session_enumerator.GetCount().unwrap_or(0);
                            for i in 0..count {
                                if let Ok(session) = session_enumerator.GetSession(i) {
                                    if let Ok(session2) = session.cast::<IAudioSessionControl2>() {
                                        if let Ok(pid) = session2.GetProcessId() {
                                            if pid > 0 {
                                                audio_pids.insert(pid);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    audio_pids
}

// Dieselbe Wrapper-Struktur wendest du auf list_audio_devices, get_monitor_statuses etc. an.