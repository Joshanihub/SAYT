# SAYT design implementation map

**Status:** IN PROGRESS<br>
**Owner:** SAYT project owner<br>
**Date:** 2026-11-27<br>
**Version:** 0.2.4

This maps every screen in the supplied Stitch archive to the current product boundary. The task handoff identifies the designs as founder-approved; that provenance is recorded here without implying that the Stitch prototype or every screen's behavior was independently re-tested. Prototype simulations are reference material, not evidence of working SAYT features.

## Shared visual system and shell

| Design element                                   | Reference                                                  | Current implementation                                                                                                                                                                                                                                                            |
| ------------------------------------------------ | ---------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Canvas / primary / secondary / elevated surfaces | `#08090B` / `#111318` / `#191D23` / `#242830`              | CSS tokens in `apps/desktop/src/styles.css`.                                                                                                                                                                                                                                      |
| Deck A / Deck B signals                          | Signal Lime `#C7F36B` / Ice Cyan `#38BDF8`                 | Used for deck identity, waveform, transport, and crate accents.                                                                                                                                                                                                                   |
| Text / warning                                   | Neutral text ramp / coral `#F43F5E` / amber `#FB923C`      | Applied to text, errors, and actual master-meter presentation.                                                                                                                                                                                                                    |
| Typography                                       | Space Grotesk, Inter, JetBrains Mono                       | 16 local variable text-font subsets and the 66-ligature Material Symbols subset are loaded via `fonts.css`; build manifest records sources and hashes. The corrected Windows render showed the roles/icons at a 1280×800 client viewport; matched Stitch comparison remains open. |
| Application shell                                | Dense desktop header, navigation, engine and device status | DJ, Library, and Settings routes work in source. Future navigation is disabled with an explanation.                                                                                                                                                                               |
| Responsive layout                                | Desktop 1440×900, compact 1280, narrow stacked layout      | The current Windows Tauri window rendered at a 1280×800 client viewport. 1440×900 and narrow captures, plus matched Stitch comparisons at all target sizes, remain NOT RUN.                                                                                                       |

## Screen-by-screen implementation record

### Shared shell

- **Screen / reference:** Shared application shell; implemented across the 13 screen references. Shared tokens are in `sayt_audio_performance_engine/DESIGN.md`.
- **Workspace / target:** Desktop application frame; M1.
- **Required frontend:** Brand, implemented navigation, engine status, output-device selector, status bar, and responsive layout.
- **Required backend:** Engine snapshot and device enumeration/selection.
- **Current source:** `AppShell` in `apps/desktop/src/App.tsx`; CSS in `apps/desktop/src/styles.css`; `AudioService` in `apps/desktop/src-tauri/src/audio_service.rs`.
- **Verification:** Design complete: yes. Design approved: yes, per supplied handoff. Prototype interaction complete: not verified; prototype specification contains simulated interactions. Implemented in source: yes. Automated: shell/future-navigation render assertion and output-device selector callback in `App.test.tsx` (21 tests passed). Running desktop: DJ workspace rendered at a 1280×800 client viewport. Physical hardware: not applicable to shell.
- **Manual requirements / limitations:** Exercise navigation and device selection in the running desktop app; verify compact/narrow sizes. Device operations still need physical output-device verification.

### Screen 01 — SAYT DJ Main Performance Workspace

- **Reference / asset:** `design/reference/SAYT-Stitch-v1/stitch_sayt_dj_audio_studio/sayt_dj_live_performance_workspace/screen.png` and `code.html`.
- **Workspace / target:** DJ performance workspace; M1.
- **Required frontend:** Two deck panels, accurate loaded-audio waveforms and positions, transport, seek, gain/EQ controls, eight identifiable colorable Hot Cue pads per deck, actual master meter, master level, crossfader, and two-slot quick crate.
- **Required backend:** Existing `sayt-engine` deck/DSP/output behavior, mixer snapshots and commands, native WAV picker, and generation-bound Hot Cue seeks with versioned local per-track metadata. Tauri owns `Engine` and cue file access on a bounded actor thread.
- **Current source:** `DeckPanel`, `HotCueControls`, `Waveform`, `MixerConsole`, and `QuickCrateBrowser` in `apps/desktop/src/App.tsx`; engine code remains in `crates/sayt-engine`; commands, cue store and actor are in `apps/desktop/src-tauri/src/`.
- **Verification:** Design complete/approved: yes, per supplied handoff. Prototype interaction complete: not verified; simulations are not product evidence. Hot Cue source, automated tests, and Windows desktop release build pass locally; real Tauri fixture loading, cue triggering, restart restoration, and screenshot artifact remain pending because owner input/minimization interrupted the live-window run. Initial empty workspace render observed at 1920×1032, not retained as a repository image. Physical playback and sample-accurate timing are not verified.
- **Manual requirements / limitations:** Complete `docs/hot-cues.md` fixture and restart procedure when the window is available. Follow the loaded-file, simultaneous playback, independent transport, audibility, gain/EQ/crossfader, device recovery, and shutdown checks in `docs/REQUIREMENTS.md`. One actual master meter is available; per-deck meters and cue routing remain unavailable.

