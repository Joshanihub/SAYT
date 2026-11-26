//! Device lifecycle, command handling, transport state, and audio callback.

use std::{
    path::Path,
    sync::{
        atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering},
        Arc,
    },
};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    FromSample, Sample, SizedSample,
};
use rtrb::{Consumer, Producer, PushError, RingBuffer};
use serde::{Deserialize, Serialize};

use crate::{
    decoder::{decode_track, StereoFrame, Track},
    dsp::{EqBank, EqCoefficients},
    hot_cue::{empty_hot_cue_slots, validate_hot_cue_slots, HotCueSlots},
};

const AUDIO_COMMAND_CAPACITY: usize = 64;
const RETIRED_TRACK_CAPACITY: usize = 128;
const MAX_COMMANDS_PER_CALLBACK: usize = 16;
const METER_DECAY_PER_CALLBACK: f32 = 0.92;

type OpenedStream = (
    cpal::Stream,
    Producer<AudioCommand>,
    Consumer<Arc<Track>>,
    u32,
);

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DeviceInfo {
    pub id: String,
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct DeckSnapshot {
    pub loaded: bool,
    pub filename: String,
    pub source_track_id: Option<String>,
    pub duration_seconds: f64,
    pub position_seconds: f64,
    pub playing: bool,
    pub sample_rate: u32,
    pub channels: u16,
    pub gain: f32,
    pub low: f32,
    pub mid: f32,
    pub high: f32,
    pub waveform: Vec<f32>,
    pub hot_cues: HotCueSlots,
    pub hot_cue_warning: Option<String>,
    pub active_hot_cue_slot: Option<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct MixerSnapshot {
    pub deck_a: DeckSnapshot,
    pub deck_b: DeckSnapshot,
    pub master_volume: f32,
    pub crossfader: f32,
    pub master_meter: f32,
    pub selected_device: Option<String>,
    pub output_error: bool,
}

#[derive(Clone)]
struct DeckSettings {
    track: Option<Arc<Track>>,
    generation: u64,
    position_seconds: f64,
    playing: bool,
    gain: f32,
    low: f32,
    mid: f32,
    high: f32,
    hot_cues: HotCueSlots,
    hot_cue_warning: Option<String>,
    active_hot_cue_slot: Option<usize>,
}

impl Default for DeckSettings {
    fn default() -> Self {
        Self {
            track: None,
            generation: 0,
            position_seconds: 0.0,
            playing: false,
            gain: 1.0,
            low: 0.0,
            mid: 0.0,
            high: 0.0,
            hot_cues: empty_hot_cue_slots(),
            hot_cue_warning: None,
            active_hot_cue_slot: None,
        }
    }
}

struct EngineSettings {
    decks: [DeckSettings; 2],
    master_volume: f32,
    crossfader: f32,
}

impl Default for EngineSettings {
    fn default() -> Self {
        Self {
            decks: std::array::from_fn(|_| DeckSettings::default()),
            master_volume: 0.8,
            crossfader: 0.5,
        }
    }
}

pub struct Engine {
    settings: EngineSettings,
    output_sample_rate: u32,
    command_sender: Option<Producer<AudioCommand>>,
    retired_tracks: Option<Consumer<Arc<Track>>>,
    stream: Option<cpal::Stream>,
    selected_device: Option<String>,
    telemetry: Arc<Telemetry>,
}

impl Engine {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            settings: EngineSettings::default(),
            output_sample_rate: 48_000,
            command_sender: None,
            retired_tracks: None,
            stream: None,
            selected_device: None,
            telemetry: Arc::new(Telemetry::default()),
        })
    }

    pub fn devices(&self) -> Result<Vec<DeviceInfo>, String> {
        enumerate_devices()
    }

    pub fn select_device(&mut self, device_id: Option<&str>) -> Result<MixerSnapshot, String> {
        let requested_id = device_id.unwrap_or("default");
        if self.selected_device.as_deref() == Some(requested_id)
            && !self.telemetry.output_error.load(Ordering::Acquire)
        {
            return Ok(self.snapshot());
        }

        self.drain_retired_tracks();
        let previous_stream = self.stream.take();
        let pause_error = if let Some(stream) = &previous_stream {
            let error = stream.pause().err();
            self.capture_playback_position();
            error.map(|error| {
                tracing::warn!(
                    component = "audio-engine",
                    check = "pause_current_output_stream",
                    error = %error,
                    "Could not pause the current output stream before switching devices"
                );
                format!("Could not pause the current output device: {error}")
            })
        } else {
            None
        };

        let opened = attempt_output_replacement(
            previous_stream,
            pause_error,
            || self.open_stream(requested_id),
            |stream| {
                stream
                    .play()
                    .map_err(|error| format!("Could not resume the current output device: {error}"))
            },
        );
        match opened {
            Ok((stream, command_sender, retired_tracks, sample_rate)) => {
                self.stream = Some(stream);
                self.command_sender = Some(command_sender);
                self.retired_tracks = Some(retired_tracks);
                self.output_sample_rate = sample_rate;
                self.selected_device = Some(requested_id.to_string());
                self.telemetry.output_error.store(false, Ordering::Release);
                self.telemetry
                    .failure_logged
                    .store(false, Ordering::Release);
                Ok(self.snapshot())
            }
            Err((failure, previous_stream)) => {
                self.stream = previous_stream;
                if !failure.previous_stream_usable {
                    self.telemetry.output_error.store(true, Ordering::Release);
                    self.telemetry
                        .failure_logged
                        .store(false, Ordering::Release);
                }
                Err(failure.message)
            }
        }
    }

    pub fn load_track(&mut self, deck: usize, path: &Path) -> Result<MixerSnapshot, String> {
        validate_deck(deck)?;
        self.ensure_output_selected()?;
        let track = Arc::new(decode_track(path)?);
        let generation = self.settings.decks[deck].generation.wrapping_add(1);
        self.enqueue(AudioCommand::LoadTrack {
            deck,
            generation,
            track: Arc::clone(&track),
        })?;
        self.settings.decks[deck].track = Some(track);
        self.settings.decks[deck].generation = generation;
        self.settings.decks[deck].position_seconds = 0.0;
        self.settings.decks[deck].playing = false;
        self.settings.decks[deck].active_hot_cue_slot = None;
        self.settings.decks[deck].hot_cues = empty_hot_cue_slots();
        self.settings.decks[deck].hot_cue_warning = None;
        Ok(self.snapshot())
    }

    pub fn set_playing(&mut self, deck: usize, playing: bool) -> Result<MixerSnapshot, String> {
        validate_deck(deck)?;
        self.ensure_output_selected()?;
        if self.settings.decks[deck].track.is_none() && playing {
            return Err("Load a WAV file before starting playback".to_string());
        }
        self.enqueue(AudioCommand::SetPlaying { deck, playing })?;
        self.settings.decks[deck].playing = playing;
        self.telemetry.playing[deck].store(playing, Ordering::Release);
        Ok(self.snapshot())
    }

    pub fn stop(&mut self, deck: usize) -> Result<MixerSnapshot, String> {
        validate_deck(deck)?;
        self.enqueue(AudioCommand::Stop { deck })?;
        self.settings.decks[deck].position_seconds = 0.0;
        self.settings.decks[deck].playing = false;
        self.settings.decks[deck].active_hot_cue_slot = None;
        self.telemetry
            .set_deck(deck, self.settings.decks[deck].generation, 0.0, false);
        Ok(self.snapshot())
    }

    pub fn seek(&mut self, deck: usize, position_seconds: f64) -> Result<MixerSnapshot, String> {
        self.seek_internal(deck, position_seconds, None)
    }

    pub fn set_hot_cues(
        &mut self,
        deck: usize,
        expected_source_track_id: &str,
        hot_cues: HotCueSlots,
        warning: Option<String>,
    ) -> Result<MixerSnapshot, String> {
        validate_deck(deck)?;
        let settings = &self.settings.decks[deck];
        let Some(track) = &settings.track else {
            return Err("Load a WAV into this deck before editing hot cues".to_string());
        };
        if track.source_track_id != expected_source_track_id {
            return Err("This deck changed tracks. Select the hot cue again.".to_string());
        }
        validate_hot_cue_slots(&hot_cues, track.duration_seconds)?;

        let settings = &mut self.settings.decks[deck];
        settings.hot_cues = hot_cues;
        settings.hot_cue_warning = warning;
        if settings
            .active_hot_cue_slot
            .is_some_and(|slot| settings.hot_cues[slot].is_none())
        {
            settings.active_hot_cue_slot = None;
        }
        Ok(self.snapshot())
    }

    pub fn trigger_hot_cue(
        &mut self,
        deck: usize,
        slot: usize,
        expected_source_track_id: &str,
    ) -> Result<MixerSnapshot, String> {
        validate_deck(deck)?;
        let settings = &self.settings.decks[deck];
        let Some(track) = &settings.track else {
            return Err("Load a WAV into this deck before triggering a hot cue".to_string());
        };
        if track.source_track_id != expected_source_track_id {
            return Err("This deck changed tracks. Select the hot cue again.".to_string());
        }
        let Some(cue) = settings.hot_cues.get(slot).and_then(Option::as_ref) else {
            return Err("This hot cue slot is empty. Set a cue before triggering it.".to_string());
        };
        self.seek_internal(deck, cue.position_seconds, Some(slot))
    }

    fn seek_internal(
        &mut self,
        deck: usize,
        position_seconds: f64,
        active_hot_cue_slot: Option<usize>,
    ) -> Result<MixerSnapshot, String> {
        validate_deck(deck)?;
        if !position_seconds.is_finite() {
            return Err("Seek position must be a finite number of seconds".to_string());
        }
        let Some(track) = &self.settings.decks[deck].track else {
            if active_hot_cue_slot.is_some() {
                return Err("Load a WAV into this deck before triggering a hot cue".to_string());
            }
            self.ensure_output_selected()?;
            return Ok(self.snapshot());
        };
        let position = position_seconds.clamp(0.0, track.duration_seconds);
        let generation = self.settings.decks[deck].generation;
        self.enqueue(AudioCommand::Seek {
            deck,
            generation,
            position,
        })?;
        self.settings.decks[deck].position_seconds = position;
        self.settings.decks[deck].active_hot_cue_slot = active_hot_cue_slot;
        self.telemetry.position_seconds[deck].store(position.to_bits(), Ordering::Release);
        Ok(self.snapshot())
    }

    pub fn set_control(
        &mut self,
        deck: usize,
        gain: Option<f32>,
        low: Option<f32>,
        mid: Option<f32>,
        high: Option<f32>,
    ) -> Result<MixerSnapshot, String> {
        validate_deck(deck)?;
        self.ensure_output_selected()?;
        let current = &self.settings.decks[deck];
        let gain = validated_control(gain, current.gain, 0.0, 1.5, "gain")?;
        let low = validated_control(low, current.low, -12.0, 12.0, "low EQ")?;
        let mid = validated_control(mid, current.mid, -12.0, 12.0, "mid EQ")?;
        let high = validated_control(high, current.high, -12.0, 12.0, "high EQ")?;
        let coefficients = EqCoefficients::from_gains(self.output_sample_rate, low, mid, high);
        self.enqueue(AudioCommand::SetDeckControls {
            deck,
            gain,
            coefficients,
        })?;
        let settings = &mut self.settings.decks[deck];
        settings.gain = gain;
        settings.low = low;
        settings.mid = mid;
        settings.high = high;
        Ok(self.snapshot())
    }

    pub fn set_mixer(
        &mut self,
        master_volume: Option<f32>,
        crossfader: Option<f32>,
    ) -> Result<MixerSnapshot, String> {
        self.ensure_output_selected()?;
        let master_volume = validated_control(
            master_volume,
            self.settings.master_volume,
            0.0,
            1.0,
            "master volume",
        )?;
        let crossfader =
            validated_control(crossfader, self.settings.crossfader, 0.0, 1.0, "crossfader")?;
        let (deck_a_gain, deck_b_gain) = crossfader_gains(crossfader);
        self.enqueue(AudioCommand::SetMixer {
            master_volume,
            deck_a_gain,
            deck_b_gain,
        })?;
        self.settings.master_volume = master_volume;
        self.settings.crossfader = crossfader;
        Ok(self.snapshot())
    }

    pub fn snapshot(&mut self) -> MixerSnapshot {
        self.drain_retired_tracks();
        let output_error = self.telemetry.output_error.load(Ordering::Acquire);
        if output_error && !self.telemetry.failure_logged.swap(true, Ordering::AcqRel) {
            tracing::error!(
                component = "audio-engine",
                check = "cpal_output_stream",
                "Audio output stream reported a device error"
            );
        }
        let deck_a = self.snapshot_deck(0);
        let deck_b = self.snapshot_deck(1);
        MixerSnapshot {
            deck_a,
            deck_b,
            master_volume: self.settings.master_volume,
            crossfader: self.settings.crossfader,
            master_meter: f32::from_bits(self.telemetry.master_meter.load(Ordering::Acquire)),
            selected_device: self.selected_device.clone(),
            output_error,
        }
    }

    fn snapshot_deck(&self, deck: usize) -> DeckSnapshot {
        let settings = &self.settings.decks[deck];
        let Some(track) = &settings.track else {
            return DeckSnapshot {
                playing: settings.playing,
                gain: settings.gain,
                low: settings.low,
                mid: settings.mid,
                high: settings.high,
                hot_cues: settings.hot_cues.clone(),
                hot_cue_warning: settings.hot_cue_warning.clone(),
                ..DeckSnapshot::default()
            };
        };
        let observed_generation = self.telemetry.track_generation[deck].load(Ordering::Acquire);
        let (position_seconds, playing) = if observed_generation == settings.generation {
            let position_bits = self.telemetry.position_seconds[deck].load(Ordering::Acquire);
            let playing = self.telemetry.playing[deck].load(Ordering::Acquire);
            let confirmed_generation =
                self.telemetry.track_generation[deck].load(Ordering::Acquire);
            if confirmed_generation == settings.generation {
                (f64::from_bits(position_bits), playing)
            } else {
                (settings.position_seconds, settings.playing)
            }
        } else {
            (settings.position_seconds, settings.playing)
        };
        DeckSnapshot {
            loaded: true,
            filename: track.filename.clone(),
            source_track_id: Some(track.source_track_id.clone()),
            duration_seconds: track.duration_seconds,
            position_seconds: position_seconds.clamp(0.0, track.duration_seconds),
            playing,
            sample_rate: track.sample_rate,
            channels: track.channels,
            gain: settings.gain,
            low: settings.low,
            mid: settings.mid,
            high: settings.high,
            waveform: track.waveform.to_vec(),
            hot_cues: settings.hot_cues.clone(),
            hot_cue_warning: settings.hot_cue_warning.clone(),
            active_hot_cue_slot: settings
                .active_hot_cue_slot
                .filter(|slot| settings.hot_cues.get(*slot).is_some_and(Option::is_some)),
        }
    }

    fn open_stream(&mut self, device_id: &str) -> Result<OpenedStream, String> {
        let host = cpal::default_host();
        let device = if device_id == "default" {
            host.default_output_device()
                .ok_or_else(|| "No default audio output device is available".to_string())?
        } else {
            let index = device_id.parse::<usize>().map_err(|_| {
                "The selected output device is invalid. Refresh the device list and choose again."
                    .to_string()
            })?;
            host.output_devices()
                .map_err(|error| format!("Could not enumerate output devices: {error}"))?
                .nth(index)
                .ok_or_else(|| {
                    "The selected output device is no longer available. Refresh the device list and choose again."
                        .to_string()
                })?
        };
        let device_name = device
            .name()
            .unwrap_or_else(|_| "Unnamed output device".to_string());
        let supported = device.default_output_config().map_err(|error| {
            format!("Could not read the output format for '{device_name}': {error}")
        })?;
        let config = supported.config();
        let output_rate = config.sample_rate.0;
        if output_rate == 0 {
            return Err(format!(
                "Output device '{device_name}' reported a zero sample rate"
            ));
        }
        if config.channels == 0 {
            return Err(format!(
                "Output device '{device_name}' reported zero channels"
            ));
        }

        let (command_sender, command_receiver) = RingBuffer::new(AUDIO_COMMAND_CAPACITY);
        let (retired_sender, retired_tracks) = RingBuffer::new(RETIRED_TRACK_CAPACITY);
        let playback = PlaybackState::new(&self.settings, output_rate);
        let telemetry = Arc::clone(&self.telemetry);
        let stream_result = match supported.sample_format() {
            cpal::SampleFormat::F32 => build_stream::<f32>(
                &device,
                &config,
                config.channels as usize,
                playback,
                command_receiver,
                retired_sender,
                telemetry,
            ),
            cpal::SampleFormat::I16 => build_stream::<i16>(
                &device,
                &config,
                config.channels as usize,
                playback,
                command_receiver,
                retired_sender,
                telemetry,
            ),
            cpal::SampleFormat::U16 => build_stream::<u16>(
                &device,
                &config,
                config.channels as usize,
                playback,
                command_receiver,
                retired_sender,
                telemetry,
            ),
            other => Err(format!(
                "Output device '{device_name}' uses unsupported sample format {other:?}"
            )),
        };
        let stream = stream_result?;
        stream
            .play()
            .map_err(|error| format!("Could not start audio output on '{device_name}': {error}"))?;

        Ok((stream, command_sender, retired_tracks, output_rate))
    }

    fn enqueue(&mut self, command: AudioCommand) -> Result<(), String> {
        self.drain_retired_tracks();
        let sender = self
            .command_sender
            .as_mut()
            .ok_or_else(|| "Select an audio output device before using the decks".to_string())?;
        sender.push(command).map_err(|_| {
            "The audio control queue is full; wait briefly and retry the command".to_string()
        })
    }

    fn drain_retired_tracks(&mut self) {
        if let Some(retired_tracks) = &mut self.retired_tracks {
            while retired_tracks.pop().is_ok() {}
        }
    }

    fn capture_playback_position(&mut self) {
        for deck in 0..2 {
            let generation = self.telemetry.track_generation[deck].load(Ordering::Acquire);
            if generation == self.settings.decks[deck].generation {
                let position = self.telemetry.position_seconds[deck].load(Ordering::Acquire);
                let playing = self.telemetry.playing[deck].load(Ordering::Acquire);
                if self.telemetry.track_generation[deck].load(Ordering::Acquire) == generation {
                    self.settings.decks[deck].position_seconds = f64::from_bits(position);
                    self.settings.decks[deck].playing = playing;
                }
            }
        }
    }

    fn ensure_output_selected(&self) -> Result<(), String> {
        if self.stream.is_some() {
            Ok(())
        } else {
            Err("Select an audio output device before using the decks".to_string())
        }
    }
}

