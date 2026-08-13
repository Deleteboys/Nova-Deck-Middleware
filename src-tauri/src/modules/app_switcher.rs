use crate::action::actions::Action;
use crate::protocol::{HostToPico, IconType};
use std::sync::{mpsc, Arc, Mutex};

pub fn parse_icon_str(icon: &str) -> IconType {
    match icon.to_uppercase().as_str() {
        "MASTER" => IconType::Master,
        "SPOTIFY" => IconType::Spotify,
        "DISCORD" => IconType::Discord,
        "BROWSER" => IconType::Browser,
        "MIC" => IconType::Mic,
        "CAMERA" => IconType::Camera,
        "PLAY_PAUSE" => IconType::PlayPause,
        "LIGHT" => IconType::Light,
        "ACTIVE_WINDOW" => IconType::ActiveWindow,
        "JELLYFIN" => IconType::Jellyfin,
        "HOME" => IconType::Home,
        _ => IconType::None,
    }
}

#[derive(Debug, Clone, Default)]
pub struct AppEntry {
    pub process_name: String,
    pub icon: Option<String>,
}

#[derive(Debug, Default)]
pub struct AppSwitcherRuntime {
    pub apps: Vec<AppEntry>,
    pub shared_icon: Option<String>,
    pub current_index: usize,
}

#[derive(Debug)]
pub struct AppSwitcherCycleAction {
    pub direction: i8,
    pub encoder_slot: Option<u8>,
    pub runtime: Arc<Mutex<AppSwitcherRuntime>>,
    pub tx: mpsc::Sender<HostToPico>,
    pub monitor_slots: Option<Arc<Mutex<[Option<String>; 4]>>>,
    /// Überspringt beim Rotieren alle Apps ohne offene Audio-Session
    pub hide_closed_apps: bool,
}

/// Prüft, ob zu einem Prozessnamen eine der offenen Audio-Sessions passt.
fn is_app_open(process_name: &str, open_sessions: &[String]) -> bool {
    if process_name.is_empty() {
        return false;
    }
    let needle = process_name.to_lowercase();
    open_sessions.iter().any(|id| id.contains(&needle))
}

/// Sucht ab `current` in Richtung `direction` den nächsten Eintrag mit offener
/// Audio-Session. Ist keine der Apps offen, wird None zurückgegeben.
fn next_open_index(
    apps: &[AppEntry],
    current: usize,
    direction: i8,
    open_sessions: &[String],
) -> Option<usize> {
    let len = apps.len();
    let step: i32 = if direction < 0 { -1 } else { 1 };

    (1..=len)
        .map(|offset| (current as i32 + step * offset as i32).rem_euclid(len as i32) as usize)
        .find(|&index| is_app_open(&apps[index].process_name, open_sessions))
}

impl Action for AppSwitcherCycleAction {
    fn execute(&self) {
        // Vor dem Lock ermitteln, damit die COM-Abfrage die Runtime nicht blockiert
        let open_sessions = if self.hide_closed_apps {
            Some(unsafe { crate::audio::list_open_session_identifiers() }.unwrap_or_default())
        } else {
            None
        };

        let mut rt = self.runtime.lock().unwrap();
        if rt.apps.is_empty() {
            return;
        }
        let len = rt.apps.len();
        if rt.current_index >= len {
            rt.current_index = 0;
        }

        let next_index = match &open_sessions {
            Some(open) => next_open_index(&rt.apps, rt.current_index, self.direction, open),
            None => Some(
                (rt.current_index as i32 + self.direction as i32).rem_euclid(len as i32) as usize,
            ),
        };

        // Keine der Apps ist offen -> Auswahl unverändert lassen
        let Some(next_index) = next_index else {
            return;
        };
        rt.current_index = next_index;

        if let Some(slot) = self.encoder_slot {
            let icon_str = rt.apps[rt.current_index]
                .icon
                .clone()
                .or_else(|| rt.shared_icon.clone())
                .unwrap_or_default();
            let icon = parse_icon_str(&icon_str);
            let _ = self.tx.send(HostToPico::SetIconSlot { slot, icon });

            if let Some(monitor_slots) = &self.monitor_slots {
                if let Ok(mut slots) = monitor_slots.lock() {
                    let process = rt.apps[rt.current_index].process_name.clone();
                    slots[slot as usize] = if process.is_empty() { None } else { Some(process) };
                }
            }
        }
    }
}
