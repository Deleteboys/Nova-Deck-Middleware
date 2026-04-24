use enigo::{Enigo, Key, Keyboard, Settings};
use enigo::Direction::Click;
use crate::action::actions::Action;

#[derive(Debug)]
pub struct PressKeyAction {
    pub key: Key,
}

impl Action for PressKeyAction {
    fn execute(&self) {
        let mut enigo = Enigo::new(&Settings::default()).unwrap();
        let _ = enigo.key(self.key.clone(), Click);
        println!("Taste {:?} gedrückt!", self.key);
    }
}