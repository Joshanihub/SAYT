//! Two-deck native audio engine with bounded command transport.

mod decoder;
mod dsp;
mod engine;

pub use engine::{DeckSnapshot, DeviceInfo, Engine, MixerSnapshot};
