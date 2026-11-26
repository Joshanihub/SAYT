# SAYT product vision

**Status:** APPROVED DIRECTION; implementation remains in progress<br>
**Owner:** SAYT project owner<br>
**Date:** 2026-11-26<br>
**Version:** 1.0.0

## Promise

SAYT is an independent music creation and performance platform. Its desktop product starts with a dependable local DJ workflow and grows into connected library preparation, performance, and studio production. A user must be able to trust that every displayed control, meter, waveform, status, and exported result reflects real application state.

The desktop core remains useful without an account, subscription, cloud connection, or streaming provider. External connections are optional, user-selected, permission-scoped, and implemented only through authorised interfaces.

## Product modes

| Mode         | Promise                                                           | Current boundary                                                                             |
| ------------ | ----------------------------------------------------------------- | -------------------------------------------------------------------------------------------- |
| SAYT DJ      | Organise and perform with local music.                            | M1 proves a two-deck PCM WAV workflow; four-deck and cue routing remain owner decisions.     |
| SAYT Bridge  | Interoperate lawfully with playlists, collections, and libraries. | No provider is selected; local library remains independent of provider availability.         |
| SAYT Studio  | Arrange, sequence, mix, and render real projects.                 | Requires a shared project, clock, persistence, audio graph, MIDI, and render model.          |
| SAYT Stage   | Dedicated live-performance mode.                                  | Behaviour, safety state, and final design require an approved brief.                         |
| SAYT Connect | User-controlled integrations.                                     | Purpose, connection types, permissions, and security architecture require an approved brief. |

Web, mobile, collaboration, cloud, and plugin expansion follow a reliable desktop and audio foundation. They are not prerequisites for local DJ use.

## Users and evidence

The initial target is a DJ learner or working DJ who has local audio files and wants to practise or mix without a streaming account. Creators need a later path to prepare and produce their own work; performers need dependable controls and truthful live feedback. The initial pain and willingness to adopt have not been validated through interviews or observed sessions. This remains a product hypothesis, not demand evidence. See [`PRODUCT_BRIEF.md`](PRODUCT_BRIEF.md).

## Principles

1. Local-first by default; network use is visible and optional.
2. A feature is real only when connected to authoritative domain state and its required device, persistence, or provider.
3. No fake tracks, simulated meters, decorative controls, detached playheads, or success without a completed operation.
4. Keep real-time audio bounded, observable outside the callback, and safe to stop.
5. Preserve approved Stitch composition, information hierarchy, typography, palette, interaction states, and motion, while meeting accessibility and operational needs.
6. Keep each capability reviewable, reversible where practical, and evidenced at the state actually reached.
7. Do not claim licensing, provider rights, performance, or cross-platform hardware support without the corresponding evidence and owner decisions.

## Product boundaries

- The 13 screens in the preserved Stitch v1 archive are approved visual references. Prototype HTML and its simulated data are not production requirements where they conflict with this vision, M1 requirements, or real engine capabilities.
- Screen 09 onboarding is absent from the archive; do not invent its final design.
- Stage and Connect need product and security briefs before implementation.
- Provider metadata access, media import/cache, playback, recording, and export are separate permissions. No private API scraping, credential theft, DRM bypass, or access-control circumvention.
- SAYT's project licence is undecided. The repository does not grant reuse rights until the owner chooses a licence.

## Success and stop conditions

M1 succeeds only when its functional criteria are demonstrated, including real local WAV selection, independent two-deck control, audible mix, gain/EQ/crossfader, honest device failure/retry behaviour, and clean shutdown. See [`REQUIREMENTS.md`](REQUIREMENTS.md) and [`FEATURE_ACCEPTANCE_MATRIX.md`](FEATURE_ACCEPTANCE_MATRIX.md).

The overall product is complete only when every accepted capability and later-approved design is implemented, tested, accessible, documented, reviewed, and released with evidence. Keep the product in progress while any required acceptance row is blocked, failed, or unrun. Revisit the audio foundation if physical tests expose starvation, dropouts, unsafe recovery, or stuck output; revisit the product hypothesis if users do not confirm the local workflow is useful.

## Authority

This summary is governed by the owner-supplied SAYT Full Product Implementation Goal v1.1.0, accepted ADRs, current requirements, and the versioned Stitch archive. The detailed acceptance source is [`FEATURE_ACCEPTANCE_MATRIX.md`](FEATURE_ACCEPTANCE_MATRIX.md); milestone ordering is in [`MASTER_IMPLEMENTATION_ROADMAP.md`](MASTER_IMPLEMENTATION_ROADMAP.md). Future approved designs are versioned and reconciled screen by screen.
