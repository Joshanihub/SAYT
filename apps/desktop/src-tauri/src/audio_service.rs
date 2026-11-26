//! Owns the non-Send CPAL engine on a dedicated control thread.

use std::{
    path::{Path, PathBuf},
    sync::{
        mpsc::{self, Receiver, SyncSender, TrySendError},
        Mutex,
    },
    thread::{self, JoinHandle},
};

use sayt_engine::{
    default_hot_cue, empty_hot_cue_slots, validate_hot_cue, validate_hot_cue_slots, DeviceInfo,
    Engine, HotCueColor, HotCueSlots, MixerSnapshot, HOT_CUE_SLOT_COUNT,
};

use crate::hot_cues::{HotCueLoad, HotCueStore};

const CONTROL_QUEUE_CAPACITY: usize = 64;
const REPLY_QUEUE_CAPACITY: usize = 1;

type EngineReply<T> = SyncSender<Result<T, String>>;

enum ServiceCommand {
    ListDevices(EngineReply<Vec<DeviceInfo>>),
    SelectDevice(Option<String>, EngineReply<MixerSnapshot>),
    SetPlaying(usize, bool, EngineReply<MixerSnapshot>),
    StopTrack(usize, EngineReply<MixerSnapshot>),
    SeekTrack(usize, f64, EngineReply<MixerSnapshot>),
    SetDeckControl(
        usize,
        Option<f32>,
        Option<f32>,
        Option<f32>,
        Option<f32>,
        EngineReply<MixerSnapshot>,
    ),
    SetMixer(Option<f32>, Option<f32>, EngineReply<MixerSnapshot>),
    LoadTrack(usize, std::path::PathBuf, EngineReply<MixerSnapshot>),
    SetHotCue(usize, usize, String, EngineReply<MixerSnapshot>),
    TriggerHotCue(usize, usize, String, EngineReply<MixerSnapshot>),
    ClearHotCue(usize, usize, String, EngineReply<MixerSnapshot>),
    RenameHotCue(usize, usize, String, String, EngineReply<MixerSnapshot>),
    SetHotCueColor(
        usize,
        usize,
        String,
        HotCueColor,
        EngineReply<MixerSnapshot>,
    ),
    Snapshot(EngineReply<MixerSnapshot>),
    Shutdown,
}

/// Thread-safe Tauri state that forwards bounded requests to the engine owner.
pub struct AudioService {
    commands: SyncSender<ServiceCommand>,
    actor: Mutex<Option<JoinHandle<()>>>,
}

impl AudioService {
    pub fn new(hot_cue_directory: PathBuf) -> Result<Self, String> {
        let (commands, receiver) = mpsc::sync_channel(CONTROL_QUEUE_CAPACITY);
        let (startup_sender, startup_receiver) = mpsc::sync_channel(REPLY_QUEUE_CAPACITY);
        let actor = thread::Builder::new()
            .name("sayt-audio-control".to_string())
            .spawn(move || match Engine::new() {
                Ok(engine) => {
                    let _ = startup_sender.send(Ok(()));
                    run_engine_actor(engine, HotCueStore::new(hot_cue_directory), receiver);
                }
                Err(error) => {
                    let _ = startup_sender.send(Err(error));
                }
            })
            .map_err(|error| format!("Could not start the audio control thread: {error}"))?;

        match startup_receiver.recv() {
            Ok(Ok(())) => Ok(Self {
                commands,
                actor: Mutex::new(Some(actor)),
            }),
            Ok(Err(error)) => {
                let _ = actor.join();
                Err(error)
            }
            Err(_) => {
                let _ = actor.join();
                Err("Audio control thread stopped during initialization".to_string())
            }
        }
    }

    pub fn list_devices(&self) -> Result<Vec<DeviceInfo>, String> {
        self.request(ServiceCommand::ListDevices)
    }

    pub fn select_device(&self, device_id: Option<&str>) -> Result<MixerSnapshot, String> {
        let device_id = device_id.map(str::to_owned);
        self.request(|reply| ServiceCommand::SelectDevice(device_id, reply))
    }

    pub fn set_playing(&self, deck: usize, playing: bool) -> Result<MixerSnapshot, String> {
        self.request(|reply| ServiceCommand::SetPlaying(deck, playing, reply))
    }

    pub fn stop_track(&self, deck: usize) -> Result<MixerSnapshot, String> {
        self.request(|reply| ServiceCommand::StopTrack(deck, reply))
    }

