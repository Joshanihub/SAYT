"""Tests for expected frequency content in generated fixtures.

Uses a radix-2 Cooley-Tukey FFT coarse peak search followed by a local 1 Hz
Goertzel refinement (standard-library math/cmath only) to confirm the dominant
frequency in each tonal fixture is within ±5 Hz of spec.
"""

from __future__ import annotations

import cmath
import math
import shutil
import struct
import tempfile
import unittest
import wave
from pathlib import Path

import sys
sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

import generate_fixtures as gen  # noqa: E402

_WINDOW_FRAMES = 4096
_MIN_FREQ_HZ = 20
_MAX_FREQ_HZ = 10000
_REFINE_HALF_SPAN_HZ = 25


def _fft_magnitudes(samples: list[int]) -> list[float]:
    """Compute magnitude-squared spectrum for a power-of-two real input."""
    n = len(samples)
    if n == 0 or (n & (n - 1)) != 0:
        raise ValueError(f"FFT length must be a power of 2, got {n}")

    # Bit-reversal permutation
    data = [complex(float(x), 0.0) for x in samples]
    j = 0
    for i in range(1, n):
        bit = n >> 1
        while j & bit:
            j ^= bit
            bit >>= 1
        j ^= bit
        if i < j:
            data[i], data[j] = data[j], data[i]

    # Iterative Cooley-Tukey butterfly stages
    length = 2
    while length <= n:
        half = length // 2
        w_step = cmath.exp(-2j * math.pi / length)
        for start in range(0, n, length):
            w = 1.0 + 0j
            for k in range(half):
                u = data[start + k]
                v = data[start + k + half] * w
                data[start + k] = u + v
                data[start + k + half] = u - v
                w *= w_step
        length <<= 1

    half_bins = n // 2 + 1
    return [data[k].real * data[k].real + data[k].imag * data[k].imag
            for k in range(half_bins)]


def _goertzel_mag2(block: list[int], freq_hz: float, sr: int) -> float:
    """Evaluate DFT magnitude-squared at *freq_hz* via Goertzel."""
    n = len(block)
    omega = 2.0 * math.pi * freq_hz / sr
    coeff = 2.0 * math.cos(omega)
    s0 = 0.0
    s1 = 0.0
    s2 = 0.0
    for sample in block:
        s0 = sample + coeff * s1 - s2
        s2 = s1
        s1 = s0
    return s1 * s1 + s2 * s2 - coeff * s1 * s2


def _dominant_frequency(filepath: Path, channel: int = 0) -> float:
    """Return the dominant frequency (20–10000 Hz, 1 Hz step) of a channel.

    Reads only the middle 4096-frame window, uses a radix-2 FFT to locate the
    coarse spectral peak across the 20–10000 Hz band, then refines at 1 Hz
    resolution within ±25 Hz via Goertzel.
    """
    with wave.open(str(filepath), "rb") as wf:
        sr = wf.getframerate()
        n_channels = wf.getnchannels()
        n_frames = wf.getnframes()
        n = min(_WINDOW_FRAMES, n_frames)
        start = max(0, (n_frames - n) // 2)
        wf.setpos(start)
        raw = wf.readframes(n)

    fmt = "<" + "h" * (n * n_channels)
    all_samples = struct.unpack(fmt, raw)
    block = [all_samples[i * n_channels + channel] for i in range(n)]

    # Zero-pad if a short fixture has fewer than _WINDOW_FRAMES samples
    fft_n = 1
    while fft_n < len(block):
        fft_n <<= 1
    fft_input = block + [0] * (fft_n - len(block))

    mags = _fft_magnitudes(fft_input)
    k_min = max(1, math.ceil(_MIN_FREQ_HZ * fft_n / sr))
    k_max = min(len(mags) - 1, math.floor(_MAX_FREQ_HZ * fft_n / sr))
    peak_k = max(range(k_min, k_max + 1), key=lambda k: mags[k])
    coarse_hz = round(peak_k * sr / fft_n)

    # Local 1 Hz Goertzel refinement around the coarse FFT peak
    lo = max(_MIN_FREQ_HZ, coarse_hz - _REFINE_HALF_SPAN_HZ)
    hi = min(_MAX_FREQ_HZ, coarse_hz + _REFINE_HALF_SPAN_HZ)

    best_freq = float(lo)
    best_mag = -1.0
    for freq in range(lo, hi + 1):
        mag = _goertzel_mag2(block, float(freq), sr)
        if mag > best_mag:
            best_mag = mag
            best_freq = float(freq)

    return best_freq


class TestFrequencyContent(unittest.TestCase):
    """Verify dominant frequency of each tonal fixture."""

    @classmethod
    def setUpClass(cls):
        cls.tmpdir = tempfile.mkdtemp(prefix="sayt_freq_")
        cls.output_dir = Path(cls.tmpdir)
        gen.generate(cls.output_dir, force=True)

    @classmethod
    def tearDownClass(cls):
        shutil.rmtree(cls.tmpdir, ignore_errors=True)

    def _assert_freq(self, filename: str, expected_hz: float,
                     channel: int = 0, tolerance: float = 5.0):
        filepath = self.output_dir / filename
        actual = _dominant_frequency(filepath, channel)
        self.assertAlmostEqual(
            actual, expected_hz, delta=tolerance,
            msg=f"{filename} ch{channel}: expected {expected_hz} Hz, "
                f"got {actual} Hz")

    def test_deck_a_440hz(self):
        self._assert_freq("deck_a_440hz.wav", 440.0)

    def test_deck_b_880hz(self):
        self._assert_freq("deck_b_880hz.wav", 880.0)

    def test_eq_low_100hz(self):
        self._assert_freq("eq_low_100hz.wav", 100.0)

    def test_eq_mid_1000hz(self):
        self._assert_freq("eq_mid_1000hz.wav", 1000.0)

    def test_eq_high_5000hz(self):
        self._assert_freq("eq_high_5000hz.wav", 5000.0)

    def test_stereo_left_440hz(self):
        self._assert_freq("stereo_channel_id.wav", 440.0, channel=0)

    def test_stereo_right_660hz(self):
        self._assert_freq("stereo_channel_id.wav", 660.0, channel=1)


if __name__ == "__main__":
    unittest.main()
