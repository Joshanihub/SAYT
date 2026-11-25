# SAYT coding-agent rules

**Owner:** SAYT project owner<br>
**Status:** PROPOSED project working rules<br>
**Date:** 2026-11-25<br>
**Version:** 1.0

## Read before changing code

1. Read this file, the relevant requirement in `docs/REQUIREMENTS.md`, the component's README, and applicable decisions in `docs/DECISIONS/` or `docs/architecture/adr/`.
2. Inspect the current implementation and working tree before editing. Preserve concurrent changes unless the owner directs otherwise.
3. Treat `docs/PRODUCT_BRIEF.md`, `docs/REQUIREMENTS.md`, and accepted ADRs as the current product boundary. The founding charter is strategic context, not evidence that a feature exists.

## Audio-engine invariants

- The CPAL output callback must not allocate or free memory, lock, perform I/O, log, call UI code, or cross Tauri/JavaScript boundaries.
- Decode WAV files and compute EQ coefficients on the control side. Pass bounded commands through the SPSC queue. Retire replaced tracks through the bounded return queue so their final `Arc` drop occurs off the callback.
- Bound work per callback. A queue-full condition must remain visible to the caller or defer work; never report an unapplied command as success.
- Keep audio state and telemetry tied to the current track generation. Add a regression test when changing replacement, seek, device-switch, or telemetry behavior.
- Keep callback DSP deterministic and independently testable without physical hardware.
- Never claim physical playback, dropout, latency, or soak verification from compilation or a unit test.

## Files and trust boundaries

- Open local audio only through the native file picker. Do not reintroduce a frontend-supplied arbitrary path command.
- Maintain explicit file-size, frame-count, channel, bit-depth, and sample-rate limits in the decoder. Test malformed, unsupported, missing, and boundary files.
- Keep user-facing errors actionable and avoid exposing secrets or unrelated local paths.
- Do not add streaming-provider extraction, private APIs, credentials, telemetry, or network access without an accepted requirement and threat review.

## Architecture and licensing

- The current native engine is Rust per ADR-001 and `docs/DECISIONS/ADR-003-rust-engine-milestone1.md`. Do not switch languages or add a second native boundary without an ADR and owner review.
- Keep sample buffers out of ordinary Tauri IPC. Tauri commands carry controls and low-frequency snapshots only.
- SAYT's project license is pending. Do not add a SAYT license grant, SPDX project-license metadata, or a release claim until the owner decides. Third-party component licenses are separate and must be inventoried.
- Pin direct dependency versions exactly and update lockfiles with manifests. Review provenance, license, and advisory output before accepting new dependencies.

## Verification and reporting

Run from the repository root after code changes:

```powershell
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo check --workspace --all-targets --locked
corepack pnpm format:check
corepack pnpm lint
corepack pnpm typecheck
corepack pnpm test
corepack pnpm build
```

Run dependency audits when dependencies or lockfiles change. Record exact commands and results. Separate written code, executed checks, and verified behavior. State when a check needs hardware or an unavailable OS. Do not claim CI passed until the remote matrix is green.

Keep changes within the active milestone. Update the requirement, implementation status, runbook, and tests when behavior changes. Prefer a small regression test that can fail over a claim based on inspection alone.
