# ADR-006: Keep manual loop state in the real-time engine

**Status:** PROPOSED for the professional DJ workflow expansion<br>
**Owner:** SAYT project owner<br>
**Date:** 2026-11-27<br>
**Version:** 1.0

## Decision

Implement per-deck manual loops in the existing Rust engine, with bounded generation-bound commands and loop durations expressed in seconds until BPM/beatgrid support exists.

## Context

The supplied Stitch DJ screen includes Loop as a performance-pad mode. SAYT currently has a Rust/CPAL callback, generation-bound deck state, a Tauri actor, and a shared UI/keyboard command dispatcher. The callback must remain allocation-free and isolated from filesystem and UI operations. No BPM or beatgrid system is implemented, so beat-sized loop claims would be false.

## Options

- **Existing Rust callback and bounded command queue:** preserves the accepted engine boundary and gives deterministic, hardware-independent loop tests; the callback must maintain minimal loop state.
- **React timer or Web Audio loop:** shortens an initial UI prototype; it cannot guarantee audio-thread boundaries and creates a second audio path.
- **Add a new native engine boundary:** could enable other audio APIs; it expands lifecycle and packaging risk without evidence that CPAL is the limiting factor.

## Consequences

Choose the existing Rust callback. Commands include the current source identity and generation; stale commands are ignored by the callback, queue saturation returns failure, and snapshots expose loop markers and release state. Loop sizes are explicit seconds. A currently active loop can be released at its next boundary; a paused loop releases immediately.

The current callback uses a hard boundary wrap without beat alignment or crossfade. This leaves audible seam quality to manual listening and future DSP work. Loop positions are not persisted; a track replacement resets them. The UI offers only implemented Hot Cue and Loop modes.

## Verification and review trigger

Automated verification covers boundaries, stale identity/generation, queue saturation, wrap, release, and shared UI/keyboard commands. Actual audio quality and Tauri runtime interaction remain separate acceptance gates. Revisit the timing unit when an approved BPM/beatgrid implementation exists; revisit crossfade behavior after listening tests expose representative seam failures. Do not infer physical performance from tests or compilation.

## Reversibility

Loop commands and state are internal and versioned by source-track generation. The decision is reversible while the command contract is not a public plugin API; changing it later requires migration of keyboard command IDs and a loop-behavior regression suite.
