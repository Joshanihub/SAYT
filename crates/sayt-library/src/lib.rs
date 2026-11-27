//! Durable local music catalog, bounded metadata scanning, and user collections.
//!
//! The catalog stores references to source files. It never edits, moves, or copies
//! audio. Track identity is a SHA-256 digest of the complete file bytes.

use std::{
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
    time::{SystemTime, UNIX_EPOCH},
};

mod bpm_analysis;

use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use hound::{SampleFormat, WavReader};
use lofty::{
    config::ParseOptions,
    file::{AudioFile, TaggedFileExt},
    prelude::ItemKey,
    probe::Probe,
};
use rusqlite::{backup::Backup, params, types::Value, Connection, OptionalExtension, Transaction};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

const SCHEMA_VERSION: i32 = 2;
const SEARCH_PAGE_SIZE: u32 = 100;
const MAX_PAGE_SIZE: u32 = 200;
const MAX_SEARCH_CHARS: usize = 256;
const MAX_COLLECTION_NAME_CHARS: usize = 64;
const HASH_BUFFER_BYTES: usize = 64 * 1024;
const MIN_PCM_SAMPLE_RATE_HZ: u32 = 8_000;
const MAX_PCM_SAMPLE_RATE_HZ: u32 = 192_000;
const MAX_PCM_FILE_BYTES: u64 = 512 * 1024 * 1024;
const MAX_PCM_FRAMES: u64 = 32_000_000;
const MAX_ARTWORK_SOURCE_BYTES: u64 = 128 * 1024 * 1024;
const MAX_ARTWORK_BYTES: usize = 3 * 1024 * 1024;
const MIN_BPM_CORRECTION: f64 = 30.0;
const MAX_BPM_CORRECTION: f64 = 300.0;
const SUPPORTED_AUDIO_EXTENSIONS: &[&str] = &[
    "aac", "aif", "aiff", "ape", "flac", "m4a", "m4b", "mp1", "mp2", "mp3", "mp4", "mpc", "ogg",
    "opus", "spx", "wav", "wv",
];

