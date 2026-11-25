# Implementation log

**Status:** ACTIVE<br>
**Owner:** SAYT project owner<br>
**Date:** 2026-11-26<br>
**Version:** 0.2.0

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
