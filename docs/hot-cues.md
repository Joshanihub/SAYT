# Hot Cues

**Status:** Implemented; automated verified; Windows runtime verified for fixture loading, paused cue triggers, deck isolation, and restart restoration<br>
**Owner:** SAYT project owner<br>
**Date:** 2026-11-27<br>
**Version:** 1.0

## User behavior

Each loaded deck shows eight per-track pads. An empty pad stores the deck's current engine position. An assigned pad seeks to its cue immediately, without changing whether the deck is playing or paused. Reassign records the current position again; Clear removes that slot. Assigned cues have a color and editable name. The active highlight identifies the last triggered cue and clears on manual seek, stop, cue removal, or track replacement.

Keyboard set/trigger/clear commands use the same typed command factory and dispatcher as the pads. Shortcuts target the focused deck and its current source identity. Existing editable-field and focused-native-control exclusions still apply. The native actor rejects a command whose source identity no longer matches the deck.

## Track identity and local storage

See [ADR-005](DECISIONS/ADR-005-hot-cue-identity-and-persistence.md) for the decision and trade-offs.

The decoder computes `sha256:<hex>` from a versioned identity domain marker, sample rate, channel count, bit depth, and decoded integer PCM samples. It excludes path, filename, and tags. Identical decoded audio can share cues between either deck and after rename/restart. Changed audio has a different key even when the filename is unchanged. A different PCM encoding or format is treated as a different source even if it sounds similar.

Cue JSON is stored beneath Tauri's application-local data directory in `hot-cues-v1`, one file per validated SHA-256 identity. The version-1 schema stores the source identity and eight optional cues. It is capped at 8 KiB per document and 10,000 track files, with three retained backups per track. Unsupported or damaged files produce an actionable warning, are not silently associated, and can recover from a valid backup. Data is local; no credentials, audio, or network access are involved.

## Verification record

Automated status and Windows runtime evidence are recorded in `FEATURE_ACCEPTANCE_MATRIX.md` and `VERIFICATION_EVIDENCE.md`. The run used generated fixtures at 1282×832, saved cues on both decks, restarted SAYT, reloaded both tracks, and observed paused cue triggers without moving the other deck. Cue-trigger behavior while playing, edit controls in the live owner workflow, physical audibility, sample-accurate timing, and long-duration performance remain open.

## Owner testing

1. Open the SAYT development desktop app and set Master to a safe level before starting playback.
2. Load a WAV into either deck using the native picker. Set a pad at the current position; seek away, then click the assigned pad or use the shortcut shown on it.
3. Set another cue on the other deck and confirm its seek does not move the first deck.
4. Rename a cue, choose its color, reassign it, and clear it from the selected-cue editor.
5. Close SAYT normally, reopen it, and load the same decoded PCM WAV. Confirm the cue name, color, and position return.
6. Load different audio under the same filename and confirm it starts with empty cues. Re-encoded audio with different format metadata is also a separate identity.

Increase output level only after confirming the operating-system output is safe. Playback/audibility and physical device verification remain owner checks.
