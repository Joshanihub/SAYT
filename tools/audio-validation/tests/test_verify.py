"""Tests for the SAYT M1 audio fixture verifier."""

from __future__ import annotations

import json
import shutil
import struct
import tempfile
import unittest
import wave
from pathlib import Path

import sys
sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

import generate_fixtures as gen  # noqa: E402
import verify_fixtures as ver    # noqa: E402


class TestVerifyHappyPath(unittest.TestCase):
    """Verifier passes on a correctly generated fixture set."""

    def setUp(self):
        self.tmpdir = tempfile.mkdtemp(prefix="sayt_ver_ok_")
        self.output_dir = Path(self.tmpdir)
        gen.generate(self.output_dir, force=True)

    def tearDown(self):
        shutil.rmtree(self.tmpdir, ignore_errors=True)

    def test_verify_all_passes(self):
        """Full verification returns 0 (success)."""
        result = ver.verify_all(self.output_dir)
        self.assertEqual(result, 0)

    def test_verify_individual_files(self):
        """Each file passes individual verification."""
        manifest = json.loads(
            (self.output_dir / "manifest.json").read_text())
        for entry in manifest["files"]:
            result = ver.verify_file(entry, self.output_dir)
            self.assertTrue(result.passed,
                            f"{entry['filename']} failed verification: "
                            + str([(n, d) for n, ok, d in result.checks if not ok]))


class TestVerifyMissingFile(unittest.TestCase):
    """Verifier detects and reports a missing WAV file."""

    def setUp(self):
        self.tmpdir = tempfile.mkdtemp(prefix="sayt_ver_miss_")
        self.output_dir = Path(self.tmpdir)
        gen.generate(self.output_dir, force=True)

    def tearDown(self):
        shutil.rmtree(self.tmpdir, ignore_errors=True)

    def test_missing_file_detected(self):
        """Removing a fixture file causes verification to fail."""
        # Remove one file
        (self.output_dir / "deck_a_440hz.wav").unlink()
        result = ver.verify_all(self.output_dir)
        self.assertEqual(result, 1)

    def test_missing_file_individual_check(self):
        """Individual verify_file reports file_exists failure."""
        (self.output_dir / "deck_a_440hz.wav").unlink()
        manifest = json.loads(
            (self.output_dir / "manifest.json").read_text())
        entry = next(e for e in manifest["files"]
                     if e["filename"] == "deck_a_440hz.wav")
        result = ver.verify_file(entry, self.output_dir)
        self.assertFalse(result.passed)
        file_check = next((n, ok) for n, ok, _ in result.checks
                          if n == "file_exists")
        self.assertFalse(file_check[1])


class TestVerifyModifiedFile(unittest.TestCase):
    """Verifier detects a file whose content has been altered."""

    def setUp(self):
        self.tmpdir = tempfile.mkdtemp(prefix="sayt_ver_mod_")
        self.output_dir = Path(self.tmpdir)
        gen.generate(self.output_dir, force=True)

    def tearDown(self):
        shutil.rmtree(self.tmpdir, ignore_errors=True)

    def test_modified_file_sha256_mismatch(self):
        """Altering file content causes SHA-256 failure."""
        filepath = self.output_dir / "deck_a_440hz.wav"
        data = filepath.read_bytes()
        # Flip a byte in the audio data region (after header)
        modified = data[:100] + bytes([data[100] ^ 0xFF]) + data[101:]
        filepath.write_bytes(modified)

        manifest = json.loads(
            (self.output_dir / "manifest.json").read_text())
        entry = next(e for e in manifest["files"]
                     if e["filename"] == "deck_a_440hz.wav")
        result = ver.verify_file(entry, self.output_dir)
        sha_check = next((n, ok) for n, ok, _ in result.checks
                         if n == "sha256")
        self.assertFalse(sha_check[1])


class TestVerifyInvalidWAV(unittest.TestCase):
    """Verifier detects files that are not valid WAV."""

    def setUp(self):
        self.tmpdir = tempfile.mkdtemp(prefix="sayt_ver_inv_")
        self.output_dir = Path(self.tmpdir)
        gen.generate(self.output_dir, force=True)

    def tearDown(self):
        shutil.rmtree(self.tmpdir, ignore_errors=True)

    def test_non_riff_file(self):
        """A file that is not RIFF is flagged."""
        filepath = self.output_dir / "deck_a_440hz.wav"
        filepath.write_bytes(b"NOT A WAV FILE AT ALL" * 100)
        manifest = json.loads(
            (self.output_dir / "manifest.json").read_text())
        entry = next(e for e in manifest["files"]
                     if e["filename"] == "deck_a_440hz.wav")
        result = ver.verify_file(entry, self.output_dir)
        self.assertFalse(result.passed)

    def test_truncated_file(self):
        """A truncated WAV is flagged."""
        filepath = self.output_dir / "deck_a_440hz.wav"
        data = filepath.read_bytes()
        filepath.write_bytes(data[:20])  # Only RIFF header fragment
        manifest = json.loads(
            (self.output_dir / "manifest.json").read_text())
        entry = next(e for e in manifest["files"]
                     if e["filename"] == "deck_a_440hz.wav")
        result = ver.verify_file(entry, self.output_dir)
        self.assertFalse(result.passed)


