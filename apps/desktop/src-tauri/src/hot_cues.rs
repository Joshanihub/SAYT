//! Versioned, source-identified local hot-cue persistence.

use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use sayt_engine::{empty_hot_cue_slots, validate_hot_cue_slots, HotCueSlots};
use serde::{Deserialize, Serialize};

const HOT_CUE_DOCUMENT_VERSION: u32 = 1;
const MAX_HOT_CUE_DOCUMENT_BYTES: u64 = 8 * 1024;
const MAX_HOT_CUE_TRACK_FILES: usize = 10_000;
const MAX_HOT_CUE_BACKUPS_PER_TRACK: usize = 3;
static TEMP_FILE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Deserialize, Serialize)]
struct HotCueDocument {
    version: u32,
    source_track_id: String,
    slots: HotCueSlots,
}

#[derive(Debug)]
pub struct HotCueLoad {
    pub slots: HotCueSlots,
    pub warning: Option<String>,
}

pub struct HotCueStore {
    directory: PathBuf,
}

impl HotCueStore {
    pub fn new(directory: PathBuf) -> Self {
        Self { directory }
    }

    pub fn load(&self, source_track_id: &str, duration_seconds: f64) -> Result<HotCueLoad, String> {
        validate_source_track_id(source_track_id)?;
        let document_path = self.document_path(source_track_id);
        let primary_error = match read_document(&document_path, source_track_id, duration_seconds) {
            Ok(Some(slots)) => {
                return Ok(HotCueLoad {
                    slots,
                    warning: None,
                });
            }
            Ok(None) => None,
            Err(error) => Some(error),
        };

        for backup_path in self.backup_paths(source_track_id) {
            if let Ok(Some(slots)) = read_document(&backup_path, source_track_id, duration_seconds)
            {
                return Ok(HotCueLoad {
                    slots,
                    warning: Some(
                        "Recovered hot cues from a local backup after an interrupted or damaged save."
                            .to_string(),
                    ),
                });
            }
        }

        if let Some(error) = primary_error {
            return Err(error);
        }

        Ok(HotCueLoad {
            slots: empty_hot_cue_slots(),
            warning: None,
        })
    }

    pub fn save(
        &self,
        source_track_id: &str,
        duration_seconds: f64,
        slots: &HotCueSlots,
    ) -> Result<(), String> {
        validate_source_track_id(source_track_id)?;
        validate_hot_cue_slots(slots, duration_seconds)?;
        fs::create_dir_all(&self.directory).map_err(|_| {
            "Could not create SAYT's local hot-cue storage. Check disk space and permissions."
                .to_string()
        })?;

        let document_path = self.document_path(source_track_id);
        if !document_path.exists() && self.count_documents()? >= MAX_HOT_CUE_TRACK_FILES {
            return Err(format!(
                "SAYT has reached its local hot-cue limit of {MAX_HOT_CUE_TRACK_FILES} tracks."
            ));
        }

        let document = HotCueDocument {
            version: HOT_CUE_DOCUMENT_VERSION,
            source_track_id: source_track_id.to_string(),
            slots: slots.clone(),
        };
        let serialized = serde_json::to_vec(&document)
            .map_err(|_| "Could not encode the hot-cue data for saving.".to_string())?;
        if serialized.len() as u64 > MAX_HOT_CUE_DOCUMENT_BYTES {
            return Err("The hot-cue data exceeds SAYT's local storage limit.".to_string());
        }

        self.replace_document(&document_path, &serialized)
    }

    fn document_path(&self, source_track_id: &str) -> PathBuf {
        let digest = source_track_id
            .strip_prefix("sha256:")
            .expect("source ID is validated before building its path");
        self.directory.join(format!("{digest}.json"))
    }

