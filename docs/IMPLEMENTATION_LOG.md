# Implementation log

**Status:** ACTIVE<br>
**Owner:** SAYT project owner<br>
**Date:** 2026-11-26<br>
**Version:** 0.4.1

## 2026-11-26 — M1 output-switch recovery

- Source review found that `Engine::select_device` returned immediately when pausing the current CPAL stream failed, so the replacement device was never attempted. This contradicted M1 requirement 7's failed-switch recovery contract.
- Updated the switch path to attempt the replacement despite a pause error. If opening the replacement fails, it best-effort resumes and retains the previous stream; a failed resume sets output-error telemetry and is included in the actionable error. No stream or command is silently discarded.
- Added four hardware-independent tests for replacement after pause failure, successful restore, failed restore, and initial selection with no previous stream. They test transition policy only; no physical device behavior is claimed.
- Updated `docs/REQUIREMENTS.md`, the audio-engine notes, and the physical device test guide. Local format, Clippy, workspace tests (19 engine tests and 1 audio-service test), locked all-target check, frontend format/lint/typecheck, 21 UI tests, and production build passed on the working tree based on `8ecbb9e3079c1b04848fa8480a515b08c4626c85`. Exact-head CI is pending commit and push.
- `corepack pnpm desktop dev` rebuilt and launched the current working tree. No picker, device-control, visual comparison, or physical playback action has been performed by the agent.

## 2026-11-26 — Current-head desktop launch for owner verification

- Launched `corepack pnpm desktop dev` from commit `17693ba8341817e9152b8f66f55a07009f6139ad` with Node.js 24.22.3 and pnpm 12.23.0. Vite became ready, Rust compilation finished, and `sayt-desktop.exe` remained alive at the last process check.
- Cargo printed a non-fatal output filename collision warning for `sayt_desktop.pdb` shared by the binary and library targets; launch did not fail.
- No native picker, UI control, Tauri IPC, viewport, or physical audio interaction was executed by the agent. The app is left running for owner-operated M1 verification. Do not infer that launch proves playback or device behavior.

## 2026-11-26 — PR #2 audio-validation consolidation

- Audited PR #2 at `57bbe6234842a4cd9f3ff05de8049301055cacbe` through GitHub/FETCH_HEAD without checking out its branch or touching the separate validation checkout. Reviewed the generator, verifier, FFT/Goertzel frequency tests, path tests, device guide and results template.
- Imported only the reviewed `tools/audio-validation/` and `docs/audio-validation/` files. Preserved PR #1's CI setup and ALSA fixes; added a Linux-only Python test step so Linux executes the symlink escape regression.
- Fixed Windows console encoding failure by using ASCII CLI status messages. Hardened output paths against symlink redirection and partial directory collisions; fixture, sentinel and manifest writes now use atomic replacement to avoid writing through symlinks or hard links.
- Added three regression tests for symlink output rejection, preflight directory rejection, and hard-link-safe replacement. Existing verifier tests retain Windows/POSIX absolute, drive-relative, traversal and symlink confinement checks.
- Reproduced 52 Python tests: 50 passed and 2 symlink cases skipped under Windows privilege policy. Generated seven fixtures in `%TEMP%\SAYT-Audio-Fixtures`; verification passed all 77 checks. SHA-256 of `manifest.json`: `1FA8F84E47C17F6FC017F99D5023AC8CE4C683B400BE788BE98C5E6E586A4701`.
- Re-ran Rust and frontend checks; Rust format/check/Clippy/tests passed, and frontend frozen install/format/lint/typecheck/20 tests/build passed under pinned Node 24.22.3 and pnpm 12.23.0.
- Committed and pushed `853e51477e56593471c7126378d290524be391af` (`test(m1): consolidate audio validation toolkit`) to `feat/stitch-m1-dj-workspace`; PR #1 remains open and unmerged; PR #2 remains open/draft. Push run `45721302725` and PR run `45721308789` passed Windows, macOS, and Ubuntu; the PR run's Linux log shows all 52 tests and the symlink regression passing.
- M1 hardware/runtime gates remain open. The physical-device guide is ready, but native loading, actual Tauri IPC, audible output, device recovery and shutdown during playback were not exercised in this increment.

## 2026-11-26 — M1 closeout preparation

- Audited the repository at `1d0323799d0a833a40b20cd58b1c79e43a11044b`, branch `main`, clean worktree. Preserved the existing Rust engine and existing frontend control paths.
- Created feature branch `feat/stitch-m1-dj-workspace`.
- Removed premature `cache: pnpm` setup from `.github/workflows/ci.yml` in its own commit, `49676c2`. The commit is pushed. The first remote run passed Node/pnpm initialization across all three OSes but exposed Windows formatting, missing Linux ALSA headers, and a macOS CoreAudio `Send` compile failure. Follow-up corrections were committed as `c517a49` and pushed with the implementation. Run [45710169519](https://github.com/Joshanihub/SAYT/actions/runs/45710169519) on `693983f` passed on all three OSes.
- Verified the Stitch ZIP CRC, path safety, source/copy SHA-256, and extracted it under `design/reference/SAYT-Stitch-v1/`; see `design/reference/README.md`.
- Implemented a shared shell, real engine-backed DJ workspace, single master meter, native-picker quick crate, limited output-device settings, and explicit library empty state. The `sayt-engine` DSP/decoder source, dependencies, package manifests, lockfile, and project-license metadata are unchanged. The Tauri shell now owns the engine through a bounded request/reply actor thread because CoreAudio's CPAL stream is not `Send` on macOS; no unsafe `Send` implementation was added.
- Added component and callback tests for workspace semantics, empty/loading states, actual waveform peak/position mapping, deck-specific transport/seek/gain/EQ, master/crossfader, device listing/selection/error actions, crate routing, and the mocked `pick_track` Tauri command contract.
- Updated M1/product copy and added the design map and current status documents.

### Verification record

Passed on 2026-11-26: `cargo fmt --all --check`, `cargo check --workspace --all-targets --locked`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, and `cargo test --workspace --locked` (15 engine tests and 1 actor test). Frontend checks passed using Node.js 24.22.3 and pnpm 12.23.0: `corepack pnpm install --frozen-lockfile`, `corepack pnpm format:check`, `corepack pnpm lint`, `corepack pnpm typecheck`, `corepack pnpm test` (20 tests), and `corepack pnpm build`. Dependency checks passed locally; the refreshed advisory details are in `docs/DEPENDENCIES_AND_LICENSES.md`. GitHub Actions run [45710169519](https://github.com/Joshanihub/SAYT/actions/runs/45710169519) passed all required steps on Ubuntu, Windows, and macOS. The Windows desktop app rendered its empty DJ workspace at 1920×1080 before the final output-label wording correction; no picker or playback interaction was completed in that render pass. Physical output remains unverified.

- Canary failure check: temporarily changed the play-command name to a wrong value and confirmed the focused Deck B callback test failed. Restored the command and reran the full UI suite successfully (20/20).
