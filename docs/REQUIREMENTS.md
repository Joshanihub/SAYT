# Milestone 1 requirements

**Status:** PROPOSED acceptance contract<br>
**Owner:** SAYT project owner<br>
**Date:** 2026-11-28<br>
**Version:** 0.1.11

## Functional requirements

1. Launch a desktop application with a two-deck workspace and output-device selection.
2. Load an actual local WAV file into either deck using the native file picker.
3. Play, pause, stop, and seek each deck independently; playback position reflects the engine state.
4. Mix both decks to a selected output device. The crossfader routes A at its left endpoint and B at its right endpoint.
5. Deck gain and low, mid, and high controls change the signal sent to the output.
6. Display duration, sample rate, channel count, and waveform peaks derived from the selected recording.
7. Return a clear error for absent devices, unsupported WAV formats, malformed data, and output stream failures. Distinguish device-list loading, an empty device list, and enumeration failure; never label an unselected output as connected. A device switch still attempts to open the replacement if pausing the current stream errors. If opening the replacement fails, attempt to resume and retain the previous stream; report whether recovery succeeded. A failed switch leaves the prior stream usable when the OS permits recovery.
8. Release the stream and track resources on application shutdown.
9. Give button and select state changes short visual feedback, honor the operating-system reduced-motion preference, and retain clear keyboard focus indicators.
10. Keep both decks and the mixer in a deterministic, usable order at desktop, compact, and narrow window sizes; the resizable desktop window supports the existing 480 px narrow breakpoint.

## Supported WAV input in the current decoder

- Uncompressed integer PCM, 16-, 24-, or 32-bit.
- Mono or stereo.
- Sample rate 8,000 through 192,000 Hz.
- Maximum file size 512 MiB and maximum decoded length 32,000,000 frames per deck.
- Floating-point WAV, compressed WAV, more than two channels, empty audio, malformed sample data, and larger files are rejected.

The configured maximum is a safety bound, not a promise that every machine can comfortably hold two maximum-size tracks in memory. Two 32-million-frame stereo tracks require about 512 MiB for frame samples alone, before vector capacity and application overhead.

## Verification matrix

| Requirement                           | Automated evidence                                                                                                                                                                                                          | Manual evidence required                                                                                                                                                                             |
| ------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Decode actual PCM and derive waveform | Decoder tests cover valid mono PCM16 and stereo PCM24/PCM32 normalization, floating-point and three-channel rejection, sample-rate limits, empty/truncated data, missing paths, file-size limit, and decoded-frame boundary | Windows native-picker runtime rejected a floating-point WAV with the decoder message and preserved the previously loaded valid track. Load two distinct PCM WAV files and confirm metadata/waveform. |
| Independent transport and seek        | Engine tests for state, bounds, and queue behavior                                                                                                                                                                          | Confirm A and B play, pause, stop, and seek independently                                                                                                                                            |
| Crossfader                            | Endpoint and equal-power math tests; rendered A-only test                                                                                                                                                                   | Listen at left, center, and right positions                                                                                                                                                          |
| Three-band EQ                         | Frequency-response tests for low, mid, and high bands                                                                                                                                                                       | Sweep each band using test tones or known material and listen for change                                                                                                                             |
| Output-device selection               | Component tests cover device discovery loading, empty results, enumeration errors, output errors, refresh, and retry; transition-policy tests cover pause failure, replacement failure, and resume failure                  | Confirm output on a physical device; verify a real no-device state and safely test failed-switch recovery                                                                                            |
| Error handling and shutdown           | Decoder error cases, native command tests, and UI regression tests separating decoder errors from output-recovery controls                                                                                                  | Unsupported float WAV rejection through the Windows native picker is verified. Physical no-device/output-failure recovery and clean shutdown during active playback remain open.                     |
| Interaction motion and reduced motion | Stylesheet regression test checks short button/select transitions and the reduced-motion override                                                                                                                           | Enable the OS reduced-motion preference; verify transitions are suppressed and keyboard focus remains visible                                                                                        |
| Responsive deck placement             | Stylesheet regression test checks desktop, compact, and narrow named grid areas                                                                                                                                             | Capture the running Tauri window at 1440×900, 1280×800, and 480×700; verify both decks and mixer appear in order                                                                                     |

