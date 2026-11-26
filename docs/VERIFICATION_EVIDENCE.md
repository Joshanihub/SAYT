# SAYT verification evidence

**Status:** ACTIVE; M1 acceptance incomplete<br>
**Owner:** SAYT project owner<br>
**Date:** 2026-11-26<br>
**Version:** 0.3.3

This file records reproduced evidence separately from historical agent reports. The audio-validation source integration is commit `853e51477e56593471c7126378d290524be391af`; the latest branch head is `6de029fbaa936d3e4c06c39a781a3143c77bec45` on `feat/stitch-m1-dj-workspace`, based on `1d0323799d0a833a40b20cd58b1c79e43a11044b`. PR #1 is open and unmerged; PR #2 remains open/draft. Exact-head push run [45733825348](https://github.com/Joshanihub/SAYT/actions/runs/45733825348) and PR run [45733831972](https://github.com/Joshanihub/SAYT/actions/runs/45733831972) both passed Windows, macOS, and Ubuntu. The PR Ubuntu log records 52 Python tests passing and 77/77 fixture checks. Some negative verifier tests intentionally print failed-verification output and assert that invalid manifests or missing files are rejected.

## Reproduced locally on Windows

| Check                 | Command / environment                                                                                                                                                 | Result                                                                                                                                                    |
| --------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Rust format           | `cargo fmt --all --check`; rustc/cargo 1.100.0                                                                                                                        | PASS, exit 0                                                                                                                                              |
| Rust all-target check | `cargo check --workspace --all-targets --locked`                                                                                                                      | PASS, exit 0                                                                                                                                              |
| Rust lint             | `cargo clippy --workspace --all-targets --locked -- -D warnings`                                                                                                      | PASS, exit 0                                                                                                                                              |
| Rust tests            | `cargo test --workspace --locked`                                                                                                                                     | PASS, exit 0; 15 engine tests and 1 desktop audio-service actor test passed; remaining crates report zero tests                                           |
| Frontend runtime pin  | Node.js 24.22.3 extracted to temp from the official release archive; archive SHA-256 `e612af99f626ab415ea225af6bce5c971feb448cb5cae250bb1c3fa50c5f7d9f`; pnpm 12.23.0 | Verified exact `.node-version` and package-manager pins. Node release metadata: [official v24.22.3 release](https://nodejs.org/en/blog/release/v24.22.3). |
| Frontend checks       | `corepack pnpm install --frozen-lockfile`, `format:check`, `lint`, `typecheck`, `test`, `build`                                                                       | PASS, exit 0; 20 tests passed; Vite production build completed                                                                                            |
| Python toolkit tests  | Python 3.14.5; `py -3 -m unittest discover -s tools/audio-validation/tests -v`                                                                                        | PASS, exit 0; 52 tests, 50 passed, 2 symlink tests skipped because this Windows host denies symlink creation                                              |
| Fixture generation    | `py -3 tools/audio-validation/generate_fixtures.py --output-dir "%TEMP%\SAYT-Audio-Fixtures"`                                                                         | PASS, exit 0; seven deterministic synthetic WAV fixtures and manifest created outside Git                                                                 |
| Fixture verification  | `py -3 tools/audio-validation/verify_fixtures.py --input-dir "%TEMP%\SAYT-Audio-Fixtures"`                                                                            | PASS; 77 checks passed, 0 failed; `ALL CHECKS PASSED`                                                                                                     |

Manifest SHA-256: `1FA8F84E47C17F6FC017F99D5023AC8CE4C683B400BE788BE98C5E6E586A4701`. Fixtures are synthetic tones, not music and not physical device measurements. They remain in `%TEMP%\SAYT-Audio-Fixtures` and are excluded from Git.

The first local toolkit run failed because CP1252 could not encode the contributed checkmark output. The generator and verifier output was changed to ASCII; the rerun above passed. A first frontend check used Node 24.16.0, which did not match `.node-version`; it was superseded by the exact Node 24.22.3 run recorded above.

## Latest exact-head CI — documentation/status commit

Branch head `6de029fbaa936d3e4c06c39a781a3143c77bec45` passed both workflow events on all three operating systems:

- Push run [45733825348](https://github.com/Joshanihub/SAYT/actions/runs/45733825348): Windows, macOS, and Ubuntu passed.
- Pull request run [45733831972](https://github.com/Joshanihub/SAYT/actions/runs/45733831972): Windows, macOS, and Ubuntu passed. Its Ubuntu job records `Ran 52 tests`, `test_symlink_escape_rejected ... ok`, and `77 checks passed, 0 checks failed` across seven fixtures.

The tested desktop source is unchanged from integration commit `853e51477e56593471c7126378d290524be391af`; intervening commits were documentation-only.

## GitHub Actions for exact integration commit

| Event           | Run                                                                        | PR head SHA                                | Result                                                                            |
| --------------- | -------------------------------------------------------------------------- | ------------------------------------------ | --------------------------------------------------------------------------------- |
| Push            | [45721302725](https://github.com/Joshanihub/SAYT/actions/runs/45721302725) | `853e51477e56593471c7126378d290524be391af` | PASS — Windows, macOS, Ubuntu                                                     |
| Pull request #1 | [45721308789](https://github.com/Joshanihub/SAYT/actions/runs/45721308789) | `853e51477e56593471c7126378d290524be391af` | PASS — Windows, macOS, Ubuntu; Ubuntu audio toolkit suite and full CI jobs passed |

The passing PR run's Ubuntu log records `Ran 52 tests`, `test_symlink_escape_rejected ... ok`, and fixture verification `77 checks passed, 0 checks failed`. Windows and macOS completed their full workflow jobs successfully; the Linux-only toolkit step is intentionally skipped on those platforms.

## Desktop runtime and hardware

- Historical Codex Tauri run: Windows desktop rendered the initial empty DJ workspace and connected engine snapshot at 1920×1032. Two decks were empty. This is a window/render check only.
- Current running session: `corepack pnpm desktop dev` was launched with Node.js 24.22.3/pnpm 12.23.0 while the checkout was at documentation commit `c79a448b433f4f29f50236254621d8020105a126`; the product-code tree matches integration commit `853e51477e56593471c7126378d290524be391af`. Vite became ready on `127.0.0.1:1420`, Cargo finished, and `sayt-desktop.exe` remains alive. Cargo emitted a non-fatal output filename collision warning for `sayt_desktop.pdb` between the library and binary targets.
- Actual native picker, WAV load, real Tauri transport/mixer IPC, 1440×900/1280×800/narrow responsive screenshots, and physical playback remain NOT RUN. No UI automation was retried after the prior helper detected concurrent input; owner-operated manual results have not yet been received, and the dev process remains available.
- Physical dual playback, independent audible control, EQ/gain/crossfader, device disconnect/recovery, and shutdown during playback are NOT RUN. A screenshot, UI meter, moving waveform, or engine snapshot is not hardware evidence. See [`audio-validation/PHYSICAL_DEVICE_TEST_GUIDE.md`](audio-validation/PHYSICAL_DEVICE_TEST_GUIDE.md) and its blank [`RESULTS_TEMPLATE.md`](audio-validation/RESULTS_TEMPLATE.md).

## Scope, security, dependency and rollback notes

- The Stitch archive remains byte-identical to the supplied reference; SHA-256 `E97F43E20EDE4BC3993CFB8E4177BB53C3AF6296C58AEE11E0E1DAA347A4B2F1`.
- The reviewed PR #2 workflow was not copied. The PR #1 workflow's existing setup-node and Linux ALSA corrections remain; only the Linux Python toolkit test step was added.
- Toolkit generator now rejects symbolic-link output destinations, preflights owned destinations before writing, and atomically replaces fixture, sentinel and manifest files so symlinks/hard links cannot redirect writes to an external file. Regression coverage includes symlink, directory, and hard-link output cases; Windows skipped the symlink case due host privilege policy.
- No dependency, lockfile, database, persistence schema, user library or migration changed. Rollback is a normal revert of `853e51477e56593471c7126378d290524be391af` on the feature branch; `main` was not changed. Generated WAVs are outside the checkout.
- Project licence remains undecided. Existing BlueOak package strings, Rust advisory warnings, and font/icon rights remain owner-review items in [`DEPENDENCIES_AND_LICENSES.md`](DEPENDENCIES_AND_LICENSES.md) and [`DECISION_REGISTER.md`](DECISION_REGISTER.md).
