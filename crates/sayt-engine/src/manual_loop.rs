//! Bounded manual loop region validation.

pub const MIN_MANUAL_LOOP_DURATION_SECONDS: f64 = 0.05;
pub const DEFAULT_MANUAL_LOOP_SIZE_SECONDS: f64 = 1.0;

pub fn validate_manual_loop(
    loop_in_seconds: f64,
    loop_out_seconds: f64,
    track_duration_seconds: f64,
) -> Result<(), String> {
    if !track_duration_seconds.is_finite() || track_duration_seconds <= 0.0 {
        return Err("The loaded track has an invalid duration for looping".to_string());
    }
    if !loop_in_seconds.is_finite()
        || !loop_out_seconds.is_finite()
        || loop_in_seconds < 0.0
        || loop_out_seconds > track_duration_seconds
        || loop_out_seconds - loop_in_seconds < MIN_MANUAL_LOOP_DURATION_SECONDS
    {
        return Err(format!(
            "Loop Out must be at least {:.0} ms after Loop In and within the track",
            MIN_MANUAL_LOOP_DURATION_SECONDS * 1_000.0
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_zero_start_and_track_end_boundaries() {
        assert!(validate_manual_loop(0.0, 0.05, 10.0).is_ok());
        assert!(validate_manual_loop(9.95, 10.0, 10.0).is_ok());
    }

    #[test]
    fn rejects_reversed_short_non_finite_and_out_of_track_regions() {
        assert!(validate_manual_loop(2.0, 1.0, 10.0).is_err());
        assert!(validate_manual_loop(1.0, 1.049, 10.0).is_err());
        assert!(validate_manual_loop(f64::NAN, 2.0, 10.0).is_err());
        assert!(validate_manual_loop(1.0, f64::INFINITY, 10.0).is_err());
        assert!(validate_manual_loop(9.0, 10.01, 10.0).is_err());
        assert!(validate_manual_loop(0.0, 1.0, f64::NAN).is_err());
    }
}
