//! Tauri commands for device selection and the local two-deck workflow.

use std::error::Error;

use crate::audio_service::AudioService;
use sayt_engine::{DeviceInfo, HotCueColor, MixerSnapshot};
use tauri::Manager;
use tracing_subscriber::EnvFilter;

mod audio_service;
mod hot_cues;

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
            let cue_directory = app
                .path()
                .app_local_data_dir()
                .map_err(|error| std::io::Error::other(error.to_string()))?
                .join("hot-cues-v1");
            let audio = AudioService::new(cue_directory).map_err(std::io::Error::other)?;
            app.manage(audio);
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
            set_hot_cue_color
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
