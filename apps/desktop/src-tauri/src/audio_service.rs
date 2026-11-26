//! Owns the non-Send CPAL engine on a dedicated control thread.

use std::{
    path::Path,
    sync::{
        mpsc::{self, Receiver, SyncSender, TrySendError},
        Mutex,
    },
    thread::{self, JoinHandle},
};

use sayt_engine::{DeviceInfo, Engine, MixerSnapshot};

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
    Snapshot(EngineReply<MixerSnapshot>),
    Shutdown,
}

/// Thread-safe Tauri state that forwards bounded requests to the engine owner.
pub struct AudioService {
    commands: SyncSender<ServiceCommand>,
    actor: Mutex<Option<JoinHandle<()>>>,
}

impl AudioService {
    pub fn new() -> Result<Self, String> {
        let (commands, receiver) = mpsc::sync_channel(CONTROL_QUEUE_CAPACITY);
        let (startup_sender, startup_receiver) = mpsc::sync_channel(REPLY_QUEUE_CAPACITY);
        let actor = thread::Builder::new()
            .name("sayt-audio-control".to_string())
            .spawn(move || match Engine::new() {
                Ok(engine) => {
                    let _ = startup_sender.send(Ok(()));
                    run_engine_actor(engine, receiver);
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

fn run_engine_actor(mut engine: Engine, commands: Receiver<ServiceCommand>) {
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
                reply_with(reply, engine.load_track(deck, &path));
            }
            ServiceCommand::Snapshot(reply) => reply_with(reply, Ok(engine.snapshot())),
            ServiceCommand::Shutdown => break,
        }
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
        let service = AudioService::new().expect("audio control actor starts");
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
}
