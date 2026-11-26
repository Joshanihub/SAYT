# ADR-004: Use one typed command path for keyboard and interface actions

**Status:** PROPOSED for the first DJ expansion slice<br>
**Owner:** SAYT project owner<br>
**Date:** 2026-11-26<br>
**Version:** 1.0

## Decision

Route interface and keyboard actions through the same typed application command dispatcher; keep audio work behind the existing Tauri control commands.

## Context

The M1 workspace calls Tauri commands directly from React controls. The expansion brief requires laptop operation, remappable shortcuts, global and deck contexts, conflict prevention, persistence, searchable help, and no duplicated business logic. The engine is Rust per [ADR-003](ADR-003-rust-engine-milestone1.md); its callback and bounded-queue rules continue to apply.

## Options

- **Shared typed dispatcher in the existing React/Tauri shell:** small reversible change; requires UI callbacks to become command actions before keyboard bindings can reuse them.
- **Separate keyboard action implementations:** quick to prototype; duplicates state transitions and risks keyboard controls diverging from buttons.
- **Native hotkey service/plugin:** enables system-wide shortcuts; adds permissions, platform behavior, packaging and a new dependency before an in-app need exists.

## Choice and consequences

Choose the shared typed dispatcher. UI handlers and the keyboard registry construct the same `AppCommand`; one handler maps that command to the existing Tauri control call or application state update. Only commands backed by real M1 behavior appear in the shortcut reference. Future cue, loop, sync, FX, sampler, search, and recording commands remain absent until implemented.

Default bindings are SAYT-authored. The in-app editor stores validated overrides in the Tauri webview's local storage under a schema-versioned key; no credentials or audio paths are stored. Control is the primary modifier on Windows/Linux, Command on macOS, and the overlay renders the platform label. Corrupt, unsupported, or unavailable preference storage falls back to defaults with an actionable notice. A later durable-settings ADR may move preferences into native app configuration without changing command IDs.

The keyboard listener ignores editable text and native range controls, avoids double-activating focused buttons, and rejects operating-system-reserved combinations. The editor detects conflicts against every currently active binding and persists only after validation succeeds.

## Verification

Automated tests must cover default binding uniqueness per platform, event matching, editable-target exclusion, reserved shortcuts, conflict detection, storage round-trip/corruption/failure, and command payload routing. Component tests must cover overlay search, capture, conflict feedback, and reset. Runtime checks must cover transport, loading via the native picker, crossfader and EQ changes, workspace switching, overlay behavior, and persistence across app restart on Windows and macOS.

These checks verify the in-app command path; they do not establish support for external MIDI/HID controllers or system-wide shortcuts.

## Reversibility and review trigger

The command registry and shortcut schema are versioned and local. Revisit the persistence adapter when a native preferences store is introduced, or the command model when controller input is added. Do not change the Rust callback or create a second engine boundary for this feature.
