// Modul-Deklarationen einbinden
pub mod action;
mod audio;
mod modules;
mod protocol;
mod serial;
// Das bindet den Ordner "action" über die mod.rs ein

use crate::action::actions::{ButtonEvent, EncoderEvent, HardwareTrigger};
use crate::action::manager::ActionManager;
use crate::protocol::{HostToPico, IconType};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::sync::{mpsc, Arc};
use std::thread;
use sysinfo::{ProcessesToUpdate, System};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, RunEvent, State, WindowEvent,
};

const WINDOW_STATE_FILE: &str = "window-state.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedWindowState {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    maximized: bool,
}

// State, um vom UI Befehle an den Hintergrund-Thread zu schicken
struct AppState {
    serial_tx: Mutex<Option<mpsc::Sender<HostToPico>>>,
    is_quitting: Mutex<bool>,
    is_device_connected: Arc<AtomicBool>,
    action_manager: Arc<Mutex<ActionManager>>,
}

// Der Command, den dein JavaScript aufruft
#[tauri::command]
fn send_to_pico(state: State<AppState>, command: HostToPico) -> Result<(), String> {
    let tx_guard = state.serial_tx.lock().unwrap();
    if let Some(tx) = tx_guard.as_ref() {
        tx.send(command).map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("Keine Verbindung zum seriellen Thread".into())
    }
}

#[tauri::command]
fn get_connection_status(state: State<AppState>) -> bool {
    state.is_device_connected.load(Ordering::Relaxed)
}

fn is_quitting(app: &AppHandle) -> bool {
    app.state::<AppState>()
        .is_quitting
        .lock()
        .map(|v| *v)
        .unwrap_or(false)
}

fn window_state_path(app: &AppHandle) -> Option<PathBuf> {
    let mut base = app.path().app_data_dir().ok()?;
    if fs::create_dir_all(&base).is_err() {
        return None;
    }
    base.push(WINDOW_STATE_FILE);
    Some(base)
}

fn capture_window_state(window: &tauri::WebviewWindow) -> Option<PersistedWindowState> {
    let position = window.outer_position().ok()?;
    let size = window.outer_size().ok()?;

    Some(PersistedWindowState {
        x: position.x,
        y: position.y,
        width: size.width,
        height: size.height,
        maximized: window.is_maximized().unwrap_or(false),
    })
}

fn persist_window_state(window: &tauri::WebviewWindow) {
    let Some(state) = capture_window_state(window) else {
        return;
    };
    let Some(path) = window_state_path(&window.app_handle()) else {
        return;
    };

    if let Ok(serialized) = serde_json::to_string(&state) {
        let _ = fs::write(path, serialized);
    }
}

fn load_window_state(app: &AppHandle) -> Option<PersistedWindowState> {
    let path = window_state_path(app)?;
    let content = fs::read_to_string(path).ok()?;
    let state: PersistedWindowState = serde_json::from_str(&content).ok()?;

    if state.width == 0 || state.height == 0 {
        return None;
    }

    Some(state)
}

fn apply_window_state(window: &tauri::WebviewWindow, state: &PersistedWindowState) {
    if state.maximized {
        let _ = window.maximize();
        return;
    }

    let _ = window.unmaximize();
    let _ = window.set_size(tauri::Size::Physical(tauri::PhysicalSize::new(
        state.width,
        state.height,
    )));
    let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition::new(
        state.x, state.y,
    )));
}

fn shutdown_app(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        persist_window_state(&window);
    }

    let state = app.state::<AppState>();
    if let Ok(mut quitting) = state.is_quitting.lock() {
        *quitting = true;
    }

    // Sender droppen, damit der Serial-Thread uber "Disconnected" sauber aussteigt.
    if let Ok(mut tx_guard) = state.serial_tx.lock() {
        tx_guard.take();
    }

    app.exit(0);
}