    pub fn seek_track(&self, deck: usize, position: f64) -> Result<MixerSnapshot, String> {
        self.request(|reply| ServiceCommand::SeekTrack(deck, position, reply))
    }

    pub fn set_deck_control(
        &self,
        deck: usize,
        gain: Option<f32>,
        low: Option<f32>,
        mid: Option<f32>,
        high: Option<f32>,
    ) -> Result<MixerSnapshot, String> {
        self.request(|reply| ServiceCommand::SetDeckControl(deck, gain, low, mid, high, reply))
    }

    pub fn set_mixer(
        &self,
        master: Option<f32>,
        crossfader: Option<f32>,
    ) -> Result<MixerSnapshot, String> {
        self.request(|reply| ServiceCommand::SetMixer(master, crossfader, reply))
    }

    pub fn load_track(&self, deck: usize, path: &Path) -> Result<MixerSnapshot, String> {
        let path = path.to_path_buf();
        self.request(|reply| ServiceCommand::LoadTrack(deck, path, reply))
    }

    pub fn set_hot_cue(
        &self,
        deck: usize,
        slot: usize,
        source_track_id: &str,
    ) -> Result<MixerSnapshot, String> {
        let source_track_id = source_track_id.to_string();
        self.request(|reply| ServiceCommand::SetHotCue(deck, slot, source_track_id, reply))
    }

    pub fn trigger_hot_cue(
        &self,
        deck: usize,
        slot: usize,
        source_track_id: &str,
    ) -> Result<MixerSnapshot, String> {
        let source_track_id = source_track_id.to_string();
        self.request(|reply| ServiceCommand::TriggerHotCue(deck, slot, source_track_id, reply))
    }

    pub fn clear_hot_cue(
        &self,
        deck: usize,
        slot: usize,
        source_track_id: &str,
    ) -> Result<MixerSnapshot, String> {
        let source_track_id = source_track_id.to_string();
        self.request(|reply| ServiceCommand::ClearHotCue(deck, slot, source_track_id, reply))
    }

    pub fn rename_hot_cue(
        &self,
        deck: usize,
        slot: usize,
        source_track_id: &str,
        name: String,
    ) -> Result<MixerSnapshot, String> {
        let source_track_id = source_track_id.to_string();
        self.request(|reply| ServiceCommand::RenameHotCue(deck, slot, source_track_id, name, reply))
    }

    pub fn set_hot_cue_color(
        &self,
        deck: usize,
        slot: usize,
        source_track_id: &str,
        color: HotCueColor,
    ) -> Result<MixerSnapshot, String> {
        let source_track_id = source_track_id.to_string();
        self.request(|reply| {
            ServiceCommand::SetHotCueColor(deck, slot, source_track_id, color, reply)
        })
    }

    pub fn snapshot(&self) -> Result<MixerSnapshot, String> {
        self.request(ServiceCommand::Snapshot)
    }

    fn request<T>(
        &self,
        create_command: impl FnOnce(EngineReply<T>) -> ServiceCommand,
    ) -> Result<T, String>
    where
        T: Send + 'static,
    {
        let (reply_sender, reply_receiver) = mpsc::sync_channel(REPLY_QUEUE_CAPACITY);
        match self.commands.try_send(create_command(reply_sender)) {
            Ok(()) => {}
            Err(TrySendError::Full(_)) => {
                return Err(
                    "Audio engine command queue is busy; try the control again shortly".to_string(),
                );
            }
            Err(TrySendError::Disconnected(_)) => {
                return Err("Audio engine control thread is unavailable".to_string());
            }
        }
        reply_receiver
            .recv()
            .map_err(|_| "Audio engine stopped before completing the command".to_string())?
    }
}

impl Drop for AudioService {
    fn drop(&mut self) {
        let _ = self.commands.send(ServiceCommand::Shutdown);
        let actor = match self.actor.get_mut() {
            Ok(actor) => actor.take(),
            Err(poisoned) => poisoned.into_inner().take(),
        };
        if let Some(actor) = actor {
            let _ = actor.join();
        }
    }
}

