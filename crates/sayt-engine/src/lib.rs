//! Two-deck native audio engine with bounded command transport.

mod beat_performance;
mod decoder;
mod dsp;
mod engine;
mod hot_cue;
mod manual_loop;

pub use beat_performance::{
    beat_duration_seconds, validate_beat_jump_size, validate_loop_roll_region,
    validate_loop_roll_size, validate_manual_bpm, BEAT_JUMP_SIZES, LOOP_ROLL_SIZES_BEATS,
    MAX_MANUAL_BPM, MIN_MANUAL_BPM,
};
pub use engine::{DeckSnapshot, DeviceInfo, Engine, MixerSnapshot};
pub use hot_cue::{
    default_hot_cue, empty_hot_cue_slots, validate_hot_cue, validate_hot_cue_slots, HotCue,
    HotCueColor, HotCueSlots, HOT_CUE_NAME_MAX_CHARS, HOT_CUE_SLOT_COUNT,
};
pub use manual_loop::{
    validate_manual_loop, DEFAULT_MANUAL_LOOP_SIZE_SECONDS, MIN_MANUAL_LOOP_DURATION_SECONDS,
};
