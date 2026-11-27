//! Tauri commands for device selection and the local two-deck workflow.

use std::{
    error::Error,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, OnceLock,
    },
    thread,
    time::{Duration, Instant},
};

use crate::audio_service::AudioService;
use sayt_engine::{DeviceInfo, HotCueColor, MixerSnapshot};
use sayt_library::{
    LibraryCatalog, LibraryCollection, LibraryFolder, ScanCancellation, ScanProgress, ScanStart,
    TrackPage, TrackQuery,
};
use tauri::{Emitter, Manager};
use tracing_subscriber::EnvFilter;

mod audio_service;
mod hot_cues;

#[derive(Clone)]
struct LibraryService {
    data_directory: PathBuf,
    scan_active: Arc<AtomicBool>,
    cancellation: ScanCancellation,
    initialized: Arc<OnceLock<Result<(), String>>>,
}

impl LibraryService {
    fn new(data_directory: PathBuf) -> Self {
        Self {
            data_directory,
            scan_active: Arc::new(AtomicBool::new(false)),
            cancellation: ScanCancellation::default(),
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
        self.scan_active
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| "A library scan is already running".to_string())?;
        self.cancellation.reset();
        Ok(())
    }

    fn end_scan(&self) {
        self.scan_active.store(false, Ordering::Release);
    }
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
                    let _ = worker_app.emit(
                        "library-scan-progress",
                        ScanProgress {
                            state: "failed".to_string(),
                            roots_total: 0,
                            roots_done: 0,
                            files_seen: 0,
                            tracks_indexed: 0,
                            unsupported: 0,
                            unreadable: 0,
                            traversal_errors: 0,
                            current_file: None,
                            message: error,
                        },
                    );
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
    let path = library.catalog()?.resolve_track_path(&track_id)?;
    audio.load_track(deck, &path)
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
