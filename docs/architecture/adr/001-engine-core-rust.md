# ADR-001: Real-time engine core in Rust

**Status:** Proposed<br>
**Owner:** SAYT project owner<br>
**Date:** 2026-11-25<br>
**Version:** 1

## Decision

Use Rust for the initial real-time audio engine and keep any future native plugin host in a separate process.

## Context

Audio callback deadlines are strict, while this project is initially staffed by a small team. Plugin crashes and UI failures must not take down audio. The charter proposes Rust for the engine and process isolation for plugin hosting.

## Options

- Rust engine with process-isolated plugin host: one language for the first real-time core; plugin-host integration waits until Studio.
- C++ engine with plugin APIs in-process: direct reuse of native audio/plugin ecosystems; larger memory-safety and crash-isolation burden in the core.
- Rust and C++ mixed in the engine: wider library choice; adds FFI and build complexity before a stretch or plugin library is selected.

## Evidence

This is a provisional architecture choice from the founding charter, not a measured performance result. M1 S1 must record underruns and callback-duration histograms at 44.1/48 kHz and 64/128/256/512 frames. M1 S5 compares time-stretch candidates without integrating one.

## Consequences

Enables a single-language engine and a later WASM reuse path. Defers plugin hosting and any native C ABI until evidence shows it is needed.

## Revisit when

Studio starts, a required DSP dependency cannot be used from Rust without unacceptable FFI, or S1 misses its decision rule.
