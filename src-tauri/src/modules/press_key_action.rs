use crate::action::actions::Action;
use enigo::Direction::Click;
use enigo::{Enigo, Key, Keyboard, Settings};
use log::debug;

#[derive(Debug)]
pub struct PressKeyAction {
    pub key: Key,
}

impl Action for PressKeyAction {
    fn execute(&self) {
        let mut enigo = Enigo::new(&Settings::default()).unwrap();
        let _ = enigo.key(self.key.clone(), Click);
        debug!("Taste {:?} gedrückt!", self.key);
    }
}
