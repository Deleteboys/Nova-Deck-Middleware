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
}

impl Action for AppSwitcherCycleAction {
    fn execute(&self) {
        let mut rt = self.runtime.lock().unwrap();
        if rt.apps.is_empty() {
            return;
        }
        let len = rt.apps.len();
        rt.current_index = (rt.current_index as i32 + self.direction as i32)
            .rem_euclid(len as i32) as usize;

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
