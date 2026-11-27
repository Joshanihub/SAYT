//! Validation and timing rules for manual-tempo performance controls.

use crate::manual_loop::MIN_MANUAL_LOOP_DURATION_SECONDS;

pub const MIN_MANUAL_BPM: f64 = 30.0;
pub const MAX_MANUAL_BPM: f64 = 300.0;
pub const BEAT_JUMP_SIZES: [f64; 5] = [1.0, 2.0, 4.0, 8.0, 16.0];
pub const LOOP_ROLL_SIZES_BEATS: [f64; 7] = [0.25, 0.5, 1.0, 2.0, 4.0, 8.0, 16.0];
const BEAT_SIZE_EPSILON: f64 = 1.0e-9;
const ROLL_DURATION_EPSILON_SECONDS: f64 = 1.0e-9;

pub fn validate_manual_bpm(bpm: f64) -> Result<(), String> {
    if !bpm.is_finite() || !(MIN_MANUAL_BPM..=MAX_MANUAL_BPM).contains(&bpm) {
        return Err(format!(
            "Manual BPM must be between {MIN_MANUAL_BPM:.0} and {MAX_MANUAL_BPM:.0}"
        ));
    }
    Ok(())
}

pub fn validate_beat_jump_size(beats: f64) -> Result<(), String> {
    if !beats.is_finite()
        || beats == 0.0
        || beats.abs() > *BEAT_JUMP_SIZES.last().expect("beat jump sizes are defined")
        || !BEAT_JUMP_SIZES
            .iter()
            .any(|size| (size - beats.abs()).abs() < BEAT_SIZE_EPSILON)
    {
        return Err(
            "Beat Jump size must be 1, 2, 4, 8, or 16 beats in either direction".to_string(),
        );
    }
    Ok(())
}

pub fn validate_loop_roll_size(beats: f64) -> Result<(), String> {
    if !beats.is_finite()
        || !LOOP_ROLL_SIZES_BEATS
            .iter()
            .any(|size| (size - beats).abs() < BEAT_SIZE_EPSILON)
    {
        return Err("Loop Roll size must be 1/4, 1/2, 1, 2, 4, 8, or 16 beats".to_string());
    }
    Ok(())
}

pub fn beat_duration_seconds(bpm: f64, beats: f64) -> f64 {
    60.0 * beats / bpm
}

pub fn validate_loop_roll_region(
    start_seconds: f64,
    duration_seconds: f64,
    track_duration_seconds: f64,
) -> Result<(), String> {
    let end_seconds = start_seconds + duration_seconds;
    if !start_seconds.is_finite()
        || !duration_seconds.is_finite()
        || !track_duration_seconds.is_finite()
        || start_seconds < 0.0
        || duration_seconds + ROLL_DURATION_EPSILON_SECONDS < MIN_MANUAL_LOOP_DURATION_SECONDS
        || track_duration_seconds <= 0.0
        || !end_seconds.is_finite()
        || end_seconds > track_duration_seconds
    {
        return Err("Loop Roll region must fit within the remaining track duration".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_manual_bpm_inclusive_boundaries_and_rejects_invalid_values() {
        assert!(validate_manual_bpm(MIN_MANUAL_BPM).is_ok());
        assert!(validate_manual_bpm(MAX_MANUAL_BPM).is_ok());
        assert!(validate_manual_bpm(120.5).is_ok());
        assert!(validate_manual_bpm(MIN_MANUAL_BPM - 0.1).is_err());
        assert!(validate_manual_bpm(MAX_MANUAL_BPM + 0.1).is_err());
        assert!(validate_manual_bpm(f64::NAN).is_err());
        assert!(validate_manual_bpm(f64::INFINITY).is_err());
    }

    #[test]
    fn validates_only_exposed_beat_jump_and_loop_roll_sizes() {
        for size in BEAT_JUMP_SIZES {
            assert!(validate_beat_jump_size(size).is_ok());
            assert!(validate_beat_jump_size(-size).is_ok());
        }
        for size in LOOP_ROLL_SIZES_BEATS {
            assert!(validate_loop_roll_size(size).is_ok());
        }
        assert!(validate_beat_jump_size(0.0).is_err());
        assert!(validate_beat_jump_size(3.0).is_err());
        assert!(validate_loop_roll_size(0.125).is_err());
        assert!(validate_loop_roll_size(f64::NAN).is_err());
    }

    #[test]
    fn derives_beat_duration_from_manual_tempo() {
        assert!((beat_duration_seconds(120.0, 1.0) - 0.5).abs() < f64::EPSILON);
        assert!((beat_duration_seconds(120.0, 0.25) - 0.125).abs() < f64::EPSILON);
    }

    #[test]
    fn validates_loop_roll_duration_at_arbitrary_positions_and_track_end() {
        let minimum_roll = beat_duration_seconds(300.0, 0.25);
        assert!(validate_loop_roll_region(1.3, minimum_roll, 5.0).is_ok());
        assert!(validate_loop_roll_region(4.95, minimum_roll, 5.0).is_ok());
        assert!(validate_loop_roll_region(4.951, minimum_roll, 5.0).is_err());
        assert!(validate_loop_roll_region(-0.01, 0.5, 5.0).is_err());
        assert!(validate_loop_roll_region(1.0, f64::NAN, 5.0).is_err());
    }
}
