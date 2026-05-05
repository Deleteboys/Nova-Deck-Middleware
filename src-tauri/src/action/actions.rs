use std::fmt::Debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ButtonEvent {
    ShortPress,
    LongPress,
    DoublePress,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EncoderEvent {
    TurnLeft,
    TurnRight,
    PushTurnLeft,
    PushTurnRight,
    PushPress,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HardwareTrigger {
    Button { id: u8, event: ButtonEvent },
    Encoder { id: u8, event: EncoderEvent },
}

pub trait Action: Send + Sync + Debug {
    fn execute(&self);
}

use enigo::{Direction::Click, Enigo, Key, Keyboard, Settings};
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
