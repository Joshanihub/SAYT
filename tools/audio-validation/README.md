# SAYT M1 Audio Validation Toolkit

**Purpose:** Generate and verify reproducible WAV fixtures for manual testing
of SAYT's two-deck audio engine on physical hardware.

> **⚠ HEARING SAFETY**
>
> Generated files produce audible tones.
>
> - **Start playback at LOW volume.**
> - Increase gradually.
> - Stop immediately if output sounds unexpectedly loud or distorted.
> - Do not use test tones at high power on speakers or headphones.

---

## Prerequisites

- Python 3.10 or later (standard library only — no `pip install` needed).
- Windows: `py -3`; macOS/Linux: `python3`.

## Quick start

### 1. Generate fixtures

```powershell
# Windows PowerShell
py -3 tools/audio-validation/generate_fixtures.py `
    --output-dir "$env:TEMP\SAYT-Audio-Fixtures"
```

```bash
# macOS / Linux
python3 tools/audio-validation/generate_fixtures.py \
    --output-dir /tmp/SAYT-Audio-Fixtures
```

Output: seven WAV files and a `manifest.json` in the output directory.

To regenerate (overwrite previous output):

```powershell
py -3 tools/audio-validation/generate_fixtures.py `
    --output-dir "$env:TEMP\SAYT-Audio-Fixtures" --force
```

### 2. Verify fixtures

```powershell
py -3 tools/audio-validation/verify_fixtures.py `
    --input-dir "$env:TEMP\SAYT-Audio-Fixtures"
```

The verifier checks every WAV header, SHA-256 hash, frame count, PCM
encoding, and clipping status against the manifest. Exit code 0 means all
checks passed.

### 3. Run automated tests

```powershell
py -3 -m unittest discover -s tools/audio-validation/tests -v
```

---

## Generated fixtures

| File                    | Frequency        | Duration | Purpose                        |
| ----------------------- | ---------------- | -------- | ------------------------------ |
| `deck_a_440hz.wav`      | 440 Hz           | 8 s      | Deck A identification          |
| `deck_b_880hz.wav`      | 880 Hz           | 8 s      | Deck B identification          |
| `eq_low_100hz.wav`      | 100 Hz           | 6 s      | Low EQ band verification       |
| `eq_mid_1000hz.wav`     | 1 000 Hz         | 6 s      | Mid EQ band verification       |
| `eq_high_5000hz.wav`    | 5 000 Hz         | 6 s      | High EQ band verification      |
| `stereo_channel_id.wav` | L:440 / R:660 Hz | 8 s      | Stereo channel identification  |
| `silence_reference.wav` | —                | 5 s      | Reference silence / leak check |

All files: 48 kHz, stereo, 16-bit PCM, ≈ −18 dBFS peak, 10 ms fade-in/out.

## Shared format

- Uncompressed integer PCM — compatible with the SAYT WAV decoder.
- No third-party codecs or compressed formats.
- Deterministic: same version + options → bit-identical output.

## What this toolkit does NOT do

- Measure physical output latency, frequency response, or distortion.
- Verify CPAL callback performance or dropout rates.
- Replace a manual listening test on physical hardware.
- Constitute a calibrated measurement or certification system.

## Physical testing

See [`docs/audio-validation/PHYSICAL_DEVICE_TEST_GUIDE.md`](../../docs/audio-validation/PHYSICAL_DEVICE_TEST_GUIDE.md)
for the step-by-step manual test procedure.

Record results in
[`docs/audio-validation/RESULTS_TEMPLATE.md`](../../docs/audio-validation/RESULTS_TEMPLATE.md).

## Repository rules

- **Do not commit generated WAV files.** They are test artefacts only.
- Add the output directory to `.gitignore` or use a `$TEMP` path.
- This toolkit lives in `tools/audio-validation/` and `docs/audio-validation/`.
  It does not modify any other part of the SAYT repository.
