use crate::AppState;
use crate::action::actions::{ButtonEvent, EncoderEvent, HardwareTrigger};
use crate::protocol::{HostToPico, IconType};
use crate::modules;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use sysinfo::{Disks, ProcessesToUpdate, System};
use tauri::{AppHandle, Emitter, Manager, State};

// --- Datenstrukturen für Mappings ---

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum ActionConfig {
    PressKey { key: String },
    MediaControl { key: String },
    SpotifyVolume { step: i8 },
    ToggleAudio { device1: String, device2: String },
    MasterVolume { step: i8 },
    ToggleAppAudio { process_name: String },
    ToggleMasterMute,
    AppVolume { process_name: String, step: i8 },
    ForegroundVolume { step: i8 },
    ToggleForegroundAudio,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct MappingPayload {
    pub element_id: String,
    pub trigger_type: String,
    pub action_config: ActionConfig,
}

#[derive(Deserialize)]
pub struct UnmapPayload {
    pub element_id: String,
    pub trigger_type: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FirmwareUpdateInfo {
    pub version: String,
    pub download_url: String,
}

// --- Hilfsfunktionen für das Mapping & Parsing ---

fn parse_key(key_str: &str) -> enigo::Key {
    let key_upper = key_str.to_uppercase();

    if key_upper.starts_with('F') {
        if let Ok(num) = key_upper[1..].parse::<u8>() {
            return match num {
                13 => enigo::Key::F13,
                14 => enigo::Key::F14,
                15 => enigo::Key::F15,
                16 => enigo::Key::F16,
                17 => enigo::Key::F17,
                18 => enigo::Key::F18,
                19 => enigo::Key::F19,
                20 => enigo::Key::F20,
                21 => enigo::Key::F21,
                22 => enigo::Key::F22,
                23 => enigo::Key::F23,
                24 => enigo::Key::F24,
                _ => enigo::Key::Return,
            };
        }
    }

    match key_upper.as_str() {
        "A" => enigo::Key::Unicode('a'),
        "MEDIAPLAYPAUSE" => enigo::Key::MediaPlayPause,
        "MEDIANEXT" => enigo::Key::MediaNextTrack,
        "MEDIAPREV" => enigo::Key::MediaPrevTrack,
        "MEDIAMUTE" => enigo::Key::VolumeMute,
        _ => enigo::Key::Return,
    }
}

fn create_action(
    config: ActionConfig,
    tx: mpsc::Sender<HostToPico>,
) -> Box<dyn crate::action::actions::Action> {
    match config {
        ActionConfig::PressKey { key } => Box::new(modules::press_key_action::PressKeyAction {
            key: parse_key(&key),
        }),
        ActionConfig::MediaControl { key } => Box::new(modules::press_key_action::PressKeyAction {
            key: parse_key(&key),
        }),
        ActionConfig::SpotifyVolume { step } => {
            Box::new(modules::spotify_volume::SpotifyVolumeAction { step })
        }
        ActionConfig::MasterVolume { step } => {
            Box::new(modules::master_volume::MasterVolumeAction { step, tx })
        }
        ActionConfig::ToggleAppAudio { process_name } => {
            Box::new(modules::toggle_app_audio::ToggleAppAudioAction { process_name })
        }
        ActionConfig::ToggleMasterMute => {
            Box::new(modules::toggle_master_mute::ToggleMasterMuteAction {})
        }
        ActionConfig::AppVolume { process_name, step } => {
            Box::new(modules::app_volume_action::AppVolumeAction {
                process_name,
                step,
                tx,
            })
        }
        ActionConfig::ForegroundVolume { step } => {
            Box::new(modules::foreground_volume::ForegroundVolumeAction { step, tx })
        }
        ActionConfig::ToggleForegroundAudio => {
            Box::new(modules::toggle_foreground_audio::ToggleForegroundAudioAction {})
        }
        _ => {
            println!("WARNUNG: Aktion noch nicht implementiert!");
            Box::new(modules::press_key_action::PressKeyAction {
                key: enigo::Key::F14,
            })
        }
    }
}

fn parse_element_id(element_id: &str) -> Result<(bool, u8), String> {
    let is_button = element_id.starts_with("btn-");
    let is_encoder = element_id.starts_with("enc-");

    if !is_button && !is_encoder {
        return Err(format!("Unknown element_id: {element_id}"));
    }

    let id_str = element_id.replace("btn-", "").replace("enc-", "");
    let id = id_str
        .parse::<u8>()
        .map_err(|_| format!("Invalid element_id: {element_id}"))?;

    Ok((is_button, id))
}

fn trigger_from_payload(element_id: &str, trigger_type: &str) -> Result<HardwareTrigger, String> {
    let (is_button, id) = parse_element_id(element_id)?;

    if is_button {
        let event = match trigger_type {
            "ShortPress" => ButtonEvent::ShortPress,
            "LongPress" => ButtonEvent::LongPress,
            "DoublePress" => ButtonEvent::DoublePress,
            _ => return Err(format!("Unknown button trigger: {trigger_type}")),
        };
        return Ok(HardwareTrigger::Button { id, event });
    }

    let event = match trigger_type {
        "TurnLeft" => EncoderEvent::TurnLeft,
        "TurnRight" => EncoderEvent::TurnRight,
        "PushTurnLeft" => EncoderEvent::PushTurnLeft,
        "PushTurnRight" => EncoderEvent::PushTurnRight,
        "PushPress" => EncoderEvent::PushPress,
        _ => return Err(format!("Unknown encoder trigger: {trigger_type}")),
    };
    Ok(HardwareTrigger::Encoder { id, event })
}

fn parse_icon(icon_str: &str) -> IconType {
    match icon_str.to_uppercase().as_str() {
        "MASTER" => IconType::Master,
        "SPOTIFY" => IconType::Spotify,
        "DISCORD" => IconType::Discord,
        "BROWSER" => IconType::Browser,
        "MIC" => IconType::Mic,
        "CAMERA" => IconType::Camera,
        "PLAY_PAUSE" => IconType::PlayPause,
        "LIGHT" => IconType::Light,
        "ACTIVE_WINDOW" => IconType::ActiveWindow,
        _ => IconType::None,
    }
}

// --- Tauri Commands ---

#[tauri::command]
pub fn send_to_pico(state: State<AppState>, command: HostToPico) -> Result<(), String> {
    let tx_guard = state.serial_tx.lock().unwrap();
    if let Some(tx) = tx_guard.as_ref() {
        tx.send(command).map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("Keine Verbindung zum seriellen Thread".into())
    }
}

#[tauri::command]
pub fn get_connection_status(state: State<AppState>) -> bool {
    state.is_device_connected.load(std::sync::atomic::Ordering::Relaxed)
}

#[tauri::command]
pub fn update_mapping(state: State<AppState>, payload: MappingPayload) -> Result<(), String> {
    let tx = state
        .serial_tx
        .lock()
        .unwrap()
        .clone()
        .ok_or("Keine serielle Verbindung verfügbar")?;

    let trigger = trigger_from_payload(&payload.element_id, &payload.trigger_type)?;
    let action = create_action(payload.action_config, tx);

    if let Ok(mut manager) = state.action_manager.lock() {
        manager.register(trigger, action);
    }
    Ok(())
}

#[tauri::command]
pub fn remove_mapping(state: State<AppState>, payload: UnmapPayload) -> Result<(), String> {
    let trigger = trigger_from_payload(&payload.element_id, &payload.trigger_type)?;

    if let Ok(mut manager) = state.action_manager.lock() {
        manager.unregister(&trigger);
    }
    Ok(())
}

#[tauri::command]
pub fn sync_mappings(state: State<AppState>, mappings: Vec<MappingPayload>) -> Result<(), String> {
    let tx = state
        .serial_tx
        .lock()
        .unwrap()
        .clone()
        .ok_or("Keine serielle Verbindung verfügbar")?;

    if let Ok(mut manager) = state.action_manager.lock() {
        manager.clear();
        for payload in mappings {
            if let Ok(trigger) = trigger_from_payload(&payload.element_id, &payload.trigger_type) {
                let action = create_action(payload.action_config.clone(), tx.clone());
                manager.register(trigger, action);
            }
        }
    }
    Ok(())
}

#[tauri::command]
pub fn set_icon_slot(state: State<AppState>, slot: u8, icon: String) -> Result<(), String> {
    let icon_enum = parse_icon(&icon);
    let command = HostToPico::SetIconSlot { slot, icon: icon_enum };

    let tx_guard = state.serial_tx.lock().unwrap();
    if let Some(tx) = tx_guard.as_ref() {
        tx.send(command).map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("Keine Verbindung zum seriellen Thread".into())
    }
}

#[tauri::command]
pub fn get_active_processes() -> Vec<String> {
    let mut sys = System::new_all();
    sys.refresh_processes(ProcessesToUpdate::All, true);

    let mut names: Vec<String> = sys
        .processes()
        .values()
        .map(|p| p.name().to_string_lossy().into_owned())
        .collect();

    names.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
    names.dedup();
    names
}

#[tauri::command]
pub async fn check_firmware_update() -> Result<Option<FirmwareUpdateInfo>, String> {
    let client = reqwest::Client::builder()
        .user_agent("StreamDeck-Middleware")
        .build()
        .map_err(|e| e.to_string())?;

    let url = "https://api.github.com/repos/Deleteboys/Stream-Deck-Firmeware/releases/latest";
    let response = client.get(url).send().await.map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Ok(None);
    }

    let json: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
    let version = json["tag_name"].as_str().unwrap_or("").to_string();

    let download_url = json["assets"].as_array()
        .and_then(|assets| {
            assets.iter().find(|a| a["name"].as_str().unwrap_or("").ends_with(".uf2"))
        })
        .and_then(|asset| asset["browser_download_url"].as_str())
        .map(|s| s.to_string());

    if let Some(url) = download_url {
        Ok(Some(FirmwareUpdateInfo { version, download_url: url }))
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub async fn download_and_flash_firmware(app: AppHandle, download_url: String) -> Result<(), String> {
    let _ = app.emit("fw-status", "Lade Firmware herunter...");

    let response = reqwest::get(&download_url).await.map_err(|e| e.to_string())?;
    let bytes = response.bytes().await.map_err(|e| e.to_string())?;

    let temp_dir = std::env::temp_dir();
    let fw_path = temp_dir.join("update.uf2");
    fs::write(&fw_path, bytes).map_err(|e| e.to_string())?;

    let _ = app.emit("fw-status", "Warte auf Bootloader-Laufwerk...");

    let mut disks = Disks::new();
    let mut pico_mount_point: Option<PathBuf> = None;

    for _ in 0..30 {
        disks.refresh(true);
        for disk in &disks {
            if disk.name().to_string_lossy().contains("RPI-RP2") {
                pico_mount_point = Some(disk.mount_point().to_path_buf());
                break;
            }
        }
        if pico_mount_point.is_some() { break; }
        thread::sleep(Duration::from_millis(500));
    }

    let dest_dir = pico_mount_point.ok_or("Pico Laufwerk nicht gefunden.")?;
    let dest_file = dest_dir.join("update.uf2");

    let _ = app.emit("fw-status", "Kopiere Firmware...");
    fs::copy(&fw_path, &dest_file).map_err(|e| e.to_string())?;
    let _ = fs::remove_file(fw_path);

    Ok(())
}

#[tauri::command]
pub fn update_monitor_mapping(state: State<AppState>, slot: u8, process_name: String) {
    if slot < 4 {
        let mut slots = state.monitor_slots.lock().unwrap();
        slots[slot as usize] = if process_name.is_empty() { None } else { Some(process_name) };
    }
}

#[tauri::command]
pub fn set_start_minimized(app: AppHandle, value: bool) {
    if let Some(_window) = app.get_webview_window("main") {
        let mut state = crate::window::load_window_state(&app).unwrap_or(crate::window::PersistedWindowState {
            x: 100, y: 100, width: 800, height: 600, maximized: false, start_minimized: value
        });
        state.start_minimized = value;

        if let Some(path) = crate::window::window_state_path(&app) {
            if let Ok(serialized) = serde_json::to_string(&state) {
                let _ = fs::write(path, serialized);
            }
        }
    }
}

#[tauri::command]
pub fn get_start_minimized(app: AppHandle) -> bool {
    crate::window::load_window_state(&app).map(|s| s.start_minimized).unwrap_or(false)
}