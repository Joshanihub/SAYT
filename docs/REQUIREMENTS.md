# Milestone 1 requirements

**Status:** PROPOSED acceptance contract<br>
**Owner:** SAYT project owner<br>
**Date:** 2026-11-25<br>
**Version:** 0.1.1

## Functional requirements

1. Launch a desktop application with a two-deck workspace and output-device selection.
2. Load an actual local WAV file into either deck using the native file picker.
3. Play, pause, stop, and seek each deck independently; playback position reflects the engine state.
4. Mix both decks to a selected output device. The crossfader routes A at its left endpoint and B at its right endpoint.
5. Deck gain and low, mid, and high controls change the signal sent to the output.
6. Display duration, sample rate, channel count, and waveform peaks derived from the selected recording.
7. Return a clear error for absent devices, unsupported WAV formats, malformed data, and output stream failures. A device switch still attempts to open the replacement if pausing the current stream errors. If opening the replacement fails, attempt to resume and retain the previous stream; report whether recovery succeeded. A failed switch leaves the prior stream usable when the OS permits recovery.
8. Release the stream and track resources on application shutdown.

## Supported WAV input in the current decoder

- Uncompressed integer PCM, 16-, 24-, or 32-bit.
- Mono or stereo.
- Sample rate 8,000 through 192,000 Hz.
- Maximum file size 512 MiB and maximum decoded length 32,000,000 frames per deck.
- Floating-point WAV, compressed WAV, more than two channels, empty audio, malformed sample data, and larger files are rejected.

The configured maximum is a safety bound, not a promise that every machine can comfortably hold two maximum-size tracks in memory. Two 32-million-frame stereo tracks require about 512 MiB for frame samples alone, before vector capacity and application overhead.

## Verification matrix

| Requirement                           | Automated evidence                                                                                                    | Manual evidence required                                                        |
| ------------------------------------- | --------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------- |
| Decode actual PCM and derive waveform | Decoder unit tests use generated WAV fixtures                                                                         | Load two different user-selected WAV files in the desktop app                   |
| Independent transport and seek        | Engine tests for state, bounds, and queue behavior                                                                    | Confirm A and B play, pause, stop, and seek independently                       |
| Crossfader                            | Endpoint and equal-power math tests; rendered A-only test                                                             | Listen at left, center, and right positions                                     |
| Three-band EQ                         | Frequency-response tests for low, mid, and high bands                                                                 | Sweep each band using test tones or known material and listen for change        |
| Output-device selection               | Compile and command-path checks; transition-policy tests cover pause failure, replacement failure, and resume failure | Confirm output on a physical device; unplug/reselect and observe error recovery |
| Error handling and shutdown           | Decoder error cases and native command tests                                                                          | Unsupported file, no-device, stream failure, and clean shutdown scenarios       |

## Manual hardware checklist

1. Start SAYT on a machine with a physical audio output and select that device.
2. Load distinct PCM WAV files into Deck A and Deck B; verify each waveform, duration, and format metadata.
3. Start both decks. Confirm both are audible simultaneously, with position advancing only while each deck plays.
4. Independently pause, resume, seek, and stop each deck.
5. Change each deck's gain and each EQ band; confirm audible and metered changes.
6. Move the crossfader fully left, center, and fully right; confirm the intended mix.
7. Disconnect or disable the selected output if safe to do so; refresh and reselect an available output. Record the message, recovery action, whether the previous output remains usable after a failed switch, and whether playback resumes after reselection.
8. Close the app while audio is active; confirm the output stops and the process exits without a stuck device.

Record device model, OS, sample rate, WAV properties, observed failures, and test duration with the result. Until this checklist is run, hardware playback is **not verified**.
