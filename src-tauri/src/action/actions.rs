use std::fmt::Debug;

// --- 1. Die Logischen Trigger ---
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
    PushPress, // Der Druck auf den Encoder selbst
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HardwareTrigger {
    Button { id: u8, event: ButtonEvent },
    Encoder { id: u8, event: EncoderEvent },
}

// --- 2. Das Action Trait ---
// Jedes Modul, das du baust, implementiert dieses Trait.
pub trait Action: Send + Sync + Debug {
    fn execute(&self);
}

// Beispiel 1: Enigo (Tastendruck)
use enigo::{Direction::Click, Enigo, Key, Keyboard, Settings};

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