static NEXT_COLLECTION_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TrackFilter {
    #[default]
    All,
    Recent,
    Missing,
    Unanalyzed,
    Favorites,
    Unsupported,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackQuery {
    #[serde(default)]
    pub search: String,
    #[serde(default)]
    pub filter: TrackFilter,
    pub collection_id: Option<String>,
    pub folder_id: Option<i64>,
    #[serde(default = "default_sort")]
    pub sort: String,
    #[serde(default)]
    pub descending: bool,
    #[serde(default)]
    pub offset: u32,
    #[serde(default = "default_limit")]
    pub limit: u32,
}

fn default_sort() -> String {
    "title".to_string()
}

fn default_limit() -> u32 {
    SEARCH_PAGE_SIZE
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrackRecord {
    pub id: String,
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration_seconds: Option<f64>,
    pub format: String,
    pub bpm: Option<f64>,
    pub bpm_analysis_candidate: Option<f64>,
    pub bpm_analysis_confidence: Option<f64>,
    pub bpm_analysis_status: Option<String>,
    pub bpm_correction: Option<f64>,
    pub musical_key: Option<String>,
    pub path: String,
    pub file_size: u64,
    pub import_state: String,
    pub load_supported: bool,
    pub load_status: String,
    pub missing: bool,
    pub favorite: bool,
    pub duplicate_locations: u32,
    pub artwork_available: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BpmAnalysisRecord {
    pub track_id: String,
    pub candidate_bpm: Option<f64>,
    pub confidence: f64,
    pub status: String,
    pub algorithm_version: String,
    pub analyzed_at: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BpmAnalysisProgress {
    pub track_id: String,
    pub state: String,
    pub processed_frames: u64,
    pub total_frames: Option<u64>,
    pub candidate_bpm: Option<f64>,
    pub confidence: Option<f64>,
    pub message: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BpmAnalysisStart {
    pub started: bool,
    pub message: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrackPage {
    pub items: Vec<TrackRecord>,
    pub total: u64,
    pub offset: u32,
    pub limit: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LibraryFolder {
    pub id: i64,
    pub path: String,
    pub track_count: u64,
    pub missing_count: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LibraryCollection {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub item_count: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScanProgress {
    pub state: String,
    pub roots_total: u32,
    pub roots_done: u32,
    pub partial_roots: u32,
    pub files_seen: u64,
    pub tracks_indexed: u64,
    pub unsupported: u64,
    pub unreadable: u64,
    pub traversal_errors: u64,
    pub current_file: Option<String>,
    pub message: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScanSummary {
    pub roots_scanned: u32,
    pub partial_roots: u32,
    pub files_seen: u64,
    pub tracks_indexed: u64,
    pub unsupported: u64,
    pub unreadable: u64,
    pub traversal_errors: u64,
    pub cancelled: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScanStart {
    pub started: bool,
    pub message: String,
}

#[derive(Clone, Copy, Debug)]
enum CollectionKind {
    Crate,
    Playlist,
}

impl CollectionKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Crate => "crate",
            Self::Playlist => "playlist",
        }
    }
}

/// A short-lived SQLite catalog handle. Open one per command or worker thread.
pub struct LibraryCatalog {
    database_path: PathBuf,
}

impl LibraryCatalog {
    pub fn open(data_directory: impl AsRef<Path>) -> Result<Self, String> {
        let data_directory = data_directory.as_ref();
        fs::create_dir_all(data_directory)
            .map_err(|error| format!("Could not create the SAYT library data folder: {error}"))?;
        let database_path = data_directory.join("music-library.sqlite3");
        let mut initialize_journal = !database_path.exists();
        if database_path.exists() {
            let integrity = Self::connect(&database_path).and_then(|connection| {
                connection.query_row("PRAGMA quick_check", [], |row| row.get::<_, String>(0))
            });
            if !matches!(integrity.as_deref(), Ok("ok")) {
                Self::quarantine_database(&database_path)?;
                initialize_journal = true;
            }
        }

        let catalog = Self { database_path };
        let connection = catalog.connection()?;
        if initialize_journal {
            connection
                .execute_batch("PRAGMA journal_mode = WAL;")
                .map_err(|error| {
                    format!("Could not initialize local catalog journaling: {error}")
                })?;
        }
        catalog.migrate(&connection)?;
        Ok(catalog)
    }

    /// Opens an already-initialized catalog for a short-lived command connection.
    /// The one-time full integrity check belongs to application startup, not search requests.
    pub fn open_for_commands(data_directory: impl AsRef<Path>) -> Result<Self, String> {
        let data_directory = data_directory.as_ref();
        fs::create_dir_all(data_directory)
            .map_err(|error| format!("Could not create the SAYT library data folder: {error}"))?;
        let catalog = Self {
            database_path: data_directory.join("music-library.sqlite3"),
        };
        let connection = catalog.connection()?;
        catalog.migrate(&connection)?;
        Ok(catalog)
    }

    fn connect(database_path: &Path) -> Result<Connection, rusqlite::Error> {
        let connection = Connection::open(database_path)?;
        connection.busy_timeout(std::time::Duration::from_secs(5))?;
        connection.execute_batch("PRAGMA foreign_keys = ON;")?;
        Ok(connection)
    }

    fn connection(&self) -> Result<Connection, String> {
        Self::connect(&self.database_path)
            .map_err(|error| format!("Could not open the local music catalog: {error}"))
    }

    fn migrate(&self, connection: &Connection) -> Result<(), String> {
        let mut version = connection
            .pragma_query_value(None, "user_version", |row| row.get::<_, i32>(0))
            .map_err(|error| format!("Could not read the music catalog version: {error}"))?;
        if version > SCHEMA_VERSION {
            return Err(format!(
                "This SAYT build cannot read music catalog version {version}. Update SAYT before opening this catalog."
            ));
        }
        if version == 0 {
            connection
                .execute_batch(
                    "BEGIN IMMEDIATE;
                     CREATE TABLE roots (
                         id INTEGER PRIMARY KEY,
                         path TEXT NOT NULL UNIQUE,
                         scan_generation INTEGER NOT NULL DEFAULT 0,
                         last_scan_at INTEGER,
                         scan_state TEXT NOT NULL DEFAULT 'never'
                     );
                     CREATE TABLE tracks (
                         id TEXT PRIMARY KEY,
                         content_sha256 TEXT UNIQUE,
                         title TEXT NOT NULL,
                         artist TEXT,
                         album TEXT,
                         user_title TEXT,
                         user_artist TEXT,
                         user_album TEXT,
                         duration_seconds REAL,
                         format TEXT NOT NULL,
                         bpm REAL,
                         musical_key TEXT,
                         file_size INTEGER NOT NULL,
                         import_state TEXT NOT NULL,
                         load_supported INTEGER NOT NULL,
                         load_status TEXT NOT NULL,
                         missing INTEGER NOT NULL DEFAULT 0,
                         favorite INTEGER NOT NULL DEFAULT 0,
                         artwork_path TEXT,
                         search_text TEXT NOT NULL,
                         first_seen_at INTEGER NOT NULL,
                         last_seen_at INTEGER NOT NULL
                     );
                     CREATE TABLE locations (
                         id INTEGER PRIMARY KEY,
                         track_id TEXT NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
                         root_id INTEGER NOT NULL REFERENCES roots(id) ON DELETE CASCADE,
                         path TEXT NOT NULL,
                         file_size INTEGER NOT NULL,
                         modified_at INTEGER NOT NULL,
                         last_seen_generation INTEGER NOT NULL,
                         missing INTEGER NOT NULL DEFAULT 0,
                         UNIQUE(root_id, path)
                     );
                     CREATE INDEX locations_track_id_idx ON locations(track_id);
                     CREATE INDEX locations_root_idx ON locations(root_id, last_seen_generation);
                     CREATE TABLE collections (
                         id TEXT PRIMARY KEY,
                         name TEXT NOT NULL COLLATE NOCASE UNIQUE,
                         kind TEXT NOT NULL CHECK(kind IN ('crate', 'playlist')),
                         created_at INTEGER NOT NULL,
                         updated_at INTEGER NOT NULL
                     );
                     CREATE TABLE collection_items (
                         collection_id TEXT NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
                         track_id TEXT NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
                         position INTEGER NOT NULL,
                         PRIMARY KEY(collection_id, track_id)
                     );
                     CREATE INDEX collection_items_order_idx ON collection_items(collection_id, position);
                     CREATE TABLE user_metadata (
                         track_id TEXT PRIMARY KEY REFERENCES tracks(id) ON DELETE CASCADE,
                         notes TEXT,
                         created_at INTEGER NOT NULL,
                         updated_at INTEGER NOT NULL
                     );
                     PRAGMA user_version = 1;
                     COMMIT;",
                )
                .map_err(|error| format!("Could not create the music catalog schema: {error}"))?;
            version = 1;
        }
        if version == 1 {
            connection
                .execute_batch(
                    "BEGIN IMMEDIATE;
                     CREATE TABLE bpm_analysis (
                         track_id TEXT PRIMARY KEY REFERENCES tracks(id) ON DELETE CASCADE,
                         algorithm_version TEXT NOT NULL,
                         source_sha256 TEXT NOT NULL,
                         candidate_bpm REAL,
                         confidence REAL NOT NULL CHECK(confidence >= 0.0 AND confidence <= 1.0),
                         status TEXT NOT NULL CHECK(status IN ('estimated', 'uncertain')),
                         analyzed_at INTEGER NOT NULL
                     );
                     CREATE TABLE bpm_corrections (
                         track_id TEXT PRIMARY KEY REFERENCES tracks(id) ON DELETE CASCADE,
                         bpm REAL NOT NULL CHECK(bpm >= 30.0 AND bpm <= 300.0),
                         modified_at INTEGER NOT NULL
                     );
                     PRAGMA user_version = 2;
                     COMMIT;",
                )
                .map_err(|error| format!("Could not add track analysis storage: {error}"))?;
        }
        Ok(())
    }

    fn quarantine_database(database_path: &Path) -> Result<(), String> {
        let stamp = unix_time_millis();
        let original_name = database_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("music-library.sqlite3");
        let quarantine_path = database_path.with_file_name(format!(
            "{original_name}.corrupt-{stamp}-{}.bak",
            std::process::id()
        ));
        fs::rename(database_path, &quarantine_path).map_err(|error| {
            format!(
                "The music catalog failed its integrity check, and SAYT could not preserve a recovery copy: {error}"
            )
        })?;
        for suffix in ["-wal", "-shm"] {
            let sidecar = PathBuf::from(format!("{}{suffix}", database_path.display()));
            if sidecar.exists() {
                let quarantined = PathBuf::from(format!("{}{suffix}", quarantine_path.display()));
                fs::rename(sidecar, quarantined).map_err(|error| {
                    format!("Could not preserve a catalog recovery sidecar: {error}")
                })?;
            }
        }
        Ok(())
    }

    pub fn refresh_missing_state(&self) -> Result<(), String> {
        let connection = self.connection()?;
        Self::refresh_missing_state_connection(&connection)
    }

    fn refresh_missing_state_connection(connection: &Connection) -> Result<(), String> {
        let mut statement = connection
            .prepare("SELECT id, track_id, path FROM locations")
            .map_err(|error| format!("Could not inspect saved music locations: {error}"))?;
        let locations = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .map_err(|error| format!("Could not read saved music locations: {error}"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("Could not read a saved music location: {error}"))?;
        drop(statement);

        let transaction = connection
            .unchecked_transaction()
            .map_err(|error| format!("Could not update missing-file state: {error}"))?;
        for (location_id, _, path) in locations {
            transaction
                .execute(
                    "UPDATE locations SET missing = ?2 WHERE id = ?1",
                    params![
                        location_id,
                        i64::from(location_is_missing(Path::new(&path)))
                    ],
                )
                .map_err(|error| format!("Could not update a missing location: {error}"))?;
        }
        transaction
            .execute(
                "UPDATE tracks SET missing = NOT EXISTS (
                    SELECT 1 FROM locations WHERE locations.track_id = tracks.id AND locations.missing = 0
                )",
                [],
            )
            .map_err(|error| format!("Could not refresh missing-file state: {error}"))?;
        transaction
            .commit()
            .map_err(|error| format!("Could not save missing-file state: {error}"))
    }

    pub fn list_tracks(&self, query: &TrackQuery) -> Result<TrackPage, String> {
        let connection = self.connection()?;
        if query.search.chars().count() > MAX_SEARCH_CHARS {
            return Err(format!(
                "Search is limited to {MAX_SEARCH_CHARS} characters"
            ));
        }
        let limit = query.limit.clamp(1, MAX_PAGE_SIZE);
        let offset = query.offset;
        let search = query.search.trim().to_lowercase();
        let sort = match query.sort.as_str() {
            "artist" => "COALESCE(t.user_artist, t.artist, '')",
            "album" => "COALESCE(t.user_album, t.album, '')",
            "duration" => "COALESCE(t.duration_seconds, 0)",
            "format" => "t.format",
            "bpm" => "COALESCE(
                (SELECT correction.bpm FROM bpm_corrections correction WHERE correction.track_id = t.id),
                (SELECT CASE WHEN analysis.status = 'estimated' THEN analysis.candidate_bpm END
                 FROM bpm_analysis analysis WHERE analysis.track_id = t.id),
                t.bpm, 0
            )",
            _ => "COALESCE(t.user_title, t.title)",
        };
        let (order_by, direction) = if query.collection_id.is_some() {
            (
                "(SELECT ci.position FROM collection_items ci WHERE ci.collection_id = ?3 AND ci.track_id = t.id)",
                "ASC",
            )
        } else {
            (sort, if query.descending { "DESC" } else { "ASC" })
        };
        let where_sql = "(?1 = '' OR instr(t.search_text, ?1) > 0)
             AND (?2 = 0 OR (?2 = 1 AND t.first_seen_at >= ?5)
                  OR (?2 = 2 AND t.missing = 1)
                  OR (?2 = 3 AND t.bpm IS NULL AND t.musical_key IS NULL
                      AND NOT EXISTS (SELECT 1 FROM bpm_analysis analysis WHERE analysis.track_id = t.id))
                  OR (?2 = 4 AND t.favorite = 1)
                  OR (?2 = 5 AND t.load_supported = 0))
             AND (?3 IS NULL OR EXISTS (
                  SELECT 1 FROM collection_items ci WHERE ci.collection_id = ?3 AND ci.track_id = t.id))
             AND (?4 IS NULL OR EXISTS (
                  SELECT 1 FROM locations fl WHERE fl.track_id = t.id AND fl.root_id = ?4))";
        let count_sql = format!("SELECT COUNT(*) FROM tracks t WHERE {where_sql}");
        let mut count_statement = connection
            .prepare(&count_sql)
            .map_err(|error| format!("Could not prepare the music search: {error}"))?;
        let filter_code = match query.filter {
            TrackFilter::All => 0_i64,
            TrackFilter::Recent => 1,
            TrackFilter::Missing => 2,
            TrackFilter::Unanalyzed => 3,
            TrackFilter::Favorites => 4,
            TrackFilter::Unsupported => 5,
        };
        let recent_after = unix_time_millis().saturating_sub(30 * 24 * 60 * 60 * 1000);
        let collection_parameter = query
            .collection_id
            .as_ref()
            .map_or(Value::Null, |value| Value::Text(value.clone()));
        let folder_parameter = query.folder_id.map_or(Value::Null, Value::Integer);
        let total = count_statement
            .query_row(
                params![
                    search,
                    filter_code,
                    collection_parameter,
                    folder_parameter,
                    recent_after
                ],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|error| format!("Could not count matching music tracks: {error}"))?;
        drop(count_statement);

        let list_sql = format!(
            "SELECT t.id, COALESCE(t.user_title, t.title), COALESCE(t.user_artist, t.artist),
                    COALESCE(t.user_album, t.album), t.duration_seconds, t.format, t.bpm,
                    (SELECT analysis.candidate_bpm FROM bpm_analysis analysis WHERE analysis.track_id = t.id),
                    (SELECT analysis.confidence FROM bpm_analysis analysis WHERE analysis.track_id = t.id),
                    (SELECT analysis.status FROM bpm_analysis analysis WHERE analysis.track_id = t.id),
                    (SELECT correction.bpm FROM bpm_corrections correction WHERE correction.track_id = t.id),
                    t.musical_key,
                    COALESCE((SELECT l.path FROM locations l WHERE l.track_id = t.id ORDER BY l.missing ASC, l.last_seen_generation DESC, l.id LIMIT 1), ''),
                    t.file_size, t.import_state, t.load_supported, t.load_status, t.missing, t.favorite,
                    (SELECT COUNT(*) FROM locations d WHERE d.track_id = t.id AND d.missing = 0), t.artwork_path IS NOT NULL
             FROM tracks t WHERE {where_sql}
             ORDER BY {order_by} {direction}, t.id ASC LIMIT ?6 OFFSET ?7"
        );
        let mut statement = connection
            .prepare(&list_sql)
            .map_err(|error| format!("Could not prepare the music list: {error}"))?;
        let items = statement
            .query_map(
                params![
                    search,
                    filter_code,
                    collection_parameter,
                    folder_parameter,
                    recent_after,
                    limit,
                    offset
                ],
                |row| {
                    Ok(TrackRecord {
                        id: row.get(0)?,
                        title: row.get(1)?,
                        artist: row.get(2)?,
                        album: row.get(3)?,
                        duration_seconds: row.get(4)?,
                        format: row.get(5)?,
                        bpm: row.get(6)?,
                        bpm_analysis_candidate: row.get(7)?,
                        bpm_analysis_confidence: row.get(8)?,
                        bpm_analysis_status: row.get(9)?,
                        bpm_correction: row.get(10)?,
                        musical_key: row.get(11)?,
                        path: row.get(12)?,
                        file_size: row.get::<_, i64>(13)?.max(0) as u64,
                        import_state: row.get(14)?,
                        load_supported: row.get::<_, i64>(15)? != 0,
                        load_status: row.get(16)?,
                        missing: row.get::<_, i64>(17)? != 0,
                        favorite: row.get::<_, i64>(18)? != 0,
                        duplicate_locations: row.get::<_, u32>(19)?,
                        artwork_available: row.get::<_, i64>(20)? != 0,
                    })
                },
            )
            .map_err(|error| format!("Could not search the music catalog: {error}"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("Could not read a music catalog entry: {error}"))?;
        Ok(TrackPage {
            items,
            total: total.max(0) as u64,
            offset,
            limit,
        })
    }

    /// Analyze a catalog-owned PCM WAV file outside the real-time audio path.
    /// Results are cached by content identity and algorithm version.
    pub fn analyze_bpm(
        &self,
        track_id: &str,
        cancellation: &AtomicBool,
        mut report: impl FnMut(BpmAnalysisProgress),
    ) -> Result<Option<BpmAnalysisRecord>, String> {
        let path = self.resolve_track_path(track_id)?;
        let expected_digest = track_id.strip_prefix("sha256:").ok_or_else(|| {
            "This catalog track has no supported content identity. Re-scan its folder.".to_string()
        })?;

        report(BpmAnalysisProgress {
            track_id: track_id.to_string(),
            state: "preparing".to_string(),
            processed_frames: 0,
            total_frames: None,
            candidate_bpm: None,
            confidence: None,
            message: "Checking the selected library file before analysis".to_string(),
        });
        let Some(current_digest) = hash_file_cancellable(&path, cancellation)? else {
            report(analysis_progress(
                track_id,
                "cancelled",
                0,
                None,
                None,
                None,
                "BPM analysis was cancelled",
            ));
            return Ok(None);
        };
        if current_digest != expected_digest {
            return Err(
                "This file changed after it was added to the library. Re-scan the folder before analyzing it."
                    .to_string(),
            );
        }

        let connection = self.connection()?;
        let cached = read_bpm_analysis(&connection, track_id)?;
        if let Some(cached) =
            cached.filter(|record| record.algorithm_version == bpm_analysis::algorithm_version())
        {
            let state = if cached.status == "estimated" {
                "complete"
            } else {
                "uncertain"
            };
            report(analysis_progress(
                track_id,
                state,
                0,
                None,
                cached.candidate_bpm,
                Some(cached.confidence),
                "Using the saved result for this track and analysis version",
            ));
            return Ok(Some(cached));
        }
        drop(connection);

        let analysis = bpm_analysis::analyze_wav(&path, cancellation, |processed, total| {
            report(analysis_progress(
                track_id,
                "analyzing",
                processed,
                total,
                None,
                None,
                "Analyzing local PCM audio",
            ));
        })?;
        let bpm_analysis::AnalysisAttempt::Complete(estimate) = analysis else {
            report(analysis_progress(
                track_id,
                "cancelled",
                0,
                None,
                None,
                None,
                "BPM analysis was cancelled",
            ));
            return Ok(None);
        };
        if cancellation.load(Ordering::Acquire) {
            report(analysis_progress(
                track_id,
                "cancelled",
                0,
                None,
                None,
                None,
                "BPM analysis was cancelled",
            ));
            return Ok(None);
        }

        let Some(final_digest) = hash_file_cancellable(&path, cancellation)? else {
            report(analysis_progress(
                track_id,
                "cancelled",
                0,
                None,
                None,
                None,
                "BPM analysis was cancelled",
            ));
            return Ok(None);
        };
        if final_digest != expected_digest {
            return Err(
                "This file changed while BPM analysis was running. Re-scan the folder and retry."
                    .to_string(),
            );
        }

        let analyzed_at = unix_time_millis();
        let status = estimate.status.as_str();
        let connection = self.connection()?;
        connection
            .execute(
                "INSERT INTO bpm_analysis(
                    track_id, algorithm_version, source_sha256, candidate_bpm, confidence, status, analyzed_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                 ON CONFLICT(track_id) DO UPDATE SET
                    algorithm_version = excluded.algorithm_version,
                    source_sha256 = excluded.source_sha256,
                    candidate_bpm = excluded.candidate_bpm,
                    confidence = excluded.confidence,
                    status = excluded.status,
                    analyzed_at = excluded.analyzed_at",
                params![
                    track_id,
                    bpm_analysis::algorithm_version(),
                    current_digest,
                    estimate.candidate_bpm,
                    estimate.confidence,
                    status,
                    analyzed_at,
                ],
            )
            .map_err(|error| format!("Could not save the BPM analysis result: {error}"))?;
        let record = BpmAnalysisRecord {
            track_id: track_id.to_string(),
            candidate_bpm: estimate.candidate_bpm,
            confidence: estimate.confidence,
            status: status.to_string(),
            algorithm_version: bpm_analysis::algorithm_version().to_string(),
            analyzed_at,
        };
        report(analysis_progress(
            track_id,
            if status == "estimated" {
                "complete"
            } else {
                "uncertain"
            },
            0,
            None,
            estimate.candidate_bpm,
            Some(estimate.confidence),
            if status == "estimated" {
                "BPM estimate saved"
            } else {
                "Analysis completed, but the tempo is uncertain"
            },
        ));
        Ok(Some(record))
    }

    /// Store a user BPM correction independently from file tags and analysis.
    pub fn set_bpm_correction(&self, track_id: &str, bpm: Option<f64>) -> Result<(), String> {
        if let Some(bpm) = bpm {
            if !bpm.is_finite() || !(MIN_BPM_CORRECTION..=MAX_BPM_CORRECTION).contains(&bpm) {
                return Err(format!(
                    "Manual BPM must be between {MIN_BPM_CORRECTION:.0} and {MAX_BPM_CORRECTION:.0}"
                ));
            }
        }
        let connection = self.connection()?;
        let exists = connection
            .query_row("SELECT 1 FROM tracks WHERE id = ?1", [track_id], |_| Ok(()))
            .optional()
            .map_err(|error| {
                format!("Could not find the catalog track for BPM correction: {error}")
            })?
            .is_some();
        if !exists {
            return Err("That track is no longer in the library. Refresh and retry.".to_string());
        }
        match bpm {
            Some(bpm) => {
                connection
                    .execute(
                        "INSERT INTO bpm_corrections(track_id, bpm, modified_at)
                         VALUES (?1, ?2, ?3)
                         ON CONFLICT(track_id) DO UPDATE SET bpm = excluded.bpm, modified_at = excluded.modified_at",
                        params![track_id, bpm, unix_time_millis()],
                    )
                    .map_err(|error| format!("Could not save the manual BPM correction: {error}"))?;
            }
            None => {
                connection
                    .execute(
                        "DELETE FROM bpm_corrections WHERE track_id = ?1",
                        [track_id],
                    )
                    .map_err(|error| {
                        format!("Could not clear the manual BPM correction: {error}")
                    })?;
            }
        }
        Ok(())
    }

    /// Return a tempo safe for existing beat controls: correction, confident
    /// analysis, then a usable file tag. Uncertain analysis is intentionally skipped.
    pub fn playback_bpm(&self, track_id: &str) -> Result<Option<f64>, String> {
        let connection = self.connection()?;
        let bpm = connection
            .query_row(
                "SELECT COALESCE(
                    correction.bpm,
                    CASE WHEN analysis.status = 'estimated' THEN analysis.candidate_bpm END,
                    track.bpm
                 )
                 FROM tracks track
                 LEFT JOIN bpm_analysis analysis ON analysis.track_id = track.id
                 LEFT JOIN bpm_corrections correction ON correction.track_id = track.id
                 WHERE track.id = ?1",
                [track_id],
                |row| row.get::<_, Option<f64>>(0),
            )
            .optional()
            .map_err(|error| format!("Could not read the selected track tempo: {error}"))?
            .flatten();
        Ok(bpm.filter(|bpm| {
            bpm.is_finite() && (MIN_BPM_CORRECTION..=MAX_BPM_CORRECTION).contains(bpm)
        }))
    }

    pub fn list_folders(&self) -> Result<Vec<LibraryFolder>, String> {
        let connection = self.connection()?;
        let mut statement = connection
            .prepare(
                "SELECT r.id, r.path,
                        COUNT(DISTINCT CASE WHEN l.missing = 0 THEN t.id END),
                        COUNT(DISTINCT CASE WHEN l.missing = 1 THEN t.id END)
                 FROM roots r LEFT JOIN locations l ON l.root_id = r.id
                 LEFT JOIN tracks t ON t.id = l.track_id GROUP BY r.id ORDER BY r.path COLLATE NOCASE",
            )
            .map_err(|error| format!("Could not prepare the music folders: {error}"))?;
        let folders = statement
            .query_map([], |row| {
                Ok(LibraryFolder {
                    id: row.get(0)?,
                    path: row.get(1)?,
                    track_count: row.get::<_, i64>(2)?.max(0) as u64,
                    missing_count: row.get::<_, i64>(3)?.max(0) as u64,
                })
            })
            .map_err(|error| format!("Could not read music folders: {error}"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("Could not read a music folder: {error}"))?;
        Ok(folders)
    }

    pub fn list_collections(&self) -> Result<Vec<LibraryCollection>, String> {
        let connection = self.connection()?;
        let mut statement = connection
            .prepare(
                "SELECT c.id, c.name, c.kind, COUNT(ci.track_id)
                 FROM collections c LEFT JOIN collection_items ci ON ci.collection_id = c.id
                 GROUP BY c.id ORDER BY c.kind, c.name COLLATE NOCASE",
            )
            .map_err(|error| format!("Could not prepare the music collections: {error}"))?;
        let collections = statement
            .query_map([], |row| {
                Ok(LibraryCollection {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    kind: row.get(2)?,
                    item_count: row.get::<_, i64>(3)?.max(0) as u64,
                })
            })
            .map_err(|error| format!("Could not read music collections: {error}"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("Could not read a music collection: {error}"))?;
        Ok(collections)
    }

    pub fn create_collection(&self, name: &str, kind: &str) -> Result<LibraryCollection, String> {
        let name = validate_collection_name(name)?;
        let kind = match kind {
            "crate" => CollectionKind::Crate,
            "playlist" => CollectionKind::Playlist,
            _ => return Err("Choose a crate or playlist collection type".to_string()),
        };
        let id = new_collection_id();
        let now = unix_time_millis();
        let connection = self.connection()?;
        connection
            .execute(
                "INSERT INTO collections(id, name, kind, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?4)",
                params![id, name, kind.as_str(), now],
            )
            .map_err(|error| collection_error("create", error))?;
        Ok(LibraryCollection {
            id,
            name,
            kind: kind.as_str().to_string(),
            item_count: 0,
        })
    }

    pub fn rename_collection(&self, id: &str, name: &str) -> Result<(), String> {
        let name = validate_collection_name(name)?;
        let connection = self.connection()?;
        let changed = connection
            .execute(
                "UPDATE collections SET name = ?2, updated_at = ?3 WHERE id = ?1",
                params![id, name, unix_time_millis()],
            )
            .map_err(|error| collection_error("rename", error))?;
        if changed == 0 {
            return Err(
                "That crate or playlist no longer exists. Refresh the library and retry."
                    .to_string(),
            );
        }
        Ok(())
    }

    pub fn delete_collection(&self, id: &str) -> Result<(), String> {
        let connection = self.connection()?;
        let changed = connection
            .execute("DELETE FROM collections WHERE id = ?1", [id])
            .map_err(|error| format!("Could not delete the collection: {error}"))?;
        if changed == 0 {
            return Err(
                "That crate or playlist no longer exists. Refresh the library and retry."
                    .to_string(),
            );
        }
        Ok(())
    }

    pub fn add_track_to_collection(
        &self,
        collection_id: &str,
        track_id: &str,
    ) -> Result<(), String> {
        let connection = self.connection()?;
        let transaction = connection
            .unchecked_transaction()
            .map_err(|error| format!("Could not add the track to the collection: {error}"))?;
        let exists = transaction
            .query_row("SELECT 1 FROM tracks WHERE id = ?1", [track_id], |_| Ok(()))
            .optional()
            .map_err(|error| format!("Could not find the selected track: {error}"))?
            .is_some();
        if !exists {
            return Err(
                "That track is no longer in the library. Refresh and select it again.".to_string(),
            );
        }
        let position = transaction
            .query_row(
                "SELECT COALESCE(MAX(position) + 1, 0) FROM collection_items WHERE collection_id = ?1",
                [collection_id],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|error| format!("Could not find the collection: {error}"))?;
        transaction
            .execute(
                "INSERT OR IGNORE INTO collection_items(collection_id, track_id, position) VALUES (?1, ?2, ?3)",
                params![collection_id, track_id, position],
            )
            .map_err(|error| format!("Could not add the track to the collection: {error}"))?;
        transaction
            .execute(
                "UPDATE collections SET updated_at = ?2 WHERE id = ?1",
                params![collection_id, unix_time_millis()],
            )
            .map_err(|error| format!("Could not update the collection timestamp: {error}"))?;
        transaction
            .commit()
            .map_err(|error| format!("Could not save the collection item: {error}"))
    }

    pub fn remove_track_from_collection(
        &self,
        collection_id: &str,
        track_id: &str,
    ) -> Result<(), String> {
        let connection = self.connection()?;
        let transaction = connection
            .unchecked_transaction()
            .map_err(|error| format!("Could not remove the track from the collection: {error}"))?;
        transaction
            .execute(
                "DELETE FROM collection_items WHERE collection_id = ?1 AND track_id = ?2",
                params![collection_id, track_id],
            )
            .map_err(|error| format!("Could not remove the track from the collection: {error}"))?;
        normalize_collection_positions(&transaction, collection_id)?;
        transaction
            .execute(
                "UPDATE collections SET updated_at = ?2 WHERE id = ?1",
                params![collection_id, unix_time_millis()],
            )
            .map_err(|error| format!("Could not update the collection timestamp: {error}"))?;
        transaction
            .commit()
            .map_err(|error| format!("Could not save the collection change: {error}"))
    }

    pub fn move_collection_track(
        &self,
        collection_id: &str,
        track_id: &str,
        delta: i32,
    ) -> Result<(), String> {
        if !matches!(delta, -1 | 1) {
            return Err("Playlist movement must be one position up or down".to_string());
        }
        let connection = self.connection()?;
        let transaction = connection
            .unchecked_transaction()
            .map_err(|error| format!("Could not reorder the playlist: {error}"))?;
        let mut statement = transaction
            .prepare(
                "SELECT track_id FROM collection_items WHERE collection_id = ?1 ORDER BY position, track_id",
            )
            .map_err(|error| format!("Could not read playlist order: {error}"))?;
        let mut ids = statement
            .query_map([collection_id], |row| row.get::<_, String>(0))
            .map_err(|error| format!("Could not read playlist order: {error}"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("Could not read a playlist item: {error}"))?;
        drop(statement);
        let index = ids
            .iter()
            .position(|item_id| item_id == track_id)
            .ok_or_else(|| "That track is not in this playlist".to_string())?;
        let target = index as i32 + delta;
        if target < 0 || target as usize >= ids.len() {
            return Err("The track is already at the end of the playlist".to_string());
        }
        ids.swap(index, target as usize);
        for (position, item_id) in ids.iter().enumerate() {
            transaction
                .execute(
                    "UPDATE collection_items SET position = ?3 WHERE collection_id = ?1 AND track_id = ?2",
                    params![collection_id, item_id, position as i64],
                )
                .map_err(|error| format!("Could not save playlist order: {error}"))?;
        }
        transaction
            .execute(
                "UPDATE collections SET updated_at = ?2 WHERE id = ?1",
                params![collection_id, unix_time_millis()],
            )
            .map_err(|error| format!("Could not update the playlist timestamp: {error}"))?;
        transaction
            .commit()
            .map_err(|error| format!("Could not save playlist order: {error}"))
    }

    pub fn set_favorite(&self, track_id: &str, favorite: bool) -> Result<(), String> {
        let connection = self.connection()?;
        let changed = connection
            .execute(
                "UPDATE tracks SET favorite = ?2 WHERE id = ?1",
                params![track_id, i64::from(favorite)],
            )
            .map_err(|error| format!("Could not update the track favorite: {error}"))?;
        if changed == 0 {
            return Err("That track is no longer in the library. Refresh and retry.".to_string());
        }
        Ok(())
    }

    pub fn update_user_metadata(
        &self,
        track_id: &str,
        title: &str,
        artist: &str,
        album: &str,
    ) -> Result<(), String> {
        let title = title.trim();
        if title.is_empty() {
            return Err("Track title cannot be empty".to_string());
        }
        if title.chars().count() > 200
            || artist.chars().count() > 200
            || album.chars().count() > 200
        {
            return Err(
                "Track title, artist, and album must each be 200 characters or fewer".to_string(),
            );
        }
        let mut connection = self.connection()?;
        let transaction = connection
            .transaction()
            .map_err(|error| format!("Could not start the metadata update: {error}"))?;
        let changed = transaction
            .execute(
                "UPDATE tracks SET user_title = ?2, user_artist = ?3, user_album = ?4
                 WHERE id = ?1",
                params![track_id, title, nonempty(artist), nonempty(album)],
            )
            .map_err(|error| format!("Could not save track metadata: {error}"))?;
        if changed == 0 {
            return Err("That track is no longer in the library. Refresh and retry.".to_string());
        }
        let (title, artist, album, format, path) = transaction
            .query_row(
                "SELECT COALESCE(user_title, title), COALESCE(user_artist, artist),
                        COALESCE(user_album, album), format,
                        COALESCE((SELECT path FROM locations WHERE track_id = tracks.id
                                  ORDER BY missing ASC, last_seen_generation DESC, id LIMIT 1), '')
                 FROM tracks WHERE id = ?1",
                [track_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, Option<String>>(1)?,
                        row.get::<_, Option<String>>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                    ))
                },
            )
            .map_err(|error| format!("Could not refresh track search details: {error}"))?;
        let search_text = searchable_text(
            &title,
            artist.as_deref(),
            album.as_deref(),
            &format,
            Path::new(&path),
        );
        transaction
            .execute(
                "UPDATE tracks SET search_text = ?2 WHERE id = ?1",
                params![track_id, search_text],
            )
            .map_err(|error| format!("Could not update the track search index: {error}"))?;
        transaction
            .commit()
            .map_err(|error| format!("Could not save track metadata: {error}"))
    }

    pub fn resolve_track_path(&self, track_id: &str) -> Result<PathBuf, String> {
        let connection = self.connection()?;
        let result = connection
            .query_row(
                "SELECT t.load_supported, t.load_status, l.path, r.path
                 FROM tracks t JOIN locations l ON l.track_id = t.id JOIN roots r ON r.id = l.root_id
                 WHERE t.id = ?1 ORDER BY l.missing ASC, l.last_seen_generation DESC, l.id LIMIT 1",
                [track_id],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)? != 0,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                    ))
                },
            )
            .optional()
            .map_err(|error| format!("Could not resolve this library track: {error}"))?;
        let Some((load_supported, load_status, path, root)) = result else {
            return Err(
                "This track is missing from its saved location. Re-scan its folder to locate it."
                    .to_string(),
            );
        };
        if !load_supported {
            return Err(load_status);
        }
        let canonical_root = Path::new(&root).canonicalize().map_err(|_| {
            "The library folder is unavailable. Re-scan it after reconnecting the drive."
                .to_string()
        })?;
        let canonical_path = Path::new(&path).canonicalize().map_err(|_| {
            "The track file is missing or unreadable. Re-scan its folder to recover it.".to_string()
        })?;
        if !canonical_path.starts_with(&canonical_root) || !canonical_path.is_file() {
            return Err("The saved track location no longer matches its selected library folder. Re-scan the folder.".to_string());
        }
        Ok(canonical_path)
    }

    pub fn artwork_data_url(&self, track_id: &str) -> Result<Option<String>, String> {
        let connection = self.connection()?;
        let location = connection
            .query_row(
                "SELECT l.path, r.path FROM locations l
                 JOIN roots r ON r.id = l.root_id WHERE l.track_id = ?1
                 ORDER BY l.missing ASC, l.last_seen_generation DESC, l.id LIMIT 1",
                [track_id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )
            .optional()
            .map_err(|error| format!("Could not find the selected track artwork: {error}"))?;
        let Some((path, root)) = location else {
            return Err("This track no longer has a saved music file location".to_string());
        };
        let canonical_root = Path::new(&root)
            .canonicalize()
            .map_err(|_| "The library folder is unavailable".to_string())?;
        let canonical_path = Path::new(&path)
            .canonicalize()
            .map_err(|_| "The track file is missing or unreadable".to_string())?;
        if !canonical_path.starts_with(&canonical_root) || !canonical_path.is_file() {
            return Err("The saved track location is outside its selected folder".to_string());
        }
        let size = fs::metadata(&canonical_path)
            .map_err(|_| "The track file could not be inspected".to_string())?
            .len();
        if size > MAX_ARTWORK_SOURCE_BYTES {
            return Ok(None);
        }

        let mut options = ParseOptions::new();
        options.read_cover_art(true);
        let probe = match Probe::open(&canonical_path) {
            Ok(probe) => probe.options(options),
            Err(_) => return Ok(None),
        };
        let probe = match probe.guess_file_type() {
            Ok(probe) => probe,
            Err(_) => return Ok(None),
        };
        let tagged_file = match probe.read() {
            Ok(file) => file,
            Err(_) => return Ok(None),
        };
        let picture = tagged_file
            .primary_tag()
            .or_else(|| tagged_file.first_tag())
            .and_then(|tag| tag.pictures().first());
        let Some(picture) = picture else {
            return Ok(None);
        };
        let data = picture.data();
        if data.is_empty() || data.len() > MAX_ARTWORK_BYTES {
            return Ok(None);
        }
        let detected_mime = image_mime_type(data);
        let tagged_mime = picture.mime_type().map(ToString::to_string);
        let mime = match (tagged_mime.as_deref(), detected_mime) {
            (Some("image/jpeg"), Some("image/jpeg")) => "image/jpeg",
            (Some("image/png"), Some("image/png")) => "image/png",
            (Some("image/webp"), Some("image/webp")) => "image/webp",
            (None, Some(mime)) => mime,
            _ => return Ok(None),
        };
        Ok(Some(format!(
            "data:{mime};base64,{}",
            BASE64_STANDARD.encode(data)
        )))
    }

    pub fn backup_to(&self, destination: impl AsRef<Path>) -> Result<(), String> {
        let source = self.connection()?;
        let destination_path = destination.as_ref();
        if destination_path == self.database_path {
            return Err("Choose a different location for the library backup".to_string());
        }
        if let Some(parent) = destination_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("Could not create the backup folder: {error}"))?;
        }
        let mut target = Connection::open(destination_path)
            .map_err(|error| format!("Could not create the library backup: {error}"))?;
        let backup = Backup::new(&source, &mut target)
            .map_err(|error| format!("Could not start the library backup: {error}"))?;
        backup
            .run_to_completion(128, std::time::Duration::from_millis(10), None)
            .map_err(|error| format!("Could not finish the library backup: {error}"))?;
        drop(backup);
        target
            .execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
            .map_err(|error| format!("Could not finalize the library backup: {error}"))?;
        Ok(())
    }

    pub fn scan_roots(
        &self,
        roots: &[PathBuf],
        cancellation: &AtomicBool,
        mut report: impl FnMut(ScanProgress),
    ) -> Result<ScanSummary, String> {
        let mut summary = ScanSummary {
            roots_scanned: 0,
            partial_roots: 0,
            files_seen: 0,
            tracks_indexed: 0,
            unsupported: 0,
            unreadable: 0,
            traversal_errors: 0,
            cancelled: false,
        };
        let mut database = self.connection()?;
        let roots_total = roots.len().min(u32::MAX as usize) as u32;

        for root_path in roots {
            if cancellation.load(Ordering::Relaxed) {
                summary.cancelled = true;
                break;
            }
            let root = root_path
                .canonicalize()
                .map_err(|error| format!("Could not access a selected music folder: {error}"))?;
            if !root.is_dir() {
                return Err("A selected library location is not a folder".to_string());
            }
            let root_string = root.to_string_lossy().into_owned();
            let root_id = database
                .query_row(
                    "INSERT INTO roots(path) VALUES (?1) ON CONFLICT(path) DO UPDATE SET path = excluded.path RETURNING id",
                    [&root_string],
                    |row| row.get::<_, i64>(0),
                )
                .map_err(|error| format!("Could not register a music folder: {error}"))?;
            database
                .execute(
                    "UPDATE roots SET scan_generation = scan_generation + 1, scan_state = 'scanning' WHERE id = ?1",
                    [root_id],
                )
                .map_err(|error| format!("Could not start a folder scan: {error}"))?;
            let generation = database
                .query_row(
                    "SELECT scan_generation FROM roots WHERE id = ?1",
                    [root_id],
                    |row| row.get::<_, i64>(0),
                )
                .map_err(|error| format!("Could not read folder scan state: {error}"))?;
            let mut root_had_incomplete_entries = false;

            for entry in WalkDir::new(&root).follow_links(false).into_iter() {
                if cancellation.load(Ordering::Relaxed) {
                    summary.cancelled = true;
                    break;
                }
                let entry = match entry {
                    Ok(entry) => entry,
                    Err(_error) => {
                        summary.traversal_errors += 1;
                        root_had_incomplete_entries = true;
                        report(progress(
                            "scanning",
                            roots_total,
                            summary.roots_scanned,
                            &summary,
                            None,
                            "Some folders could not be read; existing entries are being preserved.",
                        ));
                        continue;
                    }
                };
                if !entry.file_type().is_file() {
                    continue;
                }
                let path = entry.path();
                if !is_candidate_audio(path) {
                    continue;
                }
                summary.files_seen += 1;
                report(progress(
                    "scanning",
                    roots_total,
                    summary.roots_scanned,
                    &summary,
                    Some(path.to_string_lossy().into_owned()),
                    "Reading audio file metadata and identity…",
                ));

                let entry_result = match database.transaction() {
                    Ok(transaction) => match index_file(&transaction, root_id, generation, path) {
                        Ok(indexed) => transaction.commit().map(|()| indexed).map_err(|error| {
                            format!("Could not commit the indexed audio file: {error}")
                        }),
                        Err(error) => Err(error),
                    },
                    Err(error) => Err(format!("Could not start indexing this audio file: {error}")),
                };
                match entry_result {
                    Ok(indexed) => {
                        summary.tracks_indexed += 1;
                        if !indexed.load_supported {
                            summary.unsupported += 1;
                        }
                        if indexed.import_state != "metadata" {
                            summary.unreadable += 1;
                        }
                    }
                    Err(error) => {
                        root_had_incomplete_entries = true;
                        summary.unreadable += 1;
                        report(progress(
                            "scanning",
                            roots_total,
                            summary.roots_scanned,
                            &summary,
                            Some(path.to_string_lossy().into_owned()),
                            &error,
                        ));
                    }
                }
            }

            if cancellation.load(Ordering::Relaxed) {
                summary.cancelled = true;
            }
            if !summary.cancelled {
                if root_had_incomplete_entries {
                    summary.partial_roots += 1;
                    database
                        .execute(
                            "UPDATE roots SET scan_state = 'partial', last_scan_at = ?2 WHERE id = ?1",
                            params![root_id, unix_time_millis()],
                        )
                        .map_err(|error| format!("Could not save partial scan state: {error}"))?;
                } else {
                    let transaction = database
                        .transaction()
                        .map_err(|error| format!("Could not finish the folder scan: {error}"))?;
                    transaction
                        .execute(
                            "UPDATE locations SET missing = 1 WHERE root_id = ?1 AND last_seen_generation != ?2",
                            params![root_id, generation],
                        )
                        .map_err(|error| format!("Could not refresh moved or missing files: {error}"))?;
                    transaction
                        .execute(
                            "UPDATE roots SET scan_state = 'complete', last_scan_at = ?2 WHERE id = ?1",
                            params![root_id, unix_time_millis()],
                        )
                        .map_err(|error| format!("Could not save completed scan state: {error}"))?;
                    transaction
                        .execute(
                            "UPDATE tracks SET missing = NOT EXISTS (
                                SELECT 1 FROM locations WHERE locations.track_id = tracks.id AND locations.missing = 0
                            )",
                            [],
                        )
                        .map_err(|error| {
                            format!("Could not refresh missing track state: {error}")
                        })?;
                    transaction
                        .commit()
                        .map_err(|error| format!("Could not commit the folder scan: {error}"))?;
                    Self::refresh_missing_state_connection(&database)?;
                }
                summary.roots_scanned += 1;
            } else {
                database
                    .execute(
                        "UPDATE roots SET scan_state = 'cancelled' WHERE id = ?1",
                        [root_id],
                    )
                    .map_err(|error| format!("Could not save cancelled scan state: {error}"))?;
                break;
            }
        }

        report(progress(
            if summary.cancelled {
                "cancelled"
            } else if summary.partial_roots > 0 {
                "partial"
            } else {
                "complete"
            },
            roots_total,
            summary.roots_scanned,
            &summary,
            None,
            if summary.cancelled {
                "Scan cancelled. Tracks already indexed remain available."
            } else {
                "Folder scan finished."
            },
        ));
        Ok(summary)
    }
}

#[derive(Debug)]
struct IndexedFile {
    import_state: String,
    load_supported: bool,
}

fn index_file(
    database: &Connection,
    root_id: i64,
    generation: i64,
    path: &Path,
) -> Result<IndexedFile, String> {
    let file_metadata = fs::metadata(path)
        .map_err(|error| format!("Could not read audio file metadata ({})", error.kind()))?;
    let file_size = file_metadata.len();
    let modified_at = file_metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis().min(i64::MAX as u128) as i64)
        .unwrap_or_default();
    let digest = hash_file(path).map_err(|error| {
        format!(
            "Could not read this audio file for duplicate detection ({})",
            error.kind()
        )
    })?;
    let track_id = format!("sha256:{digest}");
    let details = extract_metadata(path);
    let title_fallback = path
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .filter(|stem| !stem.trim().is_empty())
        .unwrap_or_else(|| "Untitled track".to_string());
    let loadability = wav_loadability(path, file_size);
    let now = unix_time_millis();
    let (title, artist, album, duration, format, bpm, key, import_state) = match details {
        Ok(details) => (
            details.title.unwrap_or(title_fallback),
            details.artist,
            details.album,
            details.duration_seconds,
            details.format,
            details.bpm,
            details.musical_key,
            "metadata",
        ),
        Err((format, _message)) => (
            title_fallback,
            None,
            None,
            None,
            format,
            None,
            None,
            "unreadable",
        ),
    };
    let search_text = searchable_text(&title, artist.as_deref(), album.as_deref(), &format, path);
    database
        .execute(
            "INSERT INTO tracks(
                id, content_sha256, title, artist, album, duration_seconds, format, bpm, musical_key,
                file_size, import_state, load_supported, load_status, missing, search_text, first_seen_at, last_seen_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, 0, ?14, ?15, ?15)
             ON CONFLICT(id) DO UPDATE SET
                title = excluded.title, artist = excluded.artist, album = excluded.album,
                duration_seconds = excluded.duration_seconds, format = excluded.format,
                bpm = excluded.bpm, musical_key = excluded.musical_key,
                file_size = excluded.file_size, import_state = excluded.import_state,
                load_supported = excluded.load_supported, load_status = excluded.load_status,
                missing = 0, search_text = excluded.search_text, last_seen_at = excluded.last_seen_at",
            params![
                track_id,
                digest,
                title,
                artist,
                album,
                duration,
                format,
                bpm,
                key,
                file_size.min(i64::MAX as u64) as i64,
                import_state,
                i64::from(loadability.0),
                loadability.1,
                search_text,
                now
            ],
        )
        .map_err(|error| format!("Could not add the audio file to the catalog: {error}"))?;
    database
        .execute(
            "INSERT INTO locations(track_id, root_id, path, file_size, modified_at, last_seen_generation, missing)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0)
             ON CONFLICT(root_id, path) DO UPDATE SET track_id = excluded.track_id,
                 file_size = excluded.file_size, modified_at = excluded.modified_at,
                 last_seen_generation = excluded.last_seen_generation, missing = 0",
            params![
                track_id,
                root_id,
                path.to_string_lossy(),
                file_size.min(i64::MAX as u64) as i64,
                modified_at,
                generation
            ],
        )
        .map_err(|error| format!("Could not save the audio file location: {error}"))?;
    database
        .execute("UPDATE tracks SET missing = 0 WHERE id = ?1", [&track_id])
        .map_err(|error| format!("Could not clear the track missing state: {error}"))?;
    Ok(IndexedFile {
        import_state: import_state.to_string(),
        load_supported: loadability.0,
    })
}