fn show_or_create_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    } else {
        let window = tauri::WebviewWindowBuilder::new(
            app,
            "main",
            tauri::WebviewUrl::App("index.html".into()),
        )
        .title("Mein Programm")
        .build();

        if let Ok(window) = window {
            if let Some(state) = load_window_state(app) {
                apply_window_state(&window, &state);
            }
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Channel fur den seriellen Thread erstellen
    let (tx, rx) = mpsc::channel::<HostToPico>();

    let action_manager = Arc::new(Mutex::new(ActionManager::new()));
    let manager_for_thread = Arc::clone(&action_manager);
    let is_device_connected = Arc::new(AtomicBool::new(false));
    let is_device_connected_for_thread = Arc::clone(&is_device_connected);

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        // 1. AppState registrieren, damit `send_to_pico` darauf zugreifen kann
        .manage(AppState {
            serial_tx: Mutex::new(Some(tx)),
            is_quitting: Mutex::new(false),
            is_device_connected,
            action_manager,
        })
        // 2. Den Command fur das Frontend registrieren
        .invoke_handler(tauri::generate_handler![
            send_to_pico,
            get_connection_status,
            update_mapping,
            remove_mapping,
            get_active_processes,
            sync_mappings,
            set_icon_slot
        ])
        .setup(move |app| {
            // --- TRAY MENU SETUP ---
            let quit_i = MenuItem::with_id(app, "quit", "Beenden", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "show", "Einstellungen", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        shutdown_app(app);
                    }
                    "show" => {
                        show_or_create_main_window(app);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: tauri::tray::MouseButton::Left,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        show_or_create_main_window(&app);
                    }
                })
                .build(app)?;

            // --- SERIELLER THREAD SETUP ---
            let app_handle = app.handle().clone();
            thread::spawn(move || {
                serial::start_serial_thread(
                    app_handle,
                    rx,
                    manager_for_thread,
                    is_device_connected_for_thread,
                );
            });

            // --- FENSTER LOGIK ---
            if let Some(window) = app.get_webview_window("main") {
                if let Some(state) = load_window_state(&app.handle()) {
                    apply_window_state(&window, &state);
                }
            }

            let should_show_gui = true; // Placeholder fur Settings
            if !should_show_gui {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }

            Ok(())
        })
        .on_window_event(|window, event| match event {
            WindowEvent::CloseRequested { .. } => {
                if let Some(webview_window) = window.app_handle().get_webview_window(window.label())
                {
                    persist_window_state(&webview_window);
                }

                if is_quitting(&window.app_handle()) {
                    return;
                }
                // Nicht nur verstecken: WebView wirklich schließen, damit keine UI-Ressourcen mehr laufen.
                // ExitRequested wird unten bereits abgefangen, sodass die Tray-App weiterlebt.
            }
            WindowEvent::Moved(_) | WindowEvent::Resized(_) => {
                if let Some(webview_window) = window.app_handle().get_webview_window(window.label())
                {
                    persist_window_state(&webview_window);
                }
            }
            WindowEvent::Focused(false) => {
                if let Some(webview_window) = window.app_handle().get_webview_window(window.label())
                {
                    persist_window_state(&webview_window);
                }
            }
            _ => {}
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app, event| {
        if let RunEvent::ExitRequested { api, .. } = event {
            if let Some(window) = app.get_webview_window("main") {
                persist_window_state(&window);
            }

            let state = app.state::<AppState>();
            let should_exit = state.is_quitting.lock().map(|v| *v).unwrap_or(false);
            if !should_exit {
                api.prevent_exit();
            }
        }
    });
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(tag = "type")] // <-- Das ist die Magie!
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
    // ... hier kommen alle deine zukünftigen Module rein
}

// Deine Payload ändert sich: Wir erwarten jetzt das ActionConfig Enum
#[derive(Deserialize, Serialize, Clone)]
pub struct MappingPayload {
    pub element_id: String,
    pub trigger_type: String,
    pub action_config: ActionConfig, // <-- Statt action_id als String
}

#[derive(serde::Deserialize)]
pub struct UnmapPayload {
    pub element_id: String,
    pub trigger_type: String,
}

// Hilfsfunktion: Wandelt String in Enigo-Key um
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
                _ => enigo::Key::Return, // Fallback
            };
        }
    }

    match key_upper.to_uppercase().as_str() {
        "A" => enigo::Key::Unicode('a'),
        "MEDIAPLAYPAUSE" => enigo::Key::MediaPlayPause,
        "MEDIANEXT" => enigo::Key::MediaNextTrack,
        "MEDIAPREV" => enigo::Key::MediaPrevTrack,
        "MEDIAMUTE" => enigo::Key::VolumeMute,
        _ => enigo::Key::Return,
    }
}

