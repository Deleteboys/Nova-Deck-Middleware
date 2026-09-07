//! Aktives Fenster über Win32.

use crate::platform::ActiveWindow;
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

/// PID des Vordergrundfensters, oder `None`, wenn gerade keines fokussiert ist.
pub fn foreground_process_id() -> Option<u32> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_invalid() {
            return None;
        }

        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));

        (pid != 0).then_some(pid)
    }
}

/// Auf Windows genügt die PID – jede WASAPI-Session trägt die PID ihres
/// Fensterprozesses, deshalb bleibt `app_id` leer.
pub fn active_window() -> Option<ActiveWindow> {
    Some(ActiveWindow {
        pid: Some(foreground_process_id()?),
        app_id: None,
    })
}
