# SAYT project status

**Status:** IN PROGRESS<br>
**Owner:** SAYT project owner<br>
**Date:** 2026-11-26<br>
**Version:** 0.4.5

## Current verified position — audio-validation integration

- **Repository/worktree:** `C:\Dev\SAYT`; branch `feat/stitch-m1-dj-workspace`; base `1d0323799d0a833a40b20cd58b1c79e43a11044b`; integration commit `853e51477e56593471c7126378d290524be391af`.
- **Remote review:** [PR #1](https://github.com/Joshanihub/SAYT/pull/1) is open, unmerged, and contains the integrated toolkit. [PR #2](https://github.com/Joshanihub/SAYT/pull/2) is still open/draft as required; it was not merged or closed. `main` remains unchanged.
- **Current review state:** PR #1 is open, unmerged, mergeable, and points to `8e38a432f529f0b118bc3f804d7867e5a3f5dda7`; PR #2 remains open/draft. GitHub confirms neither is merged. `main` remains unchanged.
- **Latest exact-head CI:** push run [45736072298](https://github.com/Joshanihub/SAYT/actions/runs/45736072298) and PR run [45736076490](https://github.com/Joshanihub/SAYT/actions/runs/45736076490) passed Windows, macOS, and Ubuntu for branch head `8e38a432f529f0b118bc3f804d7867e5a3f5dda7`. The PR Ubuntu log records 52 toolkit tests and 77/77 fixture checks; expected-failure checks deliberately exercise invalid paths.
- **Exact-head CI:** push run [45721302725](https://github.com/Joshanihub/SAYT/actions/runs/45721302725) and PR run [45721308789](https://github.com/Joshanihub/SAYT/actions/runs/45721308789) passed Windows, macOS, and Ubuntu for PR head `853e51477e56593471c7126378d290524be391af`; GitHub tested merge ref `198d769fb8439a5a1471441e50c29cb0b07711cb` on the PR event. The Ubuntu PR log records 52 toolkit tests, `test_symlink_escape_rejected ... ok`, and 77/77 fixture checks.
- **Toolkit:** accepted source/docs from PR #2 were reviewed and imported. PR #2's `.github/workflows/ci.yml` was excluded; the proven PR #1 workflow was preserved and only gained the Linux Python test step. The generator/verifier received symlink/hard-link-safe atomic writes and Windows-compatible ASCII output.
- **Local evidence:** Rust format/check/Clippy/tests passed on rustc/cargo 1.100.0. Frontend frozen install/format/lint/typecheck/20 tests/build passed on Node 24.22.3 and pnpm 12.23.0. Python 3.14.5 toolkit suite: 52 total, 50 passed, 2 symlink tests skipped because Windows host permissions disallow symlink creation. Seven fixtures were generated and verified: 77 checks passed, 0 failed; manifest SHA-256 `1FA8F84E47C17F6FC017F99D5023AC8CE4C683B400BE788BE98C5E6E586A4701`.
- **M1 remains IN PROGRESS:** actual native-picker load, real Tauri control IPC, exact target-size comparisons, audible dual playback, physical gain/EQ/crossfader, device recovery, and shutdown during playback remain NOT RUN. The owner has said they can run the manual checks; no results have been received yet. Previous Tauri evidence is only the initial Windows render and connected engine snapshot at 1920×1032.
- **Current running desktop session:** `corepack pnpm desktop dev` was launched when the checkout was at documentation commit `c79a448b433f4f29f50236254621d8020105a126` with Node.js 24.22.3/pnpm 12.23.0. The product-code tree is unchanged from integration commit `853e51477e56593471c7126378d290524be391af`; current head `8e38a43` contains documentation only. The process remains available for owner operation; the two requested WAV fixtures exist in `%TEMP%\SAYT-Audio-Fixtures`. Cargo emitted a non-fatal `sayt_desktop.pdb` output filename collision warning; picker, controls, IPC and physical output have not been exercised in this launch.
- **Design fidelity gap:** the app has no bundled font files, `@font-face`, or Material Symbols implementation. The Windows host lacks the three Stitch text fonts; its current body font request falls through to a system font. App text marks `♫` and `◉` also differ from the Stitch icon font. Exact candidate asset URLs, hashes, licenses and the audit-order variance are recorded in `docs/DEPENDENCIES_AND_LICENSES.md`; no assets entered the checkout.
- Detailed current commands, test results, runtime boundary, rollback and CI evidence are in [`VERIFICATION_EVIDENCE.md`](VERIFICATION_EVIDENCE.md); feature states are in [`FEATURE_ACCEPTANCE_MATRIX.md`](FEATURE_ACCEPTANCE_MATRIX.md).

## Current milestone

**M1 — desktop foundation and two-deck audio.** The Rust engine and the existing Tauri control API are retained. This change implements the Stitch-informed desktop shell, truthful DJ workspace, output settings, and library empty state. It does not close M1.

## Verified baseline before this change

- Started from `main` at `1d0323799d0a833a40b20cd58b1c79e43a11044b`; the worktree was clean.
- Created `feat/stitch-m1-dj-workspace`.
- CI setup fix is separately committed as `49676c2` (`fix(ci): remove premature pnpm cache setup`) and pushed to the feature branch.
- Initial run [45702184357](https://github.com/Joshanihub/SAYT/actions/runs/45702184357) exposed Windows Prettier line-ending failures, missing Ubuntu ALSA headers, and a macOS CoreAudio `Send` compile failure. Follow-up run [45710169519](https://github.com/Joshanihub/SAYT/actions/runs/45710169519) passed all required checks on Ubuntu, Windows, and macOS.

## Design reference

The original Stitch export and extracted references are in `design/reference/`. SHA-256: `E97F43E20EDE4BC3993CFB8E4177BB53C3AF6296C58AEE11E0E1DAA347A4B2F1`. See `docs/DESIGN_IMPLEMENTATION_MAP.md` for per-screen states.

## M1 implementation state

- Existing Rust decoder, CPAL output, two-deck transport, gain/EQ, crossfader, master gain/meter, native asynchronous WAV picker, and output-device commands: retained in source; physical behavior remains unverified here. Tauri commands reach the engine through a bounded control-thread actor to satisfy CoreAudio's non-`Send` stream ownership.
- Shared shell and DJ / Library / Settings navigation: implemented in source; 20 frontend component/callback tests passed locally under Node.js 24.22.3 and pnpm 12.23.0.
- Decks, waveform, transport, seek, and controls: connected to existing Tauri commands; actual source WAV and physical playback checks remain required.
- Mixer: actual master meter, master gain, and crossfader only; no invented independent channel meters.
- Quick crate: two deck slots using the native picker; no directory scan or persistent library.
- Audio settings: device list/selection, refresh, and retry only; unsupported prototype controls are identified as unavailable.
- Library: explicit empty state because M1 has no index storage.
- Future screens: disabled navigation; not implemented.

## Outstanding exit gates

1. Local Rust and frontend format, lint, typecheck, unit, and build checks passed; exact commands, runtime pins, and test limits are recorded in `docs/M1_REPORT.md`.
2. Review the passing cross-platform run [45710169519](https://github.com/Joshanihub/SAYT/actions/runs/45710169519) alongside the still-open physical-device acceptance gate.
3. Record the owner’s manual native-picker, loaded-deck, transport, mixer, device-recovery and playback-shutdown results in `docs/audio-validation/RESULTS_TEMPLATE.md`.
4. Inspect the actual Tauri window at 1440×900, 1280×800, and a narrow viewport; compare the supplied Stitch screen and record deviations. Include exact font/icon parity after the owner selects the audited assets and text coverage.
5. Before a consequential release decision, assess reachability/impact of the five unmaintained Rust crates and `glib` unsoundness notice, review the two BlueOak npm license terms, and complete an SBOM/legal review. Fresh local scan results are recorded in `docs/DEPENDENCIES_AND_LICENSES.md`.
6. SAYT's project license remains undecided; no license grant is added.

## Next milestone

Do not start M2 reliability implementation until M1 exit requirements have been reviewed and approved. After M1, the roadmap prioritizes callback load/dropout measurement, device loss recovery, and shutdown verification.