struct ExtractedMetadata {
    title: Option<String>,
    artist: Option<String>,
    album: Option<String>,
    duration_seconds: Option<f64>,
    format: String,
    bpm: Option<f64>,
    musical_key: Option<String>,
}

fn extract_metadata(path: &Path) -> Result<ExtractedMetadata, (String, String)> {
    let mut options = ParseOptions::new();
    options.read_cover_art(false);
    let tagged_file = Probe::open(path)
        .map_err(|error| (file_format(path), safe_parse_error(error)))?
        .options(options)
        .guess_file_type()
        .map_err(|error| (file_format(path), safe_parse_error(error)))?
        .read()
        .map_err(|error| (file_format(path), safe_parse_error(error)))?;
    let tag = tagged_file
        .primary_tag()
        .or_else(|| tagged_file.first_tag());
    let get = |key| {
        tag.and_then(|tag| tag.get_string(key))
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
    };
    let bpm = get(ItemKey::Bpm)
        .or_else(|| get(ItemKey::IntegerBpm))
        .and_then(|value| value.parse::<f64>().ok())
        .filter(|value| value.is_finite() && (1.0..=1000.0).contains(value));
    let duration = tagged_file.properties().duration().as_secs_f64();
    Ok(ExtractedMetadata {
        title: get(ItemKey::TrackTitle),
        artist: get(ItemKey::TrackArtist),
        album: get(ItemKey::AlbumTitle),
        duration_seconds: (duration > 0.0 && duration.is_finite()).then_some(duration),
        format: format!("{:?}", tagged_file.file_type()).to_ascii_uppercase(),
        bpm,
        musical_key: get(ItemKey::InitialKey),
    })
}

