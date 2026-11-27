# ADR-009: Analyze local track tempo outside the audio callback

**Status:** PROPOSED implementation decision<br>
**Owner:** SAYT project owner<br>
**Date:** 2026-11-27<br>
**Version:** 1.0

## Decision

Use a deterministic, bounded RMS-onset autocorrelation estimator for selected catalog PCM WAV files; store its versioned result separately from source BPM tags and user corrections, and never promote an uncertain candidate into deck beat controls.

## Context

The library already extracts BPM tags, while the DJ engine accepts a deck-local manual BPM for Beat Jump and Loop Roll. Automatic analysis must not overwrite source metadata, fabricate results, run in the CPAL callback, or leave a stale result attached to changed audio. SAYT's first decoder supports validated integer PCM WAV only. The Screen 04 Stitch design also depicts beatgrid, key, phrase, vocal/stem and stored-cue features; those are separate capabilities and remain unavailable until implemented with real data and algorithms.

## Options

- **Adopt a tempo-analysis dependency:** could broaden algorithm coverage sooner; adds a direct dependency, licence/provenance review, algorithm behaviour, and possible platform/build cost.
- **Implement a small PCM-only estimator:** avoids a new dependency and fits the current decoder; limits supported formats and exposes half/double-tempo ambiguity that must remain visible.

## Choice and consequences

Choose a 100 Hz RMS envelope, positive-onset differences, and normalized autocorrelation over 60–200 BPM. The work runs on one cancellable native worker thread at a time, outside the real-time callback. It accepts only files resolved from the native-picker-owned catalog, canonicalized beneath their registered root, and already marked loadable by the current integer PCM WAV decoder. The file's SHA-256 identity is checked before and after analysis. The file-size and decoded-frame limits match the decoder.

The result has an algorithm version, candidate BPM, 0–1 periodicity score, and `estimated` or `uncertain` status. The score is a heuristic, not a probability. Silent, short, weakly periodic, and plausible half/double-tempo material must not be reported as a confident estimate. A manual BPM correction is stored in its own table and is limited to 30–300 BPM. Source tag BPM remains untouched. For deck controls the precedence is manual correction, confident analysis, then an in-range file tag; uncertain analysis is skipped. This makes current BPM values available to existing beat controls without claiming a beatgrid. The catalog track ID hashes original file bytes, while the deck identity hashes normalized decoded PCM; after loading, the integration must use the selected deck's `source_track_id` from its engine snapshot or the stale-generation guard rejects the BPM command.

SQLite schema version 2 adds `bpm_analysis` and `bpm_corrections` and migrates version 1 transactionally. Analysis is cached by content identity and algorithm version. Cancellation saves no partial result. No dependency or lockfile change is required.

## Verification and limits

Automated fixtures cover a known 60 BPM pulse, an ambiguous 120 BPM pulse, silence, short audio, unsupported float PCM, malformed input, cancellation during decode and autocorrelation, changed source identity, migration, caching, correction bounds, persistence, and deck-tempo precedence. These establish behavior on generated fixtures only. They do not establish accuracy on commercial music, personal collections, perceived beat timing, beatgrids, key detection, or physical audio output.

The current UI places the analysis and correction controls in the local library track inspector. The supplied Screen 04 workspace remains incomplete; waveform persistence, editable beatgrids, key/phrase analysis, cue/loop preparation, and a dedicated Preparation workspace remain open work.

## Reversibility and review trigger

The analysis table can be rebuilt by algorithm-version invalidation; source tags and corrections remain separate. Revisit this choice if supported formats expand, a real music test set shows unacceptable estimates, a licensed analysis dependency is selected, or beatgrid requirements need a different signal model. Do not call uncertain estimates from Beat Jump or Loop Roll.