## Manual hardware checklist

1. Start SAYT on a machine with a physical audio output and select that device.
2. Load distinct PCM WAV files into Deck A and Deck B; verify each waveform, duration, and format metadata.
3. Start both decks. Confirm both are audible simultaneously, with position advancing only while each deck plays.
4. Independently pause, resume, seek, and stop each deck.
5. Change each deck's gain and each EQ band; confirm audible and metered changes.
6. Move the crossfader fully left, center, and fully right; confirm the intended mix.
7. Disconnect or disable the selected output if safe to do so; refresh and reselect an available output. Record the message, recovery action, whether the previous output remains usable after a failed switch, and whether playback resumes after reselection.
8. Close the app while audio is active; confirm the output stops and the process exits without a stuck device.
9. With the operating-system reduced-motion preference enabled, confirm button/select transitions stop and keyboard focus remains visible.

Record device model, OS, sample rate, WAV properties, observed failures, and test duration with the result. Until every applicable item is recorded, hardware acceptance remains owner-reported and is not independently verified.

## Screen 02 local music library acceptance contract

This feature-track contract is separate from the open M1 physical-audio gate. The current implementation is in progress on `feat/dj-command-system`; it does not authorize a merge into `main`.

The normal user workflow and catalog recovery guidance are in [`music-library-runbook.md`](music-library-runbook.md).

1. The desktop accepts library roots only through the native folder picker, recursively indexes the selected local audio formats, reports progress, and can cancel. Recursive traversal does not follow symbolic links. Source audio bytes, names, and locations are never changed by the scanner.
2. The local catalog retains content identity, selected file locations, metadata actually present in tags, format, missing state, user metadata, favorites, and user-created crate/playlist membership. Search and sort inputs are bounded/whitelisted and results are paged.
3. A complete scan may mark unseen locations missing. A cancelled or permission-partial scan must not infer missing files from folders it could not finish reading. Rescanning a moved/renamed file with the same content recovers the same track identity. Same-name files with different contents remain separate.
4. Crate/playlist creation, rename, deletion, membership, and playlist order persist through application restart. Deleting a collection never deletes the source music. A catalog backup is created only at a user-selected native save location.
5. The library distinguishes metadata-indexed formats from formats supported by the current decoder. Deck loads resolve a saved track through its selected root, invoke the existing engine command, and report success only after the engine returns success.
6. Metadata, tag artwork, BPM, and key are shown only when actually present. SAYT does not invent analysis results. Artwork preview reads the selected file on demand with explicit size/MIME checks.

| Scenario                                                                                | Automated evidence                                                             | Live desktop evidence required                                                                                                              |
| --------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------- |
| Real folder import, search, sort, filters, crate/playlist persistence and order         | `sayt-library` tests over temporary directories and real generated audio files | Owner selects a genuine music folder; search a real tagged/filename track; create/reorder collections; restart and confirm persistence      |
| Deck routing and format truth                                                           | Catalog path checks plus existing engine decoder tests                         | Load one accepted PCM WAV into each deck and confirm the engine snapshot; try one unsupported indexed format and observe explicit rejection |
| Duplicate, rename/move and missing file                                                 | Content-identity, moved-file, and missing-state regression tests               | Move/rename a selected real file, rescan its folder, and confirm identity recovery and missing state                                        |
| Cancellation, malformed metadata, permission/traversal errors, and large catalog paging | Scanner cancellation, malformed-file, and 10,000-row paged-query tests         | Observe cancellation/progress against the chosen folder; permission error is NOT RUN until a safe representative folder can be selected     |
| Runtime layout                                                                          | Frontend tests and production build                                            | Capture the running library at 1440×900, 1280×800, and narrow size; check focus, scroll areas, and error/empty states                       |