class TestVerifyMissingManifest(unittest.TestCase):
    """Verifier handles a missing manifest gracefully."""

    def test_no_manifest(self):
        """verify_all returns error code when manifest is absent."""
        tmpdir = tempfile.mkdtemp(prefix="sayt_ver_noman_")
        try:
            result = ver.verify_all(Path(tmpdir))
            self.assertEqual(result, 2)
        finally:
            shutil.rmtree(tmpdir, ignore_errors=True)


class TestVerifyClippingDetection(unittest.TestCase):
    """Verifier clipping detection catches full-scale samples."""

    def test_clipping_flagged(self):
        """A WAV with max-amplitude samples is flagged for clipping."""
        tmpdir = tempfile.mkdtemp(prefix="sayt_ver_clip_")
        try:
            filepath = Path(tmpdir) / "clipped.wav"
            # Write a WAV with samples at ±32767
            num_frames = 480
            samples = [(32767, -32767)] * num_frames
            gen._write_wav(filepath, samples, 48000, 2, 16)
            clipped = ver._detect_clipping(filepath, 2, 16)
            self.assertTrue(clipped)
        finally:
            shutil.rmtree(tmpdir, ignore_errors=True)

    def test_no_clipping_on_normal_tone(self):
        """Conservative-amplitude tone is not flagged for clipping."""
        tmpdir = tempfile.mkdtemp(prefix="sayt_ver_noclip_")
        try:
            filepath = Path(tmpdir) / "quiet.wav"
            samples = gen._generate_tone(440.0, 0.01, gen.CONSERVATIVE_AMPLITUDE)
            gen._write_wav(filepath, samples, 48000, 2, 16)
            clipped = ver._detect_clipping(filepath, 2, 16)
            self.assertFalse(clipped)
        finally:
            shutil.rmtree(tmpdir, ignore_errors=True)


class TestVerifyPathValidation(unittest.TestCase):
    """Verifier rejects absolute paths and directory traversal outside input_dir."""

    @classmethod
    def setUpClass(cls):
        cls.parent_dir = Path(tempfile.mkdtemp(prefix="sayt_ver_path_"))
        cls.fixture_dir = cls.parent_dir / "fixtures"
        cls.fixture_dir.mkdir()
        gen.generate(cls.fixture_dir, force=True)

        # Place a valid WAV outside fixture_dir to prove traversal cannot read it
        cls.outside_wav = cls.parent_dir / "outside.wav"
        shutil.copy2(cls.fixture_dir / "deck_a_440hz.wav", cls.outside_wav)
        cls.manifest_text = (cls.fixture_dir / "manifest.json").read_text()
        manifest = json.loads(cls.manifest_text)
        cls.valid_entry = dict(next(
            e for e in manifest["files"]
            if e["filename"] == "deck_a_440hz.wav"
        ))

    @classmethod
    def tearDownClass(cls):
        shutil.rmtree(cls.parent_dir, ignore_errors=True)

    def _assert_rejected(self, malicious_filename: object) -> None:
        entry = dict(self.valid_entry, filename=malicious_filename)
        result = ver.verify_file(entry, self.fixture_dir)
        self.assertFalse(result.passed,
                         f"Expected rejection for {malicious_filename!r}")
        self.assertEqual(result.checks[0][0], "safe_path")
        self.assertFalse(result.checks[0][1])
        # Verification must stop at safe_path without touching the filesystem
        self.assertEqual(len(result.checks), 1)

    def test_parent_traversal_rejected(self):
        self._assert_rejected("../outside.wav")

    def test_backslash_parent_traversal_rejected(self):
        self._assert_rejected(r"..\outside.wav")

    def test_nested_traversal_rejected(self):
        self._assert_rejected("sub/../../outside.wav")

    def test_absolute_path_rejected(self):
        self._assert_rejected(str(self.outside_wav.resolve()))

    def test_posix_root_path_rejected(self):
        self._assert_rejected("/etc/passwd")

    def test_windows_drive_relative_path_rejected(self):
        self._assert_rejected("C:outside.wav")

    def test_current_dir_dot_rejected(self):
        self._assert_rejected(".")

    def test_empty_and_non_string_filename_rejected(self):
        self._assert_rejected("")
        self._assert_rejected(None)

    def test_verify_all_fails_on_traversal_manifest(self):
        manifest_path = self.fixture_dir / "manifest.json"
        try:
            manifest = json.loads(self.manifest_text)
            manifest["files"][0]["filename"] = "../outside.wav"
            manifest_path.write_text(json.dumps(manifest))
            self.assertEqual(ver.verify_all(self.fixture_dir), 1)
        finally:
            manifest_path.write_text(self.manifest_text)

    def test_symlink_escape_rejected(self):
        link_path = self.fixture_dir / "escape_link.wav"
        try:
            link_path.symlink_to(self.outside_wav)
        except OSError:
            self.skipTest("Symlink creation requires elevated privileges on this host")
        try:
            self._assert_rejected("escape_link.wav")
        finally:
            link_path.unlink(missing_ok=True)


if __name__ == "__main__":
    unittest.main()
