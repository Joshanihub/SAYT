//! Bounded WAV decoding and waveform extraction.

use std::{fs, path::Path, sync::Arc};

pub(crate) const MAX_WAV_FILE_BYTES: u64 = 512 * 1024 * 1024;
pub(crate) const MAX_TRACK_FRAMES: usize = 32_000_000;
pub(crate) const WAVEFORM_BINS: usize = 160;
const MIN_SAMPLE_RATE_HZ: u32 = 8_000;
const MAX_SAMPLE_RATE_HZ: u32 = 192_000;

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct StereoFrame {
    pub(crate) left: f32,
    pub(crate) right: f32,
}

#[derive(Debug)]
pub(crate) struct Track {
    pub(crate) frames: Arc<[StereoFrame]>,
    pub(crate) sample_rate: u32,
    pub(crate) channels: u16,
    pub(crate) duration_seconds: f64,
    pub(crate) filename: String,
    pub(crate) waveform: Arc<[f32]>,
}

pub(crate) fn decode_track(path: &Path) -> Result<Track, String> {
    let metadata =
        fs::metadata(path).map_err(|error| format!("Could not inspect WAV file: {error}"))?;
    if metadata.len() > MAX_WAV_FILE_BYTES {
        return Err(format!(
            "WAV file is {} bytes; the supported file-size limit is {} bytes",
            metadata.len(),
            MAX_WAV_FILE_BYTES
        ));
    }

    let reader = hound::WavReader::open(path)
        .map_err(|error| format!("Could not open WAV file: {error}"))?;
    let spec = reader.spec();
    if spec.sample_format != hound::SampleFormat::Int {
        return Err("Only uncompressed integer PCM WAV files are supported".to_string());
    }
    if !matches!(spec.bits_per_sample, 16 | 24 | 32) {
        return Err(format!(
            "Unsupported WAV bit depth: {} bits; use 16-, 24-, or 32-bit PCM",
            spec.bits_per_sample
        ));
    }
    if !matches!(spec.channels, 1 | 2) {
        return Err(format!(
            "Unsupported WAV channel count: {}; use mono or stereo",
            spec.channels
        ));
    }
    if !(MIN_SAMPLE_RATE_HZ..=MAX_SAMPLE_RATE_HZ).contains(&spec.sample_rate) {
        return Err(format!(
            "Unsupported WAV sample rate: {} Hz; supported range is {}–{} Hz",
            spec.sample_rate, MIN_SAMPLE_RATE_HZ, MAX_SAMPLE_RATE_HZ
        ));
    }

    let mut samples = reader.into_samples::<i32>();
    let scale = (1_i64 << (spec.bits_per_sample - 1)) as f32;
    let mut frames = Vec::new();
    while let Some(left_sample) = read_sample(&mut samples)? {
        let right_sample = if spec.channels == 2 {
            read_sample(&mut samples)?
                .ok_or_else(|| "Stereo WAV data ended with an incomplete audio frame".to_string())?
        } else {
            left_sample
        };
        if frames.len() >= MAX_TRACK_FRAMES {
            return Err(format!(
                "WAV track exceeds the decoded-frame limit of {MAX_TRACK_FRAMES}"
            ));
        }
        frames.push(StereoFrame {
            left: left_sample as f32 / scale,
            right: right_sample as f32 / scale,
        });
    }
    if frames.is_empty() {
        return Err("WAV file contains no audio frames".to_string());
    }

    let duration_seconds = frames.len() as f64 / f64::from(spec.sample_rate);
    let filename = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Selected WAV file")
        .to_string();
    let waveform = make_waveform(&frames);
    Ok(Track {
        frames: Arc::from(frames),
        sample_rate: spec.sample_rate,
        channels: spec.channels,
        duration_seconds,
        filename,
        waveform: Arc::from(waveform),
    })
}

fn read_sample(
    samples: &mut impl Iterator<Item = Result<i32, hound::Error>>,
) -> Result<Option<i32>, String> {
    samples
        .next()
        .transpose()
        .map_err(|error| format!("Invalid WAV sample data: {error}"))
}

fn make_waveform(frames: &[StereoFrame]) -> Vec<f32> {
    (0..WAVEFORM_BINS)
        .map(|bin| {
            let start = bin * frames.len() / WAVEFORM_BINS;
            let end = ((bin + 1) * frames.len() / WAVEFORM_BINS).max(start + 1);
            let end = end.min(frames.len());
            frames[start..end]
                .iter()
                .map(|frame| (frame.left.abs() + frame.right.abs()) * 0.5)
                .fold(0.0_f32, f32::max)
                .clamp(0.0, 1.0)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::{
        fs::{self, File},
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    use super::*;

    static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

    struct TempWav(PathBuf);

    impl TempWav {
        fn new(extension: &str) -> Self {
            let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
            Self(std::env::temp_dir().join(format!(
                "sayt-decoder-{}-{id}.{extension}",
                std::process::id()
            )))
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempWav {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.0);
        }
    }

    #[test]
    fn decodes_mono_wav_and_builds_actual_waveform_peaks() {
        let temp = TempWav::new("wav");
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 48_000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create(temp.path(), spec).unwrap();
        for sample in [0_i16, 8_192, -16_384, 32_767] {
            writer.write_sample(sample).unwrap();
        }
        writer.finalize().unwrap();

        let track = decode_track(temp.path()).unwrap();
        assert_eq!(track.channels, 1);
        assert_eq!(track.frames.len(), 4);
        assert_eq!(track.frames[2].left, track.frames[2].right);
        assert_eq!(track.waveform.len(), WAVEFORM_BINS);
        assert!(track.waveform.iter().any(|peak| *peak > 0.4));
    }

    #[test]
    fn rejects_float_wav_with_actionable_message() {
        let temp = TempWav::new("wav");
        let spec = hound::WavSpec {
            channels: 2,
            sample_rate: 48_000,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };
        let mut writer = hound::WavWriter::create(temp.path(), spec).unwrap();
        writer.write_sample(0.25_f32).unwrap();
        writer.write_sample(-0.25_f32).unwrap();
        writer.finalize().unwrap();

        let error = decode_track(temp.path()).unwrap_err();
        assert!(error.contains("integer PCM"));
    }

    #[test]
    fn rejects_oversized_file_before_parsing_its_header() {
        let temp = TempWav::new("wav");
        let file = File::create(temp.path()).unwrap();
        file.set_len(MAX_WAV_FILE_BYTES + 1).unwrap();

        let error = decode_track(temp.path()).unwrap_err();
        assert!(error.contains("supported file-size limit"));
    }

    #[test]
    fn missing_path_returns_a_file_error() {
        let temp = TempWav::new("missing.wav");
        let error = decode_track(temp.path()).unwrap_err();
        assert!(error.contains("Could not inspect WAV file"));
    }
}