fn run_engine_actor(
    mut engine: Engine,
    hot_cue_store: HotCueStore,
    commands: Receiver<ServiceCommand>,
) {
    while let Ok(command) = commands.recv() {
        match command {
            ServiceCommand::ListDevices(reply) => reply_with(reply, engine.devices()),
            ServiceCommand::SelectDevice(device_id, reply) => {
                reply_with(reply, engine.select_device(device_id.as_deref()));
            }
            ServiceCommand::SetPlaying(deck, playing, reply) => {
                reply_with(reply, engine.set_playing(deck, playing));
            }
            ServiceCommand::StopTrack(deck, reply) => reply_with(reply, engine.stop(deck)),
            ServiceCommand::SeekTrack(deck, position, reply) => {
                reply_with(reply, engine.seek(deck, position));
            }
            ServiceCommand::SetDeckControl(deck, gain, low, mid, high, reply) => {
                reply_with(reply, engine.set_control(deck, gain, low, mid, high));
            }
            ServiceCommand::SetMixer(master, crossfader, reply) => {
                reply_with(reply, engine.set_mixer(master, crossfader));
            }
            ServiceCommand::LoadTrack(deck, path, reply) => {
                reply_with(
                    reply,
                    load_track_with_cues(&mut engine, &hot_cue_store, deck, &path),
                );
            }
            ServiceCommand::SetHotCue(deck, slot, source_track_id, reply) => {
                reply_with(
                    reply,
                    mutate_hot_cues(
                        &mut engine,
                        &hot_cue_store,
                        deck,
                        slot,
                        &source_track_id,
                        |slots, cue_position, duration| {
                            let existing = slots[slot].clone();
                            let mut cue =
                                existing.unwrap_or_else(|| default_hot_cue(slot, cue_position));
                            cue.position_seconds = cue_position;
                            validate_hot_cue(slot, &cue, duration)?;
                            slots[slot] = Some(cue);
                            Ok(())
                        },
                    ),
                );
            }
            ServiceCommand::TriggerHotCue(deck, slot, source_track_id, reply) => {
                reply_with(reply, engine.trigger_hot_cue(deck, slot, &source_track_id));
            }
            ServiceCommand::ClearHotCue(deck, slot, source_track_id, reply) => {
                reply_with(
                    reply,
                    mutate_hot_cues(
                        &mut engine,
                        &hot_cue_store,
                        deck,
                        slot,
                        &source_track_id,
                        |slots, _, _| {
                            slots[slot] = None;
                            Ok(())
                        },
                    ),
                );
            }
            ServiceCommand::RenameHotCue(deck, slot, source_track_id, name, reply) => {
                reply_with(
                    reply,
                    mutate_hot_cues(
                        &mut engine,
                        &hot_cue_store,
                        deck,
                        slot,
                        &source_track_id,
                        |slots, _, duration| {
                            let Some(cue) = slots[slot].as_mut() else {
                                return Err("Set this hot cue before editing its name".to_string());
                            };
                            cue.name = name.trim().to_string();
                            validate_hot_cue(slot, cue, duration)
                        },
                    ),
                );
            }
            ServiceCommand::SetHotCueColor(deck, slot, source_track_id, color, reply) => {
                reply_with(
                    reply,
                    mutate_hot_cues(
                        &mut engine,
                        &hot_cue_store,
                        deck,
                        slot,
                        &source_track_id,
                        |slots, _, duration| {
                            let Some(cue) = slots[slot].as_mut() else {
                                return Err(
                                    "Set this hot cue before changing its color".to_string()
                                );
                            };
                            cue.color = color;
                            validate_hot_cue(slot, cue, duration)
                        },
                    ),
                );
            }
            ServiceCommand::Snapshot(reply) => reply_with(reply, Ok(engine.snapshot())),
            ServiceCommand::Shutdown => break,
        }
    }
}

fn load_track_with_cues(
    engine: &mut Engine,
    store: &HotCueStore,
    deck: usize,
    path: &Path,
) -> Result<MixerSnapshot, String> {
    let loaded_snapshot = engine.load_track(deck, path)?;
    let loaded_deck = deck_snapshot(&loaded_snapshot, deck)?;
    let source_track_id = loaded_deck
        .source_track_id
        .as_deref()
        .ok_or_else(|| "The loaded WAV has no source identity for hot cues".to_string())?;
    let cues = match store.load(source_track_id, loaded_deck.duration_seconds) {
        Ok(cues) => cues,
        Err(warning) => {
            tracing::warn!(
                component = "hot-cue-store",
                check = "load_source_cues",
                source_track_id,
                "Saved hot cues could not be loaded; starting this deck with empty cue slots"
            );
            HotCueLoad {
                slots: empty_hot_cue_slots(),
                warning: Some(warning),
            }
        }
    };

    let deck_ids = [
        loaded_snapshot.deck_a.source_track_id.as_deref(),
        loaded_snapshot.deck_b.source_track_id.as_deref(),
    ];
    for (target_deck, target_id) in deck_ids.into_iter().enumerate() {
        if target_id == Some(source_track_id) {
            engine.set_hot_cues(
                target_deck,
                source_track_id,
                cues.slots.clone(),
                cues.warning.clone(),
            )?;
        }
    }
    Ok(engine.snapshot())
}

