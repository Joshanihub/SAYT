# ADR-007: Use explicit per-deck tempo for the first beat controls

**Status:** PROPOSED for the professional DJ workflow expansion<br>
**Owner:** SAYT project owner<br>
**Date:** 2026-11-27<br>
**Version:** 1.0

## Decision

Implement Beat Jump and held Loop Roll using a manually entered, deck-local BPM value until track analysis and editable beatgrids are implemented.

## Context

The next approved performance slice follows Hot Cues and manual loops. The Rust/CPAL engine has no BPM estimator, beatgrid, or quantized scheduler. Guessing BPM or presenting a default tempo as track truth would make beat-sized controls unreliable. The audio callback must remain bounded, allocation-free, and isolated from the UI and filesystem.

## Options

- **Manual BPM input per loaded deck:** gives users explicit control and a measurable beat duration with no new dependency; tempo can be wrong when entered incorrectly.
- **Infer a fixed default or estimate silently:** removes a step; a wrong tempo appears authoritative and makes beat controls misleading.
- **Wait for full analysis and beatgrid implementation:** avoids interim tempo entry; blocks a useful reversible transport feature behind a larger analysis project.

## Choice and consequences

Choose manual tempo from 30 through 300 BPM. The value is session-local and clears when that deck loads a new track. Beat Jump supports 1, 2, 4, 8, and 16 beats in either direction. Loop Roll supports 1/4, 1/2, 1, 2, 4, 8, and 16 beats while held. Neither action claims beatgrid alignment or quantization.

Loop Roll runs in the existing callback. Its natural playhead advances while audible frames wrap inside the held region; release resumes at the natural position. Pause, seek, stop, replacement, output stream replacement, window blur, pointer cancellation, and control unmount end the roll. Commands are deck-generation-bound and travel through the existing bounded queue. A full queue is returned as an error.

## Verification and review trigger

Engine tests cover tempo validation, beat duration, deck isolation, jump direction and clamping, slip-position advancement, callback wrapping, release, cancellation, end-of-track, stale generations, and queue saturation. UI tests cover accessible controls, unloaded state, BPM bounds, and typed actions. Windows fixture runtime checks exercise the Tauri command path and UI states. Physical output listening, sample accuracy, pitch analysis, beatgrids, and quantized performance remain unverified or out of scope for this slice.

Review the temporary BPM seam when automatic BPM analysis and editable beatgrids enter an approved milestone. Do not silently carry the manual value across track replacement.

## Reversibility

The control API is internal to the desktop application and the value is not persisted. Replacing the manual input with analyzed tempo can be done without data migration; command behavior must keep the regression cases above.
