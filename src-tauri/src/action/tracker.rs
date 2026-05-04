use crate::action::actions::{ButtonEvent, EncoderEvent, HardwareTrigger};
use crate::protocol::PicoToHost;
use std::time::Instant;

const LONG_PRESS_MS: u128 = 500;
const DOUBLE_PRESS_TIMEOUT_MS: u128 = 250;

pub struct InputTracker {
    button_press_times: [Option<Instant>; 16],
    button_last_release: [Option<Instant>; 16],
    double_click_enabled: [bool; 16],
    long_press_notified: [bool; 16],
    encoder_pushed: [bool; 4],
    encoder_moved_while_pushed: [bool; 4],
}

impl InputTracker {
    pub fn new() -> Self {
        Self {
            button_press_times: [None; 16],
            button_last_release: [None; 16],
            double_click_enabled: [false; 16],
            long_press_notified: [false; 16],
            encoder_pushed: [false; 4],
            encoder_moved_while_pushed: [false; 4],
        }
    }

    pub fn set_double_click_enabled(&mut self, id: u8, enabled: bool) {
        if (id as usize) < 16 {
            self.double_click_enabled[id as usize] = enabled;
            if !enabled {
                self.button_last_release[id as usize] = None;
            }
        }
    }

    pub fn process_event(&mut self, event: PicoToHost) -> Option<HardwareTrigger> {
        match event {
            PicoToHost::ButtonChanged { id, pressed } => {
                let id_usize = id as usize;
                if pressed {
                    self.button_press_times[id_usize] = Some(Instant::now());
                    if self.double_click_enabled[id_usize] {
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
                    }
                    None
                } else {
                    if let Some(press_time) = self.button_press_times[id_usize] {
                        let duration = press_time.elapsed().as_millis();
                        self.button_press_times[id_usize] = None;
                        if duration >= LONG_PRESS_MS {
                            self.button_last_release[id_usize] = None;
                            self.long_press_notified[id_usize] = false;
                            return Some(HardwareTrigger::Button {
                                id,
                                event: ButtonEvent::LongPress,
                            });
                        } else {
                            if self.double_click_enabled[id_usize] {
                                self.button_last_release[id_usize] = Some(Instant::now());
                                return None;
                            } else {
                                self.button_last_release[id_usize] = None;
                                return Some(HardwareTrigger::Button {
                                    id,
                                    event: ButtonEvent::ShortPress,
                                });
                            }
                        }
                    }
                    None
                }
            }

            PicoToHost::EncoderChanged { id, pressed } => {
                let idx = id as usize;
                if pressed {
                    self.encoder_pushed[idx] = true;
                    self.encoder_moved_while_pushed[idx] = false;
                    None
                } else {
                    self.encoder_pushed[idx] = false;

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

    pub fn update(&mut self) -> Option<HardwareTrigger> {
        for id in 0..16 {
            if self.double_click_enabled[id] {
                if let Some(last_release) = self.button_last_release[id] {
                    if last_release.elapsed().as_millis() >= DOUBLE_PRESS_TIMEOUT_MS
                        && self.button_press_times[id].is_none()
                    {
                        self.button_last_release[id] = None;

                        return Some(HardwareTrigger::Button {
                            id: id as u8,
                            event: ButtonEvent::ShortPress,
                        });
                    }
                }
            }
        }
        None
    }
    
    pub fn check_long_press_feedback(&mut self) -> Option<u8> {
        for id in 0..16 {
            if let Some(press_time) = self.button_press_times[id] {
                if !self.long_press_notified[id] && press_time.elapsed().as_millis() >= LONG_PRESS_MS {
                    self.long_press_notified[id] = true;
                    return Some(id as u8);
                }
            }
        }
        None
    }
}