fn enumerate_devices() -> Result<Vec<DeviceInfo>, String> {
    let host = cpal::default_host();
    let devices = host
        .output_devices()
        .map_err(|error| format!("Could not enumerate output devices: {error}"))?;
    Ok(devices
        .enumerate()
        .map(|(index, device)| DeviceInfo {
            id: index.to_string(),
            name: device
                .name()
                .unwrap_or_else(|_| format!("Unnamed output device {index}")),
        })
        .collect())
}

#[derive(Debug)]
struct DeviceSwitchFailure {
    message: String,
    previous_stream_usable: bool,
}

fn attempt_output_replacement<T, S>(
    previous_stream: Option<S>,
    pause_error: Option<String>,
    open_replacement: impl FnOnce() -> Result<T, String>,
    resume_previous: impl FnOnce(&S) -> Result<(), String>,
) -> Result<T, (DeviceSwitchFailure, Option<S>)> {
    let open_error = match open_replacement() {
        Ok(next_stream) => return Ok(next_stream),
        Err(error) => error,
    };

    let (previous_stream_usable, resume_error) = match previous_stream.as_ref() {
        Some(stream) => match resume_previous(stream) {
            Ok(()) => (true, None),
            Err(error) => (false, Some(error)),
        },
        None => (false, None),
    };

    let mut messages = vec![open_error];
    if let Some(error) = pause_error {
        messages.push(error);
    }
    if let Some(error) = resume_error {
        messages.push(error);
    }
    messages.push(if previous_stream_usable {
        "The previous audio output stream was resumed.".to_string()
    } else if previous_stream.is_some() {
        "The previous audio output stream could not be resumed.".to_string()
    } else {
        "No previous audio output stream is available.".to_string()
    });

    Err((
        DeviceSwitchFailure {
            message: messages.join(" "),
            previous_stream_usable,
        },
        previous_stream,
    ))
}

