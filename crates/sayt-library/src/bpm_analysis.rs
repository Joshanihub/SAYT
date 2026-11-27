//! Bounded, offline BPM estimation for supported integer PCM WAV files.
//!
//! The estimate is a periodicity heuristic, not a probability. Tracks with weak
//! onset periodicity or unresolved half/double-tempo peaks are returned as
//! uncertain so callers can keep the candidate separate from a user's choice.

use std::{
    fs,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};

use hound::{SampleFormat, WavReader};

const ANALYSIS_VERSION: &str = "rms-onset-autocorrelation-v1";
const ENVELOPE_RATE_HZ: usize = 100;
const MIN_ANALYSIS_SECONDS: usize = 8;
const MIN_BPM: f64 = 60.0;
const MAX_BPM: f64 = 200.0;
const MIN_PERIODICITY: f64 = 0.35;
const OCTAVE_AMBIGUITY_RATIO: f64 = 0.80;
const MAX_FILE_BYTES: u64 = 512 * 1024 * 1024;
const MAX_AUDIO_FRAMES: u64 = 32_000_000;
const PROGRESS_FRAME_INTERVAL: u64 = 48_000;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum AnalysisAttempt {
    Cancelled,
    Complete(TempoEstimate),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TempoEstimate {
    pub candidate_bpm: Option<f64>,
    pub confidence: f64,
    pub status: TempoStatus,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum TempoStatus {
    Estimated,
    Uncertain,
}

impl TempoStatus {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Estimated => "estimated",
            Self::Uncertain => "uncertain",
        }
    }
}

pub(crate) fn algorithm_version() -> &'static str {
    ANALYSIS_VERSION
}

pub(crate) fn analyze_wav(
    path: &Path,
    cancellation: &AtomicBool,
    mut progress: impl FnMut(u64, Option<u64>),
) -> Result<AnalysisAttempt, String> {
    let metadata = fs::metadata(path)
        .map_err(|error| format!("Could not inspect the track for BPM analysis: {error}"))?;
    if metadata.len() > MAX_FILE_BYTES {
        return Err(format!(
            "This file is {} bytes; BPM analysis is limited to {} bytes",
            metadata.len(),
            MAX_FILE_BYTES
        ));
    }

    let reader = WavReader::open(path)
        .map_err(|error| format!("Could not open this WAV for BPM analysis: {error}"))?;
    let spec = reader.spec();
    if spec.sample_format != SampleFormat::Int || !matches!(spec.bits_per_sample, 16 | 24 | 32) {
        return Err(
            "BPM analysis currently supports 16-, 24-, or 32-bit integer PCM WAV files".to_string(),
        );
    }
    if !matches!(spec.channels, 1 | 2) {
        return Err("BPM analysis currently supports mono or stereo WAV files".to_string());
    }
    if !(8_000..=192_000).contains(&spec.sample_rate) {
        return Err("BPM analysis supports WAV sample rates from 8,000 to 192,000 Hz".to_string());
    }

    // `hound::WavReader::duration` is already expressed in frames, independent
    // of channel count. Keep progress in frames to match `processed_frames`.
    let total_frames = u64::from(reader.duration());
    if total_frames > MAX_AUDIO_FRAMES {
        return Err("This WAV exceeds the 32,000,000-frame BPM analysis limit".to_string());
    }
    let mut samples = reader.into_samples::<i32>();
    let mut envelope = Vec::with_capacity(
        (total_frames.min(MAX_AUDIO_FRAMES) as usize
            / usize::try_from(spec.sample_rate).unwrap_or(1))
            * ENVELOPE_RATE_HZ,
    );
    let mut next_window_end = (u64::from(spec.sample_rate) / ENVELOPE_RATE_HZ as u64).max(1);
    let mut frame_samples = 0_u16;
    let mut frame_energy = 0.0_f64;
    let mut window_energy = 0.0_f64;
    let mut window_frames = 0_u64;
    let mut processed_frames = 0_u64;
    let scale = 2.0_f64.powi(i32::from(spec.bits_per_sample) - 1);

    loop {
        if processed_frames.is_multiple_of(4_096) && cancellation.load(Ordering::Acquire) {
            return Ok(AnalysisAttempt::Cancelled);
        }
        let Some(sample) = samples.next() else {
            break;
        };
        let sample =
            sample.map_err(|error| format!("Invalid PCM data during BPM analysis: {error}"))?;
        let normalized = f64::from(sample) / scale;
        frame_energy += normalized * normalized;
        frame_samples += 1;
        if frame_samples < spec.channels {
            continue;
        }

        frame_samples = 0;
        processed_frames += 1;
        if processed_frames > MAX_AUDIO_FRAMES {
            return Err("This WAV exceeds the 32,000,000-frame BPM analysis limit".to_string());
        }
        let channel_mean_square = frame_energy / f64::from(spec.channels);
        frame_energy = 0.0;
        window_energy += channel_mean_square;
        window_frames += 1;

        if processed_frames >= next_window_end {
            envelope.push((window_energy / window_frames as f64).sqrt() as f32);
            window_energy = 0.0;
            window_frames = 0;
            let completed_windows = envelope.len() as u64;
            next_window_end = completed_windows
                .saturating_add(1)
                .saturating_mul(u64::from(spec.sample_rate))
                .div_ceil(ENVELOPE_RATE_HZ as u64);
        }

        if processed_frames.is_multiple_of(PROGRESS_FRAME_INTERVAL) {
            progress(processed_frames, Some(total_frames));
        }
    }
    if frame_samples != 0 {
        return Err("Stereo WAV data ended with an incomplete PCM frame".to_string());
    }
    if window_frames > 0 {
        envelope.push((window_energy / window_frames as f64).sqrt() as f32);
    }
    if cancellation.load(Ordering::Acquire) {
        return Ok(AnalysisAttempt::Cancelled);
    }
    progress(processed_frames, Some(total_frames));

    let Some(estimate) = estimate_envelope(&envelope, cancellation) else {
        return Ok(AnalysisAttempt::Cancelled);
    };
    Ok(AnalysisAttempt::Complete(estimate))
}