The 10,000-row test verifies search paging against catalog rows; it is not evidence of 10,000-file scan throughput. Physical audio output and sustained performance-control tests remain separate M1 acceptance items.

## Screen 04 — Local BPM analysis first slice

**Status:** PROPOSED feature-track contract<br>
**Owner:** SAYT project owner<br>
**Date:** 2026-11-28<br>
**Version:** 0.2

The first real Track Preparation capability is available in the selected-track inspector in the local library. It is not the complete supplied Screen 04 workspace. The screen reference also depicts beatgrid editing, key and phrase analysis, vocal/stem analysis, cue/loop preparation, and precision beat-jump controls; these remain unavailable and must not be represented as working until their algorithms, persisted state, controls, and verification exist.

1. Analysis is started for a selected catalog track ID; the UI cannot submit an arbitrary filesystem path. The native catalog resolves and canonicalizes the file under its registered root. Missing, unsupported, unreadable, or changed files fail with an actionable message and do not receive a result.
2. The current analyzer accepts only the existing decoder's 16/24/32-bit integer PCM WAV, mono/stereo, 8,000–192,000 Hz, up to 512 MiB and 32,000,000 frames. It reads and analyzes on a bounded cancellable worker outside the CPAL callback. Only one BPM analysis runs at a time. The file identity is checked before and after processing; cancellation persists no partial result.
3. The versioned estimator uses a 100 Hz RMS-onset envelope and normalized autocorrelation in the 60–200 BPM range. It stores a candidate, `estimated` or `uncertain` status, algorithm version, and a 0–1 periodicity heuristic. The score is not a probability. Weak/silent/short input and plausible half/double-tempo matches remain uncertain; the UI labels uncertain values as candidates for review.
4. File-tag BPM remains unchanged. A user correction is stored in a separate SQLite table and accepts finite values from 30 through 300 BPM. Results are cached by content identity and algorithm version. Effective BPM is manual correction, otherwise confident analysis, otherwise an in-range file tag. Uncertain candidates are not sent to deck beat controls. Loading the selected track passes that BPM into the existing deck-local manual BPM control using the loaded deck snapshot's engine `source_track_id`; the catalog file-byte hash and engine decoded-PCM identity are different ID domains. Passing the catalog ID fails the engine's stale-track guard.
5. The same bounded PCM pass computes 160 normalized waveform peak bins. Peaks are quantized into a 160-byte catalog record with source identity and encoding version, and shown in the selected-track inspector after analysis. Reopening the catalog restores the same peak vector; unsupported or corrupt peak records return no waveform and an actionable error.
6. No beatgrid, key result, phrase map, loudness result, cue suggestion, loop suggestion, or music-library compatibility score is inferred from BPM or waveform analysis.

| Scenario                                                     | Automated evidence                                                                                                              | Live desktop evidence                                                                                           |
| ------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------- |
| Known tempo, silence, short input, and half/double ambiguity | Deterministic WAV fixtures verify a confident 60 BPM pulse, uncertain 120 BPM pulse, silent and short cases                     | Analyze known temporary fixtures; inspect reported candidate and uncertain state                                |
| Unsupported, malformed, missing, stale, and changed source   | Decoder format checks, malformed WAV rejection, pre/post content identity, cancellation                                         | Analyze one unsupported and one changed fixture; confirm no result is saved                                     |
| Cancellation and bounds                                      | Cancellation checks during decoding and tempo correlation; file/frame limits; no partial save                                   | Cancel a long temporary WAV and confirm progress stops and no BPM result appears                                |
| Persistence and precedence                                   | Schema v1-to-v3 and v2-to-v3 migrations, cached BPM/waveform, correction persistence/clear, range, and effective-deck-BPM tests | Restart SAYT and confirm waveform, tag, analysis, correction, and deck BPM are distinct and restore as intended |

The 0–1 periodicity score and the 60–200 search range are an initial engineering choice, not a validated commercial-music accuracy claim. The 160-bin waveform is a compact display summary, not a lossless audio signal. A representative, licensed test corpus and manual beat references are required before SAYT advertises analysis accuracy beyond deterministic fixtures.
