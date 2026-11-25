# SAYT M1 — Physical Device Test Guide

**Purpose:** Step-by-step procedure for manually testing SAYT's two-deck
audio engine on a real audio output device using the generated verification
fixtures.

> [!CAUTION]
> ## Hearing safety
>
> The test fixtures produce audible tones.  Follow these rules at all times:
>
> 1. **Start with operating-system volume LOW** (≈ 10–20%).
> 2. Increase volume **gradually** after confirming output is clean.
> 3. **Stop playback immediately** if audio sounds unexpectedly loud,
>    distorted, or clipped.
> 4. **Do not use test tones at high headphone levels** — sustained pure
>    tones can cause discomfort or hearing damage.
> 5. **Do not disconnect powered professional audio equipment** unless you
>    know it is safe to do so for that specific device.
>
> This guide does **not** describe a calibrated measurement system.
> It tests whether the SAYT application delivers the expected signal to the
> output device — not the analogue performance of the device itself.

---

## Prerequisites

1. A working SAYT desktop build (M1 milestone).
2. A physical audio output device (speakers, headphones, or audio interface).
3. The generated fixture set (see `tools/audio-validation/README.md`):

```powershell
py -3 tools/audio-validation/generate_fixtures.py `
    --output-dir "$env:TEMP\SAYT-Audio-Fixtures"
```

4. The verified fixture manifest:

```powershell
py -3 tools/audio-validation/verify_fixtures.py `
    --input-dir "$env:TEMP\SAYT-Audio-Fixtures"
```

Confirm **ALL CHECKS PASSED** before proceeding.

---

## Test A — Initial playback

**Fixtures:** `deck_a_440hz.wav`, `deck_b_880hz.wav`

| Step | Action | Expected |
|------|--------|----------|
| A.1 | Start SAYT. | Application launches without errors. |
| A.2 | Select an available physical output device. | Device appears in the selector and is accepted. |
| A.3 | Set operating-system volume to LOW. | — |
| A.4 | Load `deck_a_440hz.wav` into Deck A via the file picker. | Filename, duration (≈ 8 s), sample rate (48 kHz), and waveform metadata displayed. |
| A.5 | Load `deck_b_880hz.wav` into Deck B. | Same metadata check for 880 Hz fixture. |
| A.6 | Start Deck A. | A steady 440 Hz tone is audible. Position advances. |
| A.7 | Start Deck B (while A is still playing). | Both tones are audible simultaneously — a lower (440 Hz) and higher (880 Hz) pitch. |
| A.8 | Record result: simultaneous two-deck playback. | PASS / FAIL |

---

## Test B — Independent deck control

**Fixtures:** same as Test A (both decks loaded).

| Step | Action | Expected |
|------|--------|----------|
| B.1 | Pause Deck A. | Only the 880 Hz tone (Deck B) remains audible. |
| B.2 | Resume Deck A. | Both tones return. |
| B.3 | Seek Deck A to a different position. | 440 Hz tone continues from the new position; Deck B unaffected. |
| B.4 | Stop Deck A. | Only 880 Hz audible; Deck A position resets or holds at stop point. |
| B.5 | Repeat steps B.1–B.4 with deck roles swapped. | Each control affects only its own deck. |
| B.6 | Record result: independent deck transport. | PASS / FAIL |

---

## Test C — Crossfader

**Fixtures:** both decks loaded and playing.

| Step | Action | Expected |
|------|--------|----------|
| C.1 | Move crossfader fully **left**. | Only the 440 Hz tone (Deck A) is audible; 880 Hz is silent or near-silent. |
| C.2 | Move crossfader to **centre**. | Both tones audible at roughly comparable levels. |
| C.3 | Move crossfader fully **right**. | Only the 880 Hz tone (Deck B) is audible; 440 Hz is silent or near-silent. |
| C.4 | Record result: crossfader routing. | PASS / FAIL |

> [!NOTE]
> SAYT uses an equal-power crossfader curve.  At centre, both decks may
> sound slightly louder than at the endpoints.  This test confirms correct
> *routing*, not the exact dB gain curve.

---

## Test D — Equalizer