fn validate_deck(deck: usize) -> Result<(), String> {
    if deck < 2 {
        Ok(())
    } else {
        Err("Deck index must be 0 or 1".to_string())
    }
}

fn validated_control(
    value: Option<f32>,
    current: f32,
    minimum: f32,
    maximum: f32,
    name: &str,
) -> Result<f32, String> {
    let Some(value) = value else {
        return Ok(current);
    };
    if !value.is_finite() {
        return Err(format!("{name} must be a finite number"));
    }
    Ok(value.clamp(minimum, maximum))
}

fn crossfader_gains(crossfader: f32) -> (f32, f32) {
    let angle = std::f32::consts::FRAC_PI_2 * crossfader;
    (angle.cos(), angle.sin())
}

enum AudioCommand {
    LoadTrack {
        deck: usize,
        generation: u64,
        track: Arc<Track>,
    },
    SetPlaying {
        deck: usize,
        playing: bool,
    },
    Stop {
        deck: usize,
    },
    Seek {
        deck: usize,
        generation: u64,
        position: f64,
    },
    SetDeckControls {
        deck: usize,
        gain: f32,
        coefficients: EqCoefficients,
    },
    SetMixer {
        master_volume: f32,
        deck_a_gain: f32,
        deck_b_gain: f32,
    },
}

