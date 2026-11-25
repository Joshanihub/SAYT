"""Tests for the SAYT M1 audio fixture generator."""

from __future__ import annotations

import hashlib
import json
import math
import os
import struct
import tempfile
import unittest
import wave
from pathlib import Path

# Adjust import path so we can import the generator from the parent dir.
import sys
sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

import generate_fixtures as gen  # noqa: E402


class TestSignalHelpers(unittest.TestCase):
    """Unit tests for low-level signal-generation functions."""

    def test_fade_envelope_start(self):
        """Fade-in envelope starts at 0."""
        self.assertAlmostEqual(gen._fade_envelope(0, 1000, 100), 0.0)

    def test_fade_envelope_middle(self):
        """Envelope is 1.0 in the sustain region."""
        self.assertAlmostEqual(gen._fade_envelope(500, 1000, 100), 1.0)

    def test_fade_envelope_end(self):
        """Fade-out envelope reaches 0 at the last sample."""
        self.assertAlmostEqual(gen._fade_envelope(999, 1000, 100), 0.0)

    def test_fade_envelope_at_fade_boundary(self):
        """Envelope reaches 1.0 exactly at the end of the fade-in region."""
        self.assertAlmostEqual(gen._fade_envelope(100, 1000, 100), 1.0)

    def test_generate_tone_length(self):
        """Tone has the expected number of frames."""
        samples = gen._generate_tone(440.0, 1.0, 0.5, sample_rate=48000)
        self.assertEqual(len(samples), 48000)

    def test_generate_tone_no_clipping(self):
        """No sample exceeds MAX_SAMPLE_VALUE even at full amplitude."""
        samples = gen._generate_tone(440.0, 0.1, 1.0, sample_rate=48000)
        for left, right in samples:
            self.assertLessEqual(abs(left), gen.MAX_SAMPLE_VALUE)
            self.assertLessEqual(abs(right), gen.MAX_SAMPLE_VALUE)

    def test_generate_tone_stereo_channels(self):
        """Different left/right frequencies produce different channels."""
        samples = gen._generate_tone(
            0.0, 0.05, 0.5, sample_rate=48000,
            left_freq=440.0, right_freq=880.0,
        )
        # After fade-in, check a range of samples for channel difference.
        # Individual samples may coincide at zero crossings, so we check
        # that at least some samples in the sustain region differ.
        fade_len = int(48000 * gen.FADE_SECONDS)
        sustain_start = fade_len + 10
        sustain_end = len(samples) - fade_len - 10
        any_differ = any(
            samples[i][0] != samples[i][1]
            for i in range(sustain_start, min(sustain_end, sustain_start + 200))
        )
        self.assertTrue(any_differ, "L and R channels are identical in sustain")

    def test_generate_silence(self):
        """Silence is all zeros."""
        samples = gen._generate_silence(0.1, sample_rate=48000)
        self.assertEqual(len(samples), 4800)
        for left, right in samples:
            self.assertEqual(left, 0)
            self.assertEqual(right, 0)

    def test_fade_in_increases(self):
        """Samples during fade-in should increase in absolute value."""
        samples = gen._generate_tone(440.0, 0.1, 0.5, sample_rate=48000)
        fade_len = int(48000 * gen.FADE_SECONDS)
        # Check that the envelope at sample 1 is less than at fade_len-1
        # (by checking absolute values, accounting for sine phase)
        self.assertTrue(fade_len > 2)
        # The first sample has envelope ≈ 0, mid-fade has envelope ≈ 0.5
        env_start = gen._fade_envelope(1, len(samples), fade_len)
        env_mid = gen._fade_envelope(fade_len // 2, len(samples), fade_len)
        self.assertLess(env_start, env_mid)

    def test_fade_out_decreases(self):
        """Samples during fade-out should decrease toward zero."""
        total = 4800
        fade_len = int(48000 * gen.FADE_SECONDS)
        env_before_fade = gen._fade_envelope(total - fade_len - 1, total, fade_len)
        env_in_fade = gen._fade_envelope(total - 2, total, fade_len)
        self.assertGreater(env_before_fade, env_in_fade)


class TestWAVGeneration(unittest.TestCase):
    """Integration tests: generate WAV files and inspect their headers."""

    def setUp(self):
        self.tmpdir = tempfile.mkdtemp(prefix="sayt_gen_test_")
        self.output_dir = Path(self.tmpdir)

    def tearDown(self):
        import shutil
        shutil.rmtree(self.tmpdir, ignore_errors=True)

    def test_generate_all_fixtures(self):
        """Generator creates all expected WAV files and a manifest."""
        gen.generate(self.output_dir, force=True)
        manifest_path = self.output_dir / "manifest.json"
        self.assertTrue(manifest_path.exists())

        manifest = json.loads(manifest_path.read_text())
        filenames = {e["filename"] for e in manifest["files"]}
        expected = {f["filename"] for f in gen.FIXTURES}
        self.assertEqual(filenames, expected)

        for entry in manifest["files"]:
            self.assertTrue((self.output_dir / entry["filename"]).exists())

    def test_wav_headers(self):
        """Each generated WAV has correct sample rate, channels, bit depth."""
        gen.generate(self.output_dir, force=True)
        for fixture in gen.FIXTURES:
            filepath = self.output_dir / fixture["filename"]
            with wave.open(str(filepath), "rb") as wf:
                self.assertEqual(wf.getframerate(), gen.SAMPLE_RATE,
                                 f"{fixture['filename']} sample rate")
                self.assertEqual(wf.getnchannels(), gen.NUM_CHANNELS,
                                 f"{fixture['filename']} channels")
                self.assertEqual(wf.getsampwidth(), gen.BYTES_PER_SAMPLE,
                                 f"{fixture['filename']} sample width")

    def test_duration_approximate(self):
        """Each fixture's duration matches the spec within 0.1s."""
        gen.generate(self.output_dir, force=True)
        for fixture in gen.FIXTURES:
            filepath = self.output_dir / fixture["filename"]
            with wave.open(str(filepath), "rb") as wf:
                actual_dur = wf.getnframes() / wf.getframerate()
            expected_dur = fixture["duration_sec"]
            self.assertAlmostEqual(actual_dur, expected_dur, delta=0.1,
                                   msg=f"{fixture['filename']} duration")

    def test_no_clipping_in_tones(self):
        """No tonal fixture has samples at full-scale (±32767)."""
        gen.generate(self.output_dir, force=True)
        for fixture in gen.FIXTURES:
            if fixture.get("silence"):
                continue
            filepath = self.output_dir / fixture["filename"]
            with wave.open(str(filepath), "rb") as wf:
                raw = wf.readframes(wf.getnframes())
            fmt = "<" + "h" * (len(raw) // 2)
            samples = struct.unpack(fmt, raw)
            peak = max(abs(s) for s in samples)
            self.assertLess(peak, gen.MAX_SAMPLE_VALUE,
                            f"{fixture['filename']} has clipping")

    def test_silence_is_zero(self):
        """Silence fixture contains only zero samples."""
        gen.generate(self.output_dir, force=True)
        filepath = self.output_dir / "silence_reference.wav"
        with wave.open(str(filepath), "rb") as wf:
            raw = wf.readframes(wf.getnframes())
        fmt = "<" + "h" * (len(raw) // 2)
        samples = struct.unpack(fmt, raw)
        self.assertTrue(all(s == 0 for s in samples))

    def test_stereo_channel_id_different(self):
        """Left and right channels of stereo_channel_id.wav differ."""
        gen.generate(self.output_dir, force=True)
        filepath = self.output_dir / "stereo_channel_id.wav"
        with wave.open(str(filepath), "rb") as wf:
            raw = wf.readframes(wf.getnframes())
            nframes = wf.getnframes()
        fmt = "<" + "h" * (nframes * 2)
        samples = struct.unpack(fmt, raw)
        # Check a range of samples in the sustain region for L/R difference.
        # Individual frames may coincide at zero crossings.
        fade_frames = int(gen.SAMPLE_RATE * gen.FADE_SECONDS)
        start = fade_frames + 100
        any_differ = any(
            samples[i * 2] != samples[i * 2 + 1]
            for i in range(start, start + 500)
        )
        self.assertTrue(any_differ, "L and R channels are identical")

    def test_manifest_sha256(self):
        """Manifest SHA-256 entries match actual file hashes."""
        gen.generate(self.output_dir, force=True)
        manifest = json.loads((self.output_dir / "manifest.json").read_text())
        for entry in manifest["files"]:
            filepath = self.output_dir / entry["filename"]
            h = hashlib.sha256()
            with open(filepath, "rb") as f:
                for chunk in iter(lambda: f.read(65536), b""):
                    h.update(chunk)
            self.assertEqual(h.hexdigest(), entry["sha256"],
                             f"{entry['filename']} SHA-256 mismatch")

    def test_manifest_metadata(self):
        """Manifest entries contain all required fields."""
        gen.generate(self.output_dir, force=True)
        manifest = json.loads((self.output_dir / "manifest.json").read_text())
        required_fields = {
            "filename", "purpose", "sample_rate", "channels", "bit_depth",
            "encoding", "frame_count", "duration_seconds",
            "peak_sample_value", "peak_dbfs", "sha256", "generator_version",
        }
        for entry in manifest["files"]:
            self.assertTrue(required_fields.issubset(entry.keys()),
                            f"{entry['filename']} missing manifest fields: "
                            f"{required_fields - entry.keys()}")

    def test_pcm_format_tag(self):
        """Every generated WAV has PCM format tag (1) in the fmt chunk."""
        gen.generate(self.output_dir, force=True)
        for fixture in gen.FIXTURES:
            filepath = self.output_dir / fixture["filename"]
            with open(filepath, "rb") as f:
                f.read(4)  # RIFF
                f.read(4)  # size
                f.read(4)  # WAVE
                # Find fmt chunk
                while True:
                    chunk_id = f.read(4)
                    chunk_size = struct.unpack("<I", f.read(4))[0]
                    if chunk_id == b"fmt ":
                        fmt_data = f.read(16)
                        audio_format = struct.unpack("<H", fmt_data[0:2])[0]
                        self.assertEqual(audio_format, 1,
                                         f"{fixture['filename']} is not PCM")
                        break
                    f.seek(chunk_size, 1)


class TestReproducibility(unittest.TestCase):
    """Verify that the generator produces identical output on repeated runs."""

    def test_deterministic_generation(self):
        """Two runs produce identical SHA-256 hashes for every file."""
        tmpdir1 = tempfile.mkdtemp(prefix="sayt_repro1_")
        tmpdir2 = tempfile.mkdtemp(prefix="sayt_repro2_")
        try:
            gen.generate(Path(tmpdir1), force=True)
            gen.generate(Path(tmpdir2), force=True)

            m1 = json.loads((Path(tmpdir1) / "manifest.json").read_text())
            m2 = json.loads((Path(tmpdir2) / "manifest.json").read_text())

            for e1, e2 in zip(m1["files"], m2["files"]):
                self.assertEqual(e1["sha256"], e2["sha256"],
                                 f"{e1['filename']} differs between runs")
        finally:
            import shutil
            shutil.rmtree(tmpdir1, ignore_errors=True)
            shutil.rmtree(tmpdir2, ignore_errors=True)


class TestOutputDirectorySafety(unittest.TestCase):
    """Verify that the generator refuses to overwrite foreign files."""

    def test_refuses_nonempty_foreign_dir(self):
        """Generator exits when the output dir has non-generator files."""
        tmpdir = tempfile.mkdtemp(prefix="sayt_safety_")
        try:
            (Path(tmpdir) / "my_music.wav").write_text("not a real wav")
            with self.assertRaises(SystemExit):
                gen.generate(Path(tmpdir), force=False)
        finally:
            import shutil
            shutil.rmtree(tmpdir, ignore_errors=True)

    def test_force_overwrites_own_files(self):
        """With --force, generator overwrites its own previous output."""
        tmpdir = tempfile.mkdtemp(prefix="sayt_force_")
        try:
            gen.generate(Path(tmpdir), force=True)
            # Should succeed on second run with force
            gen.generate(Path(tmpdir), force=True)
            manifest = json.loads((Path(tmpdir) / "manifest.json").read_text())
            self.assertTrue(len(manifest["files"]) > 0)
        finally:
            import shutil
            shutil.rmtree(tmpdir, ignore_errors=True)

    def test_force_rejects_symlink_destination_without_touching_target(self):
        """A generated filename symlink cannot redirect writes outside output."""
        tmpdir = tempfile.mkdtemp(prefix="sayt_symlink_output_")
        try:
            root = Path(tmpdir)
            output_dir = root / "fixtures"
            output_dir.mkdir()
            protected_file = root / "protected.txt"
            protected_content = b"user-owned file must remain unchanged"
            protected_file.write_bytes(protected_content)
            link_path = output_dir / gen.FIXTURES[0]["filename"]
            try:
                link_path.symlink_to(protected_file)
            except OSError:
                self.skipTest("Creating file symlinks is unavailable on this host")

            with self.assertRaises(SystemExit):
                gen.generate(output_dir, force=True)

            self.assertEqual(protected_file.read_bytes(), protected_content)
            self.assertFalse((output_dir / gen._SENTINEL).exists())
        finally:
            import shutil
            shutil.rmtree(tmpdir, ignore_errors=True)

    def test_force_rejects_directory_destination_before_writing(self):
        """A directory at an owned filename fails before any outputs are made."""
        tmpdir = tempfile.mkdtemp(prefix="sayt_directory_output_")
        try:
            output_dir = Path(tmpdir)
            (output_dir / gen.FIXTURES[0]["filename"]).mkdir()
            with self.assertRaises(SystemExit):
                gen.generate(output_dir, force=True)
            self.assertFalse((output_dir / gen._SENTINEL).exists())
        finally:
            import shutil
            shutil.rmtree(tmpdir, ignore_errors=True)

    def test_force_replaces_hardlink_without_modifying_link_target(self):
        """Atomic replacement avoids writing through an owned-name hard link."""
        tmpdir = tempfile.mkdtemp(prefix="sayt_hardlink_output_")
        try:
            root = Path(tmpdir)
            protected_file = root / "protected.bin"
            original_content = b"user-owned linked data"
            protected_file.write_bytes(original_content)
            fixture_path = root / gen.FIXTURES[0]["filename"]
            try:
                os.link(protected_file, fixture_path)
            except OSError:
                self.skipTest("Creating hard links is unavailable on this host")

            gen.generate(root, force=True)

            self.assertEqual(protected_file.read_bytes(), original_content)
            with wave.open(str(fixture_path), "rb") as wav_file:
                self.assertEqual(wav_file.getframerate(), gen.SAMPLE_RATE)
        finally:
            import shutil
            shutil.rmtree(tmpdir, ignore_errors=True)


if __name__ == "__main__":
    unittest.main()
