# SAYT design system

**Status:** PROPOSED implementation guide reconciled to Stitch v1<br>
**Owner:** SAYT project owner<br>
**Date:** 2026-11-26<br>
**Version:** 0.1.2

## Authority and preservation

The immutable design reference is [`design/reference/exports/SAYT-Stitch-v1.zip`](../design/reference/exports/SAYT-Stitch-v1.zip), SHA-256 `E97F43E20EDE4BC3993CFB8E4177BB53C3AF6296C58AEE11E0E1DAA347A4B2F1`. Its shared specification is `design/reference/SAYT-Stitch-v1/stitch_sayt_dj_audio_studio/sayt_audio_performance_engine/DESIGN.md`; its 13 screen references are indexed in [`design/reference/README.md`](../design/reference/README.md). Preserve reference images at their native proportions. Generated `code.html` is a reference, not production implementation. Never copy its fictitious telemetry or track data.

## Visual direction and tokens

Precision dark-mode music hardware: dense and tactile, readable in low light, with high-contrast active signals, hairline dividers, recessed controls, and restrained glow. Keep visual telemetry subordinate to real engine state.

| Semantic role            | Canonical value | Current CSS token / use                             |
| ------------------------ | --------------- | --------------------------------------------------- |
| Canvas                   | `#08090B`       | `--canvas`; window/workspace surround               |
| Surface                  | `#111318`       | `--surface`; base containers                        |
| Secondary surface        | `#191D23`       | `--surface-2`; strips and panels                    |
| Recessed / elevated well | `#242830`       | `--well`; controls and wells                        |
| Main text                | `#F5F6F2`       | `--text`                                            |
| Secondary text           | `#A7ADB5`       | `--muted`                                           |
| Subtle / inactive text   | `#747C86`       | `--subtle`                                          |
| Primary lime             | `#C7F36B`       | `--lime`; active transport and Deck A identity      |
| Electric cyan            | `#38BDF8`       | `--cyan`; Deck B identity and secondary signal      |
| Coral                    | `#F43F5E`       | `--coral`; warning/hot-cue semantics where approved |
| Amber                    | `#FB923C`       | `--amber`; warning/cue semantics where approved     |

Current CSS already uses the Stitch coral and amber values. No `#EF4444` or `#F59E0B` competing tokens were found in application/docs. Keep Deck A lime and Deck B cyan identity stable. The source specification states WCAG AA intent; actual text/control contrast still needs measurement by component and state before accessibility acceptance.

## Typography

The Stitch roles are Space Grotesk for display/hardware labels, Inter for body/navigation, and JetBrains Mono for telemetry with tabular figures. The owner's master implementation instruction approves implementing these roles and the supplied archive's Material Symbols Outlined icon style. Bundle locally versioned WOFF2 subsets with their upstream notices, include the scripts represented by each font's official CSS subsets, and retain system fallbacks for unsupported glyphs. Provenance and icon mapping are recorded in [`DEPENDENCIES_AND_LICENSES.md`](DEPENDENCIES_AND_LICENSES.md) and [`DECISION_REGISTER.md`](DECISION_REGISTER.md). Do not claim visual parity before runtime screenshot verification.

## Layout, spacing, shape, elevation

- Base spacing grid: 4 px; compact margins and gutters should use multiples where the reference permits.
- Reference desktop comparison baseline: 1440×900. DJ composition has two balanced deck areas, central mixer, waveform/transport hierarchy, and lower crate/browser; simplify only when the feature is unavailable, and label that state truthfully.
- Standard module/button radius: 4 px; telemetry tags: 2 px; rotary controls remain circular.
- Elevation uses recessed grooves, fine borders, active edge highlights, and restrained shadow. Avoid generic floating-card treatment.
- Reference active control minimum is 44×36 px; verify pointer, keyboard and touch targets in the actual application before marking accessible.

## Motion and interaction

Reference motion is short and purposeful: active controls gain a visible state, hot-cue flashes decay quickly, and waveform motion follows real playback. The shared Stitch specification describes 60 fps waveform rendering as a target; current evidence does not measure it. All buttons and selects, including navigation, transport, and load controls, use a named 120 ms ease-out transition for color, background, border, and filter changes. `prefers-reduced-motion: reduce` sets CSS animation and transition durations to 0 ms and disables smooth scrolling; keyboard focus outlines remain immediate. The master-meter width still follows the engine snapshot, with its smoothing removed for reduced-motion users. A source regression test covers these stylesheet rules; runtime behavior under the operating-system preference and waveform frame rate remain unverified. Never animate a playhead, level, latency or device state independently of engine data. Loading, error, empty, and success states still need component-level acceptance.

## Responsive rules and open comparison

The shared Stitch spec describes mixer adaptation below 1280 px and crate minimisation below 1024 px. Current `apps/desktop/src/styles.css` uses media queries at 1279, 1060, 720, and 480 px. This difference is recorded; do not silently change it. A Tauri render at 1280×800 is recorded, but it has not been compared with its reference; 1440×900 and narrow screenshots remain NOT RUN. Compare every captured target view against Stitch before settling the breakpoints.

## Screen-level fidelity

Per-screen references, production components, backend contracts, and limitations are mapped in [`DESIGN_IMPLEMENTATION_MAP.md`](DESIGN_IMPLEMENTATION_MAP.md); acceptance status is in [`FEATURE_ACCEPTANCE_MATRIX.md`](FEATURE_ACCEPTANCE_MATRIX.md). A screen is not visually accepted until its running Tauri view is compared at a matching viewport and all available states are exercised. New approved Stitch archives are versioned and compared screen by screen without replacing earlier references.
