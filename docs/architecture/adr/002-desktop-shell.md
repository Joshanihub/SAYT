# ADR-002: Tauri 2 with React and TypeScript

**Status:** Proposed<br>
**Owner:** SAYT project owner<br>
**Date:** 2026-11-25<br>
**Version:** 1

## Decision

Start with a Tauri 2 desktop shell and a strict React/TypeScript frontend, pinned to the versions recorded in the manifests.

## Context

The shell should be small, use a native OS webview, and keep UI code separate from the Rust engine. Waveform rendering is performance-sensitive and must be measured before expanding the UI.

## Options

- Tauri 2 + React/TypeScript: smaller native shell and a familiar UI stack; webview performance and per-OS differences need a spike.
- Electron + React/TypeScript: consistent Chromium rendering; larger runtime and distribution footprint.
- Native Rust UI: direct native rendering path; smaller web ecosystem and a different accessibility/localization surface.

## Evidence

As of 2026-11-25, the official Tauri release index lists `tauri` 2.12.7, `tauri-build` 2.7.6, frontend API 2.12.5, and CLI 2.12.6. These versions establish compatibility metadata, not waveform performance. M1 S2 remains the decision gate: four 10-minute waveforms and meters, measured frame times, with p95 ≤16.7 ms on Windows/macOS and ≤33 ms on Linux.

## Consequences

Enables a compact first desktop shell. It makes the OS webview a material dependency and creates an early exit gate before substantial waveform UI work.

## Revisit when

S2 misses the Windows or macOS threshold, or Linux WebKit performance makes the required feature unusable.
