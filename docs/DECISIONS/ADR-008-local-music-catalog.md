# ADR-008: Store the local music catalog in app-local SQLite

**Status:** PROPOSED for the local music library delivery<br>
**Owner:** SAYT project owner<br>
**Date:** 2026-11-27<br>
**Version:** 1.0

## Decision

Use an app-local SQLite catalog for selected folder roots, content-identified tracks, locations, and user-created crates/playlists.

## Context

Screen 02 needs durable search, sorting, filters, duplicate locations, missing-file reconciliation, collection order, user metadata, and backup. The audio remains in the user's chosen folders. Catalog queries and writes must stay outside the CPAL callback and avoid sample data in Tauri IPC. The project license is still undecided; third-party dependency licensing is tracked separately.

## Options

- **App-local SQLite:** transactions, indexed/paged queries, schema versions, and online backup; adds a direct native dependency and a catalog migration responsibility.
- **One JSON document:** easy to inspect and dependency-light; whole-file rewrites and concurrent updates become fragile as collections grow.
- **Sidecar database beside music:** portable with the folder; writes into user media directories and complicates multi-root collections and permission expectations.

## Choice and consequences

Choose `rusqlite` with bundled SQLite, WAL journaling, foreign keys, busy timeout, and an app-local database. Use SHA-256 of full file contents for track identity and separate location rows for duplicates/moves. Scan writes are per-file transactions; incomplete scans keep existing missing-state information. Schema version 1 is migrated transactionally. A failed integrity check preserves the damaged database and sidecars under a timestamped quarantine name before creating a new empty catalog. Backup uses SQLite's online backup API and a native save dialog.

The exact direct pins and third-party license strings are recorded in [`docs/licensing/third-party.md`](../licensing/third-party.md). SAYT's project license remains undecided. SQLite data contains local paths and user metadata, so it stays in app-local storage and is not uploaded.

## Verification and limits

Automated tests cover real-file indexing, unmodified source bytes, content duplicates, renamed-file recovery, cancellation, missing-file reconciliation, malformed/unsupported files, paging/search/filter/sort, collection persistence/reordering, corrupt-database preservation, and independent backup integrity. The selected real music folder, native picker, Windows app restart, and deck loading still require runtime/owner evidence. No physical playback or decoder format support is inferred from metadata parsing.

## Reversibility and review trigger

The store is local and can be exported with SQLite backup. Revisit this decision before cross-device sync, cloud accounts, multi-user catalogs, or track-analysis datasets; those add conflict, privacy, and migration requirements. Do not move catalog work into the real-time callback.
