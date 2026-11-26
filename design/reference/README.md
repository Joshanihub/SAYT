# SAYT Stitch design references

**Status:** REFERENCE<br>
**Owner:** SAYT project owner<br>
**Captured:** 2026-11-26<br>
**Archive:** `exports/SAYT-Stitch-v1.zip`<br>
**SHA-256:** `E97F43E20EDE4BC3993CFB8E4177BB53C3AF6296C58AEE11E0E1DAA347A4B2F1`

The attached archive was preserved byte-for-byte and extracted to `SAYT-Stitch-v1/`. ZIP integrity was checked before extraction (43 entries, 6,904,360 uncompressed bytes, CRC check passed). Entry names were checked for absolute paths, parent traversal, duplicate paths, and links; extraction stayed under the versioned reference directory. The copied archive hash matches the source attachment.

## Contents

The export contains 13 screen folders, each with a `screen.png` and generated `code.html`, plus the design-system document and an interactive-prototype specification:

| Screen                             | Reference folder                           | M1 implementation status                                                                                                                                |
| ---------------------------------- | ------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 01 — DJ Live Performance Workspace | `sayt_dj_live_performance_workspace/`      | Partially implemented: two real deck surfaces, real mixer state, device summary, and quick crate; physical audio verification remains open.             |
| 02 — Music Library Manager         | `sayt_music_library_manager/`              | Honest empty state only; no persistent index or fabricated tracks.                                                                                      |
| 03 — Playlist & Collection Bridge  | `sayt_playlist_collection_bridge/`         | Deferred; navigation disabled.                                                                                                                          |
| 04 — Track Preparation & Analysis  | `sayt_track_preparation_analysis/`         | Deferred; navigation disabled.                                                                                                                          |
| 05 — Audio & Device Settings       | `sayt_audio_device_settings/`              | Limited M1 settings view for real device enumeration/selection, retry, and refresh. Sample-rate, buffer, cue, and latency controls are not implemented. |
| 06 — Appearance & Customisation    | `sayt_appearance_customisation/`           | Deferred; navigation disabled.                                                                                                                          |
| 07A — Performance Effects Rack     | `sayt_performance_effects_rack/`           | Deferred; navigation disabled.                                                                                                                          |
| 07B — Sampler & Remix Performance  | `sayt_sampler_remix_performance/`          | Deferred; navigation disabled.                                                                                                                          |
| 08A — Studio Arrangement           | `sayt_studio_arrangement_daw_workspace/`   | Deferred; navigation disabled.                                                                                                                          |
| 08B — Studio Mixing Console        | `sayt_studio_professional_mixing_console/` | Deferred; navigation disabled.                                                                                                                          |
| 08C — Piano Roll & MIDI Editor     | `sayt_studio_piano_roll_midi_editor/`      | Deferred; navigation disabled.                                                                                                                          |
| 08D-A — Clip Launcher              | `sayt_studio_clip_launcher/`               | Deferred; navigation disabled.                                                                                                                          |
| 08D-B — Step Sequencer             | `sayt_studio_step_sequencer/`              | Deferred; navigation disabled.                                                                                                                          |

`sayt_audio_performance_engine/DESIGN.md` holds the shared tokens. `sayt_interactive_prototype_specification.md` describes simulated interactions; it is not an implementation contract where it conflicts with M1 requirements or actual engine capabilities.

## Handling notes

- The archive and generated HTML are preserved as design references, not imported application code.
- Generated HTML references remote Tailwind, Google Fonts, and remote image assets. The desktop application does not load those resources; it remains local-first and uses system font fallbacks.
- Prototype track names, artwork, playlists, BPM/key analysis, cue pads, sample-rate/buffer controls, per-channel meters, and simulated latency are not SAYT data or implemented SAYT features.
- The M1 implementation map is maintained in [`../../docs/DESIGN_IMPLEMENTATION_MAP.md`](../../docs/DESIGN_IMPLEMENTATION_MAP.md).