fn wav_loadability(path: &Path, file_size: u64) -> (bool, String) {
    if file_format(path) != "WAV" {
        return (
            false,
            "This format is indexed, but the current deck decoder only loads validated PCM WAV files.".to_string(),
        );
    }
    if file_size > MAX_PCM_FILE_BYTES {
        return (
            false,
            "This WAV exceeds the deck decoder's 512 MiB file-size limit.".to_string(),
        );
    }
    let reader = match WavReader::open(path) {
        Ok(reader) => reader,
        Err(_) => {
            return (
                false,
                "The deck decoder could not read this WAV header.".to_string(),
            )
        }
    };
    let spec = reader.spec();
    if spec.sample_format != SampleFormat::Int || !matches!(spec.bits_per_sample, 16 | 24 | 32) {
        return (
            false,
            "The deck decoder accepts only 16-, 24-, or 32-bit integer PCM WAV.".to_string(),
        );
    }
    if !matches!(spec.channels, 1 | 2) {
        return (
            false,
            "The deck decoder accepts only mono or stereo WAV.".to_string(),
        );
    }
    if !(MIN_PCM_SAMPLE_RATE_HZ..=MAX_PCM_SAMPLE_RATE_HZ).contains(&spec.sample_rate) {
        return (
            false,
            "The WAV sample rate is outside the deck decoder's supported range.".to_string(),
        );
    }
    let frames = u64::from(reader.duration()) / u64::from(spec.channels);
    if frames == 0 || frames > MAX_PCM_FRAMES {
        return (
            false,
            "The WAV duration is empty or exceeds the deck decoder's frame limit.".to_string(),
        );
    }
    (
        true,
        "PCM WAV candidate; the deck validates every sample when loaded.".to_string(),
    )
}