struct PlaybackDeck {
    track: Option<Arc<Track>>,
    generation: u64,
    position_frames: f64,
    playing: bool,
    gain: f32,
    eq: EqBank,
}

impl PlaybackDeck {
    fn new(settings: &DeckSettings, sample_rate: u32) -> Self {
        Self {
            track: settings.track.clone(),
            generation: settings.generation,
            position_frames: settings
                .track
                .as_ref()
                .map(|track| {
                    settings.position_seconds.clamp(0.0, track.duration_seconds)
                        * f64::from(track.sample_rate)
                })
                .unwrap_or(0.0),
            playing: settings.playing,
            gain: settings.gain,
            eq: EqBank::new(EqCoefficients::from_gains(
                sample_rate,
                settings.low,
                settings.mid,
                settings.high,
            )),
        }
    }
}

struct PlaybackState {
    decks: [PlaybackDeck; 2],
    output_sample_rate: f64,
    master_volume: f32,
    deck_a_gain: f32,
    deck_b_gain: f32,
    meter_peak: f32,
    pending_command: Option<AudioCommand>,
}

impl PlaybackState {
    fn new(settings: &EngineSettings, sample_rate: u32) -> Self {
        let (deck_a_gain, deck_b_gain) = crossfader_gains(settings.crossfader);
        Self {
            decks: std::array::from_fn(|deck| {
                PlaybackDeck::new(&settings.decks[deck], sample_rate)
            }),
            output_sample_rate: f64::from(sample_rate),
            master_volume: settings.master_volume,
            deck_a_gain,
            deck_b_gain,
            meter_peak: 0.0,
            pending_command: None,
        }
    }

