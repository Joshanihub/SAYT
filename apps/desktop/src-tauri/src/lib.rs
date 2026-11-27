//! Tauri commands for device selection and the local two-deck workflow.

use std::{
    error::Error,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex, OnceLock,
    },
    thread,
    time::{Duration, Instant},
};

use crate::audio_service::AudioService;
use sayt_engine::{DeviceInfo, HotCueColor, MixerSnapshot};
use sayt_library::{
    BpmAnalysisProgress, BpmAnalysisStart, LibraryCatalog, LibraryCollection, LibraryFolder,
    ScanCancellation, ScanProgress, ScanStart, TrackPage, TrackQuery,
};
use tauri::{Emitter, Manager};
use tracing_subscriber::EnvFilter;

mod audio_service;
mod hot_cues;

#[derive(Clone)]
struct LibraryService {
    data_directory: PathBuf,
    scan_active: Arc<AtomicBool>,
    scan_progress: Arc<Mutex<Option<ScanProgress>>>,
    cancellation: ScanCancellation,
    bpm_analysis_active: Arc<AtomicBool>,
    bpm_analysis_progress: Arc<Mutex<Option<BpmAnalysisProgress>>>,
    bpm_analysis_cancellation: Arc<AtomicBool>,
    initialized: Arc<OnceLock<Result<(), String>>>,
}

impl LibraryService {
    fn new(data_directory: PathBuf) -> Self {
        Self {
            data_directory,
            scan_active: Arc::new(AtomicBool::new(false)),
            scan_progress: Arc::new(Mutex::new(None)),
            cancellation: ScanCancellation::default(),
            bpm_analysis_active: Arc::new(AtomicBool::new(false)),
            bpm_analysis_progress: Arc::new(Mutex::new(None)),
            bpm_analysis_cancellation: Arc::new(AtomicBool::new(false)),
            initialized: Arc::new(OnceLock::new()),
        }
    }

    fn initialize(&self) -> Result<(), String> {
        self.initialized
            .get_or_init(|| {
                let catalog = LibraryCatalog::open(&self.data_directory)?;
                catalog.refresh_missing_state()
            })
            .clone()
    }

    fn catalog(&self) -> Result<LibraryCatalog, String> {
        self.initialize()?;
        LibraryCatalog::open_for_commands(&self.data_directory)
    }

    fn begin_scan(&self) -> Result<(), String> {
        let mut progress = self
            .scan_progress
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        self.scan_active
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| "A library scan is already running".to_string())?;
        *progress = None;
        self.cancellation.reset();
        Ok(())
    }

    fn scan_status(&self) -> BackgroundProgress<ScanProgress> {
        BackgroundProgress {
            active: self.scan_active.load(Ordering::Acquire),
            progress: self
                .scan_progress
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone(),
        }
    }

    fn store_scan_progress(&self, progress: ScanProgress) {
        *self
            .scan_progress
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(progress);
    }

    fn bpm_analysis_status(&self) -> BackgroundProgress<BpmAnalysisProgress> {
        BackgroundProgress {
            active: self.bpm_analysis_active.load(Ordering::Acquire),
            progress: self
                .bpm_analysis_progress
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone(),
        }
    }

    fn store_bpm_analysis_progress(&self, progress: BpmAnalysisProgress) {
        *self
            .bpm_analysis_progress
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(progress);
    }

    fn end_scan(&self) {
        self.scan_active.store(false, Ordering::Release);
    }

    fn begin_bpm_analysis(&self) -> Result<(), String> {
        self.bpm_analysis_active
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| "A track BPM analysis is already running".to_string())?;
        self.bpm_analysis_cancellation
            .store(false, Ordering::Release);
        Ok(())
    }
}

#[cfg(test)]
mod background_progress_tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn scan_status_retains_terminal_progress_for_ui_recovery() {
        let service = LibraryService::new(PathBuf::from("unused-test-data"));
        service.store_scan_progress(ScanProgress {
            state: "complete".to_string(),
            roots_total: 1,
            roots_done: 1,
            partial_roots: 0,
            files_seen: 3,
            tracks_indexed: 2,
            unsupported: 1,
            unreadable: 0,
            traversal_errors: 0,
            current_file: None,
            message: "Folder scan finished.".to_string(),
        });