// Factory, die das Config-Enum in ausführbaren Code (Action Trait) verwandelt
fn create_action(
    config: ActionConfig,
    tx: mpsc::Sender<HostToPico>,
) -> Box<dyn action::actions::Action> {
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
        // ActionConfig::SpotifyVolume { volume } => {
        //     Box::new(crate::modules::spotify_action::SetSpotifyVolumeAction {
        //         volume
        //     })
        // }
        // ActionConfig::ToggleAudio { device1, device2 } => {
        //     Box::new(crate::modules::audio_action::ToggleAudioAction {
        //         device1, device2
        //     })
        // }
        _ => {
            println!("WARNUNG: Aktion noch nicht implementiert!");
            Box::new(modules::press_key_action::PressKeyAction {
                key: enigo::Key::F14,
            })
        }
    }
}

#[tauri::command]
fn update_mapping(state: State<AppState>, payload: MappingPayload) -> Result<(), String> {
    // 1. Hole den Sender aus dem State
    let tx = state
        .serial_tx
        .lock()
        .unwrap()
        .clone()
        .ok_or("Keine serielle Verbindung verfügbar")?;

    let trigger = trigger_from_payload(&payload.element_id, &payload.trigger_type)?;

    // 2. Übergebe ihn an create_action
    let action = create_action(payload.action_config, tx);

    if let Ok(mut manager) = state.action_manager.lock() {
        manager.register(trigger, action);
    }
    Ok(())
}

#[tauri::command]
fn remove_mapping(state: State<AppState>, payload: UnmapPayload) -> Result<(), String> {
    // Nutze auch hier die vollständige Hilfsfunktion
    let trigger = trigger_from_payload(&payload.element_id, &payload.trigger_type)?;

    // Mapping aus dem Manager löschen
    if let Ok(mut manager) = state.action_manager.lock() {
        manager.unregister(&trigger);
        println!(
            "Aktion entfernt von: {} ({})",
            payload.element_id, payload.trigger_type
        );
    }

    Ok(())
}

#[tauri::command]
fn sync_mappings(state: State<AppState>, mappings: Vec<MappingPayload>) -> Result<(), String> {
    // 1. Hole den Sender
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
                // 2. Sender mitgeben (wir müssen hier tx.clone() machen, falls wir in der Schleife sind)
                let action = create_action(payload.action_config.clone(), tx.clone());
                manager.register(trigger, action);
            }
        }
    }

    Ok(())
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

#[derive(serde::Serialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
}

fn parse_icon(icon_str: &str) -> IconType {
    match icon_str.to_uppercase().as_str() {
        "MASTER" => IconType::Master,
        "SPOTIFY" => IconType::Spotify,
        "DISCORD" => IconType::Discord,
        "BROWSER" => IconType::Browser,
        _ => IconType::None,
    }
}

#[tauri::command]
fn set_icon_slot(state: State<AppState>, slot: u8, icon: String) -> Result<(), String> {
    let icon_enum = parse_icon(&icon);
    let command = HostToPico::SetIconSlot {
        slot,
        icon: icon_enum,
    };

    let tx_guard = state.serial_tx.lock().unwrap();
    if let Some(tx) = tx_guard.as_ref() {
        tx.send(command).map_err(|e| e.to_string())?;
        println!("Icon für Slot {} auf {:?} gesetzt", slot, icon_enum); // Fürs Debugging
        Ok(())
    } else {
        Err("Keine Verbindung zum seriellen Thread".into())
    }
}

#[tauri::command]
fn get_active_processes() -> Vec<String> {
    let mut sys = System::new_all();
    sys.refresh_processes(ProcessesToUpdate::All, true);

    let mut names: Vec<String> = sys
        .processes()
        .values()
        .map(|p| p.name().to_string_lossy().into_owned())
        .collect();

    // Alphabetisch sortieren und dann alle aufeinanderfolgenden Duplikate löschen!
    names.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
    names.dedup();

    names
}
