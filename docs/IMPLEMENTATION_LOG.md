# Implementation log

**Status:** ACTIVE<br>
**Owner:** SAYT project owner<br>
**Date:** 2026-11-26<br>
**Version:** 0.4.12

## 2026-11-26 — Keyboard-command CI status correction

- GitHub Actions run `45883560745` for the implementation commit was canceled when the follow-up status commit superseded it on the branch. Exact-head run [45884170431](https://github.com/Joshanihub/SAYT/actions/runs/45884170431) for branch head `2706945e62b25573cc4d4b44af2248011b878d5c` subsequently completed successfully; GitHub reports all 3 matrix jobs completed. The physical-key acceptance check remains open pending owner observation.
- No source code, dependency, or lockfile changed in this status correction.

## 2026-11-26 — First DJ expansion: keyboard command system

- Continued in the isolated `feat/dj-command-system` worktree based on M1 head `ee137e288b324a50e6f9b1574c4aee9d4eee1c09`. Added a comprehensive official-source capability tracker for Serato DJ Pro and VirtualDJ and a separate keyboard-command acceptance contract and runbook.
- Added a typed command registry with global/focused-deck contexts, platform-aware defaults, editable mappings, conflict/reserved-key checks, versioned local persistence, reset, command search, and an in-app help overlay. Existing deck, mixer, device and workspace controls plus keyboard shortcuts share the same `AppCommand` dispatcher and existing Tauri APIs.
- Added visible, remapping-aware key hints beside deck focus, transport, gain/EQ, mixer and help controls. Deck pointer/focus and successful load selection update the focused-deck target. Unsupported cue, stutter, pitch, sync, loop, effects, sampler, library search, recording and controller commands remain absent until their real systems exist.
- Extracted the production keydown path into `dispatchShortcutKeydown` so shortcut matching, typed dispatch, repeat rules, modal/input exclusion and native Space handling can be exercised directly. The App window listener calls the same helper.
- Verification checkpoint: frontend format, lint, typecheck, 46 tests and production build pass. Two intentional mutation canaries failed their expected assertions (platform default and typed command dispatch); restored focused tests passed. Rust format, Clippy, 25 engine tests plus 1 desktop actor test, and locked all-target check passed on this worktree.
- Windows Tauri runtime launched from this worktree at `127.0.0.1:1421`, 1282×832 window. The initial DJ workspace, footer help button, shortcut dialog, crossfader search results and Done close action were observed. Global key actions sent via `sky.press_key` (F1, Alt+2, Ctrl+Alt+2, Ctrl+Alt+Up and Alt+Shift+Q) produced no visible command-state change; the cause is unresolved between this automation input path and the application. No physical keyboard, fixture load, audio output or remapping persistence claim is made. Implementation commit `c5d4fa9` is pushed to `feat/dj-command-system`; exact-head Actions run [45883560745](https://github.com/Joshanihub/SAYT/actions/runs/45883560745) is in progress. No PR or merge was created.

## 2026-11-26 — M1 responsive workspace correction

- A real 800×700 client capture showed that automatic CSS grid placement put Deck B after the full-width mixer, leaving the right column empty and the second deck below the fold. The previous 800 px Tauri minimum also made the existing 720 px stacked breakpoint unreachable.
- Added named desktop, compact, and narrow grid areas so the 1060 px layout places both decks above the mixer, and the 720 px layout stacks Deck A, Deck B, then mixer. Lowered Tauri's minimum width to 480 px to make the established narrow styles usable.
- Added a stylesheet regression test for each grid mode and added responsive placement to the M1 requirements. The complete UI suite passes; four screenshots were captured from the running Tauri window using Win32 `PrintWindow` at client sizes verified by `GetClientRect` and are stored in `docs/evidence/m1-responsive/`.
- Compared the captured M1 empty workspace with the supplied Screen 01 reference. The deck colors, dark surfaces, type roles, and high-level deck/mixer arrangement are present. The screen remains a partial implementation: cue/loop/pad controls, pitch/key, per-deck meters, effects/recording, and the full library browser are not implemented.
- Local verification passed: Rust format, Clippy, workspace tests (25 engine, 1 desktop actor), and locked all-target check; frontend format, lint, typecheck, 23 UI tests, and production build. A controlled mutation of the desktop grid order made the regression test fail; source bytes were restored and the test rerun passed. Responsive change `ee137e2` is committed and pushed. Exact-head push run [45831799052](https://github.com/Joshanihub/SAYT/actions/runs/45831799052) and PR run [45831807175](https://github.com/Joshanihub/SAYT/actions/runs/45831807175) passed Windows, macOS, and Ubuntu.

## 2026-11-26 — M1 output-switch recovery

- Source review found that `Engine::select_device` returned immediately when pausing the current CPAL stream failed, so the replacement device was never attempted. This contradicted M1 requirement 7's failed-switch recovery contract.
- Updated the switch path to attempt the replacement despite a pause error. If opening the replacement fails, it best-effort resumes and retains the previous stream; a failed resume sets output-error telemetry and is included in the actionable error. No stream or command is silently discarded.
- Added four hardware-independent tests for replacement after pause failure, successful restore, failed restore, and initial selection with no previous stream. They test transition policy only; no physical device behavior is claimed.
- Updated `docs/REQUIREMENTS.md`, the audio-engine notes, and the physical device test guide. Local format, Clippy, workspace tests (19 engine tests and 1 audio-service test), locked all-target check, frontend format/lint/typecheck, 21 UI tests, and production build passed before commit. The change was committed and pushed as `14ea4d9f102e39860ee6ad76b10e44f04cb574a0`; exact-head push run [45812052568](https://github.com/Joshanihub/SAYT/actions/runs/45812052568) and PR run [45812058496](https://github.com/Joshanihub/SAYT/actions/runs/45812058496) passed on Windows, macOS, and Ubuntu.
- On 2026-11-26 the owner reported that all requested manual audio checks passed. The response did not include device, viewport, per-test observations, or error text, so the acceptance matrix records owner-reported evidence rather than independent reproduction. The current accessibility snapshot shows two populated, stopped decks but does not identify the tested fixtures.
- Two Computer Use screenshot attempts returned unrelated Chrome content while the selected SAYT window's accessibility tree exposed the SAYT interface. Target-size screenshots and Stitch visual comparisons remain NOT RUN.

## 2026-11-26 — M1 interaction motion and reduced-motion support

- The design-system audit found that control hover changes had no transition and the stylesheet did not honor `prefers-reduced-motion`.
- Added a named 120 ms ease-out transition to button and select state changes, including navigation, transport, and load controls. Added a global reduced-motion media rule that removes CSS animation/transition duration and smooth scrolling while leaving keyboard focus outlines immediate. The master meter remains tied to engine snapshot values.
- Added `apps/desktop/src/styles.test.ts` to assert the transition tokens/selectors and reduced-motion rules. The full UI suite passed 22 tests. A temporary 1 ms mutation to the reduced-motion animation rule made the regression test fail; the source was restored and the suite passed again.
- At the motion commit, local checks passed: Rust format, Clippy, all 20 workspace tests (19 engine and 1 desktop actor), locked all-target check, frontend format, lint, typecheck, 22 UI tests, and production build.
- Runtime appearance at target sizes is still open because the desktop screenshot capture returned unrelated Chrome content twice.

## 2026-11-26 — M1 WAV decoder rejection coverage

- Added tests for truncated PCM sample data, empty files, unsupported three-channel WAV, sample rates outside both supported bounds, stereo PCM24/PCM32 normalization, and the decoded-frame boundary. The focused decoder suite passes 10 tests.
- The truncated-sample test failed under a temporary mutation of the actionable error prefix; the original decoder message was restored and the test passed again.
- The frame-limit test failed when a temporary canary changed the rejection guard from `>=` to `>`; the original guard was restored and the boundary test passed.
- A first waveform assertion expected the maximum channel magnitude; it failed because waveform bins intentionally average absolute left/right levels. The assertion was corrected to the implemented 0.75 result for 0.5 and 1.0 channel amplitudes.
- Full local verification passes: Rust format, Clippy, 25 engine tests plus 1 desktop actor test, locked all-target check, and repository formatting. Frontend source remains unchanged since the motion commit.

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