    fn apply(
        &mut self,
        command: AudioCommand,
        retired_tracks: &mut Producer<Arc<Track>>,
    ) -> Result<(), AudioCommand> {
        match command {
            AudioCommand::LoadTrack {
                deck,
                generation,
                track,
            } => {
                if let Some(current_track) = self.decks[deck].track.take() {
                    if let Err(PushError::Full(current_track)) = retired_tracks.push(current_track)
                    {
                        self.decks[deck].track = Some(current_track);
                        return Err(AudioCommand::LoadTrack {
                            deck,
                            generation,
                            track,
                        });
                    }
                }
                let deck_state = &mut self.decks[deck];
                deck_state.track = Some(track);
                deck_state.generation = generation;
                deck_state.position_frames = 0.0;
                deck_state.playing = false;
                Ok(())
            }
            AudioCommand::SetPlaying { deck, playing } => {
                self.decks[deck].playing = playing && self.decks[deck].track.is_some();
                Ok(())
            }
            AudioCommand::Stop { deck } => {
                self.decks[deck].playing = false;
                self.decks[deck].position_frames = 0.0;
                Ok(())
            }
            AudioCommand::Seek {
                deck,
                generation,
                position,
            } => {
                let deck_state = &mut self.decks[deck];
                if deck_state.generation == generation {
                    if let Some(track) = &deck_state.track {
                        deck_state.position_frames = position.clamp(0.0, track.duration_seconds)
                            * f64::from(track.sample_rate);
                    }
                }
                Ok(())
            }
            AudioCommand::SetDeckControls {
                deck,
                gain,
                coefficients,
            } => {
                self.decks[deck].gain = gain;
                self.decks[deck].eq.set_coefficients(coefficients);
                Ok(())
            }
            AudioCommand::SetMixer {
                master_volume,
                deck_a_gain,
                deck_b_gain,
            } => {
                self.master_volume = master_volume;
                self.deck_a_gain = deck_a_gain;
                self.deck_b_gain = deck_b_gain;
                Ok(())
            }
        }
    }
}

#[derive(Default)]
struct Telemetry {
    position_seconds: [AtomicU64; 2],
    playing: [AtomicBool; 2],
    track_generation: [AtomicU64; 2],
    master_meter: AtomicU32,
    output_error: AtomicBool,
    failure_logged: AtomicBool,
}

impl Telemetry {
    fn set_deck(&self, deck: usize, generation: u64, position_seconds: f64, playing: bool) {
        self.position_seconds[deck].store(position_seconds.to_bits(), Ordering::Release);
        self.playing[deck].store(playing, Ordering::Release);
        self.track_generation[deck].store(generation, Ordering::Release);
    }
}

fn build_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    channels: usize,
    mut playback: PlaybackState,
    mut commands: Consumer<AudioCommand>,
    mut retired_tracks: Producer<Arc<Track>>,
    telemetry: Arc<Telemetry>,
) -> Result<cpal::Stream, String>
where
    T: SizedSample + FromSample<f32> + Sample,
{
    let callback_telemetry = Arc::clone(&telemetry);
    device
        .build_output_stream(
            config,
            move |output: &mut [T], _: &cpal::OutputCallbackInfo| {
                render::<T>(
                    output,
                    channels,
                    &mut playback,
                    &mut commands,
                    &mut retired_tracks,
                    &callback_telemetry,
                );
            },
            move |_error| {
                telemetry.output_error.store(true, Ordering::Release);
            },
            None,
        )
        .map_err(|error| format!("Could not create the audio output stream: {error}"))
}

fn render<T>(
    output: &mut [T],
    channels: usize,
    playback: &mut PlaybackState,
    commands: &mut Consumer<AudioCommand>,
    retired_tracks: &mut Producer<Arc<Track>>,
    telemetry: &Telemetry,
) where
    T: FromSample<f32> + Sample,
{
    process_commands(playback, commands, retired_tracks);
    if channels == 0 {
        output.fill(T::EQUILIBRIUM);
        return;
    }
    let frame_count = output.len() / channels;
    let mut peak = playback.meter_peak * METER_DECAY_PER_CALLBACK;
    for frame_index in 0..frame_count {
        let deck_a = next_deck_frame(&mut playback.decks[0], playback.output_sample_rate);
        let deck_b = next_deck_frame(&mut playback.decks[1], playback.output_sample_rate);
        let left = soft_limit(
            (deck_a.left * playback.deck_a_gain + deck_b.left * playback.deck_b_gain)
                * playback.master_volume,
        );
        let right = soft_limit(
            (deck_a.right * playback.deck_a_gain + deck_b.right * playback.deck_b_gain)
                * playback.master_volume,
        );
        peak = peak.max(left.abs()).max(right.abs());

        for channel in 0..channels {
            let sample = match (channels, channel) {
                (1, 0) => (left + right) * 0.5,
                (_, 0) => left,
                (_, 1) => right,
                _ => 0.0,
            };
            output[frame_index * channels + channel] = T::from_sample(sample);
        }
    }
    playback.meter_peak = peak;
    telemetry
        .master_meter
        .store(peak.to_bits(), Ordering::Release);
    for (deck, deck_state) in playback.decks.iter().enumerate() {
        let position = deck_state
            .track
            .as_ref()
            .map(|track| deck_state.position_frames / f64::from(track.sample_rate))
            .unwrap_or(0.0);
        telemetry.set_deck(deck, deck_state.generation, position, deck_state.playing);
    }
    output[frame_count * channels..].fill(T::EQUILIBRIUM);
}

