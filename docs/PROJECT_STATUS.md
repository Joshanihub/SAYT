# SAYT project status

**Status:** IN PROGRESS<br>
**Owner:** SAYT project owner<br>
**Date:** 2026-11-27<br>
**Version:** 0.4.27

## Current verified position — M1 closeout and keyboard-command expansion

- **M1 source branch:** `feat/stitch-m1-dj-workspace` is based on unchanged `main` at `1d0323799d0a833a40b20cd58b1c79e43a11044b`. Responsive correction and decoder rejection coverage are pushed as `ee137e288b324a50e6f9b1574c4aee9d4eee1c09`; exact-head push run [45831799052](https://github.com/Joshanihub/SAYT/actions/runs/45831799052) and PR run [45831807175](https://github.com/Joshanihub/SAYT/actions/runs/45831807175) both passed Windows, macOS, and Ubuntu. GitHub currently shows PR #1 open and PR #2 still draft. No merge into `main` is authorized.
- **DJ expansion branch:** Manual Loop implementation commit `809b3461dc1a80be991b134f08d63d60826a3cfb` and evidence commit `86479e9c9f8a08052b8566447ab1d4d51f2319ed` are pushed to `feat/dj-command-system`; exact-head GitHub Actions run [#52](https://github.com/Joshanihub/SAYT/actions/runs/45909756274) passed its three platform jobs. `main` remains `1d0323799d0a833a40b20cd58b1c79e43a11044b`. Windows fixture runtime checks now cover Hot Cue persistence and selected Manual Loop behaviors; physical listening and audio-device recovery remain open. Only generated fixtures were used in this run. No PR or merge is created.
- **Stitch typography and icons:** 16 local variable WOFF2 text-font subsets and a 66-ligature Material Symbols subset are integrated. Four upstream license notices are present under `apps/desktop/public/licenses`; runtime font loading makes no network request. Exact sources, hashes, sizes, Unicode ranges, and build-tool pins are recorded in the asset manifest and dependency inventory.
- **Local verification:** Rust format, Clippy, 46 engine tests plus 10 desktop actor/cue-store tests, and the locked all-target check pass. Frontend format, lint, typecheck, 61 tests, and production web build pass. The Windows Tauri release build also passes in `%TEMP%\SAYT-ManualLoop-Release-Target` without touching the running app's executable. Dependency scans from the Hot Cue batch still report duplicate versions and existing allowed unmaintained/GLib advisories; no dependencies changed in the loop slice. Exact commands and limits are in `docs/VERIFICATION_EVIDENCE.md`.
- **M1 device discovery state:** the Settings surface now distinguishes loading, an empty device list, enumeration failure, and a selected system default. The React tests verify these states; real device enumeration and recovery still require hardware verification.
- **Output-switch recovery:** source now attempts the replacement stream after a pause error and restores the previous stream if replacement fails. Resume failure is reported in the command error and output-error snapshot. Four hardware-independent transition tests pass; CPAL/OS recovery remains unverified.
- **Interaction motion:** button and select state changes, including navigation, transport, and load controls, use a named 120 ms ease-out transition. CSS honors `prefers-reduced-motion`; the stylesheet regression test passes and failed under an intentional canary edit. Runtime preference behavior and target-screen appearance remain unverified.
- **Responsive workspace:** Fixed CSS auto-placement that put Deck B below the mixer between 721 px and 1060 px. Named grid areas now preserve deck order at desktop, compact, and narrow sizes; Tauri captures at 1440×900, 1280×800, 800×700, and 480×700 are committed in `docs/evidence/m1-responsive/`. Visual comparison confirms the M1 identity and core hierarchy while the supplied Screen 01 remains only partially implemented.
- **Owner-reported audio checks:** on 2026-11-26 the owner reported that all requested manual audio checks passed, including fixture loading, deck transport, low-volume mixer/EQ controls, refresh/retry, and shutdown. Device/viewport and per-test observations were not provided, so this is attributed owner evidence, not an independent reproduction; the current accessibility snapshot shows populated, stopped decks but does not identify the tested fixtures.
- **Hot Cues:** eight colorable, editable, persisted slots per source track; real bounded generation-bound seek commands; shared UI/keyboard dispatch; local source identity and corruption recovery. Automated engine, persistence and React suites pass. On Windows at 1282×832, both generated fixtures loaded in the native picker, cues set at A 0:02 and B 0:00, and both cues restored after a full app restart and track reload. Paused cue triggers moved only their own deck. Same-file replacement retained A's cue; it had appeared absent only because A was in Loop mode, which hides the cue pads. Physical output quality and cue-trigger timing while playing remain unverified.
- **Manual Loops:** loop in/out markers, six seconds-based loop sizes, generation-bound callback wrapping, next-boundary release while playing, immediate release while paused, deck-local snapshots, and shared keyboard/pad command dispatch are implemented in source. Engine and React suites pass. Windows runtime showed a 250 ms sized loop on B, an A loop wrapping from 1–2 seconds and releasing while playing, seek-outside deactivation, paused release, and stop reset. Physical seam listening is still unverified; loop state is not persisted and no beatgrid or crossfade is claimed.
- **M1 remains IN PROGRESS:** target viewport placement is runtime verified and the feature branch passed the exact-head three-platform CI run #52. Full Screen 01 fidelity and audio failure-path/hardware acceptance remain open. Build and test evidence is recorded in `docs/VERIFICATION_EVIDENCE.md`.
- **Other open decisions:** SAYT's project license remains undecided; logo/artwork provenance and BlueOak/GLib advisory review remain open. The locked `rtrb 0.4.1` is outside the affected range in the reviewed RustSec advisory; details are in the dependency inventory. These are not claims that the app is release-ready.
- Detailed commands, outputs, screenshots boundary, and CI evidence are in [`VERIFICATION_EVIDENCE.md`](VERIFICATION_EVIDENCE.md); milestone states remain in [`FEATURE_ACCEPTANCE_MATRIX.md`](FEATURE_ACCEPTANCE_MATRIX.md).

## Current milestone

**M1 closeout plus approved DJ feature expansion.** The existing Rust engine, Tauri control API, and Stitch-informed workspace remain integrated. The command system and Hot Cues are in the isolated `feat/dj-command-system` branch. M1 hardware/failure-path acceptance remains open; this feature does not close M1 or authorize a merge.

## Verified baseline before this change

- Started from `main` at `1d0323799d0a833a40b20cd58b1c79e43a11044b`; the worktree was clean.
- Created `feat/stitch-m1-dj-workspace`.
- CI setup fix is separately committed as `49676c2` (`fix(ci): remove premature pnpm cache setup`) and pushed to the feature branch.
- Initial run [45702184357](https://github.com/Joshanihub/SAYT/actions/runs/45702184357) exposed Windows Prettier line-ending failures, missing Ubuntu ALSA headers, and a macOS CoreAudio `Send` compile failure. Follow-up run [45710169519](https://github.com/Joshanihub/SAYT/actions/runs/45710169519) passed all required checks on Ubuntu, Windows, and macOS.

## Design reference

The original Stitch export and extracted references are in `design/reference/`. SHA-256: `E97F43E20EDE4BC3993CFB8E4177BB53C3AF6296C58AEE11E0E1DAA347A4B2F1`. See `docs/DESIGN_IMPLEMENTATION_MAP.md` for per-screen states.

## M1 implementation state

- Existing Rust decoder, CPAL output, two-deck transport, gain/EQ, crossfader, master gain/meter, native asynchronous WAV picker, and output-device commands: retained in source; physical behavior remains unverified here. Device switching now attempts replacement after a pause error and reports previous-stream resume failure. Tauri commands reach the engine through a bounded control-thread actor to satisfy CoreAudio's non-`Send` stream ownership.
- Shared shell and DJ / Library / Settings navigation: implemented in source; 22 frontend tests passed locally under Node.js 24.22.3 and pnpm 12.23.0, including the stylesheet motion regression.
- Decks, waveform, transport, seek, and controls: connected to existing Tauri commands. The owner reports loading both fixtures and the requested playback/control checks passed; fixture metadata and independent physical reproduction remain unavailable.
- Mixer: actual master meter, master gain, and crossfader only; no invented independent channel meters.
- Quick crate: two deck slots using the native picker; no directory scan or persistent library.
- Audio settings: device list/selection, refresh, and retry only; discovery distinguishes loading, no listed outputs, and enumeration errors. Unsupported prototype controls remain identified as unavailable.
- Library: explicit empty state because M1 has no index storage.
- Future screens: disabled navigation; not implemented.

## Outstanding exit gates

1. Recheck cross-platform CI after any later changes to the M1 branch; `ee137e2` currently passes exact-head push and PR runs.
2. Record physical unsupported-WAV, no-device, and failed-switch recovery outcomes; decoder, UI, and transition-policy tests cover software paths, but the owner's summary did not identify those physical failure scenarios separately.
3. Complete the two-fixture Hot Cue and Manual Loop desktop run when the SAYT window is available for testing: verify fixture loads/restart and cue restore, loop sizing/in/out/release/seek/stop/track replacement on both decks, and listen for seam artifacts. Update the matched Stitch comparison after each delivery; do not mark Screen 01 complete while later pads, FX, recording, or library browsing remain unavailable.
4. Before a consequential release decision, assess reachability/impact of the five unmaintained Rust crates and `glib` unsoundness notice, review the two BlueOak npm license terms, and complete an SBOM/legal review. Fresh local scan results are recorded in `docs/DEPENDENCIES_AND_LICENSES.md`.
5. SAYT's project license remains undecided; no license grant is added.

## Next milestone

Keep M1 acceptance open until its exit requirements are reviewed and approved. Continue with Loop Roll and Beat Jump as the next approved performance-pad slice. Keep physical loop-seam listening, cue behavior while playing, shortcut-remapping persistence, device recovery, and shutdown evidence visible as open acceptance items; retain the M2 callback-load gates before claiming timing-sensitive performance quality.
