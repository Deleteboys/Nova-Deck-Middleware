use crate::action::actions::Action;
use enigo::Direction::Click;
use enigo::{Enigo, Key, Keyboard, Settings};

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
