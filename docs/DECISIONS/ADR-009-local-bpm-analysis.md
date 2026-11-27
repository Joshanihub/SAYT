# ADR-009: Analyze local tempo and retain PCM waveform peaks

**Status:** PROPOSED implementation decision<br>
**Owner:** SAYT project owner<br>
**Date:** 2026-11-27<br>
**Version:** 1.3

## Decision

Use one deterministic, bounded PCM pass for selected catalog WAV files to compute a 160-peak waveform and an RMS-onset autocorrelation BPM estimate; persist each result separately from source tags and user corrections, and never promote an uncertain candidate into deck beat controls. Provide a separate manual Tap BPM aid using monotonic UI timestamps; it stages a correction and never saves automatically.

## Context

The library already extracts BPM tags, while the DJ engine accepts a deck-local manual BPM for Beat Jump and Loop Roll. Automatic analysis must not overwrite source metadata, fabricate results, run in the CPAL callback, or leave a stale result attached to changed audio. SAYT's first decoder supports validated integer PCM WAV only. The Screen 04 Stitch design also depicts beatgrid, key, phrase, vocal/stem and stored-cue features; those are separate capabilities and remain unavailable until implemented with real data and algorithms.

## Options

- **Adopt a tempo-analysis dependency:** could broaden algorithm coverage sooner; adds a direct dependency, licence/provenance review, algorithm behaviour, and possible platform/build cost.
- **Implement a small PCM-only estimator:** avoids a new dependency and fits the current decoder; limits supported formats and exposes half/double-tempo ambiguity that must remain visible.

## Choice and consequences

Choose a 100 Hz RMS envelope, positive-onset differences, and normalized autocorrelation over 60–200 BPM. The work runs on one cancellable native worker thread at a time, outside the real-time callback. It accepts only files resolved from the native-picker-owned catalog, canonicalized beneath their registered root, and already marked loadable by the current integer PCM WAV decoder. The file's SHA-256 identity is checked before and after analysis. The file-size and decoded-frame limits match the decoder.

The tempo result has an algorithm version, candidate BPM, 0–1 periodicity score, and `estimated` or `uncertain` status. The score is a heuristic, not a probability. Silent, short, weakly periodic, and plausible half/double-tempo material must not be reported as a confident estimate. The same PCM pass also computes 160 normalized absolute-peak bins; these are quantized into a fixed-size catalog record and rendered only as a display summary, never as an audio signal. A manual BPM correction is stored in its own table and is limited to 30–300 BPM. Source tag BPM remains untouched. The separate manual Tap BPM UI retains up to four monotonic intervals from 200–2,000 ms, waits for three taps, rounds the mean tempo to one decimal, and updates only the correction draft; **Save correction** remains the persistence boundary. Track changes clear the tap sequence. Tap BPM is a user-timed input and does not inspect audio. For deck controls the precedence is manual correction, confident analysis, then an in-range file tag; uncertain analysis is skipped. This makes current BPM values available to existing beat controls without claiming a beatgrid. The catalog track ID hashes original file bytes, while the deck identity hashes normalized decoded PCM; after loading, the integration must use the selected deck's `source_track_id` from its engine snapshot or the stale-generation guard rejects the BPM command.

SQLite schema version 2 adds `bpm_analysis` and `bpm_corrections`; version 3 adds `track_waveforms`. Version 1 upgrades transactionally through both versions, and an existing version 2 catalog gains waveform storage without losing corrections. Analysis is cached by content identity and per-result version. Cancellation saves no partial result. No dependency or lockfile change is required.

## Verification and limits

Automated fixtures cover a known 60 BPM pulse, an ambiguous 120 BPM pulse, silence, short audio, unsupported float PCM, malformed input, cancellation during decode and autocorrelation, stereo frame-counted progress, waveform bounds/silence/persistence, changed source identity, v1→v3 and v2→v3 migration, caching, correction bounds, persistence, deck-tempo precedence, and Tap BPM interval/window/error boundaries. Frontend tests also cover the Preparation route, responsive layout wiring, tempo preset rounding and bounds, and Tap BPM control wiring. These establish behavior on generated fixtures and source-rendered UI only. They do not establish BPM accuracy on commercial music, personal collections, perceived beat timing, runtime appearance, beatgrids, key detection, or physical audio output.

The Library inspector and dedicated Preparation route use the same catalog selection and analysis state. Preparation widens the real waveform and BPM controls and adds half/double presets plus manual Tap BPM; all stage a correction and persistence still requires the existing Save correction action. This is a focused first workspace slice, not the full supplied Screen 04 composition. Editable beatgrids, key/phrase analysis, cue/loop preparation, and runtime verification of the current waveform UI remain open work.

## Reversibility and review trigger

The analysis table can be rebuilt by algorithm-version invalidation; source tags and corrections remain separate. Revisit this choice if supported formats expand, a real music test set shows unacceptable estimates, a licensed analysis dependency is selected, or beatgrid requirements need a different signal model. Do not call uncertain estimates from Beat Jump or Loop Roll.
