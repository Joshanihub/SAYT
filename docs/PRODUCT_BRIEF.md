# SAYT product brief

**Status:** PROPOSED<br>
**Owner:** SAYT project owner<br>
**Date:** 2026-11-25<br>
**Version:** 0.1.0

## Outcome

Build a local-first desktop music workstation, beginning with a dependable DJ workflow. The immediate proof is that a person can load two local PCM WAV recordings, control them independently, and hear their mix from a selected audio output.

## First user and pain

**Target user:** a DJ learner or working DJ who has local music files and wants to practice or mix without a streaming account.<br>
**Job:** load, cue, play, and blend two tracks.<br>
**Pain evidence:** not yet validated with user interviews or observed sessions. This is a product hypothesis, not established demand.

## Stage and scope

**Stage:** building the Milestone 1 desktop foundation and two-deck audio prototype.<br>
**In scope:** Tauri desktop shell, WAV loading, independent deck transport, gain, three-band EQ, crossfader, master output, waveform display, and output-device selection.<br>
**Out of scope:** provider streaming, library migration, recording, controllers, stems, plugins, video, mobile, accounts, cloud sync, and auto-update.

## Binding constraints

- The real-time callback must remain free of blocking locks, filesystem I/O, allocation, and UI/IPC calls.
- Local files are the only audio source in this milestone.
- Hardware behavior is not accepted as verified until tested on a physical device.
- The project's own license remains pending owner choice; the public file set carries no SAYT reuse grant.

## Success measure

Milestone 1 passes when each functional acceptance criterion in `docs/REQUIREMENTS.md` is demonstrated, including manual verification of real device output, deck controls, EQ, crossfader, device failure handling, and clean shutdown. Compilation alone is not success.

## First reversible step

Run the native and frontend checks, then follow the manual audio checklist in `docs/REQUIREMENTS.md` with two owner-approved PCM WAV fixtures and a physical output device.

## Kill or re-plan criteria

- Stop adding UI features if callback safety or device stability fails; repair the engine first.
- Reconsider the desktop shell if the planned Windows/macOS waveform performance spike fails its documented threshold.
- Keep the product local-files-only if provider agreements and rights cannot be established; do not imitate private service APIs.
