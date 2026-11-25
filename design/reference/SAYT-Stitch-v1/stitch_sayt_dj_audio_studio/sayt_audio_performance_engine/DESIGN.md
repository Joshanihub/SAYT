---
name: SAYT Audio Performance Engine
colors:
  surface: '#111318'
  surface-dim: '#111318'
  surface-bright: '#37393f'
  surface-container-lowest: '#0c0e13'
  surface-container-low: '#1a1b21'
  surface-container: '#1e2025'
  surface-container-high: '#282a2f'
  surface-container-highest: '#33353a'
  on-surface: '#e2e2e9'
  on-surface-variant: '#c4c9b3'
  inverse-surface: '#e2e2e9'
  inverse-on-surface: '#2e3036'
  outline: '#8e937f'
  outline-variant: '#444938'
  surface-tint: '#abd551'
  primary: '#ffffff'
  on-primary: '#253600'
  primary-container: '#c6f26a'
  on-primary-container: '#506e00'
  inverse-primary: '#4b6700'
  secondary: '#7bd0ff'
  on-secondary: '#00354a'
  secondary-container: '#00a6e0'
  on-secondary-container: '#00374d'
  tertiary: '#ffffff'
  on-tertiary: '#67001b'
  tertiary-container: '#ffdadb'
  on-tertiary-container: '#c51640'
  error: '#ffb4ab'
  on-error: '#690005'
  error-container: '#93000a'
  on-error-container: '#ffdad6'
  primary-fixed: '#c6f26a'
  primary-fixed-dim: '#abd551'
  on-primary-fixed: '#141f00'
  on-primary-fixed-variant: '#374e00'
  secondary-fixed: '#c4e7ff'
  secondary-fixed-dim: '#7bd0ff'
  on-secondary-fixed: '#001e2c'
  on-secondary-fixed-variant: '#004c69'
  tertiary-fixed: '#ffdadb'
  tertiary-fixed-dim: '#ffb2b7'
  on-tertiary-fixed: '#40000d'
  on-tertiary-fixed-variant: '#92002a'
  background: '#111318'
  on-background: '#e2e2e9'
  surface-variant: '#33353a'
typography:
  headline-xl:
    fontFamily: Space Grotesk
    fontSize: 28px
    fontWeight: '700'
    lineHeight: 34px
    letterSpacing: -0.02em
  headline-lg:
    fontFamily: Space Grotesk
    fontSize: 22px
    fontWeight: '600'
    lineHeight: 28px
    letterSpacing: -0.01em
  headline-md:
    fontFamily: Space Grotesk
    fontSize: 16px
    fontWeight: '600'
    lineHeight: 22px
  body-lg:
    fontFamily: Inter
    fontSize: 14px
    fontWeight: '400'
    lineHeight: 20px
  body-md:
    fontFamily: Inter
    fontSize: 12px
    fontWeight: '400'
    lineHeight: 16px
  body-sm:
    fontFamily: Inter
    fontSize: 11px
    fontWeight: '400'
    lineHeight: 14px
  label-telemetry-lg:
    fontFamily: JetBrains Mono
    fontSize: 20px
    fontWeight: '700'
    lineHeight: 24px
    letterSpacing: -0.03em
  label-telemetry-md:
    fontFamily: JetBrains Mono
    fontSize: 13px
    fontWeight: '600'
    lineHeight: 16px
    letterSpacing: -0.01em
  label-telemetry-sm:
    fontFamily: JetBrains Mono
    fontSize: 10px
    fontWeight: '500'
    lineHeight: 12px
    letterSpacing: 0.02em
  label-hardware:
    fontFamily: Space Grotesk
    fontSize: 9px
    fontWeight: '700'
    lineHeight: 10px
    letterSpacing: 0.08em
rounded:
  sm: 0.125rem
  DEFAULT: 0.25rem
  md: 0.375rem
  lg: 0.5rem
  xl: 0.75rem
  full: 9999px
spacing:
  gutter: 0.5rem
  gutter-compact: 0.25rem
  margin: 0.75rem
  margin-mobile: 0.5rem
  space-xs: 0.25rem
  space-sm: 0.5rem
  space-md: 0.75rem
  space-lg: 1rem
  space-xl: 1.5rem
---

## Brand & Style
The design system powers an open-source, studio-grade electronic music production, live performance, and DJ interface. It bridges the tangible tactile precision of flagship physical hardware (tactile encoders, multi-state silicone pads, aluminum pitch faders) with high-frame-rate digital ergonomics (smooth 60fps vector multi-band waveforms, telemetry heads-up displays, instantaneous reactive state indicators).

