use std::collections::HashMap;
use crate::action::actions::{HardwareTrigger, Action, ButtonEvent};

pub struct ActionManager {
    // Ordnet einem Trigger eine Aktion zu
    mappings: HashMap<HardwareTrigger, Box<dyn Action>>,
}

impl ActionManager {
    pub fn new() -> Self {
        Self {
            mappings: HashMap::new(),
        }
    }

    // Funktion zum Hinzufügen neuer Zuweisungen
    pub fn register(&mut self, trigger: HardwareTrigger, action: Box<dyn Action>) {
        self.mappings.insert(trigger, action);
    }

    // Funktion, die aufgerufen wird, wenn ein Event passiert
    pub fn handle_trigger(&self, trigger: HardwareTrigger) {
        // 1. Suche nach exaktem Match (z.B. Button 1 -> LongPress)
        if let Some(action) = self.mappings.get(&trigger) {
            action.execute();
            return;
        }

        // 2. Deine Fallback-Regel: Wenn kein exakter Match, aber es ist ein Button Event...
        if let HardwareTrigger::Button { id, event: _ } = trigger {
            // ...prüfe, ob es ein ShortPress Mapping für diesen Button gibt.
            let fallback_trigger = HardwareTrigger::Button {
                id,
                event: ButtonEvent::ShortPress
            };

            if let Some(fallback_action) = self.mappings.get(&fallback_trigger) {
                println!("Kein spezifisches Event gefunden. Nutze ShortPress Fallback für Button {}", id);
                fallback_action.execute();
            }
        }
    }
    pub fn unregister(&mut self, trigger: &HardwareTrigger) {
        self.mappings.remove(trigger);
    }
}