fn mutate_hot_cues(
    engine: &mut Engine,
    store: &HotCueStore,
    deck: usize,
    slot: usize,
    expected_source_track_id: &str,
    mutation: impl FnOnce(&mut HotCueSlots, f64, f64) -> Result<(), String>,
) -> Result<MixerSnapshot, String> {
    if slot >= HOT_CUE_SLOT_COUNT {
        return Err(format!(
            "Hot cue slot must be between 1 and {HOT_CUE_SLOT_COUNT}"
        ));
    }
    let snapshot = engine.snapshot();
    let deck_state = deck_snapshot(&snapshot, deck)?;
    let source_track_id = deck_state
        .source_track_id
        .as_deref()
        .ok_or_else(|| "Load a WAV into this deck before editing hot cues".to_string())?;
    if source_track_id != expected_source_track_id {
        return Err("This deck changed tracks. Select the hot cue again.".to_string());
    }

    let mut slots = deck_state.hot_cues.clone();
    mutation(
        &mut slots,
        deck_state.position_seconds,
        deck_state.duration_seconds,
    )?;
    validate_hot_cue_slots(&slots, deck_state.duration_seconds)?;
    store.save(source_track_id, deck_state.duration_seconds, &slots)?;

    let track_ids = [
        snapshot.deck_a.source_track_id.as_deref(),
        snapshot.deck_b.source_track_id.as_deref(),
    ];
    for (target_deck, track_id) in track_ids.into_iter().enumerate() {
        if track_id == Some(source_track_id) {
            engine.set_hot_cues(target_deck, source_track_id, slots.clone(), None)?;
        }
    }
    Ok(engine.snapshot())
}

fn deck_snapshot(
    snapshot: &MixerSnapshot,
    deck: usize,
) -> Result<&sayt_engine::DeckSnapshot, String> {
    match deck {
        0 => Ok(&snapshot.deck_a),
        1 => Ok(&snapshot.deck_b),
        _ => Err("Deck index must be 0 or 1".to_string()),
    }
}

fn reply_with<T>(reply: EngineReply<T>, result: Result<T, String>) {
    let _ = reply.send(result);
}

#[cfg(test)]
mod tests {
    use super::AudioService;

    #[test]
    fn actor_returns_snapshots_and_preserves_no_device_errors() {
        let service = AudioService::new(std::env::temp_dir().join("sayt-hot-cues-actor-test"))
            .expect("audio control actor starts");
        let initial = service.snapshot().expect("initial engine snapshot");
        assert_eq!(initial.crossfader, 0.5);

        let error = service
            .set_mixer(Some(0.6), Some(0.25))
            .expect_err("mixer controls require an output device");
        assert!(error.contains("Select an audio output device"));

        let after_error = service.snapshot().expect("snapshot after rejected control");
        assert_eq!(after_error.master_volume, 0.8);
        assert_eq!(after_error.crossfader, 0.5);
    }

    #[test]
    fn actor_reports_unloaded_and_stale_hot_cue_actions_without_claiming_success() {
        let storage = std::env::temp_dir().join(format!(
            "sayt-hot-cues-actor-unloaded-{}",
            std::process::id()
        ));
        let service = AudioService::new(storage).expect("audio control actor starts");
        let error = service
            .trigger_hot_cue(
                0,
                0,
                "sha256:0000000000000000000000000000000000000000000000000000000000000000",
            )
            .expect_err("an unloaded deck cannot trigger a cue");
        assert!(error.contains("Load a WAV"));
        let error = service
            .set_hot_cue(
                2,
                0,
                "sha256:0000000000000000000000000000000000000000000000000000000000000000",
            )
            .expect_err("invalid deck index must be rejected");
        assert!(error.contains("Deck index"));
    }
}
