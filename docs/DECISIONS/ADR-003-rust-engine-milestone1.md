# ADR-003: Continue the Rust audio engine for Milestone 1

**Status:** ACCEPTED for M1; review after hardware validation<br>
**Owner:** SAYT project owner<br>
**Date:** 2026-11-25<br>
**Version:** 1.0

## Decision

Keep the Milestone 1 audio engine in Rust and use CPAL plus a bounded SPSC command path; do not introduce a second C++20 engine boundary in this milestone.

## Context

The master build brief names C++20 and CMake for the native engine. The founding vision charter and existing ADR-001 select Rust to keep the application and engine in one systems language, share DSP with a future WASM target, and isolate any later plugin host out of process. The checkout already contains the Rust/CPAL prototype and DSP modules. No C++ compiler was available on the inspected Windows PATH; this is an execution constraint, not the principal architecture rationale.

## Options

- **Continue Rust:** preserves the accepted charter decision and current implementation, with fewer language boundaries now; leaves plugin hosting as a future isolated process.
- **Replace with C++20 now:** follows the master brief literally and aligns with some native audio ecosystems; adds a second toolchain and requires replacing the existing engine before its behavior is validated.
- **Bridge Rust to a C++ engine:** preserves Rust orchestration and uses C++ audio, but creates a native ABI/lifecycle boundary before M1 has a proven need for it.

## Rationale and consequences

The Rust path is retained because the owner-supplied vision charter explicitly amended the earlier C++ rationale and ADR-001 already records that choice. This is a scoped reconciliation of conflicting owner documents, not a claim that Rust is universally superior for audio. It enables M1 work without duplicating the real-time core. It forecloses C++ DSP and plugin APIs inside the process for now; any future plugin host should remain out-of-process and be evaluated separately.

## Evidence and review trigger

Rust code compiles in the current workspace subject to the checks recorded in the M1 report. Hardware timing, callback allocation behavior under profiling, and cross-platform output are not verified. Revisit this ADR if physical-device tests show the Rust/CPAL design cannot meet the documented M1/M2 reliability gate, or if an accepted plugin-host requirement creates a measured need for an isolated C++ process.

## Reversibility

Reversible before a stable public engine API; migration cost rises once external extensions depend on engine interfaces. Keep interfaces narrow and do not promise a stable plugin ABI in M1.
