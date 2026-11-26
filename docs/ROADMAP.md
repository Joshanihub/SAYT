# SAYT roadmap

**Status:** PROPOSED<br>
**Owner:** SAYT project owner<br>
**Date:** 2026-11-25<br>
**Version:** 0.2.0

Milestone dependencies and full M0–M10 exit gates are maintained in [`MASTER_IMPLEMENTATION_ROADMAP.md`](MASTER_IMPLEMENTATION_ROADMAP.md). This file remains the concise current-milestone view.

## Current milestone: M1 — desktop foundation and two-deck audio

**Status:** In progress. The Rust engine, decoder, EQ, two-deck mixer, Tauri commands, React controls, and reviewed audio-validation toolkit are implemented in source. Local Rust, frontend, and Python checks passed; push and PR cross-platform CI passed for code commit `853e51477e56593471c7126378d290524be391af`. Physical playback and the full manual acceptance checklist are not yet verified.

### M1 exit gate

- All Rust and frontend checks pass on the pinned toolchains from a clean checkout.
- The desktop application launches and two actual user-selected WAV files play through a physical output.
- Deck transport, gain, three EQ bands, crossfader, waveform, device failure handling, and shutdown pass `docs/REQUIREMENTS.md`'s manual checklist.
- Windows, macOS, and Linux CI has completed successfully; OS-specific failures are recorded and resolved.
- The reviewed fixture generator/verifier and physical-device guide are integrated; Linux CI executes the symlink-path regression.
- The owner records real picker, Tauri IPC, viewport, physical output, device recovery, and shutdown results.
- Dependency/license findings are current and the SAYT project license remains an explicit owner decision.

## Next milestones

1. **M1 closeout:** complete manual owner-operated picker/Tauri and device checks on Windows. Add macOS/Linux hardware evidence before claiming cross-platform playback.
2. **M2 — reliability:** measure callback load and dropouts; implement device loss/recovery and a tested shutdown path; use a deterministic offline renderer and failure controls before longer soak runs.
3. **M3 — library and preparation:** add local metadata persistence and migration once the owner approves the schema and data-retention behavior; keep the first library import local and reversible.
4. **Later scope:** playlist bridging, controllers, recording, plugins, video, mobile, accounts, and cloud remain gated by the founding charter and are not part of M1.

## Re-plan triggers

- A physical-device test shows audio callback starvation, repeated dropouts, or stuck output: stop feature work and repair the audio foundation.
- The Windows/macOS waveform performance spike misses the charter's threshold: reconsider the webview before adding waveform-heavy UI.
- A required dependency has an unacceptable or unclear license/security posture: replace it or re-scope before adding dependent functionality.
- User sessions do not confirm the local two-deck workflow is useful: revisit the target user and pain before building the larger workstation.
