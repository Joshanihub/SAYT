# Audio engine

**Status:** M1 source implementation; physical playback unverified<br>
**Owner:** SAYT project owner<br>
**Date:** 2026-11-25

`crates/sayt-engine` contains the CPAL output path, PCM WAV decoder, two deck transports, DSP, mixing, and engine tests. The Tauri shell owns device selection and control commands; the React UI sends controls and polls snapshots. See [`../ARCHITECTURE.md`](../ARCHITECTURE.md) for state ownership and callback boundaries and [`../REQUIREMENTS.md`](../REQUIREMENTS.md) for acceptance tests.

## Current protections

- Decode selected WAV files before sending `Arc<Track>` handles to the callback.
- Reject floating-point or compressed audio, non-mono/stereo, unsupported bit depths/rates, oversized files, malformed samples, and excess frame counts.
- Send commands through a fixed-capacity SPSC queue; process at most 16 queued commands per callback.
- Return replaced tracks through a bounded queue so their final destruction stays off the callback. A full return queue defers replacement.
- Keep CPAL error reporting callback-safe: the callback sets an atomic error signal; the control side logs and displays it.
- Compute EQ coefficients on the control side and run fixed-size biquad state in the callback.
- Beat Jump, manual Loop and Loop Roll controls share the Rust callback and the existing bounded command queue. Loop Roll advances a stored natural position while wrapping the audible deck position, and restores that natural position on release. Tempo is manually entered per loaded deck and is reset on track replacement.
- Device replacement is attempted even if pausing the current stream fails. If the new stream cannot be opened, the control side attempts to resume and retains the previous stream; resume failure is exposed through the output-error snapshot. Unit tests cover this transition policy, not OS or hardware recovery.

These properties are implemented in source and covered in part by unit tests. No allocator instrumentation or physical-device test has been run, so they are not a real-time safety or dropout guarantee.

## Current gaps

The engine itself has no BPM analyzer. The separate library crate now offers bounded local WAV analysis and corrections, and catalog loading can pass a confident/corrected BPM into the existing deck-local control; the fixed live handoff still needs UI verification. Editable beatgrids, beat quantization, Sync, key lock, headphone cue output, output-device hot-plug recovery, callback-load/xrun telemetry, deterministic offline render, controller input, and a 60-minute soak are not implemented. Device IDs are enumeration indexes and may change when devices appear or disappear. Track samples are held in memory within the limits documented in `REQUIREMENTS.md`.

## Local checks

```powershell
cargo test -p sayt-engine --locked
cargo clippy -p sayt-engine --all-targets --locked -- -D warnings
```

For a full desktop test, use the manual hardware checklist in `REQUIREMENTS.md`. Never infer real output, latency, or dropout performance from these unit tests.
