//! WASAPI-Backend: Audio-Sessions, Master-Endpoint und Ausgabegeräte.

use crate::audio::{calculate_next_position, is_master_slot, FOREGROUND_SLOT};
use crate::platform::windows::com::ComGuard;
use crate::platform::windows::device;
use crate::platform::windows::window::foreground_process_id;
use crate::platform::{AudioDeviceInfo, AudioTarget, Result};
use std::ffi::c_void;
use windows::core::Interface;
use windows::Win32::Devices::FunctionDiscovery::PKEY_Device_FriendlyName;
use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
use windows::Win32::Media::Audio::*;
use windows::Win32::System::Com::*;
use windows::Win32::UI::Shell::PropertiesSystem::IPropertyStore;

/// Hält COM für die Dauer eines Threads offen. Ohne diesen Guard müsste jeder
/// einzelne Aufruf COM neu initialisieren.
pub struct ThreadGuard {
    _com: Option<ComGuard>,
}

impl ThreadGuard {
    pub fn acquire() -> Self {
        Self {
            _com: ComGuard::init_multithreaded().ok(),
        }
    }
}

struct MonitorSessionStatus {
    pid: u32,
    identifier: String,
    volume: f32,
    muted: bool,
}

/// Öffnet den Session-Manager des Standard-Wiedergabegeräts.
unsafe fn default_session_manager() -> Result<IAudioSessionManager2> {
    let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
    let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole)?;
    Ok(device.Activate::<IAudioSessionManager2>(CLSCTX_ALL, None)?)
}

unsafe fn default_endpoint_volume() -> Result<IAudioEndpointVolume> {
    let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
    let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole)?;
    Ok(device.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None)?)
}

/// Liest den Session-Identifier aus und gibt den von COM allozierten Speicher frei.
unsafe fn session_identifier(session: &IAudioSessionControl2) -> String {
    match session.GetSessionIdentifier() {
        Ok(pwstr) => {
            let text = pwstr.to_string().unwrap_or_default().to_lowercase();
            CoTaskMemFree(Some(pwstr.as_ptr() as *const c_void));
            text
        }
        Err(_) => String::new(),
    }
}

