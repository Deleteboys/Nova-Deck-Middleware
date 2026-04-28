use crate::action::actions::{ButtonEvent, EncoderEvent, HardwareTrigger};
use crate::protocol::PicoToHost;
use std::time::Instant;

const LONG_PRESS_MS: u128 = 500;
const DOUBLE_PRESS_TIMEOUT_MS: u128 = 250;

pub struct InputTracker {
    button_press_times: [Option<Instant>; 16],
    button_last_release: [Option<Instant>; 16],
    encoder_pushed: [bool; 4],
    // NEU: Speichert, ob der Encoder bewegt wurde, während er gedrückt war
    encoder_moved_while_pushed: [bool; 4],
}

impl InputTracker {
    pub fn new() -> Self {
        Self {
            button_press_times: [None; 16],
            button_last_release: [None; 16],
            encoder_pushed: [false; 4],
            encoder_moved_while_pushed: [false; 4],
        }
    }

    pub fn process_event(&mut self, event: PicoToHost) -> Option<HardwareTrigger> {
        match event {
            PicoToHost::ButtonChanged { id, pressed } => {
                let id_usize = id as usize;
                if pressed {
                    self.button_press_times[id_usize] = Some(Instant::now());
                    if let Some(last_release) = self.button_last_release[id_usize] {
                        if last_release.elapsed().as_millis() < DOUBLE_PRESS_TIMEOUT_MS {
                            self.button_press_times[id_usize] = None;
                            self.button_last_release[id_usize] = None;
                            return Some(HardwareTrigger::Button {
                                id,
                                event: ButtonEvent::DoublePress,
                            });
                        }
                    }
                    None
                } else {
                    if let Some(press_time) = self.button_press_times[id_usize] {
                        let duration = press_time.elapsed().as_millis();
                        self.button_press_times[id_usize] = None;
                        self.button_last_release[id_usize] = Some(Instant::now());

                        if duration >= LONG_PRESS_MS {
                            return Some(HardwareTrigger::Button {
                                id,
                                event: ButtonEvent::LongPress,
                            });
                        } else {
                            return Some(HardwareTrigger::Button {
                                id,
                                event: ButtonEvent::ShortPress,
                            });
                        }
                    }
                    None
                }
            }

            PicoToHost::EncoderChanged { id, pressed } => {
                let idx = id as usize;
                if pressed {
                    self.encoder_pushed[idx] = true;
                    // Reset: Wenn neu gedrückt wird, gab es noch keine Bewegung
                    self.encoder_moved_while_pushed[idx] = false;
                    None
                } else {
                    self.encoder_pushed[idx] = false;

                    // NUR triggern, wenn in der Zwischenzeit NICHT gedreht wurde
                    if !self.encoder_moved_while_pushed[idx] {
                        return Some(HardwareTrigger::Encoder {
                            id,
                            event: EncoderEvent::PushPress,
                        });
                    }
                    None
                }
            }

            PicoToHost::EncoderTurned { id, delta } => {
                let idx = id as usize;
                let is_pushed = self.encoder_pushed[idx];

                if is_pushed {
                    // Markieren, dass der Encoder während des Drückens bewegt wurde
                    self.encoder_moved_while_pushed[idx] = true;
                }

                let event = match (delta > 0, is_pushed) {
                    (true, false) => EncoderEvent::TurnRight,
                    (false, false) => EncoderEvent::TurnLeft,
                    (true, true) => EncoderEvent::PushTurnRight,
                    (false, true) => EncoderEvent::PushTurnLeft,
                };

                Some(HardwareTrigger::Encoder { id, event })
            }
            _ => None,
        }
    }
}