    fn count_documents(&self) -> Result<usize, String> {
        let entries = match fs::read_dir(&self.directory) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(0),
            Err(_) => {
                return Err(
                    "Could not inspect SAYT's local hot-cue storage. Check its permissions."
                        .to_string(),
                );
            }
        };
        count_json_documents(entries.map(|entry| entry.map(|entry| entry.path()))).map_err(|_| {
            "Could not inspect SAYT's local hot-cue storage. Check its permissions.".to_string()
        })
    }

    fn backup_paths(&self, source_track_id: &str) -> Vec<PathBuf> {
        let Ok(entries) = fs::read_dir(&self.directory) else {
            return Vec::new();
        };
        let digest = source_track_id
            .strip_prefix("sha256:")
            .expect("source ID is validated before building its path");
        let prefix = format!("{digest}.json.backup-");
        let mut backups = entries
            .filter_map(Result::ok)
            .filter_map(|entry| {
                let name = entry.file_name();
                name.to_str()
                    .filter(|name| name.starts_with(&prefix))
                    .map(|_| entry.path())
            })
            .take(MAX_HOT_CUE_TRACK_FILES * MAX_HOT_CUE_BACKUPS_PER_TRACK)
            .collect::<Vec<_>>();
        backups.sort();
        backups.reverse();
        backups
    }

    fn replace_document(&self, document_path: &Path, serialized: &[u8]) -> Result<(), String> {
        let temporary_path = self.unique_sibling(document_path, "tmp");
        let mut temporary_file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary_path)
            .map_err(|_| {
                "Could not create a temporary hot-cue save. Check disk space and permissions."
                    .to_string()
            })?;
        if temporary_file.write_all(serialized).is_err() || temporary_file.sync_all().is_err() {
            let _ = fs::remove_file(&temporary_path);
            return Err("Could not write the hot-cue data to local storage.".to_string());
        }
        drop(temporary_file);

        let previous_path = if fs::symlink_metadata(document_path).is_ok() {
            let metadata = fs::symlink_metadata(document_path)
                .map_err(|_| "Could not inspect the existing hot-cue data.".to_string())?;
            if !metadata.file_type().is_file() {
                let _ = fs::remove_file(&temporary_path);
                return Err("The existing hot-cue storage entry is not a regular file.".to_string());
            }
            let backup_path = self.unique_sibling(document_path, "backup");
            if fs::rename(document_path, &backup_path).is_err() {
                let _ = fs::remove_file(&temporary_path);
                return Err(
                    "Could not preserve the previous hot-cue data before saving.".to_string(),
                );
            }
            Some(backup_path)
        } else {
            None
        };

        if fs::rename(&temporary_path, document_path).is_err() {
            if let Some(previous_path) = &previous_path {
                let _ = fs::rename(previous_path, document_path);
            }
            let _ = fs::remove_file(&temporary_path);
            return Err(
                "Could not finish saving hot cues. The previous data was restored where possible."
                    .to_string(),
            );
        }

        if let Some(previous_path) = previous_path {
            self.prune_backups(document_path, &previous_path);
        }
        Ok(())
    }

    fn prune_backups(&self, document_path: &Path, newest_backup: &Path) {
        let Some(filename) = document_path.file_name().and_then(|name| name.to_str()) else {
            return;
        };
        let prefix = format!("{filename}.backup-");
        let Ok(entries) = fs::read_dir(&self.directory) else {
            return;
        };
        let mut backups = entries
            .filter_map(Result::ok)
            .filter_map(|entry| {
                let name = entry.file_name();
                name.to_str()
                    .filter(|name| name.starts_with(&prefix))
                    .map(|_| entry.path())
            })
            .collect::<Vec<_>>();
        backups.sort();
        backups.reverse();
        let mut retained_previous = 0;
        for backup in backups {
            if backup == newest_backup {
                continue;
            }
            if retained_previous < MAX_HOT_CUE_BACKUPS_PER_TRACK - 1 {
                retained_previous += 1;
            } else {
                let _ = fs::remove_file(backup);
            }
        }
    }

    fn unique_sibling(&self, document_path: &Path, kind: &str) -> PathBuf {
        let sequence = TEMP_FILE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let filename = document_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("hot-cues.json");
        self.directory.join(format!(
            "{filename}.{kind}-{timestamp}-{}-{sequence}",
            std::process::id(),
        ))
    }
}