fn process_commands(
    playback: &mut PlaybackState,
    commands: &mut Consumer<AudioCommand>,
    retired_tracks: &mut Producer<Arc<Track>>,
) {
    for _ in 0..MAX_COMMANDS_PER_CALLBACK {
        let command = if let Some(pending) = playback.pending_command.take() {
            pending
        } else {
            match commands.pop() {
                Ok(command) => command,
                Err(_) => return,
            }
        };
        if let Err(command) = playback.apply(command, retired_tracks) {
            playback.pending_command = Some(command);
            return;
        }
    }
}

fn next_deck_frame(deck: &mut PlaybackDeck, output_sample_rate: f64) -> StereoFrame {
    if !deck.playing {
        return StereoFrame::default();
    }
    let Some(track) = &deck.track else {
        deck.playing = false;
        return StereoFrame::default();
    };
    let Some(frame) = sample_track(track, deck.position_frames) else {
        deck.playing = false;
        return StereoFrame::default();
    };
    deck.position_frames += f64::from(track.sample_rate) / output_sample_rate;
    if deck.position_frames >= track.frames.len() as f64 {
        deck.playing = false;
        deck.position_frames = track.frames.len() as f64;
    }
    let filtered = deck.eq.process(frame);
    StereoFrame {
        left: filtered.left * deck.gain,
        right: filtered.right * deck.gain,
    }
}

fn sample_track(track: &Track, position: f64) -> Option<crate::decoder::StereoFrame> {
    let index = position as usize;
    if index >= track.frames.len() {
        return None;
    }
    let next = (index + 1).min(track.frames.len() - 1);
    let fraction = (position - index as f64) as f32;
    let first = track.frames[index];
    let second = track.frames[next];
    Some(crate::decoder::StereoFrame {
        left: first.left + (second.left - first.left) * fraction,
        right: first.right + (second.right - first.right) * fraction,
    })
}

