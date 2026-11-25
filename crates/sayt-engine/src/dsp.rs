//! Fixed-cost, allocation-free equalizer filters for the audio callback.

use std::f32::consts::PI;

use crate::decoder::StereoFrame;

pub(crate) const LOW_CUTOFF_HZ: f32 = 200.0;
pub(crate) const MID_CENTER_HZ: f32 = 1_000.0;
pub(crate) const HIGH_CUTOFF_HZ: f32 = 5_000.0;
const FILTER_Q: f32 = std::f32::consts::FRAC_1_SQRT_2;
const SHELF_SLOPE: f32 = 1.0;

#[derive(Clone, Copy, Debug)]
pub(crate) struct BiquadCoefficients {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
}

impl BiquadCoefficients {
    fn normalized(b0: f32, b1: f32, b2: f32, a0: f32, a1: f32, a2: f32) -> Self {
        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        }
    }

    fn low_shelf(sample_rate: u32, frequency: f32, gain_db: f32) -> Self {
        let a = 10.0_f32.powf(gain_db / 40.0);
        let omega = 2.0 * PI * frequency / sample_rate as f32;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / 2.0 * (((a + 1.0 / a) * (1.0 / SHELF_SLOPE - 1.0) + 2.0).sqrt());
        let root_a = a.sqrt();
        let two_root_alpha = 2.0 * root_a * alpha;
        let b0 = a * ((a + 1.0) - (a - 1.0) * cos_omega + two_root_alpha);
        let b1 = 2.0 * a * ((a - 1.0) - (a + 1.0) * cos_omega);
        let b2 = a * ((a + 1.0) - (a - 1.0) * cos_omega - two_root_alpha);
        let a0 = (a + 1.0) + (a - 1.0) * cos_omega + two_root_alpha;
        let a1 = -2.0 * ((a - 1.0) + (a + 1.0) * cos_omega);
        let a2 = (a + 1.0) + (a - 1.0) * cos_omega - two_root_alpha;
        Self::normalized(b0, b1, b2, a0, a1, a2)
    }

    fn peaking(sample_rate: u32, frequency: f32, gain_db: f32) -> Self {
        let a = 10.0_f32.powf(gain_db / 40.0);
        let omega = 2.0 * PI * frequency / sample_rate as f32;
        let alpha = omega.sin() / (2.0 * FILTER_Q);
        let cos_omega = omega.cos();
        Self::normalized(
            1.0 + alpha * a,
            -2.0 * cos_omega,
            1.0 - alpha * a,
            1.0 + alpha / a,
            -2.0 * cos_omega,
            1.0 - alpha / a,
        )
    }

    fn high_shelf(sample_rate: u32, frequency: f32, gain_db: f32) -> Self {
        let a = 10.0_f32.powf(gain_db / 40.0);
        let omega = 2.0 * PI * frequency / sample_rate as f32;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / 2.0 * (((a + 1.0 / a) * (1.0 / SHELF_SLOPE - 1.0) + 2.0).sqrt());
        let root_a = a.sqrt();
        let two_root_alpha = 2.0 * root_a * alpha;
        let b0 = a * ((a + 1.0) + (a - 1.0) * cos_omega + two_root_alpha);
        let b1 = -2.0 * a * ((a - 1.0) + (a + 1.0) * cos_omega);
        let b2 = a * ((a + 1.0) + (a - 1.0) * cos_omega - two_root_alpha);
        let a0 = (a + 1.0) - (a - 1.0) * cos_omega + two_root_alpha;
        let a1 = 2.0 * ((a - 1.0) - (a + 1.0) * cos_omega);
        let a2 = (a + 1.0) - (a - 1.0) * cos_omega - two_root_alpha;
        Self::normalized(b0, b1, b2, a0, a1, a2)
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct EqCoefficients {
    low: BiquadCoefficients,
    mid: BiquadCoefficients,
    high: BiquadCoefficients,
}

impl EqCoefficients {
    pub(crate) fn from_gains(sample_rate: u32, low_db: f32, mid_db: f32, high_db: f32) -> Self {
        let low_frequency = LOW_CUTOFF_HZ.min(sample_rate as f32 * 0.08);
        let mid_frequency = MID_CENTER_HZ.min(sample_rate as f32 * 0.25);
        let high_frequency = HIGH_CUTOFF_HZ.min(sample_rate as f32 * 0.4);
        Self {
            low: BiquadCoefficients::low_shelf(sample_rate, low_frequency, low_db),
            mid: BiquadCoefficients::peaking(sample_rate, mid_frequency, mid_db),
            high: BiquadCoefficients::high_shelf(sample_rate, high_frequency, high_db),
        }
    }
}

#[derive(Clone, Copy)]
struct Biquad {
    coefficients: BiquadCoefficients,
    state_1: f32,
    state_2: f32,
}

impl Biquad {
    fn new(coefficients: BiquadCoefficients) -> Self {
        Self {
            coefficients,
            state_1: 0.0,
            state_2: 0.0,
        }
    }

    fn set_coefficients(&mut self, coefficients: BiquadCoefficients) {
        self.coefficients = coefficients;
    }

    fn process(&mut self, input: f32) -> f32 {
        let output = self.coefficients.b0 * input + self.state_1;
        self.state_1 = self.coefficients.b1 * input - self.coefficients.a1 * output + self.state_2;
        self.state_2 = self.coefficients.b2 * input - self.coefficients.a2 * output;
        output
    }
}

#[derive(Clone, Copy)]
pub(crate) struct EqBank {
    low: [Biquad; 2],
    mid: [Biquad; 2],
    high: [Biquad; 2],
}

impl EqBank {
    pub(crate) fn new(coefficients: EqCoefficients) -> Self {
        Self {
            low: [Biquad::new(coefficients.low), Biquad::new(coefficients.low)],
            mid: [Biquad::new(coefficients.mid), Biquad::new(coefficients.mid)],
            high: [
                Biquad::new(coefficients.high),
                Biquad::new(coefficients.high),
            ],
        }
    }

    pub(crate) fn set_coefficients(&mut self, coefficients: EqCoefficients) {
        for channel in 0..2 {
            self.low[channel].set_coefficients(coefficients.low);
            self.mid[channel].set_coefficients(coefficients.mid);
            self.high[channel].set_coefficients(coefficients.high);
        }
    }

    pub(crate) fn process(&mut self, frame: StereoFrame) -> StereoFrame {
        StereoFrame {
            left: self.high[0].process(self.mid[0].process(self.low[0].process(frame.left))),
            right: self.high[1].process(self.mid[1].process(self.low[1].process(frame.right))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn measured_rms(frequency: f32, coefficients: EqCoefficients) -> f32 {
        let mut filter = EqBank::new(coefficients);
        let sample_rate = 48_000.0_f32;
        let mut sum = 0.0_f64;
        let mut measured = 0_u32;
        for sample_index in 0..24_000 {
            let input = (2.0 * PI * frequency * sample_index as f32 / sample_rate).sin() * 0.1;
            let output = filter.process(StereoFrame {
                left: input,
                right: input,
            });
            if sample_index >= 4_000 {
                sum += f64::from(output.left * output.left);
                measured += 1;
            }
        }
        (sum / f64::from(measured)).sqrt() as f32
    }

    fn unfiltered_rms(frequency: f32) -> f32 {
        let sample_rate = 48_000.0_f32;
        let mut sum = 0.0_f64;
        let mut measured = 0_u32;
        for sample_index in 0..24_000 {
            let input = (2.0 * PI * frequency * sample_index as f32 / sample_rate).sin() * 0.1;
            if sample_index >= 4_000 {
                sum += f64::from(input * input);
                measured += 1;
            }
        }
        (sum / f64::from(measured)).sqrt() as f32
    }

    #[test]
    fn low_shelf_changes_low_frequencies_more_than_high_frequencies() {
        let coefficients = EqCoefficients::from_gains(48_000, 12.0, 0.0, 0.0);
        let low_rms = measured_rms(60.0, coefficients);
        let high_rms = measured_rms(6_000.0, coefficients);
        assert!(low_rms > high_rms * 2.5, "low={low_rms}, high={high_rms}");
    }

    #[test]
    fn mid_peak_changes_its_center_more_than_low_frequencies() {
        let coefficients = EqCoefficients::from_gains(48_000, 0.0, 12.0, 0.0);
        let center_rms = measured_rms(1_000.0, coefficients);
        let low_rms = measured_rms(100.0, coefficients);
        assert!(
            center_rms > low_rms * 2.5,
            "center={center_rms}, low={low_rms}"
        );
    }

    #[test]
    fn high_shelf_changes_high_frequencies_more_than_low_frequencies() {
        let coefficients = EqCoefficients::from_gains(48_000, 0.0, 0.0, 12.0);
        let high_rms = measured_rms(12_000.0, coefficients);
        let low_rms = measured_rms(400.0, coefficients);
        assert!(high_rms > low_rms * 2.5, "high={high_rms}, low={low_rms}");
    }

    #[test]
    fn zero_gain_is_unity_across_the_audio_band() {
        let coefficients = EqCoefficients::from_gains(48_000, 0.0, 0.0, 0.0);
        let input_rms = unfiltered_rms(1_000.0);
        let output_rms = measured_rms(1_000.0, coefficients);
        assert!((output_rms - input_rms).abs() < 1e-5);
    }
}
