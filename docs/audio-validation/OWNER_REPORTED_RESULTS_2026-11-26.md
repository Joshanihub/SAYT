# SAYT M1 owner-reported manual audio results

**Status:** OWNER-REPORTED; details incomplete<br>
**Owner:** SAYT project owner<br>
**Date:** 2026-11-26<br>
**Version:** 0.1.0

## Report source and limits

In response to a request for the live-window M1 checks, the owner reported that all checks passed and asked to continue. This record preserves that report as owner-reported evidence. The response did not include per-check observations, device or OS details, sample rate, viewport, error text, or test duration. The agent did not independently reproduce audio playback or device behavior.

## Reported checks

| Check                                                                | Owner-reported result | Details not supplied                                                                                                                                    |
| -------------------------------------------------------------------- | --------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Load the supplied 440 Hz and 880 Hz WAV fixtures into separate decks | PASS                  | Per-deck filename, duration, sample rate, channels, waveform, and fixture identity were not recorded.                                                   |
| Independently play, pause, seek, and stop both decks                 | PASS                  | Per-deck transport observations and position changes were not recorded.                                                                                 |
| At low volume, check master level, crossfader, gain, and EQ          | PASS                  | Output device, fader positions, control values, and listening observations were not recorded.                                                           |
| Check output-device refresh and retry                                | PASS                  | Device name, selected output, retry outcome, and error text were not recorded. No-device and failed-switch recovery cases were not explicitly reported. |
| Shut down cleanly during playback                                    | PASS                  | Output-stop observation and process exit were not recorded.                                                                                             |

## Visual acceptance

The owner response did not report viewport results. 1440×900, 1280×800, and narrow-size captures and Stitch comparisons remain NOT RUN. On 2026-11-26, two Computer Use screenshot attempts for the selected SAYT window returned unrelated Chrome content; its accessibility tree exposed the SAYT page, but no screenshot was accepted as visual evidence.
