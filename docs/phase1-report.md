# Archived M0 baseline report

**Owner:** SAYT project owner<br>
**Date:** 2026-11-25<br>
**Version:** 0.1.0<br>
**Status:** Historical snapshot; M0 was partial at the time of this audit and this report is superseded by [`M1_REPORT.md`](M1_REPORT.md).

All findings and next steps below record the repository before the M1 audio implementation work. They are retained as audit history, not a description of current code.

## M0 — Repository and CI scaffold

**Status:** partial

**What changed:**

- Added the Cargo and pnpm workspaces, pinned toolchains, the initial desktop shell, and the requested top-level project and governance documents.
- Added a cross-platform CI workflow for formatting, lint, typecheck, UI test, build, Rust checks, license policy, and vulnerability audit.
- Recorded the initial architecture choices as proposed ADRs and documented the scope gates.
- Preserved and audited the experimental WAV playback implementation that appeared during this M0 work. The current Rust workspace compiles, but audio behavior remains unverified and outside M0 acceptance.

**Evidence:**

- Local baseline audit: root contained only `.git`; `git status --short --branch` showed an unborn `main` branch and no changes before this scaffold.
- `git ls-remote --heads origin`: no remote branch heads were returned.
- Current versions were checked against official package and release registries on 2026-11-25; see [`third-party.md`](licensing/third-party.md).
- The 24-hour package maturity hold rejected newly published `@types/react` and Vitest patches; older patches were selected instead.
- `pnpm lint`, `pnpm typecheck`, `pnpm test`, and `pnpm build` passed locally. The UI test result is one passing shell-render test. The machine has Node 24.16.0, below the project pin 24.22.3.
- `pnpm audit` passed with `No known vulnerabilities found`.
- `pnpm licenses list --recursive` completed; the output is stored in [`npm-licenses.txt`](licensing/npm-licenses.txt). It found two development dependencies under Blue Oak Model License 1.0.0, which needs an owner decision.
- `pnpm format:check`, `pnpm lint`, `pnpm typecheck`, `pnpm test`, and `pnpm build` passed locally. The single UI test renders the shell; local Node 24.16.0 is below the 24.22.3 project pin.
- `cargo metadata --no-deps` and `cargo check --workspace --all-targets --locked` passed with the host stable compiler, which reports Rust 1.100.0. The pinned Rustup toolchain directory is incomplete. The lockfile contains 518 packages.
- `cargo fmt --all --check` failed: `apps/desktop/src-tauri/src/lib.rs` and `crates/sayt-engine/src/lib.rs` need formatting. The concurrent audio files were left untouched.
- `cargo deny check` passes bans, licenses, and sources, but exits non-zero on the transitive unmaintained `proc-macro-error` advisory. It also reports duplicate-version warnings.
- `cargo audit` scanned the 518-package lockfile against RustSec database commit `b9b8a3c` (2026-11-25): zero vulnerability advisories, five unmaintained-crate notices, and one unsoundness advisory for transitive `glib 0.18.5` (`RUSTSEC-2024-0429`). This does not establish whether the affected iterator methods are reachable in the application.
- Rust unit tests, audio-device playback, latency, dropout, and soak checks were not run. No audio device was opened.
- CI has not run on Windows, macOS, or Linux. Legal review and the Blue Oak development dependency decision remain open.

**ADRs:** [`001-engine-core-rust.md`](architecture/adr/001-engine-core-rust.md), [`002-desktop-shell.md`](architecture/adr/002-desktop-shell.md), [`012-monorepo.md`](architecture/adr/012-monorepo.md)

**[ASSUMED] log:**

- Primary dev OS: Windows 11.
- Team: solo founder with coding agents.
- Project license: pending owner decision; no SAYT license grant is included in the repository.
- Desktop identifier `org.sayt.desktop` is a development identifier and requires trademark/domain review before distribution.
- Low-end and mid-range reference machines remain the defaults in Appendix B of the founding charter; no hardware was available for this scaffold.

**REVIEW (pre-mortem):**

