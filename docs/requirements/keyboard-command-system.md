# Keyboard command system

**Status:** PROPOSED acceptance contract<br>
**Owner:** SAYT project owner<br>
**Date:** 2026-11-26<br>
**Version:** 1.0

## Purpose and scope

Provide reliable keyboard operation for commands that SAYT already implements. Keyboard shortcuts and interface controls use the same typed application command and dispatcher; native audio remains behind the existing Tauri control API and Rust engine.

This slice covers the current two-deck transport, native track loading, seeking, deck gain/EQ, master level, crossfader, audio-device refresh/retry, workspace navigation, shortcut help, Hot Cues, and manual loop in/out/release. Loop-size pads use seconds because BPM/beatgrid support is not implemented. Loop Roll, Beat Jump, stutter, pitch, sync, effects, sampler, recording, library search and controller commands remain absent until their real behavior exists.

## Functional requirements

1. Maintain a typed command registry with stable command IDs, typed payloads, a global or focused-deck context, and a real application action for every listed shortcut.
2. Route button, range-control and shortcut actions through the same command dispatcher. Do not duplicate audio, navigation or loading behavior in the key listener.
3. Allow the user to select Deck A or B as the target for focused-deck shortcuts. Clicking or keyboard-focusing a deck selects it; loading into a deck also selects it.
4. Support platform-aware defaults, editable key combinations, modifier keys, duplicate detection, reserved-key rejection, persistent local preferences, and reset-to-defaults.
5. Display current mappings beside relevant deck, gain/EQ, mixer and help controls. Hints must reflect remapped bindings.
6. Provide a searchable, keyboard-accessible shortcut dialog. Keep focus inside it while open and return focus to the previous element on close.
7. Ignore application shortcuts while text inputs, selects, editable content, or native range controls are being operated. Preserve native Space activation for focused buttons and links.
8. If preference storage is unavailable or invalid, use built-in defaults and show an actionable status message. Do not report a mapping as saved until the storage write succeeds.
9. Clamp deck and mixer adjustments to their defined control ranges before sending them through existing Tauri commands.

## Verification contract

| Requirement                                              | Automated evidence                                                                                                                     | Runtime evidence                                                                                                                          |
| -------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------- |
| Command IDs, focused-deck payloads and modifier matching | Registry tests for both decks, exact modifier matching, repeat policy and current UI command equivalence                               | Use focus, play/pause, stop, seek and gain/EQ shortcuts against loaded WAV fixtures; observe engine state                                 |
| Defaults and conflict prevention                         | Unique defaults and reserved-key tests on Windows, macOS, Linux and fallback platform; duplicate and reserved custom-binding rejection | Confirm the Windows runtime displays Windows labels; macOS runtime remains a separate OS check                                            |
| Search, remapping, persistence and reset                 | Search and storage round-trip, corrupt/unsupported storage, conflict, storage failure and empty-reset tests                            | Search the overlay, change one mapping, restart, verify it remains, reset, and verify defaults                                            |
| Inputs and accessibility                                 | Editable/native-space target tests; overlay semantic markup and focus-management implementation review                                 | Type into search and range controls without triggering app commands; use Tab, Shift+Tab, Escape and screen reader review                  |
| Engine integration                                       | UI controls and shortcut definitions emit the same typed application command; dispatcher maps only to current Tauri engine commands    | Load distinct fixtures, operate both decks and mixer controls, observe positions/meters/errors; do not infer audible output from UI state |

Manual-loop runtime acceptance uses two user-selected WAV fixtures. On each deck, verify a size-pad loop, Loop In/Out, release while playing and paused, seek outside the loop, stop, track replacement, and independence from the other deck. Listen for loop-seam artifacts; the current callback has no crossfade. Hardware sound quality remains NOT VERIFIED until this run is recorded.

## Boundaries

- Preferences use versioned WebView local storage; no audio paths, credentials or user library data are stored.
- The app does not install system-wide hotkeys and does not claim MIDI, HID, jog-wheel or controller compatibility.
- Hardware playback, physical latency, dropout, and sound-quality acceptance remain separate from keyboard dispatch tests.
- M1 device-switch and shutdown evidence remains tracked in [`REQUIREMENTS.md`](../REQUIREMENTS.md).