**Fixtures:** `eq_low_100hz.wav`, `eq_mid_1000hz.wav`, `eq_high_5000hz.wav`

For each fixture:

| Step | Action | Expected |
|------|--------|----------|
| D.1 | Load the fixture into one deck and play it. | Tone is audible. |
| D.2 | Cut the **matching** EQ band (e.g. Low for 100 Hz). | Tone becomes quieter or inaudible. |
| D.3 | Restore the band to centre. | Tone returns to its original level. |
| D.4 | Boost the matching band. | Tone becomes louder. |
| D.5 | Cut a **non-matching** band. | Tone is largely unaffected. |
| D.6 | Record result for each frequency band. | PASS / FAIL |

Repeat for all three fixtures.

> [!NOTE]
> EQ bands overlap.  A deep cut on the mid band may still attenuate 100 Hz
> or 5 kHz slightly, depending on the filter design.  The key signal is
> whether the **primary** matching band has the dominant audible effect.

---

## Test E — Gain and master volume

**Fixture:** any tonal fixture loaded into one deck.

| Step | Action | Expected |
|------|--------|----------|
| E.1 | With the deck playing, increase the **deck gain**. | Tone becomes louder. |
| E.2 | Decrease deck gain. | Tone becomes quieter. |
| E.3 | Return deck gain to unity/centre. | — |
| E.4 | Increase **master gain** (if available). | Overall output louder. |
| E.5 | Decrease master gain. | Overall output quieter. |
| E.6 | Record result: gain controls. | PASS / FAIL |

> [!WARNING]
> When testing gain boost, increase slowly.  Pure tones at boosted gain can
> become uncomfortably loud quickly.

---

## Test F — Stereo channel identification

**Fixture:** `stereo_channel_id.wav` (L: 440 Hz, R: 660 Hz)

| Step | Action | Expected |
|------|--------|----------|
| F.1 | Load `stereo_channel_id.wav` and play it. | Two distinct tones audible. |
| F.2 | Listen with stereo headphones or separate L/R monitors. | Lower pitch (440 Hz) in left ear/speaker; higher pitch (660 Hz) in right ear/speaker. |
| F.3 | If using a mono output or a single speaker: skip. | — |
| F.4 | Record result: stereo separation. | PASS / FAIL / NOT APPLICABLE |

---

## Test G — Device recovery

> [!CAUTION]
> Only perform this test if you can safely disconnect your output device.
> Do **not** unplug powered professional studio monitors under load unless
> you know that is safe for your specific equipment.

| Step | Action | Expected |
|------|--------|----------|
| G.1 | With audio playing, note the selected output device. | — |
| G.2 | Disconnect or disable the device (e.g. unplug USB headphones or disable in OS settings). | SAYT should display an error message or status change. |
| G.3 | Observe: does SAYT crash? | It should not crash. |
| G.4 | Reconnect or re-enable the device (or select another available device). | — |
| G.5 | Resume or restart playback. | Audio resumes through the new/restored device. |
| G.6 | Record result: device recovery. | PASS / FAIL / BLOCKED |

---

## Test H — Shutdown

| Step | Action | Expected |
|------|--------|----------|
| H.1 | Start playback on one or both decks. | Audio is audible. |
| H.2 | Close the SAYT application (window close or OS quit). | Audio output stops immediately. |
| H.3 | Confirm the SAYT process has exited (Task Manager / `ps`). | No orphan SAYT process remains. |
| H.4 | Record result: clean shutdown. | PASS / FAIL |

---

## Recording results

Use the template at
[`docs/audio-validation/RESULTS_TEMPLATE.md`](RESULTS_TEMPLATE.md).

Fill in every field.  Use the vocabulary:

| Status | Meaning |
|--------|---------|
| **NOT RUN** | Test was not attempted. |
| **PASS** | Expected behaviour observed. |
| **FAIL** | Unexpected behaviour observed — describe the fault. |
| **BLOCKED** | Test could not be run due to a prerequisite issue. |
| **NOT APPLICABLE** | Test does not apply to the current hardware or configuration. |

Attach screenshots, screen recordings, or audio recordings where practical.
