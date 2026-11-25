# SAYT M1 — Physical Device Test Results

> Copy this template for each test session.  Do not backfill results from
> a previous session or fabricate data.

---

## Session metadata

| Field | Value |
|-------|-------|
| **Test date** | |
| **SAYT commit** | |
| **Operating system** | |
| **Audio device model** | |
| **Audio driver / backend** | (e.g. WASAPI, ALSA, CoreAudio — if known) |
| **Device sample rate** | |
| **Fixture generator version** | |
| **Fixture manifest SHA-256** | (first 16 hex chars of `manifest.json` SHA-256) |
| **Fixture verification result** | (ALL CHECKS PASSED / FAILED — paste verifier output) |
| **Test duration** | |
| **Tester** | |

---

## Test results

### Test A — Initial playback

| Check | Status | Notes |
|-------|--------|-------|
| Application launches | NOT RUN | |
| Output device selectable | NOT RUN | |
| Deck A metadata correct | NOT RUN | |
| Deck B metadata correct | NOT RUN | |
| Both decks audible simultaneously | NOT RUN | |

### Test B — Independent deck control

| Check | Status | Notes |
|-------|--------|-------|
| Pause Deck A (B continues) | NOT RUN | |
| Resume Deck A | NOT RUN | |
| Seek Deck A (B unaffected) | NOT RUN | |
| Stop Deck A | NOT RUN | |
| Pause Deck B (A continues) | NOT RUN | |
| Resume Deck B | NOT RUN | |
| Seek Deck B (A unaffected) | NOT RUN | |
| Stop Deck B | NOT RUN | |

### Test C — Crossfader

| Position | Expected dominant | Status | Notes |
|----------|-------------------|--------|-------|
| Fully left | Deck A (440 Hz) | NOT RUN | |
| Centre | Both audible | NOT RUN | |
| Fully right | Deck B (880 Hz) | NOT RUN | |

### Test D — Equalizer

| Fixture | Band tested | Cut effect | Boost effect | Status | Notes |
|---------|-------------|------------|--------------|--------|-------|
| `eq_low_100hz.wav` | Low | NOT RUN | NOT RUN | NOT RUN | |
| `eq_mid_1000hz.wav` | Mid | NOT RUN | NOT RUN | NOT RUN | |
| `eq_high_5000hz.wav` | High | NOT RUN | NOT RUN | NOT RUN | |

### Test E — Gain and master volume

| Check | Status | Notes |
|-------|--------|-------|
| Deck gain increase | NOT RUN | |
| Deck gain decrease | NOT RUN | |
| Master gain increase | NOT RUN | |
| Master gain decrease | NOT RUN | |

### Test F — Stereo channel identification

| Check | Status | Notes |
|-------|--------|-------|
| Left channel = lower pitch (440 Hz) | NOT RUN | |
| Right channel = higher pitch (660 Hz) | NOT RUN | |

### Test G — Device recovery

| Check | Status | Notes |
|-------|--------|-------|
| Device disconnection observed | NOT RUN | |
| Error displayed (no crash) | NOT RUN | |
| Playback resumed on reconnection | NOT RUN | |

### Test H — Shutdown

| Check | Status | Notes |
|-------|--------|-------|
| Audio stops on close | NOT RUN | |
| Process exits cleanly | NOT RUN | |

---

## Observed faults

| # | Test | Description | Severity | Evidence |
|---|------|-------------|----------|----------|
| 1 | | | | |
| 2 | | | | |
| 3 | | | | |

---

## Evidence links

| Item | Link / path |
|------|-------------|
| Screen recording | |
| Audio recording | |
| Screenshot | |
| Log file | |

---

## Tester notes

<!-- Free-form observations, environment quirks, or follow-up items. -->