fn count_json_documents(
    entries: impl IntoIterator<Item = std::io::Result<PathBuf>>,
) -> std::io::Result<usize> {
    let mut document_count = 0;
    for entry in entries {
        let path = entry?;
        if path
            .extension()
            .is_some_and(|extension| extension == "json")
        {
            document_count += 1;
            if document_count >= MAX_HOT_CUE_TRACK_FILES {
                return Ok(document_count);
            }
        }
    }
    Ok(document_count)
}

fn validate_source_track_id(source_track_id: &str) -> Result<(), String> {
    let Some(digest) = source_track_id.strip_prefix("sha256:") else {
        return Err("The selected track has no valid local content identity.".to_string());
    };
    if digest.len() != 64
        || !digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err("The selected track has no valid local content identity.".to_string());
    }
    Ok(())
}

fn read_document(
    path: &Path,
    expected_source_track_id: &str,
    duration_seconds: f64,
) -> Result<Option<HotCueSlots>, String> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => {
            return Err(
                "Saved hot cues could not be inspected. Check local app-data permissions."
                    .to_string(),
            );
        }
    };
    if !metadata.file_type().is_file() {
        return Err("Saved hot cues are not a regular file and were not loaded.".to_string());
    }
    if metadata.len() > MAX_HOT_CUE_DOCUMENT_BYTES {
        return Err(
            "Saved hot cues exceed the local storage limit and were not loaded.".to_string(),
        );
    }
    let bytes = fs::read(path).map_err(|_| {
        "Saved hot cues could not be read. Check local app-data permissions.".to_string()
    })?;
    let document: HotCueDocument = serde_json::from_slice(&bytes)
        .map_err(|_| "Saved hot cues are damaged or use a newer format. The existing file was left unchanged.".to_string())?;
    if document.version != HOT_CUE_DOCUMENT_VERSION {
        return Err(
            "Saved hot cues use an unsupported version. The existing file was left unchanged."
                .to_string(),
        );
    }
    if document.source_track_id != expected_source_track_id {
        return Err(
            "Saved hot cues belong to a different source track and were not loaded.".to_string(),
        );
    }
    validate_hot_cue_slots(&document.slots, duration_seconds).map_err(|_| {
        "Saved hot cues contain invalid positions or names and were not loaded.".to_string()
    })?;
    Ok(Some(document.slots))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sayt_engine::{default_hot_cue, HotCueColor, HOT_CUE_SLOT_COUNT};
    use std::sync::atomic::{AtomicUsize, Ordering};

    static TEST_DIRECTORY_SEQUENCE: AtomicUsize = AtomicUsize::new(0);

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            let sequence = TEST_DIRECTORY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "sayt-hot-cues-test-{}-{sequence}",
                std::process::id()
            ));
            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn source_id(digit: char) -> String {
        format!("sha256:{}", digit.to_string().repeat(64))
    }

    #[test]
    fn reloads_cues_from_a_new_store_instance_for_the_same_audio_identity() {
        let directory = TestDirectory::new();
        let source_track_id = source_id('a');
        let mut slots = empty_hot_cue_slots();
        slots[0] = Some(default_hot_cue(0, 2.25));
        slots[7] = Some(default_hot_cue(7, 9.75));

        HotCueStore::new(directory.path().to_path_buf())
            .save(&source_track_id, 10.0, &slots)
            .unwrap();
        let restarted_store = HotCueStore::new(directory.path().to_path_buf());
        let loaded = restarted_store.load(&source_track_id, 10.0).unwrap();

        assert_eq!(loaded.slots, slots);
        assert!(loaded.warning.is_none());
    }

    #[test]
    fn does_not_reuse_cues_for_another_content_identity_or_tampered_document() {
        let directory = TestDirectory::new();
        let source_track_id = source_id('a');
        let other_source_track_id = source_id('b');
        let mut slots = empty_hot_cue_slots();
        slots[2] = Some(default_hot_cue(2, 3.0));
        let store = HotCueStore::new(directory.path().to_path_buf());
        store.save(&source_track_id, 10.0, &slots).unwrap();

        assert_eq!(
            store.load(&other_source_track_id, 10.0).unwrap().slots,
            empty_hot_cue_slots()
        );

        let document_path = store.document_path(&source_track_id);
        let mut document: HotCueDocument =
            serde_json::from_slice(&fs::read(&document_path).unwrap()).unwrap();
        document.source_track_id = other_source_track_id;
        fs::write(&document_path, serde_json::to_vec(&document).unwrap()).unwrap();
        assert!(store.load(&source_track_id, 10.0).is_err());
    }

    #[test]
    fn rejects_corruption_without_overwriting_it_during_load_and_preserves_it_on_repair() {
        let directory = TestDirectory::new();
        let source_track_id = source_id('c');
        let store = HotCueStore::new(directory.path().to_path_buf());
        let document_path = store.document_path(&source_track_id);
        fs::write(&document_path, b"{broken").unwrap();

        assert!(store.load(&source_track_id, 10.0).is_err());
        assert_eq!(fs::read(&document_path).unwrap(), b"{broken");

        let mut slots = empty_hot_cue_slots();
        slots[1] = Some(default_hot_cue(1, 1.0));
        store.save(&source_track_id, 10.0, &slots).unwrap();
        assert_eq!(store.load(&source_track_id, 10.0).unwrap().slots, slots);
        assert_eq!(
            fs::read_dir(directory.path())
                .unwrap()
                .filter_map(Result::ok)
                .filter(|entry| entry.file_name().to_string_lossy().contains("backup-"))
                .count(),
            1
        );
    }

    #[test]
    fn recovers_a_valid_backup_after_a_interrupted_replace() {
        let directory = TestDirectory::new();
        let source_track_id = source_id('d');
        let store = HotCueStore::new(directory.path().to_path_buf());
        let mut slots = empty_hot_cue_slots();
        slots[3] = Some(default_hot_cue(3, 4.0));
        store.save(&source_track_id, 10.0, &slots).unwrap();
        let document_path = store.document_path(&source_track_id);
        let recovery_path = store.unique_sibling(&document_path, "backup");
        fs::rename(document_path, recovery_path).unwrap();

        let recovered = store.load(&source_track_id, 10.0).unwrap();
        assert_eq!(recovered.slots, slots);
        assert!(recovered.warning.is_some());
    }

    #[test]
    fn bounds_historical_backups_per_track() {
        let directory = TestDirectory::new();
        let source_track_id = source_id('b');
        let store = HotCueStore::new(directory.path().to_path_buf());
        for position in 1..=8 {
            let mut slots = empty_hot_cue_slots();
            slots[0] = Some(default_hot_cue(0, f64::from(position)));
            store.save(&source_track_id, 10.0, &slots).unwrap();
        }

        let backup_count = fs::read_dir(directory.path())
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name().to_string_lossy().contains("backup-"))
            .count();
        assert_eq!(backup_count, MAX_HOT_CUE_BACKUPS_PER_TRACK);
    }

    #[test]
    fn counts_json_documents_after_more_non_json_entries_than_the_track_limit() {
        let noise_entries = (0..MAX_HOT_CUE_TRACK_FILES + 1)
            .map(|index| Ok(PathBuf::from(format!("entry-{index}.backup"))));
        let document_entries = (0..MAX_HOT_CUE_TRACK_FILES)
            .map(|index| Ok(PathBuf::from(format!("track-{index}.json"))));

        let document_count = count_json_documents(noise_entries.chain(document_entries)).unwrap();

        assert_eq!(document_count, MAX_HOT_CUE_TRACK_FILES);
    }

    #[test]
    fn validates_track_identity_slot_bounds_and_stored_positions() {
        let directory = TestDirectory::new();
        let store = HotCueStore::new(directory.path().to_path_buf());
        assert!(store.load("../../other", 10.0).is_err());

        let mut slots = empty_hot_cue_slots();
        slots[HOT_CUE_SLOT_COUNT - 1] = Some(default_hot_cue(7, 10.01));
        assert!(store.save(&source_id('e'), 10.0, &slots).is_err());

        let mut invalid_name = empty_hot_cue_slots();
        invalid_name[0] = Some(sayt_engine::HotCue {
            name: "\0bad".to_string(),
            color: HotCueColor::Coral,
            position_seconds: 0.0,
        });
        assert!(store.save(&source_id('f'), 10.0, &invalid_name).is_err());
    }
}
