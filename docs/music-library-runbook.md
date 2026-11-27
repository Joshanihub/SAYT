# SAYT local music library runbook

**Status:** PROPOSED operator guide<br>
**Owner:** SAYT project owner<br>
**Date:** 2026-11-27<br>
**Version:** 1.5

## What the library stores

SAYT stores selected folder paths, file content hashes, detected metadata, missing/loadable state, user-edited catalog metadata, favorites, and crate/playlist membership in an app-local SQLite database at `app_local_data_dir/music-library-v1/music-library.sqlite3`. Track audio remains in its source folder. The scanner does not rename, move, delete, or rewrite source files. No provider login, network scanning, or cloud sync is involved.

## Normal use

1. Open **Library** and choose **Import folders**.
2. In the native folder picker, select only the folder or folders you want indexed. Recursive scan ignores symbolic links and reports progress.
3. Use **All Tracks**, **Recently Added**, **Missing Files**, **Unanalyzed**, **Favorites**, or **Unsupported**, plus search, sort headers, folder, crate, and playlist views.
4. Create a crate for an unordered collection or a playlist when order matters. Select a track to edit SAYT-only title/artist/album metadata. To add it, choose a destination collection in the track action row; the destination choice does not change the collection currently being browsed.
5. Use **Load to A** or **Load to B** only when the track is marked as a current-decoder candidate. SAYT confirms only after the existing engine accepts the file.
6. Use **Export backup** and choose a local save location in the native dialog before making changes that should be recoverable.

## Local BPM preparation

1. Open **Preparation** and select a track from the catalog list, or use the same controls in the Library inspector. The focused workspace uses the existing catalog selection and does not accept an arbitrary file path.
2. For a track whose current location is available and marked as supported PCM WAV, use **Analyze local WAV**. The worker reads only the catalog-selected file and does not send PCM samples through Tauri.
3. Analysis can be cancelled while the worker reads or correlates the file. A cancelled result is not saved. An error saying the file changed after import means rescan its registered folder and retry.
4. A confident result is cached by file identity and algorithm version. An uncertain result can show a candidate, but it is not applied to Beat Jump or Loop Roll. The periodicity score is heuristic, not a probability or a claim of commercial-music accuracy.
5. Enter a manual correction from 30 to 300 BPM and choose **Save correction**; use **Clear** to remove it. **TAP BPM** stages a manually timed tempo after three taps. Keep tapping evenly; the tool averages up to four intervals between 200 and 2,000 ms, rounds to one decimal, and restarts after a tap outside that interval range. This is a timing aid and does not listen to or analyze the track. **BPM /2** and **BPM ×2** also stage one-decimal corrections from a valid draft, or from the effective tempo if the draft is not valid. These controls do not save automatically. SAYT stores corrections apart from file tags and analysis. When you load this track into a deck, a saved correction takes precedence, then confident analysis, then an in-range BPM tag. Open that deck's **Roll / Jump** tab and confirm the deck-local Manual BPM value before using Beat Jump or Loop Roll. The deck's own manual edit remains temporary and clears when its loaded track changes.
6. After analysis, the inspector displays the saved 160-bin waveform preview. It is a compact peak summary of the PCM samples, not a playback buffer. Reopening the catalog restores it; a changed or rescanned source requires analysis for the new content identity.

Preparation is a partial dedicated workspace that reuses the Library's real catalog data and analysis controls. The rest of the supplied Screen 04 panels remain unavailable: there is no beatgrid editing, automatic key detection, phrase mapping, loudness analysis, cue suggestion, or loop suggestion.

## Supported-format truth

The index recognizes files with AAC, AIF/AIFF, APE, FLAC, M4A/M4B, MP1/MP2/MP3/MP4/MPC, Ogg, Opus, Speex (`.spx`), WAV, and WavPack (`.wv`) extensions, then asks the metadata parser to read them. An extension alone does not prove a valid audio container or usable tags. The present deck decoder loads only validated uncompressed integer PCM WAV: 16/24/32-bit, mono/stereo, 8,000–192,000 Hz, no more than 512 MiB, and no more than 32,000,000 decoded frames. Every sample is revalidated during deck load. Other indexed formats remain visible as unsupported; SAYT does not claim MP3/FLAC/AIFF playback yet.

## Troubleshooting

- **A folder has no tracks:** confirm the selected location contains files with supported audio extensions and that the current Windows user can read it. Retry using **Import folders**.
- **A track is missing:** reconnect the drive and rescan the registered root. A complete scan reconciles unseen paths; a cancelled or partially unreadable scan preserves previous missing state. If a renamed/moved file is included in a selected root, its content hash can reconnect the old track identity.
- **A track is unsupported:** check its format and PCM WAV properties above. Metadata recognition does not add a decoder. Loading still passes through the engine and its sample checks.
- **A file is malformed or unreadable:** the scanner reports it and marks its import state when it can read its content identity. If an audio candidate or directory cannot be read, the affected root finishes as partial; SAYT preserves previous missing-file flags because it could not prove the scan was complete. Correct access or file issues and scan again. Automated regression covers an audio candidate disappearing after enumeration; permission-error runtime testing remains open.
- **A saved waveform is missing or invalid:** analyze the supported PCM WAV again. Unsupported waveform encodings and truncated peak data produce an actionable error; a track with no current waveform shows the analyze prompt.
- **A crate or playlist is deleted:** only the catalog collection is deleted; source tracks and files remain. Use an exported SQLite backup to preserve catalog state. The current UI exports backups but does not import/restore them.
- **The catalog fails its integrity check:** startup preserves the damaged database and its SQLite sidecars under a timestamped `.corrupt-*` name, then opens a fresh empty catalog. Do not delete the preserved files; keep the backup and contact the project owner before recovery.

## Verification commands

From the repository root, run `cargo test -p sayt-library` for catalog/scanner regressions and the full verification sequence in the root `AGENTS.md` after source changes. A clean test or release build does not prove that a physical output device produces sound. Record owner-selected folder import, real search/collections, Deck A/B loads, restart, device, viewport, and all errors separately in `docs/VERIFICATION_EVIDENCE.md`.
