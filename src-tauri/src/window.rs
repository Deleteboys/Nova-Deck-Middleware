use crate::AppState;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

const WINDOW_STATE_FILE: &str = "window-state.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedWindowState {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub maximized: bool,
    #[serde(default)]
    pub start_minimized: bool,
}

pub fn window_state_path(app: &AppHandle) -> Option<PathBuf> {
    let mut base = app.path().app_data_dir().ok()?;
    if fs::create_dir_all(&base).is_err() {
        return None;
    }
    base.push(WINDOW_STATE_FILE);
    Some(base)
}

pub fn capture_window_state(window: &tauri::WebviewWindow) -> Option<PersistedWindowState> {
    let position = window.outer_position().ok()?;
    let size = window.outer_size().ok()?;

    if window.is_minimized().unwrap_or(false) {
        return None;
    }

    let old_state = load_window_state(&window.app_handle());
    let current_start_minimized = old_state.map(|s| s.start_minimized).unwrap_or(false);

    Some(PersistedWindowState {
        x: position.x,
        y: position.y,
        width: size.width,
        height: size.height,
        maximized: window.is_maximized().unwrap_or(false),
        start_minimized: current_start_minimized,
    })
}

pub fn persist_window_state(window: &tauri::WebviewWindow) {
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

pub fn load_window_state(app: &AppHandle) -> Option<PersistedWindowState> {
    let path = window_state_path(app)?;
    let content = fs::read_to_string(path).ok()?;
    let state: PersistedWindowState = serde_json::from_str(&content).ok()?;

    if state.width == 0 || state.height == 0 || state.x <= -10000 || state.y <= -10000 {
        return None;
    }

    Some(state)
}

pub fn apply_window_state(window: &tauri::WebviewWindow, state: &PersistedWindowState) {
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

pub fn shutdown_app(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        persist_window_state(&window);
    }

    let state = app.state::<AppState>();
    if let Ok(mut quitting) = state.is_quitting.lock() {
        *quitting = true;
    }

    if let Ok(mut tx_guard) = state.serial_tx.lock() {
        tx_guard.take();
    }

    app.exit(0);
}

pub fn show_or_create_main_window(app: &AppHandle) {
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

pub fn is_quitting(app: &AppHandle) -> bool {
    app.state::<AppState>()
        .is_quitting
        .lock()
        .map(|v| *v)
        .unwrap_or(false)
}