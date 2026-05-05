pub mod action;
mod audio;
mod commands;
mod modules;
mod monitor;
mod protocol;
mod serial;
mod window;

use crate::action::manager::ActionManager;
use crate::protocol::HostToPico;
use serde::Serialize;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;
use std::sync::{mpsc, Arc};
use std::thread;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, RunEvent, WindowEvent,
};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_log::{Target, TargetKind};

#[derive(serde::Serialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
}

pub struct AppState {
    pub serial_tx: Mutex<Option<mpsc::Sender<HostToPico>>>,
    pub is_quitting: Mutex<bool>,
    pub is_device_connected: Arc<AtomicBool>,
    pub action_manager: Arc<Mutex<ActionManager>>,
    pub monitor_slots: Arc<Mutex<[Option<String>; 4]>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (tx, rx) = mpsc::channel::<HostToPico>();

    let action_manager = Arc::new(Mutex::new(ActionManager::new()));
    let manager_for_thread = Arc::clone(&action_manager);
    let is_device_connected = Arc::new(AtomicBool::new(false));
    let is_device_connected_for_thread = Arc::clone(&is_device_connected);
    let monitor_slots = Arc::new(Mutex::new([None, None, None, None]));

    let monitor_slots_for_setup = Arc::clone(&monitor_slots);
    let tx_for_monitor = tx.clone();

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--autostart"]),
        ))
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_log::Builder::new()
            .targets([
                Target::new(TargetKind::Stdout),
                Target::new(TargetKind::Webview),
            ])
            .build())
        .manage(AppState {
            serial_tx: Mutex::new(Some(tx)),
            is_quitting: Mutex::new(false),
            is_device_connected,
            action_manager,
            monitor_slots,
        })
        // 2. Den Command fur das Frontend registrieren
        .invoke_handler(tauri::generate_handler![
            commands::send_to_pico,
            commands::get_connection_status,
            commands::update_mapping,
            commands::remove_mapping,
            commands::get_active_processes,
            commands::sync_mappings,
            commands::check_firmware_update,
            commands::download_and_flash_firmware,
            commands::set_icon_slot,
            commands::update_monitor_mapping,
            commands::set_start_minimized,
            commands::get_start_minimized,
            commands::get_audio_output_devices
        ])
        .setup(move |app| {
            // --- TRAY MENU SETUP ---
            let quit_i = MenuItem::with_id(app, "quit", "Beenden", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "show", "Einstellungen", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

            let app_handle = app.handle().clone();
            monitor::start_monitoring(app_handle.clone(), monitor_slots_for_setup, tx_for_monitor);

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        window::shutdown_app(app);
                    }
                    "show" => {
                        window::show_or_create_main_window(app);
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
                        window::show_or_create_main_window(&app);
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
                let state = window::load_window_state(&app.handle());
                if let Some(s) = &state {
                    window::apply_window_state(&window, s);
                    if s.start_minimized {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                    }
                } else {
                    let _ = window.show();
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
                    window::persist_window_state(&webview_window);
                }

                if window::is_quitting(&window.app_handle()) {
                    return;
                }
            }
            WindowEvent::Moved(_) | WindowEvent::Resized(_) => {
                if let Some(webview_window) = window.app_handle().get_webview_window(window.label())
                {
                    window::persist_window_state(&webview_window);
                }
            }
            WindowEvent::Focused(false) => {
                if let Some(webview_window) = window.app_handle().get_webview_window(window.label())
                {
                    window::persist_window_state(&webview_window);
                }
            }
            _ => {}
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app, event| {
        if let RunEvent::ExitRequested { api, .. } = event {
            if let Some(window) = app.get_webview_window("main") {
                window::persist_window_state(&window);
            }

            let state = app.state::<AppState>();
            let should_exit = state.is_quitting.lock().map(|v| *v).unwrap_or(false);
            if !should_exit {
                api.prevent_exit();
            }
        }
    });
}