fn soft_limit(input: f32) -> f32 {
    input.tanh()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decoder::{StereoFrame, WAVEFORM_BINS};
    use crate::hot_cue::{default_hot_cue, HotCue, HotCueColor};

    const TEST_SOURCE_TRACK_ID: &str =
        "sha256:0000000000000000000000000000000000000000000000000000000000000000";
    const TEST_SOURCE_TRACK_ID_B: &str =
        "sha256:1111111111111111111111111111111111111111111111111111111111111111";

    fn test_track(samples: &[f32]) -> Arc<Track> {
        test_track_with_source(samples, TEST_SOURCE_TRACK_ID)
    }

    fn test_track_with_source(samples: &[f32], source_track_id: &str) -> Arc<Track> {
        Arc::new(Track {
            frames: Arc::from(
                samples
                    .iter()
                    .map(|sample| StereoFrame {
                        left: *sample,
                        right: *sample,
                    })
                    .collect::<Vec<_>>(),
            ),
            sample_rate: 48_000,
            channels: 2,
            duration_seconds: samples.len() as f64 / 48_000.0,
            filename: "fixture.wav".to_string(),
            source_track_id: source_track_id.to_string(),
            waveform: Arc::from(vec![0.0; WAVEFORM_BINS]),
        })
    }

    fn engine_with_command_queue(
        capacity: usize,
        loaded_tracks: [bool; 2],
        playing: [bool; 2],
    ) -> (Engine, Consumer<AudioCommand>) {
        let mut engine = Engine::new().unwrap();
        engine.selected_device = Some("test-output".to_string());
        let (command_sender, command_receiver) = RingBuffer::new(capacity);
        engine.command_sender = Some(command_sender);
        for deck in 0..2 {
            if loaded_tracks[deck] {
                let source_track_id = if deck == 0 {
                    TEST_SOURCE_TRACK_ID
                } else {
                    TEST_SOURCE_TRACK_ID_B
                };
                engine.settings.decks[deck].track =
                    Some(test_track_with_source(&[0.25; 48_000], source_track_id));
                engine.settings.decks[deck].generation = (deck + 1) as u64;
                engine.settings.decks[deck].playing = playing[deck];
                engine.telemetry.track_generation[deck].store((deck + 1) as u64, Ordering::Release);
                engine.telemetry.playing[deck].store(playing[deck], Ordering::Release);
                engine.settings.decks[deck].hot_cues[0] =
                    Some(default_hot_cue(0, if deck == 0 { 0.5 } else { 0.75 }));
            }
        }
        (engine, command_receiver)
    }

    fn test_playback(track_a: Arc<Track>, track_b: Arc<Track>, crossfader: f32) -> PlaybackState {
        let mut settings = EngineSettings::default();
        settings.decks[0].track = Some(track_a);
        settings.decks[1].track = Some(track_b);
        settings.decks[0].playing = true;
        settings.decks[1].playing = true;
        settings.master_volume = 1.0;
        settings.crossfader = crossfader;
        PlaybackState::new(&settings, 48_000)
    }

    #[test]
    fn equal_power_crossfader_routes_audio_to_the_expected_deck_at_each_end() {
        let (a_only, b_silent) = crossfader_gains(0.0);
        let (a_silent, b_only) = crossfader_gains(1.0);
        let (a_center, b_center) = crossfader_gains(0.5);
        assert_eq!((a_only, b_silent), (1.0, 0.0));
        assert!(a_silent.abs() < 1e-6 && (b_only - 1.0).abs() < 1e-6);
        assert!((a_center * a_center + b_center * b_center - 1.0).abs() < 1e-6);
    }

    #[test]
    fn render_outputs_deck_a_when_crossfader_is_fully_left() {
        let track_a = test_track(&[0.25; 64]);
        let track_b = test_track(&[0.75; 64]);
        let mut playback = test_playback(track_a, track_b, 0.0);
        let (_command_sender, mut commands) = RingBuffer::new(4);
        let (mut retired_tracks, _retired_consumer) = RingBuffer::new(4);
        let telemetry = Telemetry::default();
        let mut output = [0.0_f32; 8];

        render(
            &mut output,
            2,
            &mut playback,
            &mut commands,
            &mut retired_tracks,
            &telemetry,
        );

        assert!(output
            .iter()
            .all(|sample| (*sample - soft_limit(0.25)).abs() < 1e-5));
    }

    #[test]
    fn snapshot_ignores_telemetry_from_a_replaced_track() {
        let mut engine = Engine::new().unwrap();
        engine.settings.decks[0].track = Some(test_track(&[0.25; 48_000]));
        engine.settings.decks[0].generation = 2;
        engine.telemetry.position_seconds[0].store(1.0_f64.to_bits(), Ordering::Release);
        engine.telemetry.playing[0].store(true, Ordering::Release);
        engine.telemetry.track_generation[0].store(1, Ordering::Release);

        let deck = engine.snapshot_deck(0);
        assert_eq!(deck.position_seconds, 0.0);
        assert!(!deck.playing);
    }

    #[test]
    fn hot_cues_trigger_generation_bound_seeks_on_both_decks_and_preserve_play_state() {
        let (mut engine, mut commands) = engine_with_command_queue(4, [true, true], [false, true]);

        let paused = engine.trigger_hot_cue(0, 0, TEST_SOURCE_TRACK_ID).unwrap();
        let playing = engine
            .trigger_hot_cue(1, 0, TEST_SOURCE_TRACK_ID_B)
            .unwrap();

        assert_eq!(paused.deck_a.position_seconds, 0.5);
        assert!(!paused.deck_a.playing);
        assert_eq!(paused.deck_a.active_hot_cue_slot, Some(0));
        assert_eq!(playing.deck_b.position_seconds, 0.75);
        assert!(playing.deck_b.playing);
        assert_eq!(playing.deck_b.active_hot_cue_slot, Some(0));
        assert!(matches!(
            commands.pop(),
            Ok(AudioCommand::Seek {
                deck: 0,
                generation: 1,
                position: 0.5
            })
        ));
        assert!(matches!(
            commands.pop(),
            Ok(AudioCommand::Seek {
                deck: 1,
                generation: 2,
                position: 0.75
            })
        ));
    }

    #[test]
    fn hot_cue_trigger_rejects_stale_identity_empty_slots_and_unloaded_decks() {
        let (mut engine, mut commands) = engine_with_command_queue(4, [true, false], [false; 2]);

        assert!(engine.trigger_hot_cue(0, 0, "sha256:stale").is_err());
        assert!(engine.trigger_hot_cue(0, 8, TEST_SOURCE_TRACK_ID).is_err());
        assert!(engine.trigger_hot_cue(1, 0, TEST_SOURCE_TRACK_ID).is_err());
        assert!(commands.pop().is_err());
    }

    #[test]
    fn active_cue_feedback_survives_playback_and_clears_on_stop() {
        let (mut engine, _) = engine_with_command_queue(4, [true, false], [true, false]);
        engine.trigger_hot_cue(0, 0, TEST_SOURCE_TRACK_ID).unwrap();
        engine.telemetry.position_seconds[0].store(0.8_f64.to_bits(), Ordering::Release);

        let playing_snapshot = engine.snapshot();
        assert_eq!(playing_snapshot.deck_a.position_seconds, 0.8);
        assert_eq!(playing_snapshot.deck_a.active_hot_cue_slot, Some(0));

        let stopped_snapshot = engine.stop(0).unwrap();
        assert_eq!(stopped_snapshot.deck_a.active_hot_cue_slot, None);
    }

    #[test]
    fn hot_cue_updates_are_deck_local_and_reject_stale_or_invalid_data() {
        let (mut engine, _) = engine_with_command_queue(4, [true, true], [false; 2]);
        let mut deck_a_cues = empty_hot_cue_slots();
        deck_a_cues[2] = Some(HotCue {
            name: "Intro".to_string(),
            color: HotCueColor::Violet,
            position_seconds: 0.75,
        });

        assert!(engine
            .set_hot_cues(0, "sha256:stale", deck_a_cues.clone(), None)
            .is_err());
        let mut invalid_cues = deck_a_cues.clone();
        invalid_cues[2].as_mut().unwrap().position_seconds = 2.0;
        assert!(engine
            .set_hot_cues(0, TEST_SOURCE_TRACK_ID, invalid_cues, None)
            .is_err());

        let updated = engine
            .set_hot_cues(0, TEST_SOURCE_TRACK_ID, deck_a_cues, None)
            .unwrap();
        assert_eq!(updated.deck_a.hot_cues[2].as_ref().unwrap().name, "Intro");
        assert!(updated.deck_b.hot_cues[2].is_none());
        assert_eq!(
            updated.deck_b.hot_cues[0]
                .as_ref()
                .unwrap()
                .position_seconds,
            0.75
        );
    }

    #[test]
    fn full_audio_queue_reports_failed_cue_trigger_without_changing_snapshot() {
        let (mut engine, _commands) = engine_with_command_queue(1, [true, false], [false; 2]);
        engine
            .enqueue(AudioCommand::Stop { deck: 0 })
            .expect("fill the bounded audio queue");

        let error = engine
            .trigger_hot_cue(0, 0, TEST_SOURCE_TRACK_ID)
            .expect_err("a full queue must not claim cue success");
        assert!(error.contains("queue is full"));
        let snapshot = engine.snapshot();
        assert_eq!(snapshot.deck_a.position_seconds, 0.0);
        assert_eq!(snapshot.deck_a.active_hot_cue_slot, None);
    }

    #[test]
    fn callback_ignores_a_seek_for_a_replaced_track_generation() {
        let mut settings = EngineSettings::default();
        settings.decks[0].track = Some(test_track(&[0.25; 48_000]));
        settings.decks[0].generation = 9;
        let mut playback = PlaybackState::new(&settings, 48_000);
        let (mut retired_tracks, _retired_consumer) = RingBuffer::new(1);

        assert!(playback
            .apply(
                AudioCommand::Seek {
                    deck: 0,
                    generation: 8,
                    position: 0.75,
                },
                &mut retired_tracks,
            )
            .is_ok());

        assert_eq!(playback.decks[0].position_frames, 0.0);
    }

    #[test]
    fn render_fills_partial_output_frame_with_equilibrium() {
        let mut playback = test_playback(test_track(&[0.25; 4]), test_track(&[0.75; 4]), 0.0);
        let (_command_sender, mut commands) = RingBuffer::new(4);
        let (mut retired_tracks, _retired_consumer) = RingBuffer::new(4);
        let telemetry = Telemetry::default();
        let mut output = [1.0_f32; 3];

        render(
            &mut output,
            2,
            &mut playback,
            &mut commands,
            &mut retired_tracks,
            &telemetry,
        );

        assert_eq!(output[2], 0.0);
    }

    #[test]
    fn replacement_waits_when_retired_track_queue_is_full() {
        let existing = test_track(&[0.1; 4]);
        let replacement = test_track(&[0.9; 4]);
        let mut playback = PlaybackState::new(&EngineSettings::default(), 48_000);
        playback.decks[0].track = Some(Arc::clone(&existing));

        let (mut command_sender, mut command_consumer) = RingBuffer::new(4);
        assert!(command_sender
            .push(AudioCommand::LoadTrack {
                deck: 0,
                generation: 0,
                track: Arc::clone(&replacement),
            })
            .is_ok());
        let (mut retired_sender, _retired_consumer) = RingBuffer::new(1);
        assert!(retired_sender.push(test_track(&[0.2; 4])).is_ok());

        process_commands(&mut playback, &mut command_consumer, &mut retired_sender);

        assert!(Arc::ptr_eq(
            playback.decks[0].track.as_ref().unwrap(),
            &existing
        ));
        assert!(playback.pending_command.is_some());
    }

    #[test]
    fn limiter_bounds_hot_input() {
        assert!(soft_limit(12.0).abs() <= 1.0);
        assert!(soft_limit(-12.0).abs() <= 1.0);
        assert_eq!(soft_limit(0.0), 0.0);
    }

    #[test]
    fn control_values_reject_non_finite_and_clamp_boundaries() {
        assert!(validated_control(Some(f32::NAN), 0.0, -12.0, 12.0, "EQ").is_err());
        assert_eq!(
            validated_control(Some(100.0), 0.0, -12.0, 12.0, "EQ").unwrap(),
            12.0
        );
        assert_eq!(validate_deck(1), Ok(()));
        assert!(validate_deck(2).is_err());
    }

    #[test]
    fn replacement_stream_is_attempted_after_pause_failure() {
        let previous_stream = ();
        let mut open_attempted = false;

        let result = attempt_output_replacement(
            Some(previous_stream),
            Some("Could not pause the current output device".to_string()),
            || {
                open_attempted = true;
                Ok("replacement")
            },
            |_| panic!("a successful replacement must not resume the previous stream"),
        );

        assert!(open_attempted);
        assert_eq!(result.unwrap(), "replacement");
    }

    #[test]
    fn failed_replacement_resumes_the_previous_stream_when_possible() {
        let previous_stream = ();
        let mut resume_attempted = false;

        let (failure, previous_stream) = attempt_output_replacement::<(), _>(
            Some(previous_stream),
            None,
            || Err("replacement unavailable".to_string()),
            |_| {
                resume_attempted = true;
                Ok(())
            },
        )
        .expect_err("failed replacement is reported");

        assert!(resume_attempted);
        assert!(previous_stream.is_some());
        assert!(failure.previous_stream_usable);
        assert!(failure.message.contains("replacement unavailable"));
        assert!(failure
            .message
            .contains("previous audio output stream was resumed"));
    }

    #[test]
    fn failed_replacement_reports_when_previous_stream_cannot_resume() {
        let previous_stream = ();

        let (failure, previous_stream) = attempt_output_replacement::<(), _>(
            Some(previous_stream),
            Some("pause failed".to_string()),
            || Err("replacement unavailable".to_string()),
            |_| Err("resume failed".to_string()),
        )
        .expect_err("failed replacement is reported");

        assert!(previous_stream.is_some());
        assert!(!failure.previous_stream_usable);
        assert!(failure.message.contains("replacement unavailable"));
        assert!(failure.message.contains("pause failed"));
        assert!(failure.message.contains("resume failed"));
        assert!(failure.message.contains("could not be resumed"));
    }

    #[test]
    fn failed_initial_selection_reports_that_no_previous_stream_is_available() {
        let (failure, previous_stream) = attempt_output_replacement::<(), ()>(
            None,
            None,
            || Err("no default output device".to_string()),
            |_| panic!("there is no previous stream to resume"),
        )
        .expect_err("failed initial selection is reported");

        assert!(previous_stream.is_none());
        assert!(!failure.previous_stream_usable);
        assert!(failure.message.contains("no default output device"));
        assert!(failure.message.contains("No previous audio output stream"));
    }
}
