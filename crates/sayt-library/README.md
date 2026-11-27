# SAYT local music catalog

`sayt-library` indexes local audio selected by the desktop user. It stores a SQLite catalog under app-local data and keeps source audio in place. It does not contact providers or modify music files. Its first preparation capability estimates BPM from supported integer PCM WAV files outside the audio callback; it does not detect musical key or create beatgrids.

The desktop calls `scan_roots` only with paths returned by Tauri's native folder picker. Recursive traversal does not follow symbolic links. Track identity is SHA-256 of file contents; identical files can have multiple saved locations, while different contents never collapse because their names match. A cancelled or partial scan does not mark unseen files missing. Missing state is reconciled only after a complete scan or an explicit filesystem check.

Metadata is read from supported audio containers through Lofty. A format being indexed does not mean the deck can play it. The current deck decoder accepts only validated integer PCM WAV: 16/24/32-bit, mono/stereo, 8–192 kHz, within the size and frame limits in [`docs/REQUIREMENTS.md`](../../docs/REQUIREMENTS.md). The engine revalidates samples at load time. Artwork is read on demand for the selected track, bounded by source and image limits, and is not copied into the audio file or sent over the network.

The catalog uses schema versioning, SQLite transactions, corruption quarantine, a native save dialog for online backup, and paged/searchable queries with a whitelisted sort key and a 256-character search limit. Collections are local catalog records; deleting a crate or playlist never deletes tracks or source files.

Local BPM analysis is limited to the same validated PCM WAV format, 512 MiB file limit, and 32,000,000-frame limit as the current deck decoder. It computes a 100 Hz RMS onset envelope and bounded normalized autocorrelation over 60–200 BPM. The 0–1 score is a periodicity heuristic, not a calibrated probability; weak, silent, and plausible half/double-tempo results are marked uncertain. The same PCM pass produces 160 normalized waveform peak bins, stored as 160 quantized bytes with catalog content identity and encoding version. Unsupported or malformed saved waveform records return an actionable error. Results are cached by content identity and algorithm version. Manual corrections are stored separately from the file's BPM tag and the analysis result, and use the current 30–300 BPM deck-control range. Cancellation does not save a partial result. A stale or changed source file must be rescanned before analysis.

When a library track is loaded into a deck, SAYT applies a user correction first, then a confident local estimate, then a usable file BPM tag. Uncertain candidates are never passed to Beat Jump or Loop Roll. Deck-local manual edits continue to clear when the track changes.

Run the library regression suite from the repository root with `cargo test -p sayt-library`. It uses temporary directories and actual generated audio files for scanner tests; the 10,000-row search test isolates paging behavior and is not a scan-throughput benchmark or a user-library demonstration.

For the current operator flow and recovery steps, see [`docs/music-library-runbook.md`](../../docs/music-library-runbook.md).
