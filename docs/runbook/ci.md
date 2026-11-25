# CI and local checks

**Status:** Proposed<br>
**Owner:** SAYT project owner<br>
**Date:** 2026-11-25

## Local sequence

1. Install the exact Rust toolchain in `rust-toolchain.toml` and Node version in `.node-version`.
2. Install pnpm `12.23.0`, then run `pnpm install --frozen-lockfile`.
3. On Linux, install Tauri's documented GTK/WebKit prerequisites.
4. Run the commands listed under `Checks` in the root README. The advisory scan reports known unmaintained/unsound transitive components; review `docs/DEPENDENCIES_AND_LICENSES.md` rather than suppressing them.

## Failure triage

- Rust formatting: run `cargo fmt --all`; inspect the resulting diff before accepting it.
- Clippy or Rust tests: reproduce with the exact workspace command from the workflow and retain the first failing crate and diagnostic.
- Linux Tauri build: confirm the OS WebKit/GTK packages are installed before changing Rust code.
- Frontend checks: confirm Node and pnpm match the pinned files; then run the failing script from `apps/desktop` to retain package-local context.
- Dependency policy: inspect the exact crate/package and lockfile version before adding a license exception. Do not widen the global allow-list to silence one dependency. Run `cargo deny check --hide-inclusion-graph bans licenses sources`; run `cargo audit` separately for RustSec advisories.

CI is not yet verified. The runbook becomes operational only after a green three-OS run.
