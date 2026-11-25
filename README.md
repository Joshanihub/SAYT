# SAYT

SAYT is an open, local-first music workstation. The first product target is a reliable desktop DJ application, with library migration and playlist bridging as the initial differentiator.

**Project status:** Milestone 1 is in progress. The source implements local PCM WAV loading, two transport decks, gain and three-band controls, crossfader mixing, metering, device enumeration, and an OS audio-output path. Build and automated-check results are in [`docs/M1_REPORT.md`](docs/M1_REPORT.md); physical playback still requires manual verification. Current scope and gates are in [`docs/ROADMAP.md`](docs/ROADMAP.md).

## Principles

- Audio must keep playing through UI or network failures.
- Core workflows must work locally without an account or internet connection.
- Track metadata and audio rights are separate; SAYT will not extract or redistribute service audio.
- Imports and exports use documented formats, and source libraries remain untouched.
- Automatic analysis is optional, explainable, and editable.
- Controller support is claimed only after a mapping is tested.

## Current scaffold

- Rust workspace for the desktop shell and engine, DSP, decode, renderer, contract, and library crates.
- Tauri 2 desktop shell with a React and TypeScript UI.
- PCM WAV decode, deck transport, mixing, EQ controls, output-device selection, and Tauri commands are present; see [`docs/audio-engine/README.md`](docs/audio-engine/README.md).
- CI checks on Windows, macOS, and Linux.
- Dependency versions are pinned in manifests and lockfiles.

No project license has been selected. The repository has no license grant; do not reuse or redistribute SAYT code until the owner chooses one. The GPL-3.0-or-later text from the founding draft is not part of the public repository. See [`docs/DEPENDENCIES_AND_LICENSES.md`](docs/DEPENDENCIES_AND_LICENSES.md).

## Develop

Install the pinned Rust toolchain from `rust-toolchain.toml`, Node.js from `.node-version`, and pnpm from the `packageManager` field in `package.json`. Tauri's platform prerequisites are listed in its [official prerequisites guide](https://v2.tauri.app/start/prerequisites/).

```powershell
corepack pnpm install --frozen-lockfile
corepack pnpm dev
```

The commands above start the web UI through Vite. To start the native desktop shell, run:

```powershell
corepack pnpm desktop dev
```

## Checks

```powershell
corepack pnpm format:check
corepack pnpm lint
corepack pnpm typecheck
corepack pnpm test
corepack pnpm build
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo deny check --hide-inclusion-graph bans licenses sources
cargo audit
```

Linux builds need Tauri's system packages; see the runbook before interpreting a missing WebKit or GTK library as a SAYT source failure.

## Scope

M1 is the active two-deck prototype milestone. Source implementation and local checks do not establish gig safety or physical-device playback. Providers, recording, controller support, plugins, networking, accounts, and cloud services remain out of scope. See the milestone gates before expanding these systems.