fn estimate_envelope(envelope: &[f32], cancellation: &AtomicBool) -> Option<TempoEstimate> {
    if envelope.len() < ENVELOPE_RATE_HZ * MIN_ANALYSIS_SECONDS {
        return Some(uncertain(None, 0.0));
    }

    let mut onset = Vec::with_capacity(envelope.len().saturating_sub(1));
    for pair in envelope.windows(2) {
        onset.push((pair[1] - pair[0]).max(0.0) as f64);
    }
    let mean = onset.iter().sum::<f64>() / onset.len().max(1) as f64;
    for value in &mut onset {
        *value -= mean;
    }
    let total_energy = onset.iter().map(|value| value * value).sum::<f64>();
    if total_energy <= f64::EPSILON {
        return Some(uncertain(None, 0.0));
    }

    let min_lag = (60.0 * ENVELOPE_RATE_HZ as f64 / MAX_BPM).floor() as usize;
    let max_lag = (60.0 * ENVELOPE_RATE_HZ as f64 / MIN_BPM).ceil() as usize;
    let max_lag = max_lag.min(onset.len().saturating_sub(1));
    let mut scores = vec![0.0_f64; max_lag + 1];
    for lag in min_lag..=max_lag {
        if lag.is_multiple_of(8) && cancellation.load(Ordering::Acquire) {
            return None;
        }
        let overlap = onset.len() - lag;
        let (product, left_energy, right_energy) = onset[..overlap].iter().zip(&onset[lag..]).fold(
            (0.0, 0.0, 0.0),
            |(product, left_sum, right_sum), (left, right)| {
                (
                    product + left * right,
                    left_sum + left * left,
                    right_sum + right * right,
                )
            },
        );
        let normalization = (left_energy * right_energy).sqrt();
        if normalization > f64::EPSILON {
            scores[lag] = (product / normalization).clamp(-1.0, 1.0);
        }
    }

    let best_lag =
        (min_lag..=max_lag).max_by(|left, right| scores[*left].total_cmp(&scores[*right]));
    let Some(best_lag) = best_lag else {
        return Some(uncertain(None, 0.0));
    };
    let best_score = scores[best_lag].max(0.0);
    if best_score <= 0.0 {
        return Some(uncertain(None, 0.0));
    }

    let mut refined_lag = best_lag as f64;
    if best_lag > min_lag && best_lag < max_lag {
        let before = scores[best_lag - 1];
        let center = scores[best_lag];
        let after = scores[best_lag + 1];
        let curvature = before - 2.0 * center + after;
        if curvature.abs() > f64::EPSILON {
            let adjustment = (0.5 * (before - after) / curvature).clamp(-0.5, 0.5);
            refined_lag += adjustment;
        }
    }
    let candidate_bpm = (60.0 * ENVELOPE_RATE_HZ as f64 / refined_lag).clamp(MIN_BPM, MAX_BPM);

    let octave_lag = if candidate_bpm >= 2.0 * MIN_BPM {
        (60.0 * ENVELOPE_RATE_HZ as f64 / (candidate_bpm * 0.5)).round() as usize
    } else if candidate_bpm * 2.0 <= MAX_BPM {
        (60.0 * ENVELOPE_RATE_HZ as f64 / (candidate_bpm * 2.0)).round() as usize
    } else {
        0
    };
    let octave_ambiguous = octave_lag >= min_lag
        && octave_lag <= max_lag
        && octave_lag.abs_diff(best_lag) > 2
        && scores[octave_lag] >= best_score * OCTAVE_AMBIGUITY_RATIO;
    let confidence = best_score.clamp(0.0, 1.0);

    if confidence < MIN_PERIODICITY || octave_ambiguous {
        Some(uncertain(Some(candidate_bpm), confidence))
    } else {
        Some(TempoEstimate {
            candidate_bpm: Some(candidate_bpm),
            confidence,
            status: TempoStatus::Estimated,
        })
    }
}

