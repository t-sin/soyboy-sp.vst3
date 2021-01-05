//! This crate provides GameBoy's APU emulation. It includes APU controlling API via *registers* and sound processing emulation layer. The emulation layer has no dependency about platform-specific sound I/O API e.g. ALSA for GNU/Linux.
//!
//! References:
//! - <https://gbdev.gg8.se/wiki/articles/Gameboy_sound_hardware>

pub mod apu;
mod types;
mod util;

pub use types::Sample;
