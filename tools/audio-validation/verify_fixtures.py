#!/usr/bin/env python3
"""SAYT M1 Audio Verification — Fixture Verifier.

Reads *manifest.json* produced by :pymod:`generate_fixtures` and confirms
that every WAV file exists, has correct headers, matches the manifest's
metadata, and has not been modified.

Usage:
    py -3 tools/audio-validation/verify_fixtures.py \\
        --input-dir "$env:TEMP\\SAYT-Audio-Fixtures"

Exit codes:
    0  — all checks passed
    1  — one or more checks failed
    2  — usage / IO error
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import struct
import sys
import wave
from pathlib import Path, PurePosixPath, PureWindowsPath

# ---------------------------------------------------------------------------
# Constants (must match generator)
# ---------------------------------------------------------------------------

MAX_SAMPLE_VALUE = 32767
# Allowed frame-count deviation: the manifest stores the exact generator
# frame count, but we tolerate ±1 frame for rounding in duration calculations.
FRAME_COUNT_TOLERANCE = 1

# Clipping threshold: a 16-bit sample at ±32767 is full-scale.  We flag
# clipping if the peak reaches full-scale.
CLIP_THRESHOLD_16BIT = 32767


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _sha256(path: Path) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(65536), b""):
            h.update(chunk)
    return h.hexdigest()


def _read_raw_fmt(path: Path) -> dict:
    """Read WAV header fields by parsing the RIFF structure directly.

    Python's :mod:`wave` module validates basic structure; this adds the
    explicit PCM format-tag check.
    """
    with open(path, "rb") as f:
        riff = f.read(4)
        if riff != b"RIFF":
            return {"error": "Not a RIFF file"}
        f.read(4)  # file size
        wav_id = f.read(4)
        if wav_id != b"WAVE":
            return {"error": "Not a WAVE file"}

        # Walk chunks looking for 'fmt '
        while True:
            chunk_hdr = f.read(8)
            if len(chunk_hdr) < 8:
                return {"error": "fmt chunk not found"}
            chunk_id = chunk_hdr[:4]
            chunk_size = struct.unpack("<I", chunk_hdr[4:])[0]
            if chunk_id == b"fmt ":
                if chunk_size < 16:
                    return {"error": "fmt chunk too small"}
                fmt_data = f.read(16)
                if len(fmt_data) < 16:
                    return {"error": "fmt chunk truncated"}
                audio_format = struct.unpack("<H", fmt_data[0:2])[0]
                return {"audio_format": audio_format}
            else:
                f.seek(chunk_size, 1)


def _detect_clipping(path: Path, num_channels: int,
                     bits_per_sample: int) -> bool:
    """Return True if any sample is at full-scale (potential clipping)."""
    if bits_per_sample != 16:
        return False  # only 16-bit detection implemented

    with wave.open(str(path), "rb") as wf:
        n_frames = wf.getnframes()
        raw = wf.readframes(n_frames)

    fmt = "<" + "h" * (n_frames * num_channels)
    samples = struct.unpack(fmt, raw)
    peak = max(abs(s) for s in samples) if samples else 0
    return peak >= CLIP_THRESHOLD_16BIT


# ---------------------------------------------------------------------------
# Verification engine
# ---------------------------------------------------------------------------

class VerificationResult:
    """Accumulates pass/fail checks for a single file or the whole suite."""

    def __init__(self) -> None:
        self.checks: list[tuple[str, bool, str]] = []  # (name, passed, detail)

    def check(self, name: str, passed: bool, detail: str = "") -> None:
        self.checks.append((name, passed, detail))

    @property
    def passed(self) -> bool:
        return all(ok for _, ok, _ in self.checks)

    @property
    def num_passed(self) -> int:
        return sum(1 for _, ok, _ in self.checks if ok)

    @property
    def num_failed(self) -> int:
        return sum(1 for _, ok, _ in self.checks if not ok)


def _resolve_safe_fixture_path(filename: object,
                               input_dir: Path) -> tuple[Path | None, str]:
    """Validate *filename* and resolve it strictly within *input_dir*.

    Rejects non-strings, empty strings, null bytes, absolute or drive/root-
    anchored paths, parent-directory traversal (``..``), and symlinks or paths
    whose resolved target escapes *input_dir*.
    """
    if not isinstance(filename, str) or not filename:
        return None, "Manifest filename must be a non-empty string"

    if "\x00" in filename:
        return None, f"Invalid null byte in filename: {filename!r}"

    # Check both Windows and POSIX path semantics so traversal or rooted paths
    # from a foreign OS cannot sneak through on any platform.
    win_path = PureWindowsPath(filename)
    posix_path = PurePosixPath(filename)
    if (
        win_path.is_absolute()
        or posix_path.is_absolute()
        or bool(win_path.drive)
        or bool(win_path.root)
        or bool(posix_path.root)
        or ".." in win_path.parts
        or ".." in posix_path.parts
    ):
        return None, f"Unsafe path rejected (absolute or traversal): {filename}"

    base_dir = input_dir.resolve()
    candidate = Path(filename)
    resolved_path = (base_dir / candidate).resolve()

    if resolved_path == base_dir or not resolved_path.is_relative_to(base_dir):
        return None, f"Path escapes fixture directory: {filename}"

    return resolved_path, "Path confined within fixture directory"


def verify_file(entry: dict, input_dir: Path) -> VerificationResult:
    """Verify a single fixture file against its manifest entry."""
    result = VerificationResult()
    filename = entry.get("filename") if isinstance(entry, dict) else None

    # 0. Strict path confinement
    filepath, path_detail = _resolve_safe_fixture_path(filename, input_dir)
    if filepath is None:
        result.check("safe_path", False, path_detail)
        return result
    result.check("safe_path", True, path_detail)

    # 1. File existence
    result.check("file_exists", filepath.exists(),
                 f"{filename} {'found' if filepath.exists() else 'MISSING'}")
    if not filepath.exists():
        return result

    # 2. RIFF/WAVE structure and PCM format tag
    raw_fmt = _read_raw_fmt(filepath)
    if "error" in raw_fmt:
        result.check("wav_structure", False, raw_fmt["error"])
        return result
    result.check("wav_structure", True, "Valid RIFF/WAVE")

    is_pcm = raw_fmt["audio_format"] == 1
    result.check("pcm_encoding", is_pcm,
                 f"Format tag: {raw_fmt['audio_format']} "
                 f"({'PCM' if is_pcm else 'NOT PCM'})")
    if not is_pcm:
        return result

    # 3–7. WAV header fields via wave module
    try:
        with wave.open(str(filepath), "rb") as wf:
            actual_rate = wf.getframerate()
            actual_channels = wf.getnchannels()
            actual_sampwidth = wf.getsampwidth()
            actual_frames = wf.getnframes()
    except wave.Error as exc:
        result.check("wav_readable", False, str(exc))
        return result
    result.check("wav_readable", True, "wave module opened successfully")

    expected_rate = entry["sample_rate"]
    result.check("sample_rate",
                 actual_rate == expected_rate,
                 f"Expected {expected_rate}, got {actual_rate}")

    expected_channels = entry["channels"]
    result.check("channel_count",
                 actual_channels == expected_channels,
                 f"Expected {expected_channels}, got {actual_channels}")

    expected_bits = entry["bit_depth"]
    actual_bits = actual_sampwidth * 8
    result.check("bit_depth",
                 actual_bits == expected_bits,
                 f"Expected {expected_bits}, got {actual_bits}")

    expected_frames = entry["frame_count"]
    frame_ok = abs(actual_frames - expected_frames) <= FRAME_COUNT_TOLERANCE
    result.check("frame_count", frame_ok,
                 f"Expected {expected_frames} +/-{FRAME_COUNT_TOLERANCE}, "
                 f"got {actual_frames}")

    # 8. SHA-256
    actual_hash = _sha256(filepath)
    expected_hash = entry["sha256"]
    result.check("sha256",
                 actual_hash == expected_hash,
                 "Match" if actual_hash == expected_hash
                 else f"MISMATCH: expected {expected_hash[:16]}..., "
                      f"got {actual_hash[:16]}...")

    # 9. Clipping detection
    if actual_bits == 16:
        clipped = _detect_clipping(filepath, actual_channels, actual_bits)
        result.check("no_clipping", not clipped,
                     "No full-scale samples" if not clipped
                     else "WARNING: full-scale samples detected (potential clipping)")

    return result


def verify_all(input_dir: Path) -> int:
    """Verify all fixtures.  Return 0 on success, 1 on failure."""
    manifest_path = input_dir / "manifest.json"
    if not manifest_path.exists():
        print(f"Error: manifest.json not found in {input_dir}", file=sys.stderr)
        return 2

    try:
        manifest = json.loads(manifest_path.read_text())
    except (json.JSONDecodeError, OSError) as exc:
        print(f"Error reading manifest: {exc}", file=sys.stderr)
        return 2

    files = manifest.get("files", [])
    if not files:
        print("Error: manifest contains no file entries.", file=sys.stderr)
        return 2

    total_passed = 0
    total_failed = 0
    all_ok = True

    print(f"SAYT M1 Fixture Verifier")
    print(f"Input: {input_dir.resolve()}")
    print(f"Generator: {manifest.get('generator', '?')} "
          f"v{manifest.get('generator_version', '?')}")
    print(f"Files to verify: {len(files)}\n")

    for entry in files:
        filename = entry.get("filename", "<invalid>") if isinstance(entry, dict) else "<invalid>"
        result = verify_file(entry, input_dir)
        status = "PASS" if result.passed else "FAIL"
        icon = "[OK]" if result.passed else "[FAIL]"
        print(f"  {icon} {filename}: {status}")
        for name, ok, detail in result.checks:
            marker = "  [OK]" if ok else "  [FAIL]"
            print(f"    {marker} {name}: {detail}")

        total_passed += result.num_passed
        total_failed += result.num_failed
        if not result.passed:
            all_ok = False

    print(f"\nSummary: {total_passed} checks passed, "
          f"{total_failed} checks failed across {len(files)} files.")

    if all_ok:
        print("Result: ALL CHECKS PASSED")
        return 0
    else:
        print("Result: VERIFICATION FAILED", file=sys.stderr)
        return 1


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------

def main() -> None:
    parser = argparse.ArgumentParser(
        description="Verify SAYT M1 audio fixtures against their manifest.",
    )
    parser.add_argument(
        "--input-dir", type=Path, required=True,
        help="Directory containing generated WAV files and manifest.json.",
    )
    args = parser.parse_args()

    if not args.input_dir.is_dir():
        print(f"Error: {args.input_dir} is not a directory.", file=sys.stderr)
        sys.exit(2)

    sys.exit(verify_all(args.input_dir))


if __name__ == "__main__":
    main()
