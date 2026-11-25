import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

type DeckSnapshot = {
  loaded: boolean;
  filename: string;
  duration_seconds: number;
  position_seconds: number;
  playing: boolean;
  sample_rate: number;
  channels: number;
  gain: number;
  low: number;
  mid: number;
  high: number;
  waveform: number[];
};

type MixerSnapshot = {
  deck_a: DeckSnapshot;
  deck_b: DeckSnapshot;
  master_volume: number;
  crossfader: number;
  master_meter: number;
  selected_device: string | null;
  output_error: boolean;
};

type DeviceInfo = { id: string; name: string };

const emptyDeck = (): DeckSnapshot => ({
  loaded: false,
  filename: "",
  duration_seconds: 0,
  position_seconds: 0,
  playing: false,
  sample_rate: 0,
  channels: 0,
  gain: 1,
  low: 0,
  mid: 0,
  high: 0,
  waveform: [],
});

const emptyMixer = (): MixerSnapshot => ({
  deck_a: emptyDeck(),
  deck_b: emptyDeck(),
  master_volume: 0.8,
  crossfader: 0.5,
  master_meter: 0,
  selected_device: null,
  output_error: false,
});

function formatTime(seconds: number) {
  if (!Number.isFinite(seconds)) return "0:00";
  const whole = Math.max(0, Math.floor(seconds));
  return `${Math.floor(whole / 60)}:${String(whole % 60).padStart(2, "0")}`;
}

function Waveform({
  deck,
  deckIndex,
}: {
  deck: DeckSnapshot;
  deckIndex: number;
}) {
  if (!deck.loaded) {
    return <div className="waveform empty-waveform">No track loaded</div>;
  }
  const progress =
    deck.duration_seconds > 0
      ? deck.position_seconds / deck.duration_seconds
      : 0;
  return (
    <div
      className="waveform"
      aria-label={`Deck ${deckIndex === 0 ? "A" : "B"} waveform`}
    >
      {deck.waveform.map((peak, index) => (
        <span
          key={index}
          className={index / deck.waveform.length <= progress ? "played" : ""}
          style={{ height: `${Math.max(8, peak * 90)}%` }}
        />
      ))}
    </div>
  );
}

function Deck({
  deck,
  deckIndex,
  onChange,
  onError,
}: {
  deck: DeckSnapshot;
  deckIndex: number;
  onChange: (next: MixerSnapshot) => void;
  onError: (message: string) => void;
}) {
  const label = deckIndex === 0 ? "A" : "B";
  const [loading, setLoading] = useState(false);
  const invokeMixer = async (
    command: string,
    args: Record<string, unknown>,
  ) => {
    try {
      onChange(await invoke<MixerSnapshot>(command, args));
    } catch (cause) {
      onError(cause instanceof Error ? cause.message : String(cause));
    }
  };
  const loadTrack = async () => {
    setLoading(true);
    onError("");
    await invokeMixer("pick_track", { deck: deckIndex });
    setLoading(false);
  };

  const seek = (value: number) => {
    void invokeMixer("seek_track", { deck: deckIndex, position: value });
  };

  return (
    <section className="deck" aria-label={`Deck ${label}`}>
      <header className="deck-header">
        <div>
          <p className="deck-label">DECK {label}</p>
          <h2 title={deck.filename}>{deck.filename || "No track loaded"}</h2>
        </div>
        <button
          className="load-button"
          type="button"
          disabled={loading}
          onClick={() => void loadTrack()}
        >
          {loading ? "Loading…" : "Load WAV"}
        </button>
      </header>

      <Waveform deck={deck} deckIndex={deckIndex} />

      <div className="transport">
        <button
          className="transport-button"
          type="button"
          disabled={!deck.loaded || loading}
          onClick={() =>
            void invokeMixer("set_playing", {
              deck: deckIndex,
              playing: !deck.playing,
            })
          }
        >
          {deck.playing ? "Pause" : "Play"}
        </button>
        <button
          className="transport-button"
          type="button"
          disabled={!deck.loaded || loading}
          onClick={() => void invokeMixer("stop_track", { deck: deckIndex })}
        >
          Stop
        </button>
        <span className="time">
          {formatTime(deck.position_seconds)} /{" "}
          {formatTime(deck.duration_seconds)}
        </span>
      </div>

      <input
        className="seek"
        aria-label={`Deck ${label} position`}
        type="range"
        min="0"
        max={deck.duration_seconds || 1}
        step="0.01"
        value={deck.position_seconds}
        disabled={!deck.loaded || loading}
        onChange={(event) => seek(Number(event.target.value))}
      />

      <div className="track-info">
        <span>{deck.sample_rate ? `${deck.sample_rate / 1000} kHz` : "—"}</span>
        <span>
          {deck.channels
            ? `${deck.channels} channel${deck.channels === 1 ? "" : "s"}`
            : "—"}
        </span>
      </div>

      <div className="channel-strip">
        {(
          [
            ["gain", "Gain", deck.gain, 0, 1.5, 0.01],
            ["low", "Low", deck.low, -12, 12, 0.1],
            ["mid", "Mid", deck.mid, -12, 12, 0.1],
            ["high", "High", deck.high, -12, 12, 0.1],
          ] as const
        ).map(([key, name, value, min, max, step]) => (
          <label key={key}>
            <span>{name}</span>
            <input
              type="range"
              min={min}
              max={max}
              step={step}
              value={value}
              disabled={loading}
              onChange={(event) => {
                const next = Number(event.target.value);
                void invokeMixer("set_deck_control", {
                  deck: deckIndex,
                  gain: key === "gain" ? next : null,
                  low: key === "low" ? next : null,
                  mid: key === "mid" ? next : null,
                  high: key === "high" ? next : null,
                });
              }}
            />
            <output>
              {key === "gain"
                ? `${Math.round(value * 100)}%`
                : `${value.toFixed(1)} dB`}
            </output>
          </label>
        ))}
      </div>
    </section>
  );
}

