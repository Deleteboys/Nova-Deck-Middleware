use crate::action::actions::Action;
use enigo::Direction::Click;
use enigo::{Enigo, Key, Keyboard, Settings};
use log::{debug, error};

#[derive(Debug)]
pub struct PressKeyAction {
    pub key: Key,
}

impl Action for PressKeyAction {
    fn execute(&self) {
        // Unter Wayland braucht enigo das virtual-keyboard-Protokoll; fehlt es,
        // darf das den Serial-Thread nicht mit einem Panic reißen.
        let mut enigo = match Enigo::new(&Settings::default()) {
            Ok(enigo) => enigo,
            Err(e) => {
                error!("Tastatursimulation nicht verfügbar: {}", e);
                return;
            }
        };
        let _ = enigo.key(self.key.clone(), Click);
        debug!("Taste {:?} gedrückt!", self.key);
    }
}
