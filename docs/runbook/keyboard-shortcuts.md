# Keyboard shortcuts runbook

**Status:** ACTIVE operator guide<br>
**Owner:** SAYT project owner<br>
**Date:** 2026-11-27<br>
**Version:** 1.1

## Open and use the shortcut dialog

- Select **Keyboard shortcuts** in the status bar or press **F1**.
- Search by command name, description, category or focused-deck context.
- Select **Change**, then press one non-modifier key with any desired modifiers. Press **Escape** to cancel capture.
- Duplicate, operating-system, WebView and dialog-navigation combinations are rejected with an explanation.
- Select **Reset defaults** to persist the built-in mappings. The dialog shows a storage warning if settings cannot be read or saved.
- Shortcuts are saved in SAYT's local WebView profile under a versioned key. They are not synced to another machine or operating-system account.

## Built-in mappings

The defaults are SAYT-authored. On Windows and Linux, **Primary** means **Ctrl**; on macOS it means **Command**. Alt is shown as **Option** on macOS.

| Action                                  | Default                                                       |
| --------------------------------------- | ------------------------------------------------------------- |
| Focus Deck A / Deck B                   | Alt+1 / Alt+2                                                 |
| Play/pause focused deck                 | Space                                                         |
| Stop focused deck                       | Shift+Space                                                   |
| Load WAV into focused deck              | Alt+Shift+L                                                   |
| Seek backward / forward by five seconds | Alt+Left / Alt+Right                                          |
| Reduce / raise focused deck gain        | Alt+G / Alt+Shift+G                                           |
| Reduce / raise focused deck low EQ      | Alt+Q / Alt+Shift+Q                                           |
| Reduce / raise focused deck mid EQ      | Alt+W / Alt+Shift+W                                           |
| Reduce / raise focused deck high EQ     | Alt+E / Alt+Shift+E                                           |
| Move crossfader toward A / B / center   | Primary+Shift+Left / Primary+Shift+Right / Primary+Shift+Down |
| Lower / raise master output             | Primary+Alt+Down / Primary+Alt+Up                             |
| Open DJ / Library / Settings workspace  | Primary+Alt+1 / Primary+Alt+2 / Primary+Alt+3                 |
| Retry / refresh audio outputs           | Primary+Alt+R / Primary+Alt+Shift+R                           |
| Open shortcut dialog                    | F1                                                            |
| Set Loop In / Loop Out on focused deck  | Alt+I / Alt+O                                                 |
| Release focused deck loop               | Alt+Shift+O                                                   |

## Limits and troubleshooting

- Focus a deck by clicking inside it, moving keyboard focus into one of its controls, using Alt+1 or Alt+2, or loading into that deck. The active deck receives focused-deck shortcuts.
- Play, pause, stop and seek require a loaded WAV. SAYT reports an actionable error if a transport shortcut targets an empty deck.
- Text fields, select menus, content-editable regions and range controls retain normal input behavior. A focused button or link retains native Space activation.
- If stored mappings are corrupt or from an unsupported version, SAYT falls back to built-in mappings and displays a warning. Open the dialog and reset defaults if needed.
- If a mapping will not save, check that local app storage is available. The current mapping remains active only after a successful save.
- Loop shortcuts operate the focused deck. Loop In marks its current engine position; Loop Out must follow Loop In. Release exits at the next boundary while playing and immediately while paused. The Loop mode also exposes 250 ms, 500 ms, 1 s, 2 s, 4 s and 8 s pads, which start at the current playhead. These are seconds, not beat-synced sizes, and loop regions are not persisted.
- Manual BPM, signed Beat Jump, and held Loop Roll are available in each deck's performance controls. The Library can supply a saved correction or confident local BPM estimate when loading a catalog track; BPM remains deck-local session state and clears when the track changes. No beatgrid or quantization is provided. Beat Jump and Loop Roll currently use on-screen controls and do not have shortcut bindings.
- Shortcuts do not provide system-wide hotkeys, pitch/sync, effects, sampler, recording, library search or controller input. Those actions are not implemented in this slice.

## Owner verification on Windows

1. From `C:\Dev\SAYT-dj-feature-expansion`, use a free port. If the M1 app is already using 1420, run this PowerShell setup to launch the feature worktree on 1421:

   ```powershell
   $configPath = Join-Path $env:TEMP 'sayt-tauri-port-1421.json'
   $configText = '{"build":{"devUrl":"http://127.0.0.1:1421","beforeDevCommand":"corepack pnpm --filter @sayt/desktop exec vite --host 127.0.0.1 --port 1421 --strictPort"}}'
   [System.IO.File]::WriteAllText($configPath, $configText)
   corepack pnpm --filter @sayt/desktop exec tauri dev --config $configPath
   ```

   The config file only overrides the development URL and Vite port; it does not change tracked project configuration.

2. Choose an available output in SAYT. Load `%TEMP%\SAYT-Audio-Fixtures\deck_a_440hz.wav` and `deck_b_880hz.wav` into separate decks through their native file pickers.
3. Use Alt+1 / Alt+2, Space, Shift+Space, Alt+Left / Alt+Right, gain and EQ mappings. Observe the selected-deck outline, transport labels/position, engine meter and error banner.
4. Use the crossfader and master shortcuts at low volume, then compare the visible control values with their range controls.
5. Open F1, search for “crossfader,” remap a shortcut, close and reopen the dialog, and confirm the displayed mapping. Restart SAYT and confirm the remapping persists. Reset defaults and confirm the displayed mapping returns.
6. Focus the search box and range controls and type/use arrows. Verify the keyboard handler does not replace normal input. Use Tab/Shift+Tab/Escape in the shortcut dialog.
7. In the Loop mode, use a size pad on one deck, confirm that deck's position wraps and the other deck continues independently; test Loop In/Out, release during playback, release while paused, seek outside, stop, and replacing the loaded track. Note whether the audible seam clicks. Repeat with the other deck.
8. Close SAYT while playback is active and record whether the engine output stops and the process exits. Record device, viewport, OS, fixture, error text, and each result.

This procedure can verify Windows command routing and visible engine state. It cannot establish macOS behavior, physical latency/dropouts, or audible quality without the relevant OS/device and human listening.

The current tool-driven runtime session opened and searched the help dialog, but injected global key presses did not produce observable command changes. Treat real keyboard actuation as unverified until a person records the physical-key checks above.