### Screen 02 — SAYT Music Library Manager

- **Reference / asset:** `design/reference/SAYT-Stitch-v1/stitch_sayt_dj_audio_studio/sayt_music_library_manager/screen.png` and `code.html`.
- **Workspace / target:** Local music library; the empty state has been replaced by a functional, persistent catalog workspace. This advances Screen 02 independently; it does not close M1 physical audio acceptance.
- **Required frontend:** Preserve the reference's left navigation/collection rail, central searchable table, and right track inspector, with honest empty/loading/error/scan states. Supplied example tracks, counts, analysis values, and artwork are not production data.
- **Required backend:** Native folder selection, recursive no-symlink scanner, app-local SQLite catalog, metadata and selected-track artwork reads, search/sort/filter/paging, collections, missing-file recovery, backup, and deck routing through the current engine.
- **Current source:** `apps/desktop/src/MusicLibrary.tsx` and `library.css`; app integration in `App.tsx`; Tauri commands in `apps/desktop/src-tauri/src/lib.rs`; storage/scanning in `crates/sayt-library`. Storage choice is recorded in [ADR-008](DECISIONS/ADR-008-local-music-catalog.md).
- **Verification:** Source implemented. Automated: 14 library tests passed over real temporary audio files, plus a 10,000-row paged-search case; frontend typecheck, lint, formatting, 64 tests and production web build passed. Windows Tauri build, native folder-picker runtime, real user-folder search/collections/restart, deck A/B load, and viewport captures remain NOT RUN at this checkpoint. No exact-head CI exists for these unpushed edits.
- **Manual requirements / limitations:** Owner must choose a genuine music folder in the running app, search a real track, save a crate/playlist, load one accepted WAV into each deck, then restart and confirm catalog/collection restoration. The current deck decoder supports only validated integer PCM WAV; MP3/FLAC/AIFF and other recognized containers are metadata-indexed but not loadable. BPM/key are displayed only from tags; automatic analysis is not implemented. Artwork is read from the selected source file on demand and is not copied into the audio or uploaded.
- **Operator guide:** [`music-library-runbook.md`](music-library-runbook.md) documents local data, format limits, safe scanning, backup, and recovery.

### Screen 03 — Playlist & Collection Bridge

- **Reference / asset:** `design/reference/SAYT-Stitch-v1/stitch_sayt_dj_audio_studio/sayt_playlist_collection_bridge/screen.png` and `code.html`.
- **Workspace / target:** Collection/import bridge; deferred, target milestone not assigned.
- **Required frontend:** Collection source selection, mapping/preview, progress, conflict and error states.
- **Required backend:** Provider authorization, import/export, mapping persistence, and conflict handling; none exists in M1.
- **Current source:** Navigation is disabled and labelled as unavailable in M1; no bridge screen or provider integration.
- **Verification:** Design complete/approved: yes, per supplied handoff. Prototype interaction complete: not verified. Source implementation: deferred. Automated/runtime/physical verification: N/A. Deferred: yes.
- **Manual requirements / limitations:** Define supported providers, consent, credentials handling, and conflict policy before implementation. No playlist or account data is accessed.

### Screen 04 — Track Preparation & Analysis

- **Reference / asset:** `design/reference/SAYT-Stitch-v1/stitch_sayt_dj_audio_studio/sayt_track_preparation_analysis/screen.png` and `code.html`.
- **Workspace / target:** Track preparation; deferred, target milestone not assigned.
- **Required frontend:** Analysis progress, metadata, waveform, cue/loop editing, and explicit unavailable/error states.
- **Required backend:** Analysis pipeline, durable metadata and cue storage, and versioned results; none exists in M1.
- **Current source:** Navigation is disabled; only waveform peaks from loaded engine snapshots are displayed in the DJ workspace.
- **Verification:** Design complete/approved: yes, per supplied handoff. Prototype interaction complete: not verified. Source implementation: deferred. Automated/runtime/physical verification: N/A. Deferred: yes.
- **Manual requirements / limitations:** Do not present prototype BPM, key, cue, or analysis values as real. Define algorithms and quality measures before implementation.

### Screen 05 — Audio & Device Settings