export function App() {
  const [mixer, setMixer] = useState<MixerSnapshot>(emptyMixer);
  const [devices, setDevices] = useState<DeviceInfo[]>([]);
  const [error, setError] = useState("");

  useEffect(() => {
    const initialize = async () => {
      try {
        const outputDevices = await invoke<DeviceInfo[]>("list_devices");
        setDevices(outputDevices);
        const snapshot = await invoke<MixerSnapshot>("select_device", {
          deviceId: "default",
        });
        setMixer(snapshot);
      } catch (cause) {
        setError(cause instanceof Error ? cause.message : String(cause));
      }
    };
    void initialize();
    const timer = window.setInterval(() => {
      invoke<MixerSnapshot>("engine_snapshot")
        .then(setMixer)
        .catch((cause) => setError(String(cause)));
    }, 100);
    return () => window.clearInterval(timer);
  }, []);

  const setMixerControl = (command: string, args: Record<string, unknown>) => {
    setError("");
    void invoke<MixerSnapshot>(command, args)
      .then(setMixer)
      .catch((cause) => setError(String(cause)));
  };

  const refreshDevices = async () => {
    setError("");
    try {
      setDevices(await invoke<DeviceInfo[]>("list_devices"));
    } catch (cause) {
      setError(String(cause));
    }
  };

  return (
    <main className="app-shell">
      <header className="topbar">
        <div>
          <p className="eyebrow">SAYT WORKSPACE</p>
          <h1>SAYT</h1>
        </div>
        <label className="device-picker">
          <span>Output device</span>
          <select
            value={mixer.selected_device ?? "default"}
            onChange={(event) =>
              setMixerControl("select_device", { deviceId: event.target.value })
            }
          >
            <option value="default">System default</option>
            {devices.map((device) => (
              <option key={device.id} value={device.id}>
                {device.name}
              </option>
            ))}
          </select>
        </label>
      </header>

      {(error || mixer.output_error) && (
        <div className="error-banner" role="alert">
          <span>
            {error ||
              "Audio output stopped. Re-select an output device to retry."}
          </span>
          <button
            className="recovery-button"
            type="button"
            onClick={() =>
              setMixerControl("select_device", {
                deviceId: mixer.selected_device ?? "default",
              })
            }
          >
            Retry output
          </button>
          <button
            className="recovery-button"
            type="button"
            onClick={() => void refreshDevices()}
          >
            Refresh devices
          </button>
        </div>
      )}

      <div className="workspace">
        <Deck
          deck={mixer.deck_a}
          deckIndex={0}
          onChange={setMixer}
          onError={setError}
        />
        <Deck
          deck={mixer.deck_b}
          deckIndex={1}
          onChange={setMixer}
          onError={setError}
        />
      </div>

      <footer className="mixer">
        <label>
          <span>Master</span>
          <input
            type="range"
            min="0"
            max="1"
            step="0.01"
            value={mixer.master_volume}
            onChange={(event) =>
              setMixerControl("set_mixer", {
                master: Number(event.target.value),
                crossfader: null,
              })
            }
          />
        </label>
        <div className="meter" aria-label="Master audio meter">
          <span
            style={{ width: `${Math.min(100, mixer.master_meter * 100)}%` }}
          />
        </div>
        <label className="crossfader">
          <span>A &nbsp; Crossfader &nbsp; B</span>
          <input
            type="range"
            min="0"
            max="1"
            step="0.01"
            value={mixer.crossfader}
            onChange={(event) =>
              setMixerControl("set_mixer", {
                master: null,
                crossfader: Number(event.target.value),
              })
            }
          />
        </label>
      </footer>
    </main>
  );
}