fn hash_file(path: &Path) -> Result<String, std::io::Error> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0; HASH_BUFFER_BYTES];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    let digest = hasher.finalize();
    let mut hex = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        write!(&mut hex, "{byte:02x}").expect("writing a digest byte into a String cannot fail");
    }
    Ok(hex)
}

fn hash_file_cancellable(path: &Path, cancellation: &AtomicBool) -> Result<Option<String>, String> {
    let metadata = fs::metadata(path)
        .map_err(|error| format!("Could not inspect the selected library file: {error}"))?;
    if metadata.len() > MAX_PCM_FILE_BYTES {
        return Err(format!(
            "This file is {} bytes; BPM analysis is limited to {} bytes",
            metadata.len(),
            MAX_PCM_FILE_BYTES
        ));
    }
    let mut file = File::open(path)
        .map_err(|error| format!("Could not read the selected library file: {error}"))?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0; HASH_BUFFER_BYTES];
    loop {
        if cancellation.load(Ordering::Acquire) {
            return Ok(None);
        }
        let count = file
            .read(&mut buffer)
            .map_err(|error| format!("Could not read the selected library file: {error}"))?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    let mut hex = String::with_capacity(64);
    for byte in hasher.finalize() {
        use std::fmt::Write as _;
        write!(&mut hex, "{byte:02x}").expect("writing a digest byte into a String cannot fail");
    }
    Ok(Some(hex))
}

