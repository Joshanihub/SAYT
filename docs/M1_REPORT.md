# Milestone 1 implementation report

**Status:** IN PROGRESS<br>
**Owner:** SAYT project owner<br>
**Date:** 2026-11-26<br>
**Version:** 0.3.0

## Baseline and design input

- Started from clean `main` at `1d0323799d0a833a40b20cd58b1c79e43a11044b`.
- Work branch: `feat/stitch-m1-dj-workspace`.
- The attached Stitch ZIP is preserved at `design/reference/exports/SAYT-Stitch-v1.zip` and extracted under `design/reference/SAYT-Stitch-v1/`.
- Archive SHA-256: `E97F43E20EDE4BC3993CFB8E4177BB53C3AF6296C58AEE11E0E1DAA347A4B2F1`. ZIP CRC and extraction path checks passed. See `design/reference/README.md` and `docs/DESIGN_IMPLEMENTATION_MAP.md`.

## Implemented in source

- Shared desktop shell with working DJ, Library, and Settings navigation. Bridge, Preparation, Studio, and FX & Sampler are disabled and marked as later work.
- Stitch-informed desktop tokens, responsive CSS breakpoints, two independently controlled deck panels, actual peak waveforms and engine positions, a native-picker quick crate, and one actual master meter with master level and crossfader.
- Output settings limited to the existing device enumeration, selection, refresh, and retry commands. Unsupported sample-rate, buffer, cue, and latency controls are disclosed as unavailable.
- Music Library has a truthful empty state. There is no persistent local index or fabricated track list.
- The existing Rust decoder, CPAL DSP/output callback, Tauri picker, and two-deck command contract remain in place. The Tauri shell now routes controls through a bounded request/reply actor so CPAL's non-`Send` CoreAudio stream stays on its owner thread. No unsafe `Send` implementation or new dependency was added.
- Project and implementation status documents now distinguish source implementation, automated checks, desktop rendering, physical hardware, and deferred screens.

## CI repair and current remote evidence

- First repair commit: `49676c2` (`fix(ci): remove premature pnpm cache setup`), pushed separately to this feature branch. It removes the `cache: pnpm` option from pinned `actions/setup-node`; Node and pnpm versions and frozen-lockfile behavior remain pinned.
- Run [45702184357](https://github.com/Joshanihub/SAYT/actions/runs/45702184357) completed with all three OS jobs failing after setup initialization. The Node/pnpm install steps passed on Windows, macOS, and Ubuntu.
- Windows then failed `pnpm format:check` with Prettier reporting 36 project files. The follow-up adds `.gitattributes` to force LF text checkouts. **Inference:** the cross-platform pattern is consistent with Windows line-ending conversion; the next remote run must confirm this correction.
- Ubuntu failed Rust lint/compile because `alsa-sys` could not find `alsa.pc`; the workflow now installs `libasound2-dev`.
- macOS failed because `cpal::Stream`'s CoreAudio type is not `Send`, so Tauri could not manage `Mutex<Engine>`. The desktop shell now keeps `Engine` inside a dedicated owner thread and exposes only a bounded request/reply sender through Tauri state.
- Follow-up remote CI is **not yet run**. No OS matrix is claimed green until GitHub Actions reports all required checks passing.

## Local automated checks

The commands below were run on 2026-11-26 after the final source and test edits; the repository-wide frontend format check was rerun after the final documentation edits. Frontend verification used Node.js 24.22.3 and pnpm 12.23.0, matching the repository pins.

- Passed: `cargo fmt --all --check`.
- Passed: `cargo check --workspace --all-targets --locked`.
- Passed: `cargo clippy --workspace --all-targets --locked -- -D warnings`.
- Passed: `cargo test --workspace --locked` — 15 engine tests and 1 audio-service actor test passed.
- Passed: `corepack pnpm install --frozen-lockfile`, `corepack pnpm format:check`, `corepack pnpm lint`, `corepack pnpm typecheck`, `corepack pnpm test` (20 component and callback tests), and `corepack pnpm build`, using Node.js 24.22.3 and pnpm 12.23.0.
- Component callback tests cover transport, seek, gain/EQ, mixer control, deck routing, device selector callbacks, error actions, crate routing, and the `pick_track` command name/arguments using a mocked Tauri invoke. They do not prove Tauri IPC, native picker operation, or physical playback.
- The failure path was checked by temporarily changing the play command name; the targeted Deck B test failed with the expected/actual mismatch. The product code was restored and all 20 tests passed afterward.
- No dependency or lockfile changed. On 2026-11-26, `cargo audit` refreshed RustSec and exited successfully with no vulnerability advisories and six allowed warnings (five unmaintained crates and the `glib 0.18.5` unsoundness advisory). `cargo deny check --hide-inclusion-graph bans licenses sources` passed with duplicate-lockfile-entry warnings. `corepack pnpm audit` reported no known vulnerabilities. The recursive npm license inventory still contains two `BlueOak-1.0.0` development packages. See `docs/DEPENDENCIES_AND_LICENSES.md`; none of these scan results is legal approval or a reachability determination.

## Desktop and physical-device verification

- **Desktop render:** the Windows Tauri app launched and the DJ workspace appeared at 1920×1080 before the final output-label wording correction and test-only changes. Its accessibility tree showed a connected engine snapshot, two empty decks, disabled future navigation, and a single master meter. This verifies that a window rendered, not that WAV loading or control commands completed in this pass.
- **Native picker:** not exercised in this pass; no file was selected.
- **Target responsive sizes:** CSS breakpoints are implemented, but 1440×900, 1280×800, and narrow-device runtime layouts were not separately inspected.
- **Physical output:** not verified. Simultaneous audibility, gain/EQ/crossfader listening, device recovery, latency/dropouts, soak, and external-device shutdown still require the checklist in `docs/REQUIREMENTS.md`.
- The previous report's Windows fixture smoke test is historical baseline evidence only; it is not new evidence from this implementation pass.

## Not implemented or still deferred

- Persistent music library, playlists/bridge, preparation/analysis, appearance customization, performance effects, sampler, Studio/DAW workspaces, onboarding, input/recording, headphone cue bus, controller mappings, and device hot-plug recovery.
- A full 512 MiB-per-deck file is only a decoder safety bound, not a usability or performance guarantee.

## Security, dependencies, and license

- This change adds no dependencies, changes no dependency versions, and does not modify the package lockfiles.
- Current dependency checks are recorded in `docs/DEPENDENCIES_AND_LICENSES.md`: RustSec refreshed on 2026-11-26, no vulnerability advisories, six allowed Rust warnings; `cargo deny` and npm audit passed; two npm BlueOak license strings remain for owner review. Reachability and legal interpretation remain open.
- The SAYT project license is still undecided. No root license file, project SPDX grant, or legal approval was added.

## Git and next gate

- Branch: `feat/stitch-m1-dj-workspace`; base commit `1d0323799d0a833a40b20cd58b1c79e43a11044b`; separate CI setup commit `49676c2`.
- The implementation commit, follow-up CI run, and pull request URL are added after push.
- M1 remains **IN PROGRESS** until the cross-platform CI run passes and the owner records physical-device evidence for two real WAV files, independent transport, gain/EQ/crossfader, device failure/recovery, and shutdown. Do not begin M2 before that review.