- Silent failure: found; the experimental audio callback uses `Mutex::try_lock` and returns a silent output buffer when state is contended.
- Trust violation: no provider, network, or account surface exists. The local `load_track` command accepts a path supplied by the frontend, with no picker-only restriction.
- Non-idempotency: checked; no state mutation or data import exists.
- Unbounded growth: found; the WAV decoder reads the complete file into unbounded sample vectors before playback.
- Missing UI states: checked for the shell; no interactive feature is exposed.
- Environment assumption: found local Node 24.16.0 differs from the 24.22.3 pin; pnpm 12.23.0 was used. The Rustup 1.100.0 path is incomplete, so local Cargo metadata used the stable compiler that reports the same version.
- Single dependency: found Tauri/WebView is a major UI-shell dependency; S2 is the explicit performance gate.
- Observability: structured startup logging exists. The audio callback reports CPAL errors, but callback load, xrun count, and request correlation are not implemented.
- Audio callback: found `Mutex::try_lock`; the callback emits a silent buffer if the lock is busy. This violates the charter's no-lock callback rule and remains unfixed pending scope direction.
- File loading: Tauri exposes a path-taking `load_track` command. The decoder reads PCM WAV data into unbounded vectors and decodes stereo data twice; no adversarial file test exists.
- EQ behavior: the low, mid, and high controls are collapsed into one scalar applied equally to every sample; they do not provide frequency-selective bands.
- Audio state: commands and the callback share mutex-protected state. The callback avoids waiting but can emit silence under contention; no xrun metric is recorded.
- Dependency risk: the Linux GTK dependency graph includes `glib 0.18.5`, which cargo-audit reports under an unsoundness advisory. Reachability in SAYT is not established.
- Build contract: current workspace compilation passes; the previous compile errors were absent in the current tree. Formatting still fails, and no behavior test was run.

**NEXT:**

1. Review the Blue Oak development dependency against the SAYT license policy and record the owner decision in `docs/licensing/third-party.md`.
2. Resolve the `glib 0.18.5` unsoundness exposure and the five unmaintained-crate notices, then rerun `cargo audit` and `cargo deny check`.
3. Add explicit WAV file-size/frame limits and remove the stereo decoder's redundant full pass before accepting the audio implementation.
4. Replace the callback lock with a real-time-safe command/state path and add deterministic tests before any device playback or soak test.
5. Format Rust sources, then run the Windows/macOS/Linux CI matrix; complete license review before distribution.

**WATCH:**

- Revisit Tauri after S2 if p95 waveform frame time misses the charter thresholds on Windows or macOS.
- Revisit the Rust MSRV when Tauri raises its supported compiler floor or when CI identifies a required dependency floor.

## Acceptance criteria

| ID    | Criterion                                                      | Status    | Evidence                                                                                   |
| ----- | -------------------------------------------------------------- | --------- | ------------------------------------------------------------------------------------------ |
| AC-1  | Two local audio files play simultaneously                      | Not in M0 | M2–M4                                                                                      |
| AC-2  | 60-minute soak meets xrun and callback targets                 | Not in M0 | M9                                                                                         |
| AC-3  | Crossfader power law and continuity                            | Not in M0 | M4/M6                                                                                      |
| AC-4  | EQ response and kill depth                                     | Not in M0 | M4/M6                                                                                      |
| AC-5  | Offline renders are bit-identical                              | Not in M0 | M6                                                                                         |
| AC-6  | Seek latency and stale-audio discard                           | Not in M0 | M3/M6                                                                                      |
| AC-7  | Decoder accuracy                                               | Not in M0 | M1 S4                                                                                      |
| AC-8  | Real-time callback has no allocation, lock, or reachable panic | Not in M0 | M2/M6                                                                                      |
| AC-9  | UI reload does not interrupt audio                             | Not in M0 | M5                                                                                         |
| AC-10 | Device unplug/replug recovery                                  | Not in M0 | M5                                                                                         |
| AC-11 | Waveform frame-time gate                                       | Not in M0 | M1 S2                                                                                      |
| AC-12 | 50k-track library performance                                  | Not in M0 | M1 S6                                                                                      |
| AC-13 | Three-OS CI and dependency audit                               | Partial   | Local checks were run on Windows; Rust advisory/license findings remain; CI matrix not run |
| AC-14 | Memory growth during soak                                      | Not in M0 | M9                                                                                         |

**TENSIONS:** The charter asks for future crate boundaries in M0 and prohibits empty stubs. The crates are documented but future APIs remain deferred. The workspace also contains an experimental audio path beyond M0; it is preserved and audited, with its hardware behavior still unverified. The project license remains pending and no license grant is included in the public file set.

**3 a.m. test:** Partial. The CI runbook describes how to reproduce checks. The experimental runtime audio path has no xrun metric or recovery runbook.

**Successor test:** Partial. The ADRs record why the technology and repository boundaries were selected and when to revisit them.

**Calibration test:** The workspace compile and frontend checks are verified locally. No CI, playback, device, or performance claim is marked verified.