### Aesthetic Directives
- **Precision Dark-Mode Dominance**: The primary visual state is engineered for low-light club environments and high-intensity performance stages. Low visual fatigue, high luminescent contrast on focal elements, zero extraneous visual noise.
- **Hardware Skeuomorphism meets Clean Computational Tech**: Tactile toggles, segmented LED peak meters, rotary dials, and razor-sharp structural grooves combined with modern border-radius discipline, crisp hairline dividers, and fluid micro-states.
- **High Data Density & Instantaneous Visual Parsing**: Essential performance telemetry (BPM to two decimals, camelot/musical key, transient phase alignment, millisecond quantize buffers) must be parsable in peripheral vision under intense lighting shifts.

## Colors
The palette balances an ultra-deep technical canvas with laser-focused optical signals. Contrast meets rigorous WCAG AA standards while ensuring vibrant phosphor glows on key audio actions.

### Functional Palette Mapping
- **Canvas Base (`#08090B`)**: Deepest substrate layer used for window framing, global workspace margins, and outer gutters.
- **Primary Surface (`#111318`)**: Base rack container, deck body, and library tree.
- **Secondary Surface (`#191D23`)**: Channel strips, sampler slots, waveform background tracks, and inspector panels.
- **Elevated Surface (`#242830`)**: Popovers, active deck bays, rotary knob caps, hover states, and transport button housings.
- **Primary Signal Lime (`#C7F36B`)**: Transport Master (Play/Pause), Deck A identity, active sync locks, beatgrid anchors, loop active triggers.
- **Electric Cyan (`#38BDF8`)**: Deck B identity, secondary audio streams, mid/high band waveform rendering, modulation matrix routing.
- **Hot Coral / Warning Amber (`#F43F5E` / `#FB923C`)**: Hot cues 1-4, clip clipping / master limiter engagement, track over-gain, solo/mute safety states.
- **Typographic Hierarchy**:
  - Primary Text: `#F5F6F2` (100% white-lime tint for extreme clarity).
  - Secondary Text: `#A7ADB5` (BPM labels, channel metadata, track artist).
  - Muted/Inactive: `#747C86` (Unassigned slots, inactive beat markers, unselected tabs).

## Typography
Typographic rules follow strict functional roles to separate music identification, interactive control, and timecode telemetry:

1. **Display & Hardware Titles (`Space Grotesk`)**: Geometric, wide stance, engineered for track titles, plugin headers, and deck indicators.
2. **Body & Navigation (`Inter`)**: High legibility in compact table rows, file trees, sample names, and audio engine preferences.
3. **Data & Telemetry (`JetBrains Mono`)**: Always active with tabular figures (`tnum`, `zero`) for dynamic countervalues: BPM counters, remaining millisecond timecode, dBFS meters, and musical pitch semitone shifts. Prevents visual jitter during realtime playback.

## Layout & Spacing
Designed for extreme density and rapid physical-mouse/trackpad targeting. The baseline grid operates on strict 4px increments.

### Master Layout Structure (Desktop-First: 1440x900 Baseline)
- **Top Global Transport Bar**: Fixed 48px height. Hosts master audio clock, CPU performance meters, master limiter out, master recording indicators, and audio settings.
- **Waveform HUD Canvas**: Dynamic 180px–240px vertical split view. Features stacked multi-layer spectral frequencies (Low/Mid/High) with phase alignment grids.
- **Performance Bay**: 2 or 4 deck configurations arranged symmetrically or stacked horizontally. Deck A anchors to the left, Deck B to the right, mixer console aligned centrally.
- **Lower Workspace (Crate & Browser / FX Matrix)**: Docked collapsible bottom tier (300px default height) with high-density tabular view, fast search bar, and sample preview scrubber.
- **Responsive Adaptations**:
  - Below 1280px width, channel mixer strips collapse rotary EQ columns into horizontal micro-sliders.
  - Below 1024px, the crate browser auto-minimizes to an overlay drawer to preserve 100% vertical resolution for deck waveforms.

## Elevation & Depth
Elevation in this system is communicated via recessed chassis grooves, inner-bevel shadows, hairline glass separators, and luminous active states rather than floaty ambient drop shadows.

### Depth Hierarchy
1. **Level 0 (Recessed Slots & Fader Tracks)**:
   - Background: `#08090B`
   - Border: Inset 1px `rgba(0, 0, 0, 0.6)`
   - Inner Shadow: `inset 0 1px 3px rgba(0, 0, 0, 0.7)`
