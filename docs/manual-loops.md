# Manual loops

**Status:** IMPLEMENTED in source; desktop runtime verification pending<br>
**Owner:** SAYT project owner<br>
**Date:** 2026-11-27<br>
**Version:** 1.0

## Behavior

- Loop In records the focused deck's current engine position. Loop Out records a later position and activates the region. A Loop Out before Loop In is rejected.
- The size pads start a loop at the current position with one of six fixed durations: 0.25, 0.5, 1, 2, 4, or 8 seconds. Regions shorter than 50 ms, non-finite values, reversed markers, and markers outside the source track are rejected.
- While playing, the engine repeats the half-open frame region `[Loop In, Loop Out)`. It normalizes a playhead already at Loop Out before reading another frame, so the first repeated frame comes from inside the region.
- Release while playing exits at the next Loop Out boundary. Release while paused disables the loop immediately. A seek outside the region disables the active loop but retains its markers; a size pad or new markers can activate it again.
- Stop disables the active loop and returns the deck to the track start. Loading a different track clears that deck's loop markers. Deck A and Deck B own independent loop state.
- Loop regions are deck-local runtime state and are not persisted. The UI states this. Loop durations are seconds, not beats, because BPM and beatgrid analysis are not implemented.

## Boundaries

Loop commands carry source-track identity and engine generation. They travel through the bounded control queue; a full queue returns an error and does not update the accepted loop snapshot. The audio callback performs no allocation, locking, I/O, logging, UI work, or Tauri/JavaScript calls.

The callback wraps at the loop boundary without beat alignment or a crossfade. This is functional loop playback, not a claim of click-free transitions, sample-accurate controller response, physical output quality, or hardware reliability. Those require additional DSP design and manual listening against real output.

## Verification

Automated cases cover region limits, track identity, deck isolation, queue saturation, stale generations, loop wrapping, first-frame boundary normalization, immediate paused release, and release at the next playing boundary. Frontend cases cover state display, empty-deck gating, typed pad commands, and keyboard bindings sharing the same typed actions. Windows Tauri fixture playback and audio listening remain pending; a software test or desktop build does not prove physical output.

See [ADR-006](DECISIONS/ADR-006-manual-loop-runtime.md), the [acceptance matrix](FEATURE_ACCEPTANCE_MATRIX.md), and the [shortcut runbook](runbook/keyboard-shortcuts.md).
