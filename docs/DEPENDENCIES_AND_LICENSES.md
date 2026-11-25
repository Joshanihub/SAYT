# Dependencies and licenses

**Status:** PROPOSED inventory; owner and legal review pending<br>
**Owner:** SAYT project owner<br>
**Date:** 2026-11-25<br>
**Version:** 0.1.0

## SAYT license decision

The owner has not selected SAYT's project license. No project license grant or project SPDX metadata is included. The public repository must not imply that SAYT code is available for reuse or redistribution. A GPL-3.0-or-later text from an earlier local draft was excluded from the public file set; it was a proposal, not an owner decision. The project may be publicly visible on GitHub while all rights remain reserved. Update this section, Cargo package metadata, and the root license file only after the owner chooses.

## Third-party inventory

The direct Rust and npm dependencies, exact pins, recorded license strings, toolchain pins, and sources are listed in [`licensing/third-party.md`](licensing/third-party.md). The pnpm resolved-license output is [`licensing/npm-licenses.txt`](licensing/npm-licenses.txt). These inventories concern third-party components and do not license SAYT. Cargo deny skips workspace crates marked `publish = false` while the project-license decision is pending; it still checks the third-party dependency graph.

Current owner-review items:

- The npm development dependency inventory includes two packages reported as `BlueOak-1.0.0`.
- The RustSec scan recorded five unmaintained transitive crates and one unsoundness advisory for `glib 0.18.5`; reachability in SAYT was not established.
- `cargo deny check` reports an unmaintained `proc-macro-error` advisory; Cargo deny bans, license, and source checks passed in the prior local audit.
- A full SBOM and legal review are required before distributing binaries.

Do not silently widen license allowlists or describe scanner output as legal approval.