2. **Level 1 (Surface Canvas & Channel Strips)**:
   - Background: `#111318`
   - Border: 1px solid `rgba(255, 255, 255, 0.06)`
3. **Level 2 (Active Decks, Modules, Hardware Panels)**:
   - Background: `#191D23`
   - Border: 1px solid `rgba(255, 255, 255, 0.1)`
   - Highlight: Top edge hairline `1px solid rgba(255, 255, 255, 0.05)`
4. **Level 3 (Tactile Caps, Floating Tooltips, Modals, Menus)**:
   - Background: `#242830` with `backdrop-filter: blur(16px)`
   - Border: 1px solid `rgba(255, 255, 255, 0.16)`
   - Shadow: `0 8px 24px rgba(0, 0, 0, 0.6), 0 2px 6px rgba(0, 0, 0, 0.4)`
5. **Interactive Glows**: Active Cue, Sync, and Loop buttons radiate a directional 6px to 10px luminescence using their respective accent colors at 30% alpha (e.g., `0 0 12px rgba(199, 243, 107, 0.35)`).

## Shapes
To mirror industrial CNC-machined DJ gear and specialized audio workstations, shapes adhere strictly to compact, low-radius curvature.

- **Standard Containers & Modules**: `0.25rem` (4px). Provides structure without wasting canvas real estate.
- **Interactive Buttons & Fader Caps**: `0.25rem` (4px) with subtle 1px chamfers on inner active indicators.
- **Performance Pads**: `0.25rem` (4px) corners with a 1px perimeter inset channel.
- **Rotary Knobs**: 100% circular geometry with high-contrast indicator ticks, surrounded by 270° SVG indicator arcs.
- **Telemetry Tags & Key Badges**: `0.125rem` (2px) micro-radii to optimize interior text clearance.

## Components

### 1. Transport & Silicone Performance Pads
- **Play/Pause Button**: 44x36px minimum tap target. Rest state: `#191D23` with `#F5F6F2` play icon. Active state: `#C7F36B` base with `#08090B` icon and intense `#C7F36B` edge glow.
- **Cue Button**: Amber `#FB923C` reactive glow on hold; returns to muted outline when idle.
- **Hot Cue Grid**: 8-pad bank (4x2 or 8x1). Each pad features unassigned state (`#191D23` with `#747C86` index label), assigned state (dim accent tint), and active hit state (100% luminance color flash with 50ms decay).

### 2. Rotary Encoders & Knobs
- **Construction**: Concentric outer track with central circular dial (`#242830`).
- **Indicator**: Radial sweep stroke (2.5px width) rendered in `#C7F36B` (EQ Low/Mid/High) or `#38BDF8` (Filter/FX Send).
- **Center Detent**: Explicit 1px tick mark at 12 o'clock for EQ zero position (`0.0 dB`).
- **Micro-Value Readout**: Displays inline tooltip or sub-label in `JetBrains Mono` only on hover/scrub.

### 3. Faders (Channel Volume & Crossfader)
- **Track**: 6px inset groove, filled with dark track chassis `#08090B`.
- **Thumb/Cap**: High-profile 20x12px pill-chamfer cap (`#242830`) featuring a central white 1px tactile indicator line.
- **Crossfader Curve Control**: Smooth tension slider with switchable cut/smooth/sharp algorithmic curves.

### 4. Audio Waveform Scrubber & Beatgrid
- **Overview Strip**: 32px height, full-track cached amplitude profile showing frequency-based coloring (Red = Low/Bass, Green = Mid/Vocals, Blue/Cyan = High/Hi-hats).
- **Scrolling Waveform**: Dynamic 60fps WebGL/Canvas layer. Solid beat markers at 1-beat intervals, accentuated lines with bar count on beat 1. Playhead anchored at center with high-contrast `#F5F6F2` needle.

### 5. Input Fields & Search Bars
- **Chassis**: `#111318` background, 1px border `#242830`.
- **Focus**: Transitions to 1px border `#C7F36B` with zero ambient ring blowout.
- **Right Action**: Integrated clear button and shortcut chip (e.g., `⌘F` in `JetBrains Mono`).

### 6. Crate / Track Library Tables
- **Row Density**: Compact 28px height. Alternating rows with subtle `rgba(255, 255, 255, 0.015)` zebra stripes.
- **Columns**: Title, Artist, BPM (`tnum`), Key (color-coded harmonic Camelot wheel tags), Time, Rating, Bitrate.
- **Loaded State**: Highlighted row border in `#C7F36B` for Deck A, `#38BDF8` for Deck B.