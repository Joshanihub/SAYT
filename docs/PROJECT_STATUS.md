# SAYT project status

**Status:** IN PROGRESS<br>
**Owner:** SAYT project owner<br>
**Date:** 2026-11-26<br>
**Version:** 0.4.8

## Current verified position — M1 output-switch recovery

- **Repository and baseline:** `C:\Dev\SAYT`, branch `feat/stitch-m1-dj-workspace`, based on `main` at `1d0323799d0a833a40b20cd58b1c79e43a11044b`. Current local recovery changes are based on pushed commit `8ecbb9e3079c1b04848fa8480a515b08c4626c85` and still require a commit and exact-head CI.
- **GitHub review:** PR #1 is open, unmerged, and mergeable; PR #2 remains open and draft. Both target unchanged `main`; neither is merged. Exact-head push run [45804224796](https://github.com/Joshanihub/SAYT/actions/runs/45804224796) and PR run [45804236667](https://github.com/Joshanihub/SAYT/actions/runs/45804236667) passed Windows, macOS, and Ubuntu for `8ecbb9e3079c1b04848fa8480a515b08c4626c85`.
- **Stitch typography and icons:** 16 local variable WOFF2 text-font subsets and a 66-ligature Material Symbols subset are integrated. Four upstream license notices are present under `apps/desktop/public/licenses`; runtime font loading makes no network request. Exact sources, hashes, sizes, Unicode ranges, and build-tool pins are recorded in the asset manifest and dependency inventory.
- **Local verification:** after the output-switch recovery change, Rust format, Clippy, workspace tests (19 engine and 1 desktop actor), and locked all-target check passed. Frontend format, lint, typecheck, 21 tests, and production build passed. Earlier font-asset verification passed for 17 WOFF2 assets, 4 notices, all 5 M1 icon names, and font-role declarations; three font-role regression tests passed. Vite copied notices to `dist/licenses`.
- **Output-switch recovery:** source now attempts the replacement stream after a pause error and restores the previous stream if replacement fails. Resume failure is reported in the command error and output-error snapshot. Four hardware-independent transition tests pass; CPAL/OS recovery remains unverified.
- **Desktop render:** The current Windows Tauri DJ empty state was observed at a 1280×800 client viewport; the corrected typography and Material Symbols rendered. No target-size Stitch comparison has been completed.
- **M1 remains IN PROGRESS:** actual native-picker WAV loading, real Tauri transport/mixer interactions, audible dual playback, gain/EQ/crossfader listening, device refresh/retry and recovery, playback shutdown, and the three target-size visual comparisons remain NOT RUN. The dev app is running with the local recovery change for owner testing; no manual results have been received yet.
- **Other open decisions:** SAYT's project license remains undecided; logo/artwork provenance and prior BlueOak/Rust advisory review remain open. These are documented separately and are not claims that the app is release-ready.
- Detailed commands, outputs, screenshots boundary, and CI evidence are in [`VERIFICATION_EVIDENCE.md`](VERIFICATION_EVIDENCE.md); milestone states remain in [`FEATURE_ACCEPTANCE_MATRIX.md`](FEATURE_ACCEPTANCE_MATRIX.md).

## Current milestone

**M1 — desktop foundation and two-deck audio.** The existing Rust engine, Tauri control API, and Stitch-informed workspace remain integrated. This batch improves device-switch failure recovery. It does not close M1.

## Verified baseline before this change

- Started from `main` at `1d0323799d0a833a40b20cd58b1c79e43a11044b`; the worktree was clean.
- Created `feat/stitch-m1-dj-workspace`.
- CI setup fix is separately committed as `49676c2` (`fix(ci): remove premature pnpm cache setup`) and pushed to the feature branch.
- Initial run [45702184357](https://github.com/Joshanihub/SAYT/actions/runs/45702184357) exposed Windows Prettier line-ending failures, missing Ubuntu ALSA headers, and a macOS CoreAudio `Send` compile failure. Follow-up run [45710169519](https://github.com/Joshanihub/SAYT/actions/runs/45710169519) passed all required checks on Ubuntu, Windows, and macOS.

## Design reference

The original Stitch export and extracted references are in `design/reference/`. SHA-256: `E97F43E20EDE4BC3993CFB8E4177BB53C3AF6296C58AEE11E0E1DAA347A4B2F1`. See `docs/DESIGN_IMPLEMENTATION_MAP.md` for per-screen states.

## M1 implementation state

- Existing Rust decoder, CPAL output, two-deck transport, gain/EQ, crossfader, master gain/meter, native asynchronous WAV picker, and output-device commands: retained in source; physical behavior remains unverified here. Device switching now attempts replacement after a pause error and reports previous-stream resume failure. Tauri commands reach the engine through a bounded control-thread actor to satisfy CoreAudio's non-`Send` stream ownership.
- Shared shell and DJ / Library / Settings navigation: implemented in source; 21 frontend component/callback tests passed locally under Node.js 24.22.3 and pnpm 12.23.0.
- Decks, waveform, transport, seek, and controls: connected to existing Tauri commands; actual source WAV and physical playback checks remain required.
- Mixer: actual master meter, master gain, and crossfader only; no invented independent channel meters.
- Quick crate: two deck slots using the native picker; no directory scan or persistent library.
- Audio settings: device list/selection, refresh, and retry only; unsupported prototype controls are identified as unavailable.
- Library: explicit empty state because M1 has no index storage.
- Future screens: disabled navigation; not implemented.

## Outstanding exit gates

1. Local Rust and frontend format, lint, typecheck, unit, and build checks passed for the working tree; exact commands and limits are recorded in `docs/VERIFICATION_EVIDENCE.md`.
2. Commit and push the recovery change, then verify its exact-head Windows/macOS/Ubuntu CI alongside the still-open physical-device acceptance gate.
3. Record the owner’s manual native-picker, loaded-deck, transport, mixer, device-recovery and playback-shutdown results in `docs/audio-validation/RESULTS_TEMPLATE.md`.
4. Compare the current 1280×800 Tauri render and new 1440×900 and narrow screenshots against the supplied Stitch PNGs; record intentional deviations and confirm local font/icon appearance plus text fallback behavior.
5. Before a consequential release decision, assess reachability/impact of the five unmaintained Rust crates and `glib` unsoundness notice, review the two BlueOak npm license terms, and complete an SBOM/legal review. Fresh local scan results are recorded in `docs/DEPENDENCIES_AND_LICENSES.md`.
6. SAYT's project license remains undecided; no license grant is added.

## Next milestone

Do not start M2 reliability implementation until M1 exit requirements have been reviewed and approved. After M1, the roadmap prioritizes callback load/dropout measurement, device loss recovery, and shutdown verification.