- **Reference / asset:** `design/reference/SAYT-Stitch-v1/stitch_sayt_dj_audio_studio/sayt_audio_device_settings/screen.png` and `code.html`.
- **Workspace / target:** Shared audio settings; M1 limited output controls.
- **Required frontend:** Device list/selection, discovery loading, empty and enumeration-error states, refresh, output error, retry, and truthful capabilities.
- **Required backend:** Existing engine device enumeration/selection and output recovery attempt.
- **Current source:** `AudioSettings`, top-bar selector, and output-error banner in `apps/desktop/src/App.tsx`; `AudioService` forwards device commands to the engine thread. Device discovery is represented as loading, loaded, or error so an empty initial state is not reported as an absent device.
- **Verification:** Design complete/approved: yes, per supplied handoff. Prototype interaction complete: not verified. Source: limited controls. Automated: component tests cover discovery loading, empty results, enumeration failure, output errors, and refresh/retry handlers. These component tests do not exercise device switching through real Tauri IPC. Running desktop: settings page not inspected. Physical: not verified.
- **Manual requirements / limitations:** Test physical-device selection, disconnect/reselect, recovery, and shutdown. Sample-rate, buffer, cue, and latency controls are not exposed and must not be implied.

### Screen 06 — Appearance & Customisation

- **Reference / asset:** `design/reference/SAYT-Stitch-v1/stitch_sayt_dj_audio_studio/sayt_appearance_customisation/screen.png` and `code.html`.
- **Workspace / target:** Appearance settings; deferred, target milestone not assigned.
- **Required frontend:** Theme/accent/density preferences, preview, reset, persistence, and accessibility-safe contrast.
- **Required backend:** Preference persistence and migration; none exists in M1.
- **Current source:** Navigation is disabled; shared M1 tokens are static.
- **Verification:** Design complete/approved: yes, per supplied handoff. Prototype interaction complete: not verified. Source implementation: deferred. Automated/runtime/physical verification: N/A. Deferred: yes.
- **Manual requirements / limitations:** Validate contrast, keyboard operation, and persisted settings when scoped. No settings controls or saved preferences currently exist.

### Screen 07A — Performance Effects Rack

- **Reference / asset:** `design/reference/SAYT-Stitch-v1/stitch_sayt_dj_audio_studio/sayt_performance_effects_rack/screen.png` and `code.html`.
- **Workspace / target:** DJ effects; deferred, target milestone not assigned.
- **Required frontend:** Effect state, parameter controls, enable/bypass feedback, and safe failure states.
- **Required backend:** Audio-thread-safe effect graph and bounded parameter commands; none exists in M1.
- **Current source:** Navigation is disabled; no effects are exposed.
- **Verification:** Design complete/approved: yes, per supplied handoff. Prototype interaction complete: not verified. Source implementation: deferred. Automated/runtime/physical verification: N/A. Deferred: yes.
- **Manual requirements / limitations:** Requires DSP correctness, callback-load/dropout measurements, and physical listening before any effect is called verified.

### Screen 07B — Sampler & Remix Performance

- **Reference / asset:** `design/reference/SAYT-Stitch-v1/stitch_sayt_dj_audio_studio/sayt_sampler_remix_performance/screen.png` and `code.html`.
- **Workspace / target:** Sampler/remix; deferred, target milestone not assigned.
- **Required frontend:** Sample slots, trigger/stop state, loading and error feedback.
- **Required backend:** Bounded sample loading, scheduling, voices and DSP integration; none exists in M1.
- **Current source:** Navigation is disabled; no sampler behavior is exposed.
- **Verification:** Design complete/approved: yes, per supplied handoff. Prototype interaction complete: not verified. Source implementation: deferred. Automated/runtime/physical verification: N/A. Deferred: yes.
- **Manual requirements / limitations:** Requires sample limits, callback safety and latency/dropout evaluation before implementation claims.

### Screen 08A — Studio Arrangement

- **Reference / asset:** `design/reference/SAYT-Stitch-v1/stitch_sayt_dj_audio_studio/sayt_studio_arrangement_daw_workspace/screen.png` and `code.html`.
- **Workspace / target:** Studio/DAW; deferred, target milestone not assigned.
- **Required frontend:** Timeline, clip editing, transport, project loading/saving and empty/error states.
- **Required backend:** Project model, arrangement persistence, audio rendering and recovery; none exists in M1.
- **Current source:** Studio navigation is disabled; no DAW implementation.
- **Verification:** Design complete/approved: yes, per supplied handoff. Prototype interaction complete: not verified. Source implementation: deferred. Automated/runtime/physical verification: N/A. Deferred: yes.
- **Manual requirements / limitations:** Project format, compatibility, undo, and recovery behavior need design before implementation.

