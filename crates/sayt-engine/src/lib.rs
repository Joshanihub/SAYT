//! Two-deck native audio engine with bounded command transport.

mod decoder;
mod dsp;
mod engine;
mod hot_cue;
mod manual_loop;

pub use engine::{DeckSnapshot, DeviceInfo, Engine, MixerSnapshot};
pub use hot_cue::{
    default_hot_cue, empty_hot_cue_slots, validate_hot_cue, validate_hot_cue_slots, HotCue,
    HotCueColor, HotCueSlots, HOT_CUE_NAME_MAX_CHARS, HOT_CUE_SLOT_COUNT,
};
pub use manual_loop::{
    validate_manual_loop, DEFAULT_MANUAL_LOOP_SIZE_SECONDS, MIN_MANUAL_LOOP_DURATION_SECONDS,
};
