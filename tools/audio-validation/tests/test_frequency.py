"""Tests for expected frequency content in generated fixtures.

Uses a basic DFT bin-peak check (standard-library math only) to confirm
the dominant frequency in each tonal fixture is within ±5 Hz of spec.
"""

from __future__ import annotations

import json
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


def _dominant_frequency(filepath: Path, channel: int = 0) -> float:
    """Return the dominant frequency of a mono channel using a DFT peak.

    *channel*: 0 = left, 1 = right (for stereo files).

    This uses a basic Goertzel-style magnitude scan over candidate bins
    rather than a full FFT (no numpy).  Accurate enough for pure tones.
    """
    with wave.open(str(filepath), "rb") as wf:
        sr = wf.getframerate()
        n_channels = wf.getnchannels()
        n_frames = wf.getnframes()
        raw = wf.readframes(n_frames)

    fmt = "<" + "h" * (n_frames * n_channels)
    all_samples = struct.unpack(fmt, raw)

    # Extract single channel
    samples = [all_samples[i * n_channels + channel]
               for i in range(n_frames)]

    # Use a window of the middle 4096 samples (past fade-in/out)
    n = min(4096, len(samples))
    start = (len(samples) - n) // 2
    block = samples[start:start + n]

    # Scan frequencies 20–10000 Hz at 1 Hz resolution using Goertzel
    best_freq = 0.0
    best_mag = 0.0

    for freq in range(20, 10001):
        k = freq * n / sr
        omega = 2.0 * math.pi * k / n
        coeff = 2.0 * math.cos(omega)
        s0 = 0.0
        s1 = 0.0
        s2 = 0.0
        for sample in block:
            s0 = sample + coeff * s1 - s2
            s2 = s1
            s1 = s0
        mag = s1 * s1 + s2 * s2 - coeff * s1 * s2
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
