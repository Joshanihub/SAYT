# Dependencies and licenses

**Status:** PROPOSED inventory; owner and legal review pending<br>
**Owner:** SAYT project owner<br>
**Date:** 2026-11-26<br>
**Version:** 0.3.0

## SAYT license decision

The owner has not selected SAYT's project license. No project license grant or project SPDX metadata is included. The public repository must not imply that SAYT code is available for reuse or redistribution. A GPL-3.0-or-later text from an earlier local draft was excluded from the public file set; it was a proposal, not an owner decision. The project may be publicly visible on GitHub while all rights remain reserved. Update this section, Cargo package metadata, and the root license file only after the owner chooses.

## Third-party inventory

The direct Rust and npm dependencies, exact pins, recorded license strings, toolchain pins, and sources are listed in [`licensing/third-party.md`](licensing/third-party.md). The pnpm resolved-license output is [`licensing/npm-licenses.txt`](licensing/npm-licenses.txt). These inventories concern third-party components and do not license SAYT. Cargo deny skips workspace crates marked `publish = false` while the project-license decision is pending; it still checks the third-party dependency graph.

Current owner-review items:

- Exact-head push run [45725687824](https://github.com/Joshanihub/SAYT/actions/runs/45725687824) and PR run [45725694284](https://github.com/Joshanihub/SAYT/actions/runs/45725694284) passed Windows, macOS, and Ubuntu for head `c79a448b433f4f29f50236254621d8020105a126`. On Linux, the PR job's Python suite passed 52 tests; the normal fixture manifest passed 77/77 checks. Negative verifier tests also print expected failure reports while asserting those invalid inputs are rejected.
- The same exact-head Linux job ran `pnpm audit` with no known vulnerabilities. `pnpm licenses list --recursive` reports two development-only packages with `BlueOak-1.0.0`: `@isaacs/nonrecursive-glob-match@1.1.0` and `minimatch@11.0.9`. `pnpm why` places them under the ESLint development toolchain. The [Blue Oak Model License 1.0.0](https://blueoakcouncil.org/license/1.0.0) requires recipients to receive its license text or a link and includes a contributor patent license and no-liability terms. Applicability and distribution handling remain for owner/legal review; no allowlist change was made.
- `cargo audit` refreshed the RustSec database in exact-head CI on 2026-11-26 and reported no known vulnerabilities, plus five unmaintained-package warnings and the unsoundness advisory [RUSTSEC-2024-0429](https://rustsec.org/advisories/RUSTSEC-2024-0429.html) for `glib 0.18.5`. The affected `VariantStrIter` iterator methods have undefined-behaviour risk and RustSec lists `glib >=0.20.0` as patched. `glib 0.18.5` is in the Linux Tauri/GTK graph; a source scan found no `VariantStrIter` or `array_iter_str` use in SAYT or cached dependency sources outside the `glib` crate itself. This lowers observed direct reachability but is not a binary call-graph audit or a security clearance.
- `cargo tree --locked --offline --workspace --all-features --edges all --target all -i <crate>` showed `proc-macro-error@1.0.4` in the Linux GTK macro build graph, but no resolved workspace graph path for `chrono@0.4.45`, `zbus_names@4.3.5`, `zvariant@5.15.1`, or `zvariant_derive@5.15.1`. Those four remain in `Cargo.lock`, which `cargo audit` scans; they were not shown as active dependencies in the all-workspace/all-target resolution. Their RustSec records are [chrono](https://rustsec.org/advisories/RUSTSEC-2026-0543.html), [proc-macro-error](https://rustsec.org/advisories/RUSTSEC-2024-0370.html), [zbus_names](https://rustsec.org/advisories/RUSTSEC-2026-0371.html), [zvariant](https://rustsec.org/advisories/RUSTSEC-2026-0370.html), and [zvariant_derive](https://rustsec.org/advisories/RUSTSEC-2026-0372.html). No transitive upgrade was made; re-evaluate when the Linux Tauri dependency stack changes.
- `cargo deny check --hide-inclusion-graph bans licenses sources` passed on 2026-11-26. It emitted duplicate-lockfile-entry warnings; no dependency-policy allowlist was widened.
- A packaged-artifact SBOM, review of the active Linux GTK dependency path, and legal review are required before distributing binaries. The SAYT project license and third-party/font/icon approvals remain owner decisions.

Do not silently widen license allowlists or describe scanner output as legal approval.