fn uncertain(candidate_bpm: Option<f64>, confidence: f64) -> TempoEstimate {
    TempoEstimate {
        candidate_bpm,
        confidence: confidence.clamp(0.0, 1.0),
        status: TempoStatus::Uncertain,
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        sync::atomic::{AtomicBool, AtomicU64, Ordering},
    };

    use hound::{SampleFormat, WavSpec, WavWriter};

    use super::*;

    static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

    struct WavFixture(PathBuf);

    impl WavFixture {
        fn new() -> Self {
            let id = NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
            Self(
                std::env::temp_dir()
                    .join(format!("sayt-bpm-analysis-{}-{id}.wav", std::process::id())),
            )
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for WavFixture {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.0);
        }
    }

    fn write_click_track(path: &Path, bpm: f64, duration_seconds: usize) {
        const SAMPLE_RATE: u32 = 48_000;
        let mut writer = WavWriter::create(
            path,
            WavSpec {
                channels: 1,
                sample_rate: SAMPLE_RATE,
                bits_per_sample: 16,
                sample_format: SampleFormat::Int,
            },
        )
        .unwrap();
        let samples_per_beat = (f64::from(SAMPLE_RATE) * 60.0 / bpm).round() as usize;
        let total_samples = SAMPLE_RATE as usize * duration_seconds;
        for sample_index in 0..total_samples {
            let within_beat = sample_index % samples_per_beat;
            let sample = if within_beat < 240 {
                let envelope = 1.0 - within_beat as f64 / 240.0;
                (i16::MAX as f64 * envelope * (within_beat as f64 * 0.1).sin()) as i16
            } else {
                0_i16
            };
            writer.write_sample(sample).unwrap();
        }
        writer.finalize().unwrap();
    }

    #[test]
    fn reports_half_double_tempo_ambiguity_for_a_regular_120_bpm_pulse() {
        let fixture = WavFixture::new();
        write_click_track(fixture.path(), 120.0, 16);
        let result = analyze_wav(fixture.path(), &AtomicBool::new(false), |_, _| {}).unwrap();
        let AnalysisAttempt::Complete(estimate) = result else {
            panic!("analysis should complete");
        };

        let candidate = estimate
            .candidate_bpm
            .expect("a tempo candidate is available");
        assert!(
            (candidate - 60.0).abs() < 2.0 || (candidate - 120.0).abs() < 2.0,
            "candidate was {candidate}"
        );
        assert!(estimate.confidence > 0.5);
        assert_eq!(estimate.status, TempoStatus::Uncertain);
    }

    #[test]
    fn estimates_an_unambiguous_60_bpm_pulse() {
        let fixture = WavFixture::new();
        write_click_track(fixture.path(), 60.0, 16);
        let result = analyze_wav(fixture.path(), &AtomicBool::new(false), |_, _| {}).unwrap();
        let AnalysisAttempt::Complete(estimate) = result else {
            panic!("analysis should complete");
        };

        let candidate = estimate
            .candidate_bpm
            .expect("a tempo candidate is available");
        assert!((candidate - 60.0).abs() < 2.0, "candidate was {candidate}");
        assert!(estimate.confidence > 0.5);
        assert_eq!(estimate.status, TempoStatus::Estimated);
    }

    #[test]
    fn reports_silence_as_uncertain_without_inventing_a_tempo() {
        let fixture = WavFixture::new();
        let mut writer = WavWriter::create(
            fixture.path(),
            WavSpec {
                channels: 1,
                sample_rate: 48_000,
                bits_per_sample: 16,
                sample_format: SampleFormat::Int,
            },
        )
        .unwrap();
        for _ in 0..48_000 * 10 {
            writer.write_sample(0_i16).unwrap();
        }
        writer.finalize().unwrap();
        let result = analyze_wav(fixture.path(), &AtomicBool::new(false), |_, _| {}).unwrap();
        assert_eq!(result, AnalysisAttempt::Complete(uncertain(None, 0.0)));
    }

    #[test]
    fn reports_stereo_progress_in_frames_not_channel_samples() {
        const SAMPLE_RATE: u32 = 48_000;
        const DURATION_SECONDS: u32 = 10;
        let fixture = WavFixture::new();
        let mut writer = WavWriter::create(
            fixture.path(),
            WavSpec {
                channels: 2,
                sample_rate: SAMPLE_RATE,
                bits_per_sample: 16,
                sample_format: SampleFormat::Int,
            },
        )
        .unwrap();
        for _ in 0..SAMPLE_RATE * DURATION_SECONDS {
            writer.write_sample(0_i16).unwrap();
            writer.write_sample(0_i16).unwrap();
        }
        writer.finalize().unwrap();

        let mut last_progress = None;
        let result = analyze_wav(
            fixture.path(),
            &AtomicBool::new(false),
            |processed, total| {
                last_progress = Some((processed, total));
            },
        )
        .unwrap();

        assert!(matches!(result, AnalysisAttempt::Complete(_)));
        assert_eq!(
            last_progress,
            Some((
                u64::from(SAMPLE_RATE * DURATION_SECONDS),
                Some(u64::from(SAMPLE_RATE * DURATION_SECONDS))
            ))
        );
    }

    #[test]
    fn rejects_short_and_unsupported_input_and_honors_cancellation() {
        let fixture = WavFixture::new();
        write_click_track(fixture.path(), 120.0, 2);
        let result = analyze_wav(fixture.path(), &AtomicBool::new(false), |_, _| {}).unwrap();
        let AnalysisAttempt::Complete(estimate) = result else {
            panic!("short analysis should complete as uncertain");
        };
        assert_eq!(estimate.status, TempoStatus::Uncertain);
        assert_eq!(estimate.candidate_bpm, None);

        let cancelled = AtomicBool::new(true);
        assert_eq!(
            analyze_wav(fixture.path(), &cancelled, |_, _| {}).unwrap(),
            AnalysisAttempt::Cancelled
        );
        fs::write(fixture.path(), b"not a wave file").unwrap();
        assert!(analyze_wav(fixture.path(), &AtomicBool::new(false), |_, _| {}).is_err());

        let float_fixture = WavFixture::new();
        let mut writer = WavWriter::create(
            float_fixture.path(),
            WavSpec {
                channels: 1,
                sample_rate: 48_000,
                bits_per_sample: 32,
                sample_format: SampleFormat::Float,
            },
        )
        .unwrap();
        for _ in 0..48_000 {
            writer.write_sample(0.0_f32).unwrap();
        }
        writer.finalize().unwrap();
        assert!(
            analyze_wav(float_fixture.path(), &AtomicBool::new(false), |_, _| {})
                .unwrap_err()
                .contains("integer PCM WAV")
        );
    }

    #[test]
    fn cancellation_during_pcm_analysis_stops_before_estimation_finishes() {
        let fixture = WavFixture::new();
        write_click_track(fixture.path(), 120.0, 16);
        let cancellation = AtomicBool::new(false);
        let result = analyze_wav(fixture.path(), &cancellation, |processed, _| {
            if processed > 0 {
                cancellation.store(true, Ordering::Release);
            }
        })
        .unwrap();
        assert_eq!(result, AnalysisAttempt::Cancelled);
    }

    #[test]
    fn cancellation_during_tempo_correlation_exits_without_a_candidate() {
        let mut envelope = vec![0.0_f32; 100_000];
        for index in (0..envelope.len()).step_by(50) {
            envelope[index] = 1.0;
        }
        assert!(estimate_envelope(&envelope, &AtomicBool::new(true)).is_none());
    }
}
