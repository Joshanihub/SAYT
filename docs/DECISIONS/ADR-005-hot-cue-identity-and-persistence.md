# ADR-005: Persist hot cues by decoded audio identity

**Status:** PROPOSED for the professional DJ workflow expansion<br>
**Owner:** SAYT project owner<br>
**Date:** 2026-11-27<br>
**Version:** 1.0

## Decision

Store eight hot-cue slots per decoded PCM source identity in versioned JSON under Tauri's app-local data directory, and trigger cues through the existing generation-bound audio seek command.

## Context

Hot cues are source-track metadata shared by either deck and must survive application restarts. A deck index is transient, and a filename can be reused for different audio. The accepted audio engine remains Rust/CPAL per [ADR-003](ADR-003-rust-engine-milestone1.md), with the bounded real-time callback constraints in `AGENTS.md`. Interface and keyboard actions share the typed dispatcher in [ADR-004](ADR-004-keyboard-command-system.md).

## Options

- **Content identity and app-local JSON:** small, inspectable, dependency-light storage; identical decoded audio shares cues, while acoustically equivalent re-encodings may not.
- **Filename or path identity:** simple to calculate; renames, moves, and same-name replacement can misapply cues.
- **Add cue fields to a music library database:** supports richer preparation metadata later; introduces a library migration and lifecycle before that subsystem exists.

## Choice and consequences

Choose content identity and app-local JSON. The decoder hashes a domain marker, sample rate, channel count, bit depth, and the sequence of decoded integer PCM samples with SHA-256. File names and tags are excluded. Changing any decoded sample or the included format metadata changes identity; renaming or retagging identical PCM does not. Re-encoding the same performance with different bit depth, sample rate, or PCM values is treated as a different source. This is exact-content identity, not acoustic fingerprinting.

The actor owns filesystem access and all cue persistence. The CPAL callback receives only bounded seek commands and performs no cue storage, logging, allocation, locking, I/O, or UI work. Cue triggers carry the source identity currently rendered by the UI; the actor rejects a stale identity after track replacement. The engine binds seek commands to the deck's current generation and preserves the current play/pause state.

Cue documents have schema version 1, a strict source identity, an 8 KiB document limit, a 10,000-track cap, and at most three retained backups per track. Writes go through a synced temporary file and rename; damaged or unsupported data is not silently loaded, and an available backup is offered with a warning. The store stays local and does not create or depend on the unimplemented music library.

## Verification

Automated tests cover cue validation, both deck commands, cue-trigger behavior while playing and paused, stale generation rejection, queue saturation, source identity, persistence across store re-instantiation, content mismatch, corrupted documents, interrupted replacement, and backup bounds. Frontend tests cover visible empty/assigned/active states and verify that cue shortcut actions construct the same typed commands used by pads. Actual Tauri runtime checks must still confirm native picker integration, cue triggering, and persistence across application restart. Physical listening and sample-accurate timing are separate claims and are not established by these tests.

## Reversibility and review trigger

The JSON version and source identity include explicit version/domain markers. Revisit the identity when codec support, analysis metadata, or a library database is introduced; any identity change needs migration or an explicit no-reuse policy. Revisit the persistence location if the approved library becomes the canonical owner of track metadata. Do not move file I/O to the callback or place PCM buffers in Tauri IPC.
