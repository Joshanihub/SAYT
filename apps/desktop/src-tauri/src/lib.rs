//! Tauri commands for device selection and the local two-deck workflow.

use std::{error::Error, sync::Mutex};

use sayt_engine::{DeviceInfo, Engine, MixerSnapshot};
use tauri::Manager;
use tracing_subscriber::EnvFilter;

#[tauri::command]
fn list_devices(engine: tauri::State<'_, Mutex<Engine>>) -> Result<Vec<DeviceInfo>, String> {
    engine
        .lock()
        .map_err(|_| "Audio engine state is unavailable".to_string())?
        .devices()
}

#[tauri::command]
fn select_device(
    engine: tauri::State<'_, Mutex<Engine>>,
    device_id: Option<String>,
) -> Result<MixerSnapshot, String> {
    engine
        .lock()
        .map_err(|_| "Audio engine state is unavailable".to_string())?
        .select_device(device_id.as_deref())
}

#[tauri::command]
fn set_playing(
    engine: tauri::State<'_, Mutex<Engine>>,
    deck: usize,
    playing: bool,
) -> Result<MixerSnapshot, String> {
    engine
        .lock()
        .map_err(|_| "Audio engine state is unavailable".to_string())?
        .set_playing(deck, playing)
}

#[tauri::command]
fn stop_track(
    engine: tauri::State<'_, Mutex<Engine>>,
    deck: usize,
) -> Result<MixerSnapshot, String> {
    engine
        .lock()
        .map_err(|_| "Audio engine state is unavailable".to_string())?
        .stop(deck)
}

#[tauri::command]
fn seek_track(
    engine: tauri::State<'_, Mutex<Engine>>,
    deck: usize,
    position: f64,
) -> Result<MixerSnapshot, String> {
    engine
        .lock()
        .map_err(|_| "Audio engine state is unavailable".to_string())?
        .seek(deck, position)
}

#[tauri::command]
fn set_deck_control(
    engine: tauri::State<'_, Mutex<Engine>>,
    deck: usize,
    gain: Option<f32>,
    low: Option<f32>,
    mid: Option<f32>,
    high: Option<f32>,
) -> Result<MixerSnapshot, String> {
    engine
        .lock()
        .map_err(|_| "Audio engine state is unavailable".to_string())?
        .set_control(deck, gain, low, mid, high)
}

#[tauri::command]
fn set_mixer(
    engine: tauri::State<'_, Mutex<Engine>>,
    master: Option<f32>,
    crossfader: Option<f32>,
) -> Result<MixerSnapshot, String> {
    engine
        .lock()
        .map_err(|_| "Audio engine state is unavailable".to_string())?
        .set_mixer(master, crossfader)
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
        return app
            .state::<Mutex<Engine>>()
            .lock()
            .map_err(|_| "Audio engine state is unavailable".to_string())
            .map(|mut engine| engine.snapshot());
    };
    let path = file
        .as_path()
        .ok_or_else(|| "The selected WAV file does not have a local filesystem path".to_string())?;
    app.state::<Mutex<Engine>>()
        .lock()
        .map_err(|_| "Audio engine state is unavailable".to_string())?
        .load_track(deck, path)
}

#[tauri::command]
fn engine_snapshot(engine: tauri::State<'_, Mutex<Engine>>) -> Result<MixerSnapshot, String> {
    engine
        .lock()
        .map_err(|_| "Audio engine state is unavailable".to_string())
        .map(|mut engine| engine.snapshot())
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
    let engine = Engine::new().map_err(std::io::Error::other)?;
    if let Err(error) = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(Mutex::new(engine))
        .invoke_handler(tauri::generate_handler!(
            list_devices,
            select_device,
            set_playing,
            stop_track,
            seek_track,
            set_deck_control,
            set_mixer,
            pick_track,
            engine_snapshot
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
