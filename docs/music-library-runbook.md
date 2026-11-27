# SAYT local music library runbook

**Status:** PROPOSED operator guide<br>
**Owner:** SAYT project owner<br>
**Date:** 2026-11-27<br>
**Version:** 1.0

## What the library stores

SAYT stores selected folder paths, file content hashes, detected metadata, missing/loadable state, user-edited catalog metadata, favorites, and crate/playlist membership in an app-local SQLite database at `app_local_data_dir/music-library-v1/music-library.sqlite3`. Track audio remains in its source folder. The scanner does not rename, move, delete, or rewrite source files. No provider login, network scanning, or cloud sync is involved.

## Normal use

1. Open **Library** and choose **Import folders**.
2. In the native folder picker, select only the folder or folders you want indexed. Recursive scan ignores symbolic links and reports progress.
3. Use **All Tracks**, **Recently Added**, **Missing Files**, **Unanalyzed**, **Favorites**, or **Unsupported**, plus search, sort headers, folder, crate, and playlist views.
4. Create a crate for an unordered collection or a playlist when order matters. Select a track to edit SAYT-only title/artist/album metadata. To add it, choose a destination collection in the track action row; the destination choice does not change the collection currently being browsed.
5. Use **Load to A** or **Load to B** only when the track is marked as a current-decoder candidate. SAYT confirms only after the existing engine accepts the file.
6. Use **Export backup** and choose a local save location in the native dialog before making changes that should be recoverable.

## Supported-format truth

The index recognizes files with AAC, AIF/AIFF, APE, FLAC, M4A/M4B, MP1/MP2/MP3/MP4/MPC, Ogg, Opus, Speex (`.spx`), WAV, and WavPack (`.wv`) extensions, then asks the metadata parser to read them. An extension alone does not prove a valid audio container or usable tags. The present deck decoder loads only validated uncompressed integer PCM WAV: 16/24/32-bit, mono/stereo, 8,000–192,000 Hz, no more than 512 MiB, and no more than 32,000,000 decoded frames. Every sample is revalidated during deck load. Other indexed formats remain visible as unsupported; SAYT does not claim MP3/FLAC/AIFF playback yet.

## Troubleshooting

- **A folder has no tracks:** confirm the selected location contains files with supported audio extensions and that the current Windows user can read it. Retry using **Import folders**.
- **A track is missing:** reconnect the drive and rescan the registered root. A complete scan reconciles unseen paths; a cancelled or partially unreadable scan preserves previous missing state. If a renamed/moved file is included in a selected root, its content hash can reconnect the old track identity.
- **A track is unsupported:** check its format and PCM WAV properties above. Metadata recognition does not add a decoder. Loading still passes through the engine and its sample checks.
- **A file is malformed or unreadable:** the scanner reports it and marks its import state when it can read its content identity. If an audio candidate or directory cannot be read, the affected root finishes as partial; SAYT preserves previous missing-file flags because it could not prove the scan was complete. Correct access or file issues and scan again. Automated regression covers an audio candidate disappearing after enumeration; permission-error runtime testing remains open.
- **A crate or playlist is deleted:** only the catalog collection is deleted; source tracks and files remain. Use an exported SQLite backup to preserve catalog state. The current UI exports backups but does not import/restore them.
- **The catalog fails its integrity check:** startup preserves the damaged database and its SQLite sidecars under a timestamped `.corrupt-*` name, then opens a fresh empty catalog. Do not delete the preserved files; keep the backup and contact the project owner before recovery.

## Verification commands

From the repository root, run `cargo test -p sayt-library` for catalog/scanner regressions and the full verification sequence in the root `AGENTS.md` after source changes. A clean test or release build does not prove that a physical output device produces sound. Record owner-selected folder import, real search/collections, Deck A/B loads, restart, device, viewport, and all errors separately in `docs/VERIFICATION_EVIDENCE.md`.