        let status = service.scan_status();
        assert!(!status.active);
        let progress = status.progress.expect("terminal scan progress is retained");
        assert_eq!(progress.state, "complete");
        assert_eq!(progress.tracks_indexed, 2);

        let serialized = serde_json::to_value(BackgroundProgress {
            active: status.active,
            progress: Some(progress),
        })
        .unwrap();
        assert_eq!(serialized["active"], false);
        assert_eq!(serialized["progress"]["tracks_indexed"], 2);
    }

    #[test]
    fn starting_a_new_scan_clears_the_previous_terminal_snapshot() {
        let service = LibraryService::new(PathBuf::from("unused-test-data"));
        service.store_scan_progress(ScanProgress {
            state: "complete".to_string(),
            roots_total: 1,
            roots_done: 1,
            partial_roots: 0,
            files_seen: 3,
            tracks_indexed: 2,
            unsupported: 1,
            unreadable: 0,
            traversal_errors: 0,
            current_file: None,
            message: "Previous scan finished.".to_string(),
        });

        service.begin_scan().unwrap();
        let status = service.scan_status();
        assert!(status.active);
        assert!(status.progress.is_none());
        service.end_scan();
    }

    #[test]
    fn bpm_status_serializes_the_frontend_camel_case_contract() {
        let service = LibraryService::new(PathBuf::from("unused-test-data"));
        service.bpm_analysis_active.store(true, Ordering::Release);
        service.store_bpm_analysis_progress(BpmAnalysisProgress {
            track_id: "sha256:test".to_string(),
            state: "analyzing".to_string(),
            processed_frames: 48_000,
            total_frames: Some(96_000),
            candidate_bpm: None,
            confidence: None,
            message: "Analyzing local PCM audio".to_string(),
        });

        let status = service.bpm_analysis_status();
        assert!(status.active);
        let serialized = serde_json::to_value(status).unwrap();
        assert_eq!(serialized["active"], true);
        assert_eq!(serialized["progress"]["trackId"], "sha256:test");
        assert_eq!(serialized["progress"]["processedFrames"], 48_000);
        assert!(serialized["progress"].get("track_id").is_none());
    }

    #[test]
    fn bpm_load_uses_the_engine_track_identity_for_the_selected_deck() {
        let catalog_track_id = "sha256:file-bytes";
        let engine_track_id = "sha256:decoded-pcm";
        assert_ne!(catalog_track_id, engine_track_id);
        let snapshot = MixerSnapshot {
            deck_a: sayt_engine::DeckSnapshot {
                source_track_id: Some(engine_track_id.to_string()),
                ..sayt_engine::DeckSnapshot::default()
            },
            deck_b: sayt_engine::DeckSnapshot {
                source_track_id: Some("sha256:other-deck-pcm".to_string()),
                ..sayt_engine::DeckSnapshot::default()
            },
            ..MixerSnapshot::default()
        };

        assert_eq!(
            source_track_id_for_deck(&snapshot, 0).unwrap(),
            engine_track_id
        );
        assert_eq!(
            source_track_id_for_deck(&snapshot, 1).unwrap(),
            "sha256:other-deck-pcm"
        );
        assert!(source_track_id_for_deck(&snapshot, 2).is_err());
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct BackgroundProgress<T> {
    active: bool,
    progress: Option<T>,
}

struct LibraryScanGuard(Arc<AtomicBool>);

impl Drop for LibraryScanGuard {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

#[tauri::command]
fn library_tracks(
    library: tauri::State<'_, LibraryService>,
    query: TrackQuery,
) -> Result<TrackPage, String> {
    library.catalog()?.list_tracks(&query)
}

#[tauri::command]
fn library_track_artwork(
    library: tauri::State<'_, LibraryService>,
    track_id: String,
) -> Result<Option<String>, String> {
    library.catalog()?.artwork_data_url(&track_id)
}

#[tauri::command]
fn library_folders(
    library: tauri::State<'_, LibraryService>,
) -> Result<Vec<LibraryFolder>, String> {
    library.catalog()?.list_folders()
}

#[tauri::command]
fn library_collections(
    library: tauri::State<'_, LibraryService>,
) -> Result<Vec<LibraryCollection>, String> {
    library.catalog()?.list_collections()
}

#[tauri::command]
fn library_scan_status(
    library: tauri::State<'_, LibraryService>,
) -> BackgroundProgress<ScanProgress> {
    library.scan_status()
}

#[tauri::command]
fn library_bpm_analysis_status(
    library: tauri::State<'_, LibraryService>,
) -> BackgroundProgress<BpmAnalysisProgress> {
    library.bpm_analysis_status()
}

#[tauri::command]
fn library_reconcile_missing(library: tauri::State<'_, LibraryService>) -> Result<(), String> {
    library.catalog()?.refresh_missing_state()
}

#[tauri::command]
async fn library_start_scan(
    app: tauri::AppHandle,
    library: tauri::State<'_, LibraryService>,
) -> Result<ScanStart, String> {
    use tauri_plugin_dialog::DialogExt;

    if library.scan_active.load(Ordering::Acquire) {
        return Err("A library scan is already running".to_string());
    }
    let (selection_sender, selection_receiver) = futures_channel::oneshot::channel();
    app.dialog()
        .file()
        .set_title("Choose music folders for your SAYT library")
        .pick_folders(move |selection| {
            let _ = selection_sender.send(selection);
        });
    let selection = selection_receiver
        .await
        .map_err(|_| "The folder picker closed before returning a selection".to_string())?;
    let Some(selected_folders) = selection else {
        return Ok(ScanStart {
            started: false,
            message: "Folder selection cancelled".to_string(),
        });
    };
    let folders = selected_folders
        .into_iter()
        .map(|folder| {
            folder
                .into_path()
                .map_err(|_| "Choose folders stored on this computer".to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    if folders.is_empty() {
        return Ok(ScanStart {
            started: false,
            message: "No music folders were selected".to_string(),
        });
    }

    library.begin_scan()?;
    let library_service = library.inner().clone();
    let scan_state = Arc::clone(&library.scan_active);
    let worker_app = app.clone();
    if let Err(error) = thread::Builder::new()
        .name("sayt-library-scan".to_string())
        .spawn(move || {
            let _guard = LibraryScanGuard(scan_state);
            let result = library_service.catalog().and_then(|catalog| {
                let mut last_progress_at = Instant::now() - Duration::from_secs(1);
                catalog.scan_roots(&folders, library_service.cancellation.flag(), |progress| {
                    library_service.store_scan_progress(progress.clone());
                    let is_terminal = progress.state != "scanning";
                    if !is_terminal && last_progress_at.elapsed() < Duration::from_millis(100) {
                        return;
                    }
                    last_progress_at = Instant::now();
                    if let Err(error) = worker_app.emit("library-scan-progress", progress) {
                        tracing::warn!(
                            component = "music-library",
                            check = "emit_scan_progress",
                            error = %error,
                            "Library scan progress could not reach the desktop window"
                        );
                    }
                })
            });
            match result {
                Ok(summary) => {
                    tracing::info!(
                        component = "music-library",
                        event = "scan_finished",
                        roots_scanned = summary.roots_scanned,
                        partial_roots = summary.partial_roots,
                        files_seen = summary.files_seen,
                        tracks_indexed = summary.tracks_indexed,
                        unsupported = summary.unsupported,
                        unreadable = summary.unreadable,
                        traversal_errors = summary.traversal_errors,
                        cancelled = summary.cancelled,
                        "Music folder scan finished"
                    );
                }
                Err(error) => {
                    tracing::error!(
                        component = "music-library",
                        check = "scan_selected_folders",
                        error = %error,
                        "Music folder scan failed"
                    );
                    let progress = ScanProgress {
                        state: "failed".to_string(),
                        roots_total: 0,
                        roots_done: 0,
                        partial_roots: 0,
                        files_seen: 0,
                        tracks_indexed: 0,
                        unsupported: 0,
                        unreadable: 0,
                        traversal_errors: 0,
                        current_file: None,
                        message: error,
                    };
                    library_service.store_scan_progress(progress.clone());
                    let _ = worker_app.emit("library-scan-progress", progress);
                }
            }
        })
    {
        library.end_scan();
        return Err(format!("Could not start the library scan worker: {error}"));
    }
    Ok(ScanStart {
        started: true,
        message: "Music folder scan started".to_string(),
    })
}

#[tauri::command]
fn library_cancel_scan(library: tauri::State<'_, LibraryService>) -> bool {
    if !library.scan_active.load(Ordering::Acquire) {
        return false;
    }
    library.cancellation.cancel();
    true
}

#[tauri::command]
fn library_start_bpm_analysis(
    app: tauri::AppHandle,
    library: tauri::State<'_, LibraryService>,
    track_id: String,
) -> Result<BpmAnalysisStart, String> {
    library.begin_bpm_analysis()?;
    let library_service = library.inner().clone();
    let active = Arc::clone(&library.bpm_analysis_active);
    let cancellation = Arc::clone(&library.bpm_analysis_cancellation);
    let worker_app = app.clone();
    library.store_bpm_analysis_progress(BpmAnalysisProgress {
        track_id: track_id.clone(),
        state: "preparing".to_string(),
        processed_frames: 0,
        total_frames: None,
        candidate_bpm: None,
        confidence: None,
        message: "Preparing local analysis".to_string(),
    });
    if let Err(error) = thread::Builder::new()
        .name("sayt-bpm-analysis".to_string())
        .spawn(move || {
            let _guard = LibraryScanGuard(active);
            let result = library_service.catalog().and_then(|catalog| {
                catalog.analyze_bpm(&track_id, &cancellation, |progress| {
                    library_service.store_bpm_analysis_progress(progress.clone());
                    if let Err(error) = worker_app.emit("library-bpm-analysis-progress", &progress)
                    {
                        tracing::warn!(
                            component = "music-library",
                            check = "emit_bpm_analysis_progress",
                            error = %error,
                            "BPM analysis progress could not reach the desktop window"
                        );
                    }
                })
            });
            if let Err(error) = result {
                tracing::error!(
                    component = "music-analysis",
                    check = "analyze_catalog_track_bpm",
                    track_id = %track_id,
                    error = %error,
                    "Local BPM analysis failed"
                );
                let progress = BpmAnalysisProgress {
                    track_id,
                    state: "failed".to_string(),
                    processed_frames: 0,
                    total_frames: None,
                    candidate_bpm: None,
                    confidence: None,
                    message: error,
                };
                library_service.store_bpm_analysis_progress(progress.clone());
                if let Err(error) = worker_app.emit("library-bpm-analysis-progress", &progress) {
                    tracing::warn!(
                        component = "music-library",
                        check = "emit_bpm_analysis_failure",
                        error = %error,
                        "BPM analysis failure could not reach the desktop window"
                    );
                }
            }
        })
    {
        library.end_bpm_analysis();
        return Err(format!("Could not start the BPM analysis worker: {error}"));
    }
    Ok(BpmAnalysisStart {
        started: true,
        message: "Local BPM analysis started".to_string(),
    })
}

#[tauri::command]
fn library_cancel_bpm_analysis(library: tauri::State<'_, LibraryService>) -> bool {
    if !library.bpm_analysis_active.load(Ordering::Acquire) {
        return false;
    }
    library
        .bpm_analysis_cancellation
        .store(true, Ordering::Release);
    true
}

#[tauri::command]
fn library_set_bpm_correction(
    library: tauri::State<'_, LibraryService>,
    track_id: String,
    bpm: Option<f64>,
) -> Result<(), String> {
    library.catalog()?.set_bpm_correction(&track_id, bpm)
}

#[tauri::command]
fn library_track_waveform(
    library: tauri::State<'_, LibraryService>,
    track_id: String,
) -> Result<Option<Vec<f32>>, String> {
    library.catalog()?.waveform_peaks(&track_id)
}

impl LibraryService {
    fn end_bpm_analysis(&self) {
        self.bpm_analysis_active.store(false, Ordering::Release);
    }
}

#[tauri::command]
fn library_create_collection(
    library: tauri::State<'_, LibraryService>,
    name: String,
    kind: String,
) -> Result<LibraryCollection, String> {
    library.catalog()?.create_collection(&name, &kind)
}

#[tauri::command]
fn library_rename_collection(
    library: tauri::State<'_, LibraryService>,
    collection_id: String,
    name: String,
) -> Result<(), String> {
    library.catalog()?.rename_collection(&collection_id, &name)
}

#[tauri::command]
fn library_delete_collection(
    library: tauri::State<'_, LibraryService>,
    collection_id: String,
) -> Result<(), String> {
    library.catalog()?.delete_collection(&collection_id)
}

#[tauri::command]
fn library_add_track_to_collection(
    library: tauri::State<'_, LibraryService>,
    collection_id: String,
    track_id: String,
) -> Result<(), String> {
    library
        .catalog()?
        .add_track_to_collection(&collection_id, &track_id)
}

#[tauri::command]
fn library_remove_track_from_collection(
    library: tauri::State<'_, LibraryService>,
    collection_id: String,
    track_id: String,
) -> Result<(), String> {
    library
        .catalog()?
        .remove_track_from_collection(&collection_id, &track_id)
}

#[tauri::command]
fn library_move_playlist_track(
    library: tauri::State<'_, LibraryService>,
    collection_id: String,
    track_id: String,
    delta: i32,
) -> Result<(), String> {
    library
        .catalog()?
        .move_collection_track(&collection_id, &track_id, delta)
}

#[tauri::command]
fn library_set_favorite(
    library: tauri::State<'_, LibraryService>,
    track_id: String,
    favorite: bool,
) -> Result<(), String> {
    library.catalog()?.set_favorite(&track_id, favorite)
}

#[tauri::command]
fn library_update_track_metadata(
    library: tauri::State<'_, LibraryService>,
    track_id: String,
    title: String,
    artist: String,
    album: String,
) -> Result<(), String> {
    library
        .catalog()?
        .update_user_metadata(&track_id, &title, &artist, &album)
}

#[tauri::command]
async fn library_export_backup(
    app: tauri::AppHandle,
    library: tauri::State<'_, LibraryService>,
) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;

    let (selection_sender, selection_receiver) = futures_channel::oneshot::channel();
    app.dialog()
        .file()
        .set_title("Export SAYT library backup")
        .set_file_name("sayt-music-library.sqlite3")
        .add_filter("SQLite catalog backup", &["sqlite3"])
        .save_file(move |selection| {
            let _ = selection_sender.send(selection);
        });
    let selection = selection_receiver
        .await
        .map_err(|_| "The save dialog closed before returning a location".to_string())?;
    let Some(destination) = selection else {
        return Ok(None);
    };
    let path = destination
        .as_path()
        .ok_or_else(|| "Choose a backup location stored on this computer".to_string())?;
    library.catalog()?.backup_to(path)?;
    Ok(Some(path.to_string_lossy().into_owned()))
}

#[tauri::command]
fn load_library_track(
    library: tauri::State<'_, LibraryService>,
    audio: tauri::State<'_, AudioService>,
    track_id: String,
    deck: usize,
) -> Result<MixerSnapshot, String> {
    let catalog = library.catalog()?;
    load_catalog_track(&catalog, &audio, &track_id, deck)
}

fn load_catalog_track(
    catalog: &LibraryCatalog,
    audio: &AudioService,
    track_id: &str,
    deck: usize,
) -> Result<MixerSnapshot, String> {
    let path = catalog.resolve_track_path(track_id)?;
    let snapshot = audio.load_track(deck, &path)?;
    if let Some(bpm) = catalog.playback_bpm(track_id)? {
        let engine_track_id = source_track_id_for_deck(&snapshot, deck)?;
        audio.set_manual_bpm(deck, engine_track_id, bpm)
    } else {
        Ok(snapshot)
    }
}

fn source_track_id_for_deck(snapshot: &MixerSnapshot, deck: usize) -> Result<&str, String> {
    let deck_snapshot = match deck {
        0 => &snapshot.deck_a,
        1 => &snapshot.deck_b,
        _ => return Err("Deck must be A or B".to_string()),
    };
    deck_snapshot
        .source_track_id
        .as_deref()
        .ok_or_else(|| "Audio engine did not return the loaded track identity".to_string())
}

#[tauri::command]
fn list_devices(audio: tauri::State<'_, AudioService>) -> Result<Vec<DeviceInfo>, String> {
    audio.list_devices()
}

#[tauri::command]
fn select_device(
    audio: tauri::State<'_, AudioService>,
    device_id: Option<String>,
) -> Result<MixerSnapshot, String> {
    audio.select_device(device_id.as_deref())
}

#[tauri::command]
fn set_playing(
    audio: tauri::State<'_, AudioService>,
    deck: usize,
    playing: bool,
) -> Result<MixerSnapshot, String> {
    audio.set_playing(deck, playing)
}

#[tauri::command]
fn stop_track(audio: tauri::State<'_, AudioService>, deck: usize) -> Result<MixerSnapshot, String> {
    audio.stop_track(deck)
}

#[tauri::command]
fn seek_track(
    audio: tauri::State<'_, AudioService>,
    deck: usize,
    position: f64,
) -> Result<MixerSnapshot, String> {
    audio.seek_track(deck, position)
}

#[tauri::command]
fn set_deck_control(
    audio: tauri::State<'_, AudioService>,
    deck: usize,
    gain: Option<f32>,
    low: Option<f32>,
    mid: Option<f32>,
    high: Option<f32>,
) -> Result<MixerSnapshot, String> {
    audio.set_deck_control(deck, gain, low, mid, high)
}

#[tauri::command]
fn set_mixer(
    audio: tauri::State<'_, AudioService>,
    master: Option<f32>,
    crossfader: Option<f32>,
) -> Result<MixerSnapshot, String> {
    audio.set_mixer(master, crossfader)
}

#[tauri::command]
async fn pick_track(app: tauri::AppHandle, deck: usize) -> Result<MixerSnapshot, String> {
    use tauri_plugin_dialog::DialogExt;

    let (selection_sender, selection_receiver) = futures_channel::oneshot::channel();
    app.dialog()
        .file()
        .add_filter("WAV audio", &["wav"])
        .pick_file(move |selection| {
            let _ = selection_sender.send(selection);
        });
    let selection = selection_receiver
        .await
        .map_err(|_| "The file picker closed before returning a selection".to_string())?;
    let Some(file) = selection else {
        return app.state::<AudioService>().snapshot();
    };
    let path = file
        .as_path()
        .ok_or_else(|| "The selected WAV file does not have a local filesystem path".to_string())?;
    app.state::<AudioService>().load_track(deck, path)
}

#[tauri::command]
fn engine_snapshot(audio: tauri::State<'_, AudioService>) -> Result<MixerSnapshot, String> {
    audio.snapshot()
}

#[tauri::command]
fn set_hot_cue(
    audio: tauri::State<'_, AudioService>,
    deck: usize,
    slot: usize,
    source_track_id: String,
) -> Result<MixerSnapshot, String> {
    audio.set_hot_cue(deck, slot, &source_track_id)
}

#[tauri::command]
fn trigger_hot_cue(
    audio: tauri::State<'_, AudioService>,
    deck: usize,
    slot: usize,
    source_track_id: String,
) -> Result<MixerSnapshot, String> {
    audio.trigger_hot_cue(deck, slot, &source_track_id)
}

#[tauri::command]
fn clear_hot_cue(
    audio: tauri::State<'_, AudioService>,
    deck: usize,
    slot: usize,
    source_track_id: String,
) -> Result<MixerSnapshot, String> {
    audio.clear_hot_cue(deck, slot, &source_track_id)
}

#[tauri::command]
fn rename_hot_cue(
    audio: tauri::State<'_, AudioService>,
    deck: usize,
    slot: usize,
    source_track_id: String,
    name: String,
) -> Result<MixerSnapshot, String> {
    audio.rename_hot_cue(deck, slot, &source_track_id, name)
}

#[tauri::command]
fn set_hot_cue_color(
    audio: tauri::State<'_, AudioService>,
    deck: usize,
    slot: usize,
    source_track_id: String,
    color: HotCueColor,
) -> Result<MixerSnapshot, String> {
    audio.set_hot_cue_color(deck, slot, &source_track_id, color)
}

#[tauri::command]
fn set_loop_in(
    audio: tauri::State<'_, AudioService>,
    deck: usize,
    source_track_id: String,
) -> Result<MixerSnapshot, String> {
    audio.set_loop_in(deck, &source_track_id)
}

#[tauri::command]
fn set_loop_out(
    audio: tauri::State<'_, AudioService>,
    deck: usize,
    source_track_id: String,
) -> Result<MixerSnapshot, String> {
    audio.set_loop_out(deck, &source_track_id)
}

#[tauri::command]
fn set_loop_size(
    audio: tauri::State<'_, AudioService>,
    deck: usize,
    source_track_id: String,
    size_seconds: f64,
) -> Result<MixerSnapshot, String> {
    audio.set_loop_size(deck, &source_track_id, size_seconds)
}

#[tauri::command]
fn release_loop(
    audio: tauri::State<'_, AudioService>,
    deck: usize,
    source_track_id: String,
) -> Result<MixerSnapshot, String> {
    audio.release_loop(deck, &source_track_id)
}

#[tauri::command]
fn set_manual_bpm(
    audio: tauri::State<'_, AudioService>,
    deck: usize,
    source_track_id: String,
    bpm: f64,
) -> Result<MixerSnapshot, String> {
    audio.set_manual_bpm(deck, &source_track_id, bpm)
}

#[tauri::command]
fn beat_jump(
    audio: tauri::State<'_, AudioService>,
    deck: usize,
    source_track_id: String,
    beats: f64,
) -> Result<MixerSnapshot, String> {
    audio.beat_jump(deck, &source_track_id, beats)
}

#[tauri::command]
fn begin_loop_roll(
    audio: tauri::State<'_, AudioService>,
    deck: usize,
    source_track_id: String,
    beats: f64,
) -> Result<MixerSnapshot, String> {
    audio.begin_loop_roll(deck, &source_track_id, beats)
}

#[tauri::command]
fn end_loop_roll(
    audio: tauri::State<'_, AudioService>,
    deck: usize,
    source_track_id: String,
) -> Result<MixerSnapshot, String> {
    audio.end_loop_roll(deck, &source_track_id)
}

/// Starts the desktop shell and reports startup or runtime failures.
pub fn run() -> Result<(), Box<dyn Error>> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(filter)
        .try_init()
        .map_err(|error| std::io::Error::other(error.to_string()))?;

    tracing::info!(component = "desktop-shell", event = "startup");
    if let Err(error) = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let local_data_directory = app
                .path()
                .app_local_data_dir()
                .map_err(|error| std::io::Error::other(error.to_string()))?;
            let cue_directory = local_data_directory.join("hot-cues-v1");
            let audio = AudioService::new(cue_directory).map_err(std::io::Error::other)?;
            app.manage(audio);
            let library = LibraryService::new(local_data_directory.join("music-library-v1"));
            match library.initialize() {
                Ok(()) => tracing::info!(
                    component = "music-library",
                    event = "catalog_ready",
                    "Local music catalog initialized"
                ),
                Err(error) => tracing::error!(
                    component = "music-library",
                    check = "initialize_catalog",
                    error = %error,
                    "Music catalog could not be initialized; audio playback remains available"
                ),
            }
            app.manage(library);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler!(
            list_devices,
            select_device,
            set_playing,
            stop_track,
            seek_track,
            set_deck_control,
            set_mixer,
            pick_track,
            engine_snapshot,
            set_hot_cue,
            trigger_hot_cue,
            clear_hot_cue,
            rename_hot_cue,
            set_hot_cue_color,
            set_loop_in,
            set_loop_out,
            set_loop_size,
            release_loop,
            set_manual_bpm,
            beat_jump,
            begin_loop_roll,
            end_loop_roll,
            library_tracks,
            library_track_artwork,
            library_folders,
            library_collections,
            library_reconcile_missing,
            library_start_scan,
            library_cancel_scan,
            library_scan_status,
            library_bpm_analysis_status,
            library_start_bpm_analysis,
            library_cancel_bpm_analysis,
            library_set_bpm_correction,
            library_track_waveform,
            library_create_collection,
            library_rename_collection,
            library_delete_collection,
            library_add_track_to_collection,
            library_remove_track_from_collection,
            library_move_playlist_track,
            library_set_favorite,
            library_update_track_metadata,
            library_export_backup,
            load_library_track
        ))
        .run(tauri::generate_context!())
    {
        tracing::error!(
            component = "desktop-shell",
            error = %error,
            "Tauri runtime exited with an error"
        );
        return Err(Box::new(error));
    }

    Ok(())
}
