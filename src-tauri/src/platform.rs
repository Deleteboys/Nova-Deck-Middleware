#[cfg(target_os = "windows")]
pub fn get_active_window_pid() -> u32 {
    use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_invalid() {
            return 0;
        }
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        pid
    }
}

#[cfg(target_os = "linux")]
pub fn get_active_window_pid() -> u32 {
    // Unter Hyprland über hyprctl JSON abfragen
    let output = std::process::Command::new("hyprctl")
        .args(["activewindow", "-j"])
        .output();

    if let Ok(out) = output {
        if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&out.stdout) {
            return val.get("pid").and_then(|p| p.as_u64()).unwrap_or(0) as u32;
        }
    }
    0
}