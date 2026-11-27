# Third-party dependency register

**Status:** Direct inventory, pnpm transitive license output, and RustSec scan recorded; owner and legal review pending.<br>
**Owner:** SAYT project owner<br>
**Checked:** 2026-11-27<br>
**Version:** 0.1.1

Every direct dependency is pinned exactly in its manifest. The lockfiles are the resolution record. CI runs Cargo bans/license/source checks and RustSec advisories separately, plus pnpm's license inventory command. M1 remains partial until the transitive report is reviewed.

## SAYT project license: pending

The project owner has not selected a license. No SAYT license grant is included in this repository, and Cargo package metadata intentionally omits a project license. The GPL-3.0-or-later text that appeared in an early local draft is not authoritative and is not included in the published file set. Until the owner decides, assume all rights are reserved; do not treat this public source as permission to reuse or redistribute SAYT code. Third-party dependency licenses below apply to those dependencies only, not to SAYT.

## Direct dependencies

| Ecosystem | Package                |         Exact version | License           | Role                                                                                                    |
| --------- | ---------------------- | --------------------: | ----------------- | ------------------------------------------------------------------------------------------------------- |
| Rust      | `tauri`                |                2.12.7 | Apache-2.0        | Desktop application shell                                                                               |
| Rust      | `tauri-build`          |                 2.7.6 | Apache-2.0        | Tauri build-time integration                                                                            |
| Rust      | `tauri-plugin-dialog`  |                 2.4.0 | Apache-2.0 OR MIT | User-selected file dialog                                                                               |
| Rust      | `cpal`                 |                0.16.0 | Apache-2.0        | M1 OS audio output path; physical device behavior unverified; 0.18.2 compatibility review pending       |
| Rust      | `futures-channel`      |                0.3.34 | MIT OR Apache-2.0 | One-shot async bridge from native file-picker callback to Tauri command                                 |
| Rust      | `rtrb`                 |                 0.4.1 | MIT OR Apache-2.0 | Bounded single-producer/single-consumer command and retired-track queues                                |
| Rust      | `hound`                |                 3.5.1 | Apache-2.0        | Experimental PCM WAV decoder                                                                            |
| Rust      | `base64`               |                0.23.1 | MIT OR Apache-2.0 | Bounded embedded-artwork data URLs for the selected-track inspector                                     |
| Rust      | `lofty`                |                0.25.5 | MIT OR Apache-2.0 | Local tag, duration, BPM/key, and on-demand artwork metadata parsing                                    |
| Rust      | `rusqlite`             |                0.43.0 | MIT               | App-local transactional music catalog and online backup                                                 |
| Rust      | `serde`                |               1.0.228 | MIT OR Apache-2.0 | Tauri and engine snapshot serialization                                                                 |
| Rust      | `sha2`                 |                0.11.2 | MIT OR Apache-2.0 | Full-file content identity for duplicate/move recovery                                                  |
| Rust      | `walkdir`              |                 2.5.1 | Unlicense/MIT     | Recursive selected-root scan with symlink following disabled                                            |
| Rust      | `tracing`              |                0.1.44 | MIT               | Structured startup logging                                                                              |
| Rust      | `tracing-subscriber`   |                0.3.23 | MIT               | JSON logs and environment filter                                                                        |
| npm       | `react`                |                19.3.1 | MIT               | UI runtime                                                                                              |
| npm       | `react-dom`            |                19.3.1 | MIT               | DOM renderer and server-render smoke test                                                               |
| npm       | `@tauri-apps/api`      |                 2.9.1 | Apache-2.0 OR MIT | Tauri command invocation from the desktop UI; official ecosystem release index lists 2.12.5             |
| npm       | `@tauri-apps/cli`      |                2.12.6 | Apache-2.0        | Desktop development/build CLI                                                                           |
| npm       | `@types/react`         |                19.3.0 | MIT               | React types; newer patches were inside the configured 24-hour release hold                              |
| npm       | `@types/react-dom`     |                19.3.0 | MIT               | React DOM types                                                                                         |
| npm       | `typescript`           |                 6.0.3 | Apache-2.0        | Static type checking; TypeScript 7 is outside the declared range of the pinned TypeScript ESLint parser |
| npm       | `vite`                 |                 8.3.8 | MIT               | Frontend dev server and bundler                                                                         |
| npm       | `@vitejs/plugin-react` |                 6.3.0 | MIT               | React integration for Vite                                                                              |
| npm       | `eslint`               |               10.15.0 | MIT               | Linting                                                                                                 |
| npm       | `typescript-eslint`    |                8.74.0 | MIT               | TypeScript lint configuration                                                                           |
| npm       | `prettier`             |                3.10.1 | MIT               | Formatting                                                                                              |
| npm       | `vitest`               |                 5.1.1 | MIT               | UI shell test runner; 5.1.2 was inside the configured 24-hour release hold                              |
| npm       | `pnpm`                 |               12.23.0 | MIT               | Workspace package manager                                                                               |
| Python    | `fonttools`            |                4.68.0 | MIT               | Optional, pinned Material Symbols subset build tool                                                     |
| Python    | `Brotli`               |                 1.2.1 | MIT               | Optional WOFF2 compression dependency for the subset build                                              |
| CI        | `actions/checkout`     | v7.0.1, commit-pinned | MIT               | Checkout; immutable SHA in workflow                                                                     |
| CI        | `actions/setup-node`   | v7.1.0, commit-pinned | MIT               | Pinned Node runtime; immutable SHA in workflow                                                          |