fn analysis_progress(
    track_id: &str,
    state: &str,
    processed_frames: u64,
    total_frames: Option<u64>,
    candidate_bpm: Option<f64>,
    confidence: Option<f64>,
    message: &str,
) -> BpmAnalysisProgress {
    BpmAnalysisProgress {
        track_id: track_id.to_string(),
        state: state.to_string(),
        processed_frames,
        total_frames,
        candidate_bpm,
        confidence,
        message: message.to_string(),
    }
}

fn read_bpm_analysis(
    connection: &Connection,
    track_id: &str,
) -> Result<Option<BpmAnalysisRecord>, String> {
    connection
        .query_row(
            "SELECT track_id, candidate_bpm, confidence, status, algorithm_version, analyzed_at
             FROM bpm_analysis WHERE track_id = ?1",
            [track_id],
            |row| {
                Ok(BpmAnalysisRecord {
                    track_id: row.get(0)?,
                    candidate_bpm: row.get(1)?,
                    confidence: row.get(2)?,
                    status: row.get(3)?,
                    algorithm_version: row.get(4)?,
                    analyzed_at: row.get(5)?,
                })
            },
        )
        .optional()
        .map_err(|error| format!("Could not read the saved BPM analysis: {error}"))
}

fn is_candidate_audio(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| {
            SUPPORTED_AUDIO_EXTENSIONS
                .iter()
                .any(|known| extension.eq_ignore_ascii_case(known))
        })
        .unwrap_or(false)
}

fn file_format(path: &Path) -> String {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_uppercase)
        .unwrap_or_else(|| "AUDIO".to_string())
}

fn location_is_missing(path: &Path) -> bool {
    match fs::metadata(path) {
        Ok(metadata) => !metadata.is_file(),
        Err(error) => error.kind() == std::io::ErrorKind::NotFound,
    }
}

fn image_mime_type(data: &[u8]) -> Option<&'static str> {
    if data.starts_with(b"\xFF\xD8\xFF") {
        Some("image/jpeg")
    } else if data.starts_with(b"\x89PNG\r\n\x1A\n") {
        Some("image/png")
    } else if data.len() >= 12 && &data[..4] == b"RIFF" && &data[8..12] == b"WEBP" {
        Some("image/webp")
    } else {
        None
    }
}

fn safe_parse_error(error: impl std::fmt::Display) -> String {
    let _ = error;
    "Metadata could not be read. The file will remain visible and cannot load until the file itself is valid.".to_string()
}

fn searchable_text(
    title: &str,
    artist: Option<&str>,
    album: Option<&str>,
    format: &str,
    path: &Path,
) -> String {
    format!(
        "{} {} {} {} {}",
        title,
        artist.unwrap_or_default(),
        album.unwrap_or_default(),
        format,
        path.to_string_lossy()
    )
    .to_lowercase()
}

fn validate_collection_name(name: &str) -> Result<String, String> {
    let name = name.trim();
    let length = name.chars().count();
    if length == 0 || length > MAX_COLLECTION_NAME_CHARS {
        return Err(format!(
            "Collection names must contain 1 to {MAX_COLLECTION_NAME_CHARS} characters"
        ));
    }
    if name.chars().any(char::is_control) {
        return Err("Collection names cannot contain control characters".to_string());
    }
    Ok(name.to_string())
}

fn collection_error(action: &str, error: rusqlite::Error) -> String {
    match error {
        rusqlite::Error::SqliteFailure(code, _)
            if code.extended_code == rusqlite::ffi::SQLITE_CONSTRAINT_UNIQUE =>
        {
            format!(
                "A crate or playlist named this already exists; choose another name to {action} it"
            )
        }
        other => format!("Could not {action} the music collection: {other}"),
    }
}

fn normalize_collection_positions(
    transaction: &Transaction<'_>,
    collection_id: &str,
) -> Result<(), String> {
    let mut statement = transaction
        .prepare("SELECT track_id FROM collection_items WHERE collection_id = ?1 ORDER BY position, track_id")
        .map_err(|error| format!("Could not read collection order: {error}"))?;
    let ids = statement
        .query_map([collection_id], |row| row.get::<_, String>(0))
        .map_err(|error| format!("Could not read collection order: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Could not read a collection item: {error}"))?;
    drop(statement);
    for (position, track_id) in ids.iter().enumerate() {
        transaction
            .execute(
                "UPDATE collection_items SET position = ?3 WHERE collection_id = ?1 AND track_id = ?2",
                params![collection_id, track_id, position as i64],
            )
            .map_err(|error| format!("Could not save collection order: {error}"))?;
    }
    Ok(())
}

fn progress(
    state: &str,
    roots_total: u32,
    roots_done: u32,
    summary: &ScanSummary,
    current_file: Option<String>,
    message: &str,
) -> ScanProgress {
    ScanProgress {
        state: state.to_string(),
        roots_total,
        roots_done,
        partial_roots: summary.partial_roots,
        files_seen: summary.files_seen,
        tracks_indexed: summary.tracks_indexed,
        unsupported: summary.unsupported,
        unreadable: summary.unreadable,
        traversal_errors: summary.traversal_errors,
        current_file,
        message: message.to_string(),
    }
}

fn unix_time_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(i64::MAX as u128) as i64)
        .unwrap_or_default()
}

fn new_collection_id() -> String {
    let sequence = NEXT_COLLECTION_ID.fetch_add(1, Ordering::Relaxed);
    format!(
        "collection:{}:{}:{sequence}",
        unix_time_millis(),
        std::process::id()
    )
}

fn nonempty(value: &str) -> Option<&str> {
    let value = value.trim();
    (!value.is_empty()).then_some(value)
}

/// A clonable scan-control handle used by the Tauri command layer.
#[derive(Clone, Default)]
pub struct ScanCancellation(Arc<AtomicBool>);