pub fn get_monitor_statuses(slots: &[Option<String>; 4]) -> Result<[Option<(f32, bool)>; 4]> {
    let mut results = [None; 4];

    if slots.iter().all(Option::is_none) {
        return Ok(results);
    }

    unsafe {
        let _com = ComGuard::init_multithreaded()?;
        let enumerator: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
        let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole)?;

        if slots
            .iter()
            .any(|slot| slot.as_deref().is_some_and(is_master_slot))
        {
            let endpoint_volume: IAudioEndpointVolume = device.Activate(CLSCTX_ALL, None)?;
            let status = (
                endpoint_volume.GetMasterVolumeLevelScalar()?,
                endpoint_volume.GetMute()?.as_bool(),
            );

            for (index, slot) in slots.iter().enumerate() {
                if slot.as_deref().is_some_and(is_master_slot) {
                    results[index] = Some(status);
                }
            }
        }

        let needs_sessions = slots
            .iter()
            .any(|slot| slot.as_deref().is_some_and(|name| !is_master_slot(name)));

        if !needs_sessions {
            crate::diagnostics::record_audio_snapshot(0);
            return Ok(results);
        }

        let foreground_pid = if slots
            .iter()
            .any(|slot| matches!(slot.as_deref(), Some(FOREGROUND_SLOT)))
        {
            foreground_process_id().unwrap_or(0)
        } else {
            0
        };

        let manager = device.Activate::<IAudioSessionManager2>(CLSCTX_ALL, None)?;
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

            sessions.push(MonitorSessionStatus {
                pid: session2.GetProcessId().unwrap_or(0),
                identifier: session_identifier(&session2),
                volume: simple_volume.GetMasterVolume()?,
                muted: simple_volume.GetMute()?.as_bool(),
            });
        }
        crate::diagnostics::record_audio_snapshot(sessions.len() as u64);

        for (slot_index, slot) in slots.iter().enumerate() {
            let Some(name) = slot.as_deref() else {
                continue;
            };

            if is_master_slot(name) {
                continue;
            }

            let matching_session = if name == FOREGROUND_SLOT {
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
}

/// Liefert die Session-Identifier (kleingeschrieben) aller Audio-Sessions, die
/// aktuell offen sind und eine Lautstärke besitzen. Beendete (expired) Sessions
/// und Sessions ohne Prozess werden übersprungen.
pub fn list_open_session_identifiers() -> Result<Vec<String>> {
    unsafe {
        let _com = ComGuard::init_multithreaded()?;
        let manager = default_session_manager()?;
        let session_enumerator = manager.GetSessionEnumerator()?;
        let session_count = session_enumerator.GetCount()?;

        let mut identifiers = Vec::with_capacity(session_count as usize);

        for index in 0..session_count {
            let Ok(session) = session_enumerator.GetSession(index) else {
                continue;
            };
            let Ok(session2) = session.cast::<IAudioSessionControl2>() else {
                continue;
            };

            if session2.GetProcessId().unwrap_or(0) == 0 {
                continue;
            }

            let expired = session
                .GetState()
                .map(|state| state == AudioSessionStateExpired)
                .unwrap_or(true);
            if expired {
                continue;
            }

            // Ohne ISimpleAudioVolume gibt es keine Lautstärke zu steuern
            if session.cast::<ISimpleAudioVolume>().is_err() {
                continue;
            }

            let text = session_identifier(&session2);
            if !text.is_empty() {
                identifiers.push(text);
            }
        }

        Ok(identifiers)
    }
}

/// PIDs aller Prozesse, die eine Audio-Session offen haben.
pub fn list_session_pids() -> Result<Vec<u32>> {
    unsafe {
        let _com = ComGuard::init_apartment_threaded()?;
        let manager = default_session_manager()?;
        let session_enumerator = manager.GetSessionEnumerator()?;
        let count = session_enumerator.GetCount()?;

        let mut pids = Vec::with_capacity(count as usize);

        for index in 0..count {
            let Ok(session) = session_enumerator.GetSession(index) else {
                continue;
            };
            let Ok(session2) = session.cast::<IAudioSessionControl2>() else {
                continue;
            };
            if let Ok(pid) = session2.GetProcessId() {
                if pid > 0 {
                    pids.push(pid);
                }
            }
        }

        Ok(pids)
    }
}

/// Passt die Lautstärke aller Sessions des Ziels an. `true` bedeutet, dass eine
/// Session dabei 0 % oder 100 % erreicht hat (Feedback für das Gerät).
///
/// `name_hints` des Ziels werden auf Windows nicht ausgewertet – die WASAPI-Session
/// trägt die PID des Fensterprozesses, damit ist die PID-Zuordnung eindeutig.
pub fn adjust_volume(target: &AudioTarget, step: i8, snap: bool) -> Result<bool> {
    if target.pids.is_empty() {
        return Ok(false);
    }

    unsafe {
        let _com = ComGuard::init_multithreaded()?;
        let manager = default_session_manager()?;
        let session_enumerator = manager.GetSessionEnumerator()?;
        let session_count = session_enumerator.GetCount()?;

        let mut boundary_hit = false;

        for i in 0..session_count {
            let Ok(session) = session_enumerator.GetSession(i) else {
                continue;
            };
            let Ok(session2) = session.cast::<IAudioSessionControl2>() else {
                continue;
            };
            let Ok(pid) = session2.GetProcessId() else {
                continue;
            };
            if !target.pids.contains(&pid) {
                continue;
            }
            let Ok(simple_volume) = session.cast::<ISimpleAudioVolume>() else {
                continue;
            };

            let current_vol_pct = (simple_volume.GetMasterVolume()? * 100.0).round() as i32;
            let new_vol_pct = if snap {
                calculate_next_position(current_vol_pct, step as i32)
            } else {
                (current_vol_pct + step as i32).clamp(0, 100)
            };
            if new_vol_pct == 100 || new_vol_pct == 0 {
                boundary_hit = true;
            }

            simple_volume.SetMasterVolume(new_vol_pct as f32 / 100.0, std::ptr::null())?;
        }

        Ok(boundary_hit)
    }
}

pub fn toggle_mute(target: &AudioTarget) -> Result<()> {
    if target.pids.is_empty() {
        return Ok(());
    }

    unsafe {
        let _com = ComGuard::init_multithreaded()?;
        let manager = default_session_manager()?;
        let session_enumerator = manager.GetSessionEnumerator()?;
        let session_count = session_enumerator.GetCount()?;

        for i in 0..session_count {
            let Ok(session) = session_enumerator.GetSession(i) else {
                continue;
            };
            let Ok(session2) = session.cast::<IAudioSessionControl2>() else {
                continue;
            };
            let Ok(pid) = session2.GetProcessId() else {
                continue;
            };
            if !target.pids.contains(&pid) {
                continue;
            }
            let Ok(simple_volume) = session.cast::<ISimpleAudioVolume>() else {
                continue;
            };

            let new_mute = !simple_volume.GetMute()?.as_bool();
            simple_volume.SetMute(new_mute, std::ptr::null())?;
        }

        Ok(())
    }
}

pub fn get_master_volume() -> Result<f32> {
    unsafe {
        let _com = ComGuard::init_multithreaded()?;
        Ok(default_endpoint_volume()?.GetMasterVolumeLevelScalar()?)
    }
}

pub fn set_master_volume(level: f32) -> Result<()> {
    unsafe {
        let _com = ComGuard::init_multithreaded()?;
        default_endpoint_volume()?.SetMasterVolumeLevelScalar(level, std::ptr::null())?;
        Ok(())
    }
}

pub fn toggle_master_mute() -> Result<()> {
    unsafe {
        let _com = ComGuard::init_multithreaded()?;
        let interface = default_endpoint_volume()?;
        let new_mute = !interface.GetMute()?.as_bool();
        interface.SetMute(new_mute, std::ptr::null())?;
        Ok(())
    }
}

pub fn list_audio_devices() -> Result<Vec<AudioDeviceInfo>> {
    unsafe {
        let _com = ComGuard::init_multithreaded()?;
        let enumerator: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;

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
}

/// Endpoint-ID des aktuellen Standard-Wiedergabegeräts.
pub fn default_device_id() -> Result<Option<String>> {
    unsafe {
        let _com = ComGuard::init_multithreaded()?;
        let enumerator: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
        let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole)?;
        Ok(Some(device.GetId()?.to_string()?))
    }
}

/// Setzt das Standard-Wiedergabegerät. `id` ist eine Endpoint-ID aus
/// [`list_audio_devices`].
pub fn set_default_device(id: &str) -> Result<()> {
    device::set_default_endpoint(id)
}
