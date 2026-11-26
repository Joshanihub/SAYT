# SAYT verification evidence

**Status:** ACTIVE; M1 acceptance incomplete<br>
**Owner:** SAYT project owner<br>
**Date:** 2026-11-26<br>
**Version:** 0.3.8

This file records reproduced evidence separately from historical agent reports. At the start of the current batch, branch head `25c37aa414a90ad1b45d8bce39e9f8fbdaa36250` on `feat/stitch-m1-dj-workspace` was based on `main` at `1d0323799d0a833a40b20cd58b1c79e43a11044b`. PR #1 was open and unmerged; PR #2 was open/draft/unmerged. Baseline PR run [45740252743](https://github.com/Joshanihub/SAYT/actions/runs/45740252743) passed. This batch adds local Stitch font/icon assets and code; exact-head CI for each push is tracked in GitHub Actions.

## M1 output-switch recovery — local working-tree verification

This change is based on pushed commit `8ecbb9e3079c1b04848fa8480a515b08c4626c85`; the recovery implementation and status updates below were locally verified before commit. Exact-head CI for this change is pending. The previous pushed head's Windows, macOS, and Ubuntu push/PR runs [45804224796](https://github.com/Joshanihub/SAYT/actions/runs/45804224796) and [45804236667](https://github.com/Joshanihub/SAYT/actions/runs/45804236667) passed.

| Check                 | Command / environment                                                                                | Result                                                                                                                              |
| --------------------- | ---------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------- |
| Rust format           | `cargo fmt --all --check`                                                                            | PASS, exit 0                                                                                                                        |
| Rust lint             | `cargo clippy --workspace --all-targets -- -D warnings`                                              | PASS, exit 0                                                                                                                        |
| Rust tests            | `cargo test --workspace`                                                                             | PASS, exit 0; 19 engine tests and 1 desktop audio-service actor test passed                                                         |
| Rust all-target check | `cargo check --workspace --all-targets --locked`                                                     | PASS, exit 0                                                                                                                        |
| Frontend checks       | `corepack pnpm format:check`, `lint`, `typecheck`, `test`, `build`; Node.js 24.22.3 and pnpm 12.23.0 | PASS, exit 0; 21 UI tests passed and the production build completed                                                                 |
| Desktop dev launch    | `corepack pnpm desktop dev`                                                                          | Tauri window launched with the recovery change. Cargo emitted the existing non-fatal `sayt_desktop.pdb` filename-collision warning. |

The engine tests exercise the transition policy for opening a replacement after a pause error, restoring the previous stream after a failed replacement, exposing a failed resume, and initial selection with no previous stream. They do not exercise CPAL, a real device switch, audibility, device hot-plug, or recovery under a physical device failure. No native picker or playback controls were operated in this verification pass; owner-operated manual acceptance remains open.

## Reproduced locally on Windows

