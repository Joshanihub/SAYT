# SAYT project status

**Status:** IN PROGRESS<br>
**Owner:** SAYT project owner<br>
**Date:** 2026-11-26<br>
**Version:** 0.2.0

## Current milestone

**M1 — desktop foundation and two-deck audio.** The Rust engine and the existing Tauri control API are retained. This change implements the Stitch-informed desktop shell, truthful DJ workspace, output settings, and library empty state. It does not close M1.

## Verified baseline before this change

- Started from `main` at `1d0323799d0a833a40b20cd58b1c79e43a11044b`; the worktree was clean.
- Created `feat/stitch-m1-dj-workspace`.
- CI setup fix is separately committed as `49676c2` (`fix(ci): remove premature pnpm cache setup`) and pushed to the feature branch.
- GitHub Actions run [45702184357](https://github.com/Joshanihub/SAYT/actions/runs/45702184357) completed with three failures after Node/pnpm initialization. It exposed Windows Prettier line-ending failures, missing Ubuntu ALSA headers, and a macOS CoreAudio `Send` compile failure. Follow-up fixes are in the current working tree; their remote run is pending.

## Design reference

The original Stitch export and extracted references are in `design/reference/`. SHA-256: `E97F43E20EDE4BC3993CFB8E4177BB53C3AF6296C58AEE11E0E1DAA347A4B2F1`. See `docs/DESIGN_IMPLEMENTATION_MAP.md` for per-screen states.

## M1 implementation state

- Existing Rust decoder, CPAL output, two-deck transport, gain/EQ, crossfader, master gain/meter, native asynchronous WAV picker, and output-device commands: retained in source; physical behavior remains unverified here. Tauri commands reach the engine through a bounded control-thread actor to satisfy CoreAudio's non-`Send` stream ownership.
- Shared shell and DJ / Library / Settings navigation: implemented in source; 20 frontend component/callback tests passed locally under Node.js 24.22.3 and pnpm 12.23.0.
- Decks, waveform, transport, seek, and controls: connected to existing Tauri commands; actual source WAV and physical playback checks remain required.
- Mixer: actual master meter, master gain, and crossfader only; no invented independent channel meters.
- Quick crate: two deck slots using the native picker; no directory scan or persistent library.
- Audio settings: device list/selection, refresh, and retry only; unsupported prototype controls are identified as unavailable.
- Library: explicit empty state because M1 has no index storage.
- Future screens: disabled navigation; not implemented.

## Outstanding exit gates

1. Local Rust and frontend format, lint, typecheck, unit, and build checks passed; exact commands, runtime pins, and test limits are recorded in `docs/M1_REPORT.md`.
2. Push the follow-up CI corrections and verify all three OS jobs without weakening audit policy.
3. Run an owner-approved file-picker and loaded-deck desktop smoke check; inspect 1440×900, 1280×800, and a narrow viewport when the app is not being used interactively.
4. Run the physical-device checklist in `docs/REQUIREMENTS.md`: dual playback, independent controls, gain/EQ/crossfader audibility, device recovery, and shutdown. No such evidence is recorded yet.
5. Before a consequential release decision, assess reachability/impact of the five unmaintained Rust crates and `glib` unsoundness notice, review the two BlueOak npm license terms, and complete an SBOM/legal review. Fresh local scan results are recorded in `docs/DEPENDENCIES_AND_LICENSES.md`.
6. SAYT's project license remains undecided; no license grant is added.

## Next milestone

Do not start M2 reliability implementation until M1 exit requirements have been reviewed and approved. After M1, the roadmap prioritizes callback load/dropout measurement, device loss recovery, and shutdown verification.