### Screen 08B — Studio Professional Mixer

- **Reference / asset:** `design/reference/SAYT-Stitch-v1/stitch_sayt_dj_audio_studio/sayt_studio_professional_mixing_console/screen.png` and `code.html`.
- **Workspace / target:** Studio mixer; deferred, target milestone not assigned.
- **Required frontend:** Track buses, routing, meters, faders and automation.
- **Required backend:** Multi-track routing, metering, automation and persistent project state; none exists in M1.
- **Current source:** Studio navigation is disabled. M1 has only a two-deck master bus.
- **Verification:** Design complete/approved: yes, per supplied handoff. Prototype interaction complete: not verified. Source implementation: deferred. Automated/runtime/physical verification: N/A. Deferred: yes.
- **Manual requirements / limitations:** Do not conflate the M1 two-deck mixer with a multitrack console. Audio routing and project-state requirements remain undefined.

### Screen 08C — Studio Piano Roll & MIDI Editor

- **Reference / asset:** `design/reference/SAYT-Stitch-v1/stitch_sayt_dj_audio_studio/sayt_studio_piano_roll_midi_editor/screen.png` and `code.html`.
- **Workspace / target:** MIDI editor; deferred, target milestone not assigned.
- **Required frontend:** Piano roll, note edit/selection, grid/zoom, velocity and MIDI-device state.
- **Required backend:** MIDI event model, device I/O, timing and project persistence; none exists in M1.
- **Current source:** Studio navigation is disabled; no MIDI support.
- **Verification:** Design complete/approved: yes, per supplied handoff. Prototype interaction complete: not verified. Source implementation: deferred. Automated/runtime/physical verification: N/A. Deferred: yes.
- **Manual requirements / limitations:** Requires MIDI timing/device behavior and physical controller verification before claims of support.

### Screen 08D-A — Studio Clip Launcher

- **Reference / asset:** `design/reference/SAYT-Stitch-v1/stitch_sayt_dj_audio_studio/sayt_studio_clip_launcher/screen.png` and `code.html`.
- **Workspace / target:** Studio clip launcher; deferred, target milestone not assigned.
- **Required frontend:** Clip/scene state, launch/queued feedback and empty/error states.
- **Required backend:** Quantized scheduling, clip playback, project persistence and bounded audio-thread commands; none exists in M1.
- **Current source:** Studio navigation is disabled; no clip launcher.
- **Verification:** Design complete/approved: yes, per supplied handoff. Prototype interaction complete: not verified. Source implementation: deferred. Automated/runtime/physical verification: N/A. Deferred: yes.
- **Manual requirements / limitations:** Scheduling accuracy and audio callback behavior require measured verification.

### Screen 08D-B — Studio Step Sequencer

- **Reference / asset:** `design/reference/SAYT-Stitch-v1/stitch_sayt_dj_audio_studio/sayt_studio_step_sequencer/screen.png` and `code.html`.
- **Workspace / target:** Studio step sequencer; deferred, target milestone not assigned.
- **Required frontend:** Step grid, pattern state, playback position and edit feedback.
- **Required backend:** Pattern model, timing/scheduling and project persistence; none exists in M1.
- **Current source:** Studio navigation is disabled; no step sequencer.
- **Verification:** Design complete/approved: yes, per supplied handoff. Prototype interaction complete: not verified. Source implementation: deferred. Automated/runtime/physical verification: N/A. Deferred: yes.
- **Manual requirements / limitations:** Requires timing, persistence, and device/audio verification in its target milestone.

## Archive and interpretation

- Original package: `design/reference/exports/SAYT-Stitch-v1.zip`; extracted inventory and SHA-256 are recorded in `design/reference/README.md`.
- Each screen folder preserves `screen.png` and generated `code.html`; the original export also includes the design-system document and interactive-prototype specification.
- Generated HTML references remote assets and simulated/demo data. It is preserved as a design reference and is not imported as production UI.
- The Stitch specification's prototype interactions are not treated as complete SAYT functionality. “Implemented in source,” “automated tests passed,” “rendered in desktop,” and “verified on physical hardware” are separate states.
- Onboarding Screen 09 is absent from this archive and is not specified here.

## M1 verification boundary

The UI test suite uses static rendering and direct component callback checks; it does not exercise Tauri IPC or the native picker end to end. The app's DJ and Library empty states rendered at 1280×800. The native library folder picker opened, but no real folder was selected yet. Settings runtime rendering, real catalog IPC/data, 1440×900 and narrow library layouts, matched Stitch comparisons at every target size, and physical output checks remain open. M1 stays **IN PROGRESS** until its CI and physical-device exit gates in `docs/REQUIREMENTS.md` pass.