| Check                 | Command / environment                                                                                                                                                 | Result                                                                                                                                                    |
| --------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Rust format           | `cargo fmt --all --check`; rustc/cargo 1.100.0                                                                                                                        | PASS, exit 0                                                                                                                                              |
| Rust all-target check | `cargo check --workspace --all-targets --locked`                                                                                                                      | PASS, exit 0                                                                                                                                              |
| Rust lint             | `cargo clippy --workspace --all-targets -- -D warnings`                                                                                                               | PASS, exit 0                                                                                                                                              |
| Rust tests            | `cargo test --workspace`                                                                                                                                              | PASS, exit 0; 15 engine tests and 1 desktop audio-service actor test passed; remaining crates report zero tests                                           |
| Frontend runtime pin  | Node.js 24.22.3 extracted to temp from the official release archive; archive SHA-256 `e612af99f626ab415ea225af6bce5c971feb448cb5cae250bb1c3fa50c5f7d9f`; pnpm 12.23.0 | Verified exact `.node-version` and package-manager pins. Node release metadata: [official v24.22.3 release](https://nodejs.org/en/blog/release/v24.22.3). |
| Frontend checks       | `corepack pnpm format:check`, `lint`, `typecheck`, `test`, `build`                                                                                                    | PASS, exit 0; 21 UI tests passed; Vite production build completed with local WOFF2 assets and license notices in `dist/licenses`                          |
| Python toolkit tests  | Python 3.14.5; `py -3 -m unittest discover -s tools/audio-validation/tests -v`                                                                                        | PASS, exit 0; 52 tests, 50 passed, 2 symlink tests skipped because this Windows host denies symlink creation                                              |
| Fixture generation    | `py -3 tools/audio-validation/generate_fixtures.py --output-dir "%TEMP%\SAYT-Audio-Fixtures"`                                                                         | PASS, exit 0; seven deterministic synthetic WAV fixtures and manifest created outside Git                                                                 |
| Fixture verification  | `py -3 tools/audio-validation/verify_fixtures.py --input-dir "%TEMP%\SAYT-Audio-Fixtures"`                                                                            | PASS; 77 checks passed, 0 failed; `ALL CHECKS PASSED`                                                                                                     |

Manifest SHA-256: `1FA8F84E47C17F6FC017F99D5023AC8CE4C683B400BE788BE98C5E6E586A4701`. Fixtures are synthetic tones, not music and not physical device measurements. They remain in `%TEMP%\SAYT-Audio-Fixtures` and are excluded from Git.

The first local toolkit run failed because CP1252 could not encode the contributed checkmark output. The generator and verifier output was changed to ASCII; the rerun above passed. A first frontend check used Node 24.16.0, which did not match `.node-version`; it was superseded by the exact Node 24.22.3 run recorded above.

## Latest exact-head CI before this batch

The previous documentation head `8e38a432f529f0b118bc3f804d7867e5a3f5dda7` passed push run [45736072298](https://github.com/Joshanihub/SAYT/actions/runs/45736072298) and PR run [45736076490](https://github.com/Joshanihub/SAYT/actions/runs/45736076490) on all three OSes. This batch changes source assets, UI code, tests, and workflow; CI has not yet run on it.

- Push run [45736072298](https://github.com/Joshanihub/SAYT/actions/runs/45736072298): Windows, macOS, and Ubuntu passed.
- Pull request run [45736076490](https://github.com/Joshanihub/SAYT/actions/runs/45736076490): Windows, macOS, and Ubuntu passed. Its Ubuntu job records `Ran 52 tests`, `test_symlink_escape_rejected ... ok`, and `77 checks passed, 0 checks failed` across seven fixtures.

The tested desktop source is unchanged from integration commit `853e51477e56593471c7126378d290524be391af`; intervening commits through `8e38a43` were documentation-only.

## Stitch typography and icon implementation

The supplied Stitch roles are now integrated with local assets: Space Grotesk for display/hardware labels, Inter for body/navigation, JetBrains Mono for telemetry, and Material Symbols Outlined for icons. Variable WOFF2 files retain the official subset Unicode ranges and no font request is made to the network at runtime. License texts are in `apps/desktop/public/licenses` and were copied into the Vite production output under `dist/licenses`.

The asset manifest [`tools/font-assets/asset-manifest.json`](../tools/font-assets/asset-manifest.json) records the exact 16 text-font subsets and Material Symbols source/output URLs, sizes, and SHA-256 values. [`build_material_symbols.py`](../tools/font-assets/build_material_symbols.py) reproduced 66 verified ligatures from the pinned v387 source into a 51,720-byte WOFF2 subset with SHA-256 `f31f8e017925c3a6a882c1a67f0d41d6531f5a3193d1a3e1f5dc0cbbf99d4459`; optional tool pins are `fonttools==4.68.0` and `Brotli==1.2.1`. `py -3 tools/font-assets/verify_assets.py` passed: 17 assets, 4 notices, 5 M1 icon names, local-only CSS paths, and nonrecursive font roles. `py -3 -m unittest discover -s tools/font-assets/tests -v` passed 3 role-validation cases. The owner-authorized design roles and official OFL/Apache notice sources are recorded in [`DEPENDENCIES_AND_LICENSES.md`](DEPENDENCIES_AND_LICENSES.md).

The current Windows Tauri window was observed in an empty DJ workspace at a 1280×800 client viewport (the outer window screenshot was 1282×832). The corrected typography roles and Material Symbols rendered. This is a runtime render observation, not a matched Stitch comparison. The 1440×900 and narrow captures remain NOT RUN; matching the 1280×800 render against its Stitch reference is also NOT RUN. Library and Settings screens were not inspected in this runtime pass. Actual font script behavior for arbitrary user metadata and all physical audio acceptance checks remain unverified.

## Local build-tool audit

The font subsetter runs only as an optional asset-maintenance tool and is not a SAYT runtime dependency. Its exact dependencies are in `tools/font-assets/requirements.txt`. In an isolated Python 3.14.5 environment under `%TEMP%`, `python -m pip_audit -r tools/font-assets/requirements.txt --desc` using `pip-audit 2.10.1` reported `No known vulnerabilities found` (PASS, exit 0). The auditor and its dependencies are not project dependencies. The font-role regression suite (`python -m unittest discover -s tools/font-assets/tests -v`) passed 3 tests covering valid roles, a recursive fallback, and a missing font family.

## GitHub Actions for exact integration commit

| Event           | Run                                                                        | PR head SHA                                | Result                                                                            |
| --------------- | -------------------------------------------------------------------------- | ------------------------------------------ | --------------------------------------------------------------------------------- |
| Push            | [45721302725](https://github.com/Joshanihub/SAYT/actions/runs/45721302725) | `853e51477e56593471c7126378d290524be391af` | PASS — Windows, macOS, Ubuntu                                                     |
| Pull request #1 | [45721308789](https://github.com/Joshanihub/SAYT/actions/runs/45721308789) | `853e51477e56593471c7126378d290524be391af` | PASS — Windows, macOS, Ubuntu; Ubuntu audio toolkit suite and full CI jobs passed |

The passing PR run's Ubuntu log records `Ran 52 tests`, `test_symlink_escape_rejected ... ok`, and fixture verification `77 checks passed, 0 checks failed`. Windows and macOS completed their full workflow jobs successfully; the Linux-only toolkit step is intentionally skipped on those platforms.

## Desktop runtime and hardware

- Historical Codex Tauri run: Windows desktop rendered the initial empty DJ workspace and connected engine snapshot at 1920×1032. Two decks were empty. This is a window/render check only.
- Current running session: `corepack pnpm desktop dev` was launched with Node.js 24.22.3/pnpm 12.23.0 while the checkout was at documentation commit `c79a448b433f4f29f50236254621d8020105a126`; the product-code tree matches integration commit `853e51477e56593471c7126378d290524be391af`. Vite became ready on `127.0.0.1:1420`, Cargo finished, and `sayt-desktop.exe` remains alive. On 2026-11-26 the process and both requested fixture paths were confirmed present. Cargo emitted a non-fatal output filename collision warning for `sayt_desktop.pdb` between the library and binary targets.
- Actual native picker, WAV load, real Tauri transport/mixer IPC, 1440×900 and narrow screenshots, matched Stitch comparison at 1280×800, and physical playback remain NOT RUN. The owner said they can run the manual check; results have not yet been received. Only the accessibility tree and rendered UI were inspected; no playback or device-control actions were automated, and the dev process remains available.
- Physical dual playback, independent audible control, EQ/gain/crossfader, device disconnect/recovery, and shutdown during playback are NOT RUN. A screenshot, UI meter, moving waveform, or engine snapshot is not hardware evidence. See [`audio-validation/PHYSICAL_DEVICE_TEST_GUIDE.md`](audio-validation/PHYSICAL_DEVICE_TEST_GUIDE.md) and its blank [`RESULTS_TEMPLATE.md`](audio-validation/RESULTS_TEMPLATE.md).

## Scope, security, dependency and rollback notes

- The Stitch archive remains byte-identical to the supplied reference; SHA-256 `E97F43E20EDE4BC3993CFB8E4177BB53C3AF6296C58AEE11E0E1DAA347A4B2F1`.
- The reviewed PR #2 workflow was not copied. The PR #1 workflow's existing setup-node and Linux ALSA corrections remain; only the Linux Python toolkit test step was added.
- Toolkit generator now rejects symbolic-link output destinations, preflights owned destinations before writing, and atomically replaces fixture, sentinel and manifest files so symlinks/hard links cannot redirect writes to an external file. Regression coverage includes symlink, directory, and hard-link output cases; Windows skipped the symlink case due host privilege policy.
- No runtime dependency, Rust/npm lockfile, database, persistence schema, user library or migration changed. The optional Python font-building dependencies are pinned in `tools/font-assets/requirements.txt` and were audited separately. Rollback for this typography/icon batch is a normal revert of its dedicated feature-branch commit; `main` was not changed. Generated WAVs are outside the checkout.
- Project licence remains undecided. Existing BlueOak package strings and Rust advisory warnings remain owner-review items. Font family license sources are now recorded; the exact font/icon set, Unicode coverage, notices, and logo/artwork provenance remain owner decisions in [`DEPENDENCIES_AND_LICENSES.md`](DEPENDENCIES_AND_LICENSES.md) and [`DECISION_REGISTER.md`](DECISION_REGISTER.md).
