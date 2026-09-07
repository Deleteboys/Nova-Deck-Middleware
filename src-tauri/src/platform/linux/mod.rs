//! Linux-Backend: PulseAudio-Protokoll (PipeWire/PulseAudio), MPRIS und die
//! Compositor-spezifischen Wege zum aktiven Fenster.

pub mod audio;
pub mod diagnostics;
pub mod media;
pub mod shell;
pub mod window;

mod pulse;
mod window_hypr;
mod window_plasma;
mod window_x11;
