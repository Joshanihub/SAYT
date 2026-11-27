# SAYT local music catalog

`sayt-library` indexes local audio selected by the desktop user. It stores a SQLite catalog under app-local data and keeps source audio in place. It does not contact providers, modify music files, decode tracks for playback, or perform BPM/key analysis.

The desktop calls `scan_roots` only with paths returned by Tauri's native folder picker. Recursive traversal does not follow symbolic links. Track identity is SHA-256 of file contents; identical files can have multiple saved locations, while different contents never collapse because their names match. A cancelled or partial scan does not mark unseen files missing. Missing state is reconciled only after a complete scan or an explicit filesystem check.

Metadata is read from supported audio containers through Lofty. A format being indexed does not mean the deck can play it. The current deck decoder accepts only validated integer PCM WAV: 16/24/32-bit, mono/stereo, 8–192 kHz, within the size and frame limits in [`docs/REQUIREMENTS.md`](../../docs/REQUIREMENTS.md). The engine revalidates samples at load time. Artwork is read on demand for the selected track, bounded by source and image limits, and is not copied into the audio file or sent over the network.

The catalog uses schema versioning, SQLite transactions, corruption quarantine, a native save dialog for online backup, and paged/searchable queries with a whitelisted sort key and a 256-character search limit. Collections are local catalog records; deleting a crate or playlist never deletes tracks or source files.

Run the library regression suite from the repository root with `cargo test -p sayt-library`. It uses temporary directories and actual generated audio files for scanner tests; the 10,000-row search test isolates paging behavior and is not a scan-throughput benchmark or a user-library demonstration.

For the current operator flow and recovery steps, see [`docs/music-library-runbook.md`](../../docs/music-library-runbook.md).