On 2026-11-26, the optional font build requirements were scanned with `pip-audit 2.10.1` in an isolated Python 3.14.5 environment. It reported no known vulnerabilities. The auditor and its transitive dependencies were installed under `%TEMP%`; they are not project dependencies. This is an advisory scan of the two pinned build-tool requirements, not a security clearance.

Toolchain pins: Node.js 24.22.3 (Active LTS as checked on 2026-11-25), Rust 1.100.0, and pnpm 12.23.0. These are exact build-time versions; the Rust crate `rust-version` remains the minimum 1.90.0 needed by the selected Tauri line.

Versions and license fields were retrieved from crates.io and the npm registry on 2026-11-25. Tauri package versions were cross-checked against the official Tauri release index. Some dependencies in the preserved audio prototype are older than the registry's latest stable version: `cpal` 0.18.2, `tauri-plugin-dialog` 2.8.4, and `serde` 1.0.229 were available when `Cargo.lock` was refreshed. Their current pins remain unchanged pending compatibility review.

The pnpm resolver license inventory is recorded in [`npm-licenses.txt`](npm-licenses.txt). It lists two development dependencies under Blue Oak Model License 1.0.0 (`@isaacs/nonrecursive-glob-match` and `minimatch`). The Blue Oak Council describes this as a permissive license with a license-notice condition and an express patent grant; it is outside the charter's named dependency license set, so the project owner must decide whether to accept it before distribution. It is a development-only transitive dependency in this lockfile.

The Rust lockfile contains 532 crate dependencies. Local `cargo audit` on 2026-11-27 refreshed the advisory database after the catalog dependency update and reported zero known vulnerability advisories, six unmaintained-package warnings, and one unsoundness warning for transitive `glib 0.18.5` (`RUSTSEC-2024-0429`). The unmaintained notices are `chrono 0.4.45` (`RUSTSEC-2026-0543`), `paste 1.0.15` (`RUSTSEC-2024-0436`), `proc-macro-error 1.0.4` (`RUSTSEC-2024-0370`), `zbus_names 4.3.5` (`RUSTSEC-2026-0371`), `zvariant 5.15.1` (`RUSTSEC-2026-0370`), and `zvariant_derive 5.15.1` (`RUSTSEC-2026-0372`). The GTK-related `glib` finding matters to Linux builds. Reachability of affected APIs was not established by this audit. Do not treat this scan as a security clearance.

## Policy and remaining checks

- `deny.toml` limits Rust dependencies to the configured license allowlist, including Unicode-3.0 and Apache-2.0 WITH LLVM-exception observed in the current graph. This scanner configuration is not a legal compatibility determination.
- `cargo deny check --hide-inclusion-graph bans licenses sources` passed locally on 2026-11-27. It reports duplicate-version warnings in the transitive graph; this selected check does not include advisory findings.
- `cargo audit` found zero vulnerability advisories and the unmaintained/unsound notices recorded above; the RustSec database and findings must be refreshed before release.
- `pnpm audit` checks the complete resolved npm graph against registry advisories.
- `pnpm licenses list --recursive` must be run after lockfile generation; its complete output must be reviewed for unknown, proprietary, copyleft, or non-allowlisted terms.
- Current pnpm inventory: all listed packages have an identified license; two development packages use `BlueOak-1.0.0`, recorded above for owner review.
- An SBOM and license review are required before distributing binaries; a clean license scanner is not legal advice or a compatibility determination.

## Retrieved sources

- [Tauri ecosystem release index](https://v2.tauri.app/release/)
- [CPAL on crates.io](https://crates.io/crates/cpal)
- [rtrb 0.4.1 on crates.io](https://crates.io/crates/rtrb/0.4.1)
- [rtrb documentation](https://docs.rs/rtrb/0.4.1/rtrb/)
- [Tauri dialog plugin on crates.io](https://crates.io/crates/tauri-plugin-dialog)
- [Serde on crates.io](https://crates.io/crates/serde)
- [React on npm](https://www.npmjs.com/package/react)
- [Vite release policy](https://vite.dev/releases)
- [pnpm 12.23.0 release notes](https://pnpm.io/blog/releases/12.23.0)
- [Blue Oak Model License 1.0.0](https://blueoakcouncil.org/license/1.0.0)
- [Node.js release status](https://nodejs.org/en/about/previous-releases)
- [Rust 1.100.0 announcement](https://blog.rust-lang.org/2026/11/12/Rust-1.100.0/)
- [SPDX license list](https://github.com/spdx/license-list-data)
