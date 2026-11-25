#!/usr/bin/env python3
"""SAYT M1 Audio Verification — Fixture Generator.

Generates reproducible, uncompressed PCM WAV files with known acoustic
properties for manual and automated verification of the SAYT two-deck
audio engine.

**Safety warning** — these files produce audible tones.  Start playback
at LOW volume and increase gradually.  Stop immediately if output sounds
unexpectedly loud or distorted.  Do not use at high power on headphones
or speakers.

Usage:
    py -3 tools/audio-validation/generate_fixtures.py \\
        --output-dir "$env:TEMP\\SAYT-Audio-Fixtures"

The generator is deterministic: the same version and options always
produce bit-identical WAV data.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
import struct
import sys
from pathlib import Path

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

GENERATOR_VERSION = "1.0.0"
GENERATOR_ID = "sayt-m1-audio-validation"

# Shared WAV parameters
SAMPLE_RATE = 48_000
NUM_CHANNELS = 2
BITS_PER_SAMPLE = 16
BYTES_PER_SAMPLE = BITS_PER_SAMPLE // 8

# Amplitude: -18 dBFS ≈ 10^(-18/20) ≈ 0.12589
CONSERVATIVE_AMPLITUDE = 10 ** (-18.0 / 20.0)

# Fade duration in seconds (to suppress onset/offset clicks)
FADE_SECONDS = 0.010  # 10 ms

# Maximum representable sample value for 16-bit signed PCM
MAX_SAMPLE_VALUE = 32767


# ---------------------------------------------------------------------------
# WAV writing (standard library only)
# ---------------------------------------------------------------------------

def _write_wav(path: Path, samples_lr: list[tuple[int, int]],
               sample_rate: int, num_channels: int,
               bits_per_sample: int) -> None:
    """Write an uncompressed PCM WAV file from interleaved sample pairs."""
    num_frames = len(samples_lr)
    bytes_per_sample = bits_per_sample // 8
    block_align = num_channels * bytes_per_sample
    byte_rate = sample_rate * block_align
    data_size = num_frames * block_align

    fmt_chunk_size = 16
    # RIFF header (12) + fmt chunk (8 + 16) + data chunk header (8) + data
    riff_size = 4 + (8 + fmt_chunk_size) + (8 + data_size)

    with open(path, "wb") as f:
        # RIFF header
        f.write(b"RIFF")
        f.write(struct.pack("<I", riff_size))
        f.write(b"WAVE")

        # fmt sub-chunk
        f.write(b"fmt ")
        f.write(struct.pack("<I", fmt_chunk_size))
        f.write(struct.pack("<H", 1))               # PCM format
        f.write(struct.pack("<H", num_channels))
        f.write(struct.pack("<I", sample_rate))
        f.write(struct.pack("<I", byte_rate))
        f.write(struct.pack("<H", block_align))
        f.write(struct.pack("<H", bits_per_sample))

        # data sub-chunk
        f.write(b"data")
        f.write(struct.pack("<I", data_size))

        # Sample data — pack as little-endian signed 16-bit
        fmt = "<" + "h" * (num_frames * num_channels)
        interleaved: list[int] = []
        for left, right in samples_lr:
            interleaved.append(left)
            interleaved.append(right)
        f.write(struct.pack(fmt, *interleaved))


# ---------------------------------------------------------------------------
# Signal generation helpers
# ---------------------------------------------------------------------------

def _fade_envelope(frame_idx: int, total_frames: int,
                   fade_samples: int) -> float:
    """Return a [0.0, 1.0] envelope with linear fade-in/out."""
    if frame_idx < fade_samples:
        return frame_idx / fade_samples
    if frame_idx >= total_frames - fade_samples:
        return (total_frames - 1 - frame_idx) / fade_samples
    return 1.0


def _generate_tone(freq_hz: float, duration_sec: float,
                   amplitude: float,
                   sample_rate: int = SAMPLE_RATE,
                   left_freq: float | None = None,
                   right_freq: float | None = None) -> list[tuple[int, int]]:
    """Generate a stereo tone with optional per-channel frequencies.

    If *left_freq* and *right_freq* are None, both channels use *freq_hz*.
    """
    total_frames = int(sample_rate * duration_sec)
    fade_samples = int(sample_rate * FADE_SECONDS)
    lf = left_freq if left_freq is not None else freq_hz
    rf = right_freq if right_freq is not None else freq_hz

    samples: list[tuple[int, int]] = []
    for n in range(total_frames):
        env = _fade_envelope(n, total_frames, fade_samples)
        left_val = amplitude * env * math.sin(2.0 * math.pi * lf * n / sample_rate)
        right_val = amplitude * env * math.sin(2.0 * math.pi * rf * n / sample_rate)
        left_int = max(-MAX_SAMPLE_VALUE, min(MAX_SAMPLE_VALUE,
                                               int(round(left_val * MAX_SAMPLE_VALUE))))
        right_int = max(-MAX_SAMPLE_VALUE, min(MAX_SAMPLE_VALUE,
                                                int(round(right_val * MAX_SAMPLE_VALUE))))
        samples.append((left_int, right_int))
    return samples


def _generate_silence(duration_sec: float,
                      sample_rate: int = SAMPLE_RATE) -> list[tuple[int, int]]:
    """Generate stereo digital silence."""
    total_frames = int(sample_rate * duration_sec)
    return [(0, 0)] * total_frames


# ---------------------------------------------------------------------------
# Fixture definitions
# ---------------------------------------------------------------------------

FIXTURES: list[dict] = [
    {
        "filename": "deck_a_440hz.wav",
        "purpose": "Deck A identification tone — 440 Hz stereo sine",
        "freq_hz": 440.0,
        "duration_sec": 8.0,
        "amplitude": CONSERVATIVE_AMPLITUDE,
    },
    {
        "filename": "deck_b_880hz.wav",
        "purpose": "Deck B identification tone — 880 Hz stereo sine",
        "freq_hz": 880.0,
        "duration_sec": 8.0,
        "amplitude": CONSERVATIVE_AMPLITUDE,
    },
    {
        "filename": "eq_low_100hz.wav",
        "purpose": "EQ verification — 100 Hz low-band tone",
        "freq_hz": 100.0,
        "duration_sec": 6.0,
        "amplitude": CONSERVATIVE_AMPLITUDE,
    },
    {
        "filename": "eq_mid_1000hz.wav",
        "purpose": "EQ verification — 1000 Hz mid-band tone",
        "freq_hz": 1000.0,
        "duration_sec": 6.0,
        "amplitude": CONSERVATIVE_AMPLITUDE,
    },
    {
        "filename": "eq_high_5000hz.wav",
        "purpose": "EQ verification — 5000 Hz high-band tone",
        "freq_hz": 5000.0,
        "duration_sec": 6.0,
        "amplitude": CONSERVATIVE_AMPLITUDE,
    },
    {
        "filename": "stereo_channel_id.wav",
        "purpose": "Stereo channel identification — L:440 Hz, R:660 Hz",
        "freq_hz": 0.0,  # unused; per-channel freqs below
        "left_freq": 440.0,
        "right_freq": 660.0,
        "duration_sec": 8.0,
        "amplitude": CONSERVATIVE_AMPLITUDE,
    },
    {
        "filename": "silence_reference.wav",
        "purpose": "Reference digital silence for leak/residual checks",
        "freq_hz": 0.0,
        "duration_sec": 5.0,
        "amplitude": 0.0,
        "silence": True,
    },
]


# ---------------------------------------------------------------------------
# Manifest and hashing
# ---------------------------------------------------------------------------

def _sha256(path: Path) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(65536), b""):
            h.update(chunk)
    return h.hexdigest()


def _peak_sample(samples: list[tuple[int, int]]) -> int:
    """Return the maximum absolute sample value across both channels."""
    peak = 0
    for left, right in samples:
        peak = max(peak, abs(left), abs(right))
    return peak


def _peak_dbfs(peak_sample: int) -> float:
    """Convert a peak sample value to dBFS (16-bit)."""
    if peak_sample == 0:
        return float("-inf")
    return 20.0 * math.log10(peak_sample / MAX_SAMPLE_VALUE)


def _build_manifest_entry(fixture: dict, path: Path,
                          samples: list[tuple[int, int]]) -> dict:
    peak = _peak_sample(samples)
    return {
        "filename": fixture["filename"],
        "purpose": fixture["purpose"],
        "sample_rate": SAMPLE_RATE,
        "channels": NUM_CHANNELS,
        "bit_depth": BITS_PER_SAMPLE,
        "encoding": "PCM",
        "frame_count": len(samples),
        "duration_seconds": round(len(samples) / SAMPLE_RATE, 6),
        "peak_sample_value": peak,
        "peak_dbfs": round(_peak_dbfs(peak), 2) if peak > 0 else None,
        "sha256": _sha256(path),
        "generator_version": GENERATOR_VERSION,
    }


# ---------------------------------------------------------------------------
# Main generation logic
# ---------------------------------------------------------------------------

# Sentinel file written alongside fixtures so the generator can identify
# directories it previously created.
_SENTINEL = ".sayt-audio-fixtures"


def _generated_filenames() -> set[str]:
    """Return the set of filenames the generator creates (WAV + manifest + sentinel)."""
    names = {f["filename"] for f in FIXTURES}
    names.add("manifest.json")
    names.add(_SENTINEL)
    return names


def generate(output_dir: Path, *, force: bool = False) -> Path:
    """Generate all fixtures into *output_dir*.  Return path to manifest."""
    output_dir = output_dir.resolve()

    if output_dir.exists():
        if not output_dir.is_dir():
            print(f"Error: {output_dir} exists but is not a directory.",
                  file=sys.stderr)
            sys.exit(1)

        existing = set(os.listdir(output_dir))
        our_files = _generated_filenames()
        foreign = existing - our_files

        if foreign and not force:
            print(f"Error: output directory {output_dir} contains files not "
                  f"created by this generator:\n  "
                  + "\n  ".join(sorted(foreign)),
                  file=sys.stderr)
            print("\nUse --force to overwrite only generator-owned files in "
                  "this directory.", file=sys.stderr)
            sys.exit(1)

        if existing and not force:
            # Directory has only our files — still require --force to overwrite
            sentinel = output_dir / _SENTINEL
            if not sentinel.exists():
                print(f"Error: output directory {output_dir} is not empty and "
                      f"was not created by this generator.\n"
                      f"Use --force to overwrite.", file=sys.stderr)
                sys.exit(1)
    else:
        output_dir.mkdir(parents=True)

    # Write sentinel
    (output_dir / _SENTINEL).write_text(
        f"Generated by {GENERATOR_ID} v{GENERATOR_VERSION}\n"
        f"Do not edit — used to identify generator-owned directories.\n"
    )

    manifest_entries: list[dict] = []

    for fixture in FIXTURES:
        filepath = output_dir / fixture["filename"]
        is_silence = fixture.get("silence", False)

        if is_silence:
            samples = _generate_silence(fixture["duration_sec"])
        else:
            samples = _generate_tone(
                freq_hz=fixture["freq_hz"],
                duration_sec=fixture["duration_sec"],
                amplitude=fixture["amplitude"],
                left_freq=fixture.get("left_freq"),
                right_freq=fixture.get("right_freq"),
            )

        _write_wav(filepath, samples, SAMPLE_RATE, NUM_CHANNELS,
                   BITS_PER_SAMPLE)
        entry = _build_manifest_entry(fixture, filepath, samples)
        manifest_entries.append(entry)
        print(f"  ✓ {fixture['filename']}  "
              f"({entry['frame_count']} frames, "
              f"{entry['duration_seconds']}s)")

    manifest = {
        "generator": GENERATOR_ID,
        "generator_version": GENERATOR_VERSION,
        "description": (
            "SAYT M1 audio verification fixtures — synthetic test tones "
            "for manual two-deck playback testing.  These files are NOT "
            "music intended for publication."
        ),
        "sample_rate": SAMPLE_RATE,
        "channels": NUM_CHANNELS,
        "bit_depth": BITS_PER_SAMPLE,
        "encoding": "PCM",
        "files": manifest_entries,
    }

    manifest_path = output_dir / "manifest.json"
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n")
    print(f"  ✓ manifest.json")

    return manifest_path


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------

def main() -> None:
    parser = argparse.ArgumentParser(
        description="Generate SAYT M1 audio verification WAV fixtures.",
        epilog=(
            "⚠  HEARING SAFETY: generated files produce audible tones.  "
            "Start at LOW volume."
        ),
    )
    parser.add_argument(
        "--output-dir", type=Path, required=True,
        help="Directory for generated WAV files (must be empty or "
             "previously generated by this tool).",
    )
    parser.add_argument(
        "--force", action="store_true",
        help="Overwrite generator-owned files in an existing output directory.",
    )
    parser.add_argument(
        "--version", action="version",
        version=f"%(prog)s {GENERATOR_VERSION}",
    )
    args = parser.parse_args()

    print(f"SAYT M1 Audio Fixture Generator v{GENERATOR_VERSION}")
    print(f"Output: {args.output_dir.resolve()}\n")
    print("⚠  HEARING SAFETY: start playback at LOW volume.\n")

    manifest_path = generate(args.output_dir, force=args.force)

    print(f"\nGeneration complete.  Manifest: {manifest_path}")
    print(f"Verify with:\n  py -3 tools/audio-validation/verify_fixtures.py "
          f"--input-dir \"{args.output_dir}\"")


if __name__ == "__main__":
    main()