impl ScanCancellation {
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Relaxed);
    }

    pub fn reset(&self) {
        self.0.store(false, Ordering::Relaxed);
    }

    pub fn flag(&self) -> &AtomicBool {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        io::Write,
        sync::atomic::{AtomicU64, Ordering},
    };

    static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(1);

    struct FixtureDirectory(PathBuf);

    impl FixtureDirectory {
        fn new() -> Self {
            let id = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
            let path =
                std::env::temp_dir().join(format!("sayt-library-test-{}-{id}", std::process::id()));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for FixtureDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn write_pcm_wav(path: &Path, sample: i16) {
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 48_000,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create(path, spec).unwrap();
        for _ in 0..4_800 {
            writer.write_sample(sample).unwrap();
        }
        writer.finalize().unwrap();
    }

    fn write_click_wav(path: &Path, bpm: f64, seconds: usize) {
        const SAMPLE_RATE: u32 = 48_000;
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: SAMPLE_RATE,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create(path, spec).unwrap();
        let samples_per_beat = (f64::from(SAMPLE_RATE) * 60.0 / bpm).round() as usize;
        for sample_index in 0..SAMPLE_RATE as usize * seconds {
            let within_beat = sample_index % samples_per_beat;
            let sample = if within_beat < 240 {
                let envelope = 1.0 - within_beat as f64 / 240.0;
                (i16::MAX as f64 * envelope * (within_beat as f64 * 0.1).sin()) as i16
            } else {
                0_i16
            };
            writer.write_sample(sample).unwrap();
        }
        writer.finalize().unwrap();
    }

    fn query_all() -> TrackQuery {
        TrackQuery {
            search: String::new(),
            filter: TrackFilter::All,
            collection_id: None,
            folder_id: None,
            sort: "title".to_string(),
            descending: false,
            offset: 0,
            limit: SEARCH_PAGE_SIZE,
        }
    }

    #[test]
    fn imports_real_file_metadata_and_keeps_source_unchanged() {
        let fixture = FixtureDirectory::new();
        let root = fixture.path().join("music");
        fs::create_dir(&root).unwrap();
        let wav = root.join("Night Drive.wav");
        write_pcm_wav(&wav, 4_000);
        let original_bytes = fs::read(&wav).unwrap();
        let catalog = LibraryCatalog::open(fixture.path().join("app-data")).unwrap();
        let summary = catalog
            .scan_roots(std::slice::from_ref(&root), &AtomicBool::new(false), |_| {})
            .unwrap();
        assert_eq!(summary.tracks_indexed, 1);
        let page = catalog.list_tracks(&query_all()).unwrap();
        assert_eq!(page.total, 1);
        assert_eq!(page.items[0].title, "Night Drive");
        assert!(page.items[0].load_supported);
        assert_eq!(fs::read(wav).unwrap(), original_bytes);
    }

    #[test]
    fn identical_content_is_one_track_with_two_locations_and_distinct_content_stays_separate() {
        let fixture = FixtureDirectory::new();
        let root = fixture.path().join("music");
        fs::create_dir(&root).unwrap();
        write_pcm_wav(&root.join("same-name.wav"), 4_000);
        fs::copy(root.join("same-name.wav"), root.join("duplicate.wav")).unwrap();
        write_pcm_wav(&root.join("same-name-copy.wav"), -4_000);
        let catalog = LibraryCatalog::open(fixture.path().join("app-data")).unwrap();
        catalog
            .scan_roots(std::slice::from_ref(&root), &AtomicBool::new(false), |_| {})
            .unwrap();
        let page = catalog.list_tracks(&query_all()).unwrap();
        assert_eq!(page.total, 2);
        assert_eq!(
            page.items
                .iter()
                .map(|track| track.duplicate_locations)
                .max(),
            Some(2)
        );
    }

    #[test]
    fn rescanning_a_renamed_file_recovers_its_content_identity() {
        let fixture = FixtureDirectory::new();
        let root = fixture.path().join("music");
        fs::create_dir(&root).unwrap();
        let before = root.join("before.wav");
        let after = root.join("after.wav");
        write_pcm_wav(&before, 4_000);
        let catalog = LibraryCatalog::open(fixture.path().join("app-data")).unwrap();
        catalog
            .scan_roots(std::slice::from_ref(&root), &AtomicBool::new(false), |_| {})
            .unwrap();
        let id_before = catalog.list_tracks(&query_all()).unwrap().items[0]
            .id
            .clone();
        fs::rename(&before, &after).unwrap();
        catalog
            .scan_roots(&[root], &AtomicBool::new(false), |_| {})
            .unwrap();
        let track = catalog.list_tracks(&query_all()).unwrap().items.remove(0);
        assert_eq!(track.id, id_before);
        assert!(Path::new(&track.path).ends_with("after.wav"));
        assert!(!track.missing);
    }

    #[test]
    fn unsupported_or_malformed_files_are_visible_but_cannot_load() {
        let fixture = FixtureDirectory::new();
        let root = fixture.path().join("music");
        fs::create_dir(&root).unwrap();
        fs::write(root.join("song.mp3"), b"not an MP3").unwrap();
        let float_wav = root.join("float.wav");
        let spec = hound::WavSpec {
            channels: 2,
            sample_rate: 48_000,
            bits_per_sample: 32,
            sample_format: SampleFormat::Float,
        };
        let mut writer = hound::WavWriter::create(&float_wav, spec).unwrap();
        writer.write_sample(0.2_f32).unwrap();
        writer.write_sample(-0.2_f32).unwrap();
        writer.finalize().unwrap();
        let catalog = LibraryCatalog::open(fixture.path().join("app-data")).unwrap();
        catalog
            .scan_roots(&[root], &AtomicBool::new(false), |_| {})
            .unwrap();
        let page = catalog.list_tracks(&query_all()).unwrap();
        assert_eq!(page.total, 2);
        assert!(page.items.iter().all(|track| !track.load_supported));
        assert!(page
            .items
            .iter()
            .any(|track| track.import_state == "unreadable"));
    }

    #[test]
    fn collections_persist_membership_and_playlist_order_and_never_touch_audio() {
        let fixture = FixtureDirectory::new();
        let root = fixture.path().join("music");
        fs::create_dir(&root).unwrap();
        write_pcm_wav(&root.join("one.wav"), 100);
        write_pcm_wav(&root.join("two.wav"), 200);
        let audio_before = fs::read(root.join("one.wav")).unwrap();
        let data = fixture.path().join("app-data");
        let catalog = LibraryCatalog::open(&data).unwrap();
        catalog
            .scan_roots(std::slice::from_ref(&root), &AtomicBool::new(false), |_| {})
            .unwrap();
        let mut tracks = catalog.list_tracks(&query_all()).unwrap().items;
        tracks.sort_by(|left, right| left.title.cmp(&right.title));
        let playlist = catalog.create_collection("Warmup", "playlist").unwrap();
        catalog
            .add_track_to_collection(&playlist.id, &tracks[0].id)
            .unwrap();
        catalog
            .add_track_to_collection(&playlist.id, &tracks[1].id)
            .unwrap();
        catalog
            .move_collection_track(&playlist.id, &tracks[1].id, -1)
            .unwrap();
        let mut list_query = query_all();
        list_query.collection_id = Some(playlist.id.clone());
        assert_eq!(
            catalog.list_tracks(&list_query).unwrap().items[0].id,
            tracks[1].id
        );
        drop(catalog);
        let reopened = LibraryCatalog::open(&data).unwrap();
        assert_eq!(reopened.list_collections().unwrap()[0].item_count, 2);
        reopened.delete_collection(&playlist.id).unwrap();
        assert!(reopened.list_collections().unwrap().is_empty());
        assert_eq!(fs::read(root.join("one.wav")).unwrap(), audio_before);
    }

    #[test]
    fn scan_cancellation_stops_before_visiting_a_root() {
        let fixture = FixtureDirectory::new();
        let root = fixture.path().join("music");
        fs::create_dir(&root).unwrap();
        write_pcm_wav(&root.join("one.wav"), 100);
        let catalog = LibraryCatalog::open(fixture.path().join("app-data")).unwrap();
        let cancellation = AtomicBool::new(true);
        let summary = catalog.scan_roots(&[root], &cancellation, |_| {}).unwrap();
        assert!(summary.cancelled);
        assert_eq!(summary.files_seen, 0);
    }

    #[test]
    fn cancelled_partial_scan_keeps_old_files_present_until_a_complete_reconcile() {
        let fixture = FixtureDirectory::new();
        let root = fixture.path().join("music");
        fs::create_dir(&root).unwrap();
        let old_file = root.join("Old.wav");
        write_pcm_wav(&old_file, 100);
        let catalog = LibraryCatalog::open(fixture.path().join("app-data")).unwrap();
        catalog
            .scan_roots(std::slice::from_ref(&root), &AtomicBool::new(false), |_| {})
            .unwrap();
        fs::remove_file(&old_file).unwrap();
        write_pcm_wav(&root.join("One.wav"), 110);
        write_pcm_wav(&root.join("Two.wav"), 120);
        let cancellation = AtomicBool::new(false);
        let summary = catalog
            .scan_roots(std::slice::from_ref(&root), &cancellation, |progress| {
                if progress.current_file.is_some() && progress.tracks_indexed >= 1 {
                    cancellation.store(true, Ordering::Relaxed);
                }
            })
            .unwrap();

        assert!(summary.cancelled);
        assert_eq!(summary.tracks_indexed, 2);
        let mut old_query = query_all();
        old_query.search = "old".to_string();
        let old_track = catalog.list_tracks(&old_query).unwrap().items.remove(0);
        assert!(!old_track.missing);
        assert_eq!(catalog.list_tracks(&query_all()).unwrap().total, 3);
    }

    #[test]
    fn unreadable_candidate_makes_root_partial_and_does_not_mark_old_location_missing() {
        let fixture = FixtureDirectory::new();
        let root = fixture.path().join("music");
        fs::create_dir(&root).unwrap();
        let disappearing = root.join("Disappearing.wav");
        write_pcm_wav(&disappearing, 100);
        let catalog = LibraryCatalog::open(fixture.path().join("app-data")).unwrap();
        catalog
            .scan_roots(std::slice::from_ref(&root), &AtomicBool::new(false), |_| {})
            .unwrap();

        let mut removed = false;
        let mut final_state = None;
        let summary = catalog
            .scan_roots(
                std::slice::from_ref(&root),
                &AtomicBool::new(false),
                |progress| {
                    if progress.state != "scanning" {
                        final_state = Some(progress.state.clone());
                    }
                    if progress
                        .current_file
                        .as_deref()
                        .is_some_and(|path| path.ends_with("Disappearing.wav"))
                        && !removed
                    {
                        fs::remove_file(&disappearing).unwrap();
                        removed = true;
                    }
                },
            )
            .unwrap();

        assert_eq!(summary.unreadable, 1);
        assert_eq!(summary.roots_scanned, 1);
        assert_eq!(summary.partial_roots, 1);
        assert_eq!(final_state.as_deref(), Some("partial"));
        let connection = catalog.connection().unwrap();
        let scan_state: String = connection
            .query_row("SELECT scan_state FROM roots", [], |row| row.get(0))
            .unwrap();
        assert_eq!(scan_state, "partial");
        let mut query = query_all();
        query.search = "disappearing".to_string();
        assert!(!catalog.list_tracks(&query).unwrap().items[0].missing);
    }

    #[test]
    fn search_sort_filters_favorite_and_user_metadata_work_on_scanned_files() {
        let fixture = FixtureDirectory::new();
        let root = fixture.path().join("music");
        fs::create_dir(&root).unwrap();
        write_pcm_wav(&root.join("Alpha.wav"), 100);
        write_pcm_wav(&root.join("Zebra.wav"), 200);
        let catalog = LibraryCatalog::open(fixture.path().join("app-data")).unwrap();
        catalog
            .scan_roots(std::slice::from_ref(&root), &AtomicBool::new(false), |_| {})
            .unwrap();

        let mut descending = query_all();
        descending.sort = "title".to_string();
        descending.descending = true;
        assert_eq!(
            catalog.list_tracks(&descending).unwrap().items[0].title,
            "Zebra"
        );

        let mut search = query_all();
        search.search = "alpha".to_string();
        let alpha = catalog.list_tracks(&search).unwrap().items.remove(0);
        assert_eq!(alpha.title, "Alpha");
        catalog
            .update_user_metadata(&alpha.id, "After Hours", "Local Artist", "Test Album")
            .unwrap();
        let mut edited_search = query_all();
        edited_search.search = "after hours".to_string();
        assert_eq!(
            catalog.list_tracks(&edited_search).unwrap().items[0].id,
            alpha.id
        );

        catalog.set_favorite(&alpha.id, true).unwrap();
        let mut favorites = query_all();
        favorites.filter = TrackFilter::Favorites;
        assert_eq!(catalog.list_tracks(&favorites).unwrap().total, 1);
        let mut unanalyzed = query_all();
        unanalyzed.filter = TrackFilter::Unanalyzed;
        assert_eq!(catalog.list_tracks(&unanalyzed).unwrap().total, 2);
        let folder = catalog.list_folders().unwrap().remove(0);
        let mut folder_query = query_all();
        folder_query.folder_id = Some(folder.id);
        assert_eq!(catalog.list_tracks(&folder_query).unwrap().total, 2);
    }

    #[test]
    fn bpm_analysis_and_manual_correction_persist_as_separate_values() {
        let fixture = FixtureDirectory::new();
        let root = fixture.path().join("music");
        fs::create_dir(&root).unwrap();
        write_click_wav(&root.join("Tempo test.wav"), 120.0, 16);
        let data = fixture.path().join("app-data");
        let catalog = LibraryCatalog::open(&data).unwrap();
        catalog
            .scan_roots(std::slice::from_ref(&root), &AtomicBool::new(false), |_| {})
            .unwrap();
        let track = catalog.list_tracks(&query_all()).unwrap().items.remove(0);

        let mut progress = Vec::new();
        let analyzed = catalog
            .analyze_bpm(&track.id, &AtomicBool::new(false), |item| {
                progress.push(item.state);
            })
            .unwrap()
            .unwrap();
        let candidate = analyzed
            .candidate_bpm
            .expect("the fixture has a periodic pulse");
        assert!(
            (candidate - 60.0).abs() < 2.0 || (candidate - 120.0).abs() < 2.0,
            "candidate was {candidate}"
        );
        assert_eq!(analyzed.status, "uncertain");
        assert!(progress.iter().any(|state| state == "analyzing"));
        assert_eq!(progress.last().map(String::as_str), Some("uncertain"));

        catalog.set_bpm_correction(&track.id, Some(124.5)).unwrap();
        let saved = catalog.list_tracks(&query_all()).unwrap().items.remove(0);
        assert_eq!(saved.bpm, None, "analysis does not overwrite the file tag");
        assert_eq!(saved.bpm_analysis_candidate, analyzed.candidate_bpm);
        assert_eq!(saved.bpm_analysis_status.as_deref(), Some("uncertain"));
        assert_eq!(saved.bpm_correction, Some(124.5));
        assert_eq!(catalog.playback_bpm(&track.id).unwrap(), Some(124.5));

        drop(catalog);
        let reopened = LibraryCatalog::open(data).unwrap();
        let restored = reopened.list_tracks(&query_all()).unwrap().items.remove(0);
        assert_eq!(restored.bpm_analysis_candidate, analyzed.candidate_bpm);
        assert_eq!(restored.bpm_correction, Some(124.5));
        let cached = reopened
            .analyze_bpm(&track.id, &AtomicBool::new(false), |_| {})
            .unwrap()
            .unwrap();
        assert_eq!(cached, analyzed);
        reopened.set_bpm_correction(&track.id, None).unwrap();
        assert_eq!(
            reopened.list_tracks(&query_all()).unwrap().items[0].bpm_correction,
            None
        );
        assert_eq!(reopened.playback_bpm(&track.id).unwrap(), None);
    }

    #[test]
    fn confident_analysis_is_available_to_the_existing_deck_bpm_control() {
        let fixture = FixtureDirectory::new();
        let root = fixture.path().join("music");
        fs::create_dir(&root).unwrap();
        write_click_wav(&root.join("Slow pulse.wav"), 60.0, 16);
        let catalog = LibraryCatalog::open(fixture.path().join("app-data")).unwrap();
        catalog
            .scan_roots(std::slice::from_ref(&root), &AtomicBool::new(false), |_| {})
            .unwrap();
        let track = catalog.list_tracks(&query_all()).unwrap().items.remove(0);
        let result = catalog
            .analyze_bpm(&track.id, &AtomicBool::new(false), |_| {})
            .unwrap()
            .unwrap();
        assert_eq!(result.status, "estimated");
        assert_eq!(
            catalog.playback_bpm(&track.id).unwrap(),
            result.candidate_bpm
        );
    }

    #[test]
    fn cancelled_and_changed_source_analysis_never_persist_a_false_result() {
        let fixture = FixtureDirectory::new();
        let root = fixture.path().join("music");
        fs::create_dir(&root).unwrap();
        let wav = root.join("Tempo test.wav");
        write_click_wav(&wav, 120.0, 16);
        let catalog = LibraryCatalog::open(fixture.path().join("app-data")).unwrap();
        catalog
            .scan_roots(std::slice::from_ref(&root), &AtomicBool::new(false), |_| {})
            .unwrap();
        let track = catalog.list_tracks(&query_all()).unwrap().items.remove(0);

        let cancelled = AtomicBool::new(true);
        let mut final_state = String::new();
        assert!(catalog
            .analyze_bpm(&track.id, &cancelled, |progress| {
                final_state = progress.state;
            })
            .unwrap()
            .is_none());
        assert_eq!(final_state, "cancelled");
        assert!(catalog.list_tracks(&query_all()).unwrap().items[0]
            .bpm_analysis_status
            .is_none());

        write_pcm_wav(&wav, 2_000);
        assert!(catalog
            .analyze_bpm(&track.id, &AtomicBool::new(false), |_| {})
            .unwrap_err()
            .contains("changed after it was added"));
        assert!(catalog.list_tracks(&query_all()).unwrap().items[0]
            .bpm_analysis_status
            .is_none());
    }

    #[test]
    fn bpm_correction_enforces_manual_range_and_requires_a_catalog_track() {
        let fixture = FixtureDirectory::new();
        let root = fixture.path().join("music");
        fs::create_dir(&root).unwrap();
        write_pcm_wav(&root.join("Track.wav"), 2_000);
        let catalog = LibraryCatalog::open(fixture.path().join("app-data")).unwrap();
        catalog
            .scan_roots(std::slice::from_ref(&root), &AtomicBool::new(false), |_| {})
            .unwrap();
        let track = catalog.list_tracks(&query_all()).unwrap().items.remove(0);

        catalog.set_bpm_correction(&track.id, Some(30.0)).unwrap();
        catalog.set_bpm_correction(&track.id, Some(300.0)).unwrap();
        assert!(catalog.set_bpm_correction(&track.id, Some(29.9)).is_err());
        assert!(catalog.set_bpm_correction(&track.id, Some(300.1)).is_err());
        assert!(catalog
            .set_bpm_correction(&track.id, Some(f64::NAN))
            .is_err());
        assert!(catalog.set_bpm_correction("missing", Some(120.0)).is_err());
    }

    #[test]
    fn schema_v1_migration_preserves_tracks_and_adds_analysis_tables() {
        let fixture = FixtureDirectory::new();
        let root = fixture.path().join("music");
        fs::create_dir(&root).unwrap();
        write_pcm_wav(&root.join("Existing.wav"), 2_000);
        let data = fixture.path().join("app-data");
        let catalog = LibraryCatalog::open(&data).unwrap();
        catalog
            .scan_roots(std::slice::from_ref(&root), &AtomicBool::new(false), |_| {})
            .unwrap();
        let track_id = catalog
            .list_tracks(&query_all())
            .unwrap()
            .items
            .remove(0)
            .id;
        drop(catalog);

        let database = Connection::open(data.join("music-library.sqlite3")).unwrap();
        database
            .execute_batch(
                "DROP TABLE bpm_corrections;
                 DROP TABLE bpm_analysis;
                 PRAGMA user_version = 1;",
            )
            .unwrap();
        drop(database);

        let migrated = LibraryCatalog::open(&data).unwrap();
        assert_eq!(
            migrated.list_tracks(&query_all()).unwrap().items[0].id,
            track_id
        );
        let connection = migrated.connection().unwrap();
        assert_eq!(
            connection
                .pragma_query_value(None, "user_version", |row| row.get::<_, i32>(0))
                .unwrap(),
            SCHEMA_VERSION
        );
        let analysis_table: String = connection
            .query_row(
                "SELECT name FROM sqlite_master WHERE type = 'table' AND name = 'bpm_analysis'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(analysis_table, "bpm_analysis");
    }

    #[test]
    fn search_accepts_the_character_limit_and_rejects_the_next_character() {
        let fixture = FixtureDirectory::new();
        let catalog = LibraryCatalog::open(fixture.path().join("app-data")).unwrap();
        let mut query = query_all();
        query.search = "x".repeat(MAX_SEARCH_CHARS);
        assert!(catalog.list_tracks(&query).is_ok());
        query.search.push('x');
        assert_eq!(
            catalog.list_tracks(&query).unwrap_err(),
            format!("Search is limited to {MAX_SEARCH_CHARS} characters")
        );
    }

    #[test]
    fn missing_files_persist_across_catalog_reopen() {
        let fixture = FixtureDirectory::new();
        let root = fixture.path().join("music");
        fs::create_dir(&root).unwrap();
        let wav = root.join("lost.wav");
        write_pcm_wav(&wav, 100);
        let data = fixture.path().join("app-data");
        let catalog = LibraryCatalog::open(&data).unwrap();
        catalog
            .scan_roots(std::slice::from_ref(&root), &AtomicBool::new(false), |_| {})
            .unwrap();
        fs::remove_file(wav).unwrap();
        drop(catalog);
        let reopened = LibraryCatalog::open(data).unwrap();
        reopened.refresh_missing_state().unwrap();
        let page = reopened.list_tracks(&query_all()).unwrap();
        assert!(page.items[0].missing);
        let mut missing = query_all();
        missing.filter = TrackFilter::Missing;
        assert_eq!(reopened.list_tracks(&missing).unwrap().total, 1);
    }

    #[test]
    fn paged_search_scales_to_ten_thousand_catalog_entries() {
        let fixture = FixtureDirectory::new();
        let data = fixture.path().join("app-data");
        let catalog = LibraryCatalog::open(&data).unwrap();
        let connection = catalog.connection().unwrap();
        let transaction = connection.unchecked_transaction().unwrap();
        for index in 0..10_000_u32 {
            let title = format!("Track {index:05}");
            let search_text = title.to_lowercase();
            transaction.execute(
                "INSERT INTO tracks(id, title, duration_seconds, format, file_size, import_state, load_supported, load_status, search_text, first_seen_at, last_seen_at)
                 VALUES (?1, ?2, 180.0, 'WAV', 1000, 'metadata', 0, 'unsupported fixture', ?3, 1, 1)",
                params![format!("fixture:{index}"), title, search_text],
            ).unwrap();
        }
        transaction.commit().unwrap();
        let mut query = query_all();
        query.search = "track 09".to_string();
        query.limit = 25;
        let result = catalog.list_tracks(&query).unwrap();
        assert_eq!(result.total, 1_000);
        assert_eq!(result.items.len(), 25);
        assert_eq!(result.items[0].title, "Track 09000");
    }

    #[test]
    fn backup_is_an_integrity_checked_independent_sqlite_copy() {
        let fixture = FixtureDirectory::new();
        let catalog = LibraryCatalog::open(fixture.path().join("app-data")).unwrap();
        let backup_path = fixture.path().join("library-backup.sqlite3");
        catalog.backup_to(&backup_path).unwrap();
        let backup = Connection::open(backup_path).unwrap();
        let integrity = backup
            .query_row("PRAGMA quick_check", [], |row| row.get::<_, String>(0))
            .unwrap();
        assert_eq!(integrity, "ok");
        assert_eq!(
            backup
                .pragma_query_value(None, "user_version", |row| row.get::<_, i32>(0))
                .unwrap(),
            SCHEMA_VERSION
        );
    }

    #[test]
    fn corrupt_catalog_is_preserved_and_recreated_empty() {
        let fixture = FixtureDirectory::new();
        let data = fixture.path().join("app-data");
        fs::create_dir_all(&data).unwrap();
        let database = data.join("music-library.sqlite3");
        let mut file = File::create(&database).unwrap();
        file.write_all(b"not sqlite").unwrap();
        drop(file);
        let reopened = LibraryCatalog::open(&data).unwrap();
        assert!(reopened.list_tracks(&query_all()).unwrap().items.is_empty());
        let preserved = fs::read_dir(&data)
            .unwrap()
            .filter_map(Result::ok)
            .any(|entry| entry.file_name().to_string_lossy().contains(".corrupt-"));
        assert!(preserved);
    }

    #[test]
    fn catalog_resolves_indexed_file_path_within_the_selected_root() {
        let fixture = FixtureDirectory::new();
        let root = fixture.path().join("music");
        fs::create_dir(&root).unwrap();
        let wav = root.join("test.wav");
        write_pcm_wav(&wav, 300);
        let catalog = LibraryCatalog::open(fixture.path().join("app-data")).unwrap();
        catalog
            .scan_roots(std::slice::from_ref(&root), &AtomicBool::new(false), |_| {})
            .unwrap();
        let track = catalog.list_tracks(&query_all()).unwrap().items.remove(0);
        assert_eq!(
            catalog.resolve_track_path(&track.id).unwrap(),
            wav.canonicalize().unwrap()
        );
    }
}
