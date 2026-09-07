use crate::action::actions::Action;
use crate::platform::shell::run_special_shortcut;
use enigo::Direction::{Click, Press, Release};
use enigo::{Enigo, Key, Keyboard, Settings};
use log::{debug, error, warn};

#[derive(Debug)]
pub struct CustomMacroAction {
    pub keys_string: String,
}

impl Action for CustomMacroAction {
    fn execute(&self) {
        // Manche Makros lassen sich nicht sinnvoll tippen (Sperrbildschirm,
        // Systemmonitor) – die übernimmt das Plattform-Backend direkt.
        if run_special_shortcut(&self.keys_string) {
            return;
        }

        let mut enigo = match Enigo::new(&Settings::default()) {
            Ok(enigo) => enigo,
            Err(e) => {
                error!("Tastatursimulation nicht verfügbar: {}", e);
                return;
            }
        };

        let parts: Vec<&str> = self.keys_string.split(" + ").collect();

        let mut modifiers = Vec::new();
        let mut main_key = None;

        // Tasten zuordnen
        for part in parts {
            match parse_key_string(part) {
                Some(key) => {
                    match key {
                        Key::Control | Key::Shift | Key::Alt | Key::Meta => modifiers.push(key),
                        _ => main_key = Some(key), // Alles andere ist die Haupttaste
                    }
                }
                None => warn!("Unbekannte Taste im Makro: {}", part),
            }
        }

        for mod_key in &modifiers {
            let _ = enigo.key(mod_key.clone(), Press);
        }

        if let Some(key) = main_key {
            let _ = enigo.key(key, Click);
        }

        for mod_key in modifiers.iter().rev() {
            let _ = enigo.key(mod_key.clone(), Release);
        }

        debug!("Makro ausgeführt: {}", self.keys_string);
    }
}

/// Rollen-/Scroll-Lock heißt in enigo je Plattform anders.
#[cfg(windows)]
const SCROLL_LOCK: Key = Key::Scroll;
#[cfg(target_os = "linux")]
const SCROLL_LOCK: Key = Key::ScrollLock;

/// Kontextmenü-Taste. Unter Linux hat enigo dafür keine benannte Variante,
/// deshalb direkt das X11-Keysym `XK_Menu`.
#[cfg(windows)]
const CONTEXT_MENU: Key = Key::Apps;
#[cfg(target_os = "linux")]
const CONTEXT_MENU: Key = Key::Other(0xff67);

fn parse_key_string(s: &str) -> Option<Key> {
    match s {
        "Ctrl" => Some(Key::Control),
        "Shift" => Some(Key::Shift),
        "Alt" => Some(Key::Alt),
        "Win" => Some(Key::Meta),
        "SPACE" => Some(Key::Space),
        "ENTER" => Some(Key::Return),
        "ESCAPE" => Some(Key::Escape),
        "TAB" => Some(Key::Tab),
        "BACKSPACE" => Some(Key::Backspace),
        "ARROWUP" => Some(Key::UpArrow),
        "ARROWDOWN" => Some(Key::DownArrow),
        "ARROWLEFT" => Some(Key::LeftArrow),
        "ARROWRIGHT" => Some(Key::RightArrow),
        "PrintScreen" => Some(Key::PrintScr),
        "ScrollLock" => Some(SCROLL_LOCK),
        "Pause" => Some(Key::Pause),
        "Insert" => Some(Key::Insert),
        "ContextMenu" => Some(CONTEXT_MENU),
        _ => {
            if s.len() == 1 {
                let c = s.chars().next().unwrap().to_ascii_lowercase();
                Some(Key::Unicode(c))
            } else {
                None
            }
        }
    }
}
