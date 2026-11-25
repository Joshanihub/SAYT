# Milestone 1 implementation report

**Status:** IN PROGRESS<br>
**Owner:** SAYT project owner<br>
**Date:** 2026-11-25<br>
**Version:** 0.1.0

## Implemented in source

- Two independent WAV deck states with play, pause, stop, seek, gain, and low/mid/high controls.
- Bounded PCM WAV decoder and actual peak waveform extraction.
- CPAL output stream, equal-power crossfader, master control and meter, three biquad EQ bands, and output soft limiter.
- Bounded SPSC callback command path, bounded off-callback retired-track queue, atomic telemetry, and device error state.
- Nonblocking native file picker; no frontend-provided arbitrary filesystem path command.
- Required product, requirements, architecture, roadmap, dependency, and persistent agent-instruction documents.

The Windows desktop smoke check below verifies the UI transport clock and local WAV loading; it does not prove audible quality or physical-device reliability.

## Executed checks

The final local verification results are recorded below. Frontend commands used Corepack's pinned pnpm 12.23.0. The host Node.js was 24.16.0, while `.node-version` pins 24.22.3.

- Passed: `cargo fmt --all --check`.
- Passed: `cargo check --workspace --all-targets --locked`.
- Passed: `cargo clippy --workspace --all-targets --locked -- -D warnings`.
- Passed: `cargo test --workspace --locked` (15 engine tests; other workspace crates contain no tests).
- Passed: `corepack pnpm install --frozen-lockfile`.
- Passed: `corepack pnpm format:check`, `corepack pnpm lint`, `corepack pnpm typecheck`, `corepack pnpm test` (1 UI test), and `corepack pnpm build`.
- Passed: `corepack pnpm audit` (no known vulnerabilities).
- Passed: Windows desktop smoke check via `corepack pnpm desktop dev`: launched the app, kept the window responsive while the native picker was open, loaded generated 48 kHz stereo WAV fixtures into both decks, and observed Deck A reach the 4-second end position. This verifies the UI playback state and local stream path, not sound quality measured at the output.
- Passed: `cargo deny check --hide-inclusion-graph bans licenses sources`; duplicate transitive versions remain warnings. SAYT workspace crates are skipped because they are `publish = false` and no project license has been selected.
- Passed with findings: `cargo audit --no-fetch --db C:\Users\amani\.cargo\advisory-dbs\advisory-db-3157b0e258782691` scanned 519 dependencies against cached RustSec DB commit `b9b8a3c` (2026-11-25): zero vulnerability advisories, five unmaintained-crate notices, and one unsoundness advisory for transitive `glib 0.18.5` (`RUSTSEC-2024-0429`). A fresh Git fetch stalled, so the cached same-day database was used; refresh before release or another consequential dependency decision.

## Not verified

- Independently audible physical output quality or simultaneous listening; the UI position advanced, but no calibrated acoustic measurement was made.
- Real-device EQ and crossfader behavior, unplug/reselect recovery, latency, callback load, dropouts, or soak duration.
- Clean desktop shutdown and reliability with an external physical audio device.
- Windows/macOS/Linux CI matrix.

## Not implemented

- Persistent application settings and local library database.
- Hot-plug recovery, headphone cue bus, controller mappings, recording, provider integrations, and crash-isolated plugin hosting.

## Blockers and known review items

- Physical output-device access is required for end-to-end audio evidence.
- Rust dependency audit findings are described in `DEPENDENCIES_AND_LICENSES.md`.
- The project license is pending owner decision; no SAYT license grant is included.

## How to run

Install the pinned Rust toolchain and Node.js, enable Corepack, and install the operating-system prerequisites in the Tauri documentation; then run:

```powershell
corepack pnpm install --frozen-lockfile
corepack pnpm desktop dev
```

## Architecture decisions

- Rust engine: `DECISIONS/ADR-003-rust-engine-milestone1.md`, consistent with `architecture/adr/001-engine-core-rust.md`.
- Tauri desktop shell: `architecture/adr/002-desktop-shell.md`.
- Monorepo boundaries: `architecture/adr/012-monorepo.md`.

## Next milestone

Close M1 by passing local checks, completing cross-platform CI, and recording the hardware checklist in `REQUIREMENTS.md`. Only then begin the M2 reliability work in `ROADMAP.md`.
