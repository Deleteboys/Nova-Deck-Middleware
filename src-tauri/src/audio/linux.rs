use super::AudioDeviceInfo;

pub unsafe fn get_monitor_statuses(
    _slots: &[Option<String>; 4],
) -> Result<[Option<(f32, bool)>; 4], String> {
    Ok([None, None, None, None])
}

pub unsafe fn list_open_session_identifiers() -> Result<Vec<String>, String> {
    Ok(Vec::new())
}

pub unsafe fn adjust_volume_for_pids(
    _target_pids: &[u32],
    _step: i8,
    _snap: bool,
) -> Result<bool, String> {
    Ok(false)
}

pub unsafe fn toggle_mute_for_pids(_target_pids: &[u32]) -> Result<(), String> {
    Ok(())
}

pub unsafe fn get_master_volume() -> Result<f32, String> {
    // Schneller Readout über wpctl (PipeWire)
    let output = std::process::Command::new("wpctl")
        .args(["get-volume", "@DEFAULT_AUDIO_SINK@"])
        .output()
        .map_err(|e| e.to_string())?;

    let text = String::from_utf8_lossy(&output.stdout);
    // Beispielausgabe: "Volume: 0.65"
    let vol = text
        .split_whitespace()
        .nth(1)
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or(0.0);

    Ok(vol)
}

pub fn toggle_master_mute() -> Result<(), String> {
    let status = std::process::Command::new("wpctl")
        .args(["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"])
        .status()
        .map_err(|e| format!("Fehler beim Ausführen von wpctl: {e}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("wpctl beendete mit Exit-Code: {:?}", status.code()))
    }
}

pub fn set_master_volume(level: f32) -> Result<(), String> {
    let clamped = level.clamp(0.0, 1.0);

    let status = std::process::Command::new("wpctl")
        .args(["set-volume", "@DEFAULT_AUDIO_SINK@", &format!("{:.2}", clamped)])
        .status()
        .map_err(|e| format!("Fehler beim Ausführen von wpctl: {e}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("wpctl beendete mit Exit-Code: {:?}", status.code()))
    }
}

pub unsafe fn get_volume_by_process_name(_name: &str) -> Result<Option<f32>, String> {
    Ok(None)
}

pub unsafe fn get_master_status() -> Result<(f32, bool), String> {
    let vol = get_master_volume().unwrap_or(0.0);
    Ok((vol, false))
}

pub unsafe fn get_process_status(_name: &str) -> Result<Option<(f32, bool)>, String> {
    Ok(None)
}

pub unsafe fn get_foreground_status() -> Result<Option<(f32, bool)>, String> {
    // Unter Wayland/Hyprland gibt es kein GetForegroundWindow().
    // Dies erfordert Abfragen über 'hyprctl activewindow -j'
    Ok(None)
}

pub unsafe fn list_audio_devices() -> Result<Vec<AudioDeviceInfo>, String> {
    Ok(vec![AudioDeviceInfo {
        id: "default".into(),
        name: "Default Linux Audio Device".into(),
    }])
}

pub fn get_active_audio_pids() -> std::collections::HashSet<u32> {
    std::collections::HashSet::new() // Stub für Linux
}