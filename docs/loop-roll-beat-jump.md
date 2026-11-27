# Loop Roll and Beat Jump

**Status:** IMPLEMENTED in source; automated verification and Windows manual BPM/Beat Jump UI integration verified; sustained Loop Roll runtime and physical audio quality remain pending<br>
**Owner:** SAYT project owner<br>
**Date:** 2026-11-27<br>
**Version:** 1.0

## Behavior

- Each loaded deck accepts an explicit BPM between 30 and 300, including fractional values. Tempo is deck-local, session-only, and clears on track replacement. Empty tempo is not inferred or defaulted.
- Beat Jump moves the real engine playhead by 1, 2, 4, 8, or 16 beat durations in either direction. At 120 BPM one beat is 0.5 seconds. The resulting position clamps to the track start or end.
- Loop Roll has 1/4, 1/2, 1, 2, 4, 8, and 16 beat sizes. It starts only on a playing deck with a valid BPM and a region fitting the remaining track. Pointer hold or focused Space/Enter hold starts the action; release, pointer cancellation, focus loss, or window blur ends it.
- The callback loops decoded audio in the selected region while an independent natural playhead advances. Releasing resumes at that natural position. Snapshots report natural track position during the roll.
- Pause, seek, Stop, replacing a track, selecting a new output stream, or leaving the pad mode ends the roll. If its natural position has reached track end, release stops the deck at the end.
- All engine operations identify the track generation. Queue saturation returns an actionable error; it never claims an unapplied jump or held state succeeded.
- The performance mode exposes deck-local session BPM. A saved Library correction or confident local estimate may supply it when a catalog track loads; uncertain analysis is not applied. The engine controls do not build or use beatgrids or quantization.

## Audio-thread and trust boundary

WAV decoding and BPM input validation run outside the callback. The callback only updates fixed-size per-deck state and samples the existing track buffer. The feature adds no dependency, filesystem path command, sample IPC, UI callback, allocation, lock, or I/O to the CPAL callback.

## Verification

Automated coverage is recorded in [`VERIFICATION_EVIDENCE.md`](VERIFICATION_EVIDENCE.md). It includes BPM boundaries and invalid values; beat sizes and durations; deck scoping and track replacement; jump forward/back and clamping; stale identity, unloaded deck, paused playback, end boundary, and full queue; Loop Roll sample wrapping, natural playhead progress, release restoration, pause/seek cancellation, and active telemetry.

Windows Tauri runtime at 1280×832 used the generated fixtures in separate decks, System default output, and master at 10%. At 120 BPM, A moved from 0 to 2 seconds with +4 beats, then to 1 second with −2 beats; B moved from 0 to 1 second with +2 beats while A stayed at 1 second. After clean shutdown and relaunch, both fixtures reloaded, saved cues returned at A 0:02 and B 0:00, and BPM was blank for both decks as intended. This verifies the manual BPM and Beat Jump UI/engine integration and cue persistence after restart. Sustained Loop Roll hold/release, cancellation during a live hold, physical seam listening, and sample-accurate controller response remain unverified. The available UI input could click or send key presses but could not sustain a press.

See [ADR-007](DECISIONS/ADR-007-manual-beat-performance.md), [manual loops](manual-loops.md), and the [feature acceptance matrix](FEATURE_ACCEPTANCE_MATRIX.md).
