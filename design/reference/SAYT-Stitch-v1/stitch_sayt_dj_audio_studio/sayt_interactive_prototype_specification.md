# SAYT Interactive Desktop Application Prototype & Engineering Specification
**Target**: Codex Frontend & Rust Audio Engine Engineering Team  
**Architecture**: React + TypeScript + Tauri 2 + Rust Core Audio Engine  
**Visual Tokens**: `#08090B` (Canvas), `#111318` (Primary Chassis), `#191D23` (Secondary Surface), `#242830` (Hardware Wells), `#C7F36B` (Signal Lime / Deck A), `#38BDF8` (Ice Cyan / Deck B).

---

## 1. Prototype Navigation & Workspace State Graph

The interactive prototype binds all 13 approved SAYT desktop workspaces through a persistent application chrome with unified routing and cross-screen state persistence:

```
[ App Shell Header & Global Navigation ]
   ├── SAYT DJ Workspace (Live Performance 2-Deck & Mixer) <───┐
   ├── Music Library Manager (Crates, Smart Filters, Inspector) ─┼── Track Load Bridge (Deck A / B)
   ├── Playlist Bridge (Streaming Sync, Metadata Matcher)       │
   ├── Track Preparation & Analysis (Phrase Map, Hot Cues, Grid)─┘
   ├── FX & Sampler Suite
   │     ├── 07A: Performance Effects Rack (4-Slot Master/Deck Chain)
   │     └── 07B: 16-Pad MPC Performance Sampler & Wave Slicer
   ├── Studio Workspaces (DAW Mode)
   │     ├── 08A: Arrangement Timeline & Automation
   │     ├── 08B: Professional Mixing Console (Channel Strips & Parametric EQ)
   │     ├── 08C: Piano Roll & MIDI Editor (Keybed & Velocity Lanes)
   │     ├── 08D-A: Clip Launcher (Session Matrix & Scene Triggering)
   │     └── 08D-B: Step Sequencer (8-Lane Drum Matrix & Micro-Timing)
   └── Settings & Personalisation
         ├── Audio & Device Settings (Buffer, Sample Rate, Telemetry)
         └── Appearance & Customisation (Themes, Density, Live Preview)
```

---

## 2. Interactive Feature Matrix & State Specifications

| Workspace | Component | Interactive Action | Prototype Simulation Behavior | Native Backend Contract Target |
| :--- | :--- | :--- | :--- | :--- |
| **SAYT DJ** | `Deck A / Deck B` | Click `LOAD TRACK` | Opens Quick Crate modal; selecting track populates artwork, title, BPM (124–128), duration, and renders audio waveform canvas | `tauri::command::load_audio_file` (Rust WAV parser) |
| **SAYT DJ** | `Transport (PLAY/CUE)`| Click `PLAY` / `CUE` | Toggles illuminated lime/cyan active state, advances waveform playhead via `requestAnimationFrame` | Real-time CPAL thread playback trigger |
| **SAYT DJ** | `Hardware Mixer` | Drag Channel Faders / Crossfader / EQ Knobs | Live value readout (`-4.2 dB`, `+1.5 dB`, `KILL`), updates simulated stereo LED ladder VU meters | `audio_engine::set_channel_gain` & 3-band biquad filter |
| **SAYT DJ** | `Performance Pads` | Click `HOT CUE` (1–4) | Jumps playhead to stored cue timestamp (Drop, Vocals, Breakdown) with instant visual flash | Low-latency sample seek position |
| **Library** | `Track Inspector` | Select Track Row | Highlights row, displays artwork, Camelot harmonic wheel (`8B / C-Maj`), and enables `LOAD A` / `LOAD B` | SQLite local track registry |
| **Bridge** | `Match Resolver` | Click `Resolve Match` | Opens manual matching drawer with local SSD candidates; confirming marks status as `100% RECONCILED` | Local ISRC / Audio fingerprint matcher |
| **Preparation** | `Beatgrid Editor` | Click `+1ms` / `-1ms` / `Grid Zoom` | Micro-shifts transient grid lines against frequency-colored waveform | Offline audio transient detection |
| **Audio Settings** | `Sound Check` | Click `Test Audio Output` | Animates simulated 1 kHz stereo sine wave meters; displays explicit prototype disclaimer | CPAL audio test tone buffer playback |
| **Appearance** | `Theme & Ergonomics`| Toggle `OLED Black` / `Density` | Switches interface palette variables; live updates side-by-side Deck preview | Tauri `window.style` & CSS custom props |
| **Sampler** | `16-Pad MPC Array` | Click Pad (01–16) | Toggles Playing state, animates circular progress ring, and updates bottom waveform inspector | Low-latency memory buffer trigger |
| **Studio DAW** | `Transport & Tools` | Click Workspace Tabs | Switches between Arrangement, Console Mixer, Piano Roll, and Clip Launcher seamlessly | Multi-view DAW state container |

---

## 3. Engineering Safety & Telemetry Boundaries
- **Prototype Audio Simulation Disclaimer**: The prototype clearly labels simulated level meters and latency metrics as demonstrations to prevent confusion with real CPAL/ASIO driver telemetry.
- **IPC Safety**: Zero raw audio PCM buffers transported over Webview IPC. Waveform visualizations run purely on metadata peak arrays and local HTML5 Canvas rendering.
