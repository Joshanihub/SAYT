import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState, type ReactNode } from "react";
import { MaterialIcon } from "./MaterialIcon";

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
type Page = "dj" | "library" | "settings";

const SNAPSHOT_INTERVAL_MS = 100;

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

const getErrorMessage = (cause: unknown) =>
  cause instanceof Error ? cause.message : String(cause);

function formatTime(seconds: number) {
  if (!Number.isFinite(seconds)) return "0:00";
  const whole = Math.max(0, Math.floor(seconds));
  return `${Math.floor(whole / 60)}:${String(whole % 60).padStart(2, "0")}`;
}

export function Waveform({
  deck,
  deckIndex,
}: {
  deck: DeckSnapshot;
  deckIndex: number;
}) {
  if (!deck.loaded) {
    return (
      <div className="waveform empty-waveform" aria-label="No audio loaded">
        <span>No WAV loaded</span>
      </div>
    );
  }

  const progress =
    deck.duration_seconds > 0
      ? Math.min(1, Math.max(0, deck.position_seconds / deck.duration_seconds))
      : 0;
  const deckName = deckIndex === 0 ? "A" : "B";

  return (
    <div
      className={`waveform deck-waveform deck-waveform-${deckName.toLowerCase()}`}
      role="img"
      aria-label={`Deck ${deckName} waveform from loaded WAV audio`}
    >
      {deck.waveform.length === 0 ? (
        <span className="waveform-empty-label">Waveform unavailable</span>
      ) : (
        deck.waveform.map((peak, index) => {
          const amplitude = Number.isFinite(peak)
            ? Math.min(1, Math.max(0, peak))
            : 0;
          return (
            <span
              key={index}
              className={
                index / deck.waveform.length <= progress ? "played" : ""
              }
              style={{ height: `${Math.max(5, amplitude * 88)}%` }}
            />
          );
        })
      )}
      {deck.waveform.length > 0 && (
        <span className="playhead" style={{ left: `${progress * 100}%` }} />
      )}
    </div>
  );
}

export function DeckPanel({
  deck,
  deckIndex,
  loading,
  onCommand,
}: {
  deck: DeckSnapshot;
  deckIndex: number;
  loading: boolean;
  onCommand: (command: string, args: Record<string, unknown>) => void;
}) {
  const label = deckIndex === 0 ? "A" : "B";
  const color = label === "A" ? "lime" : "cyan";
  const controls = [
    {
      key: "gain",
      label: "Gain",
      value: deck.gain,
      min: 0,
      max: 1.5,
      step: 0.01,
    },
    {
      key: "low",
      label: "Low EQ",
      value: deck.low,
      min: -12,
      max: 12,
      step: 0.1,
    },
    {
      key: "mid",
      label: "Mid EQ",
      value: deck.mid,
      min: -12,
      max: 12,
      step: 0.1,
    },
    {
      key: "high",
      label: "High EQ",
      value: deck.high,
      min: -12,
      max: 12,
      step: 0.1,
    },
  ] as const;

  return (
    <section
      className={`deck-panel deck-${label.toLowerCase()}`}
      aria-labelledby={`deck-${label}-heading`}
    >
      <header className="deck-heading">
        <div className="deck-identity">
          <span className={`deck-badge ${color}`}>DECK {label}</span>
          <div>
            <h2 id={`deck-${label}-heading`} title={deck.filename || undefined}>
              {deck.filename || "Waiting for a WAV"}
            </h2>
            <p>
              {deck.loaded
                ? "Local audio · PCM WAV"
                : "Load a local file from the crate below"}
            </p>
          </div>
        </div>
        <span className={`deck-state ${deck.playing ? "is-playing" : ""}`}>
          <span aria-hidden="true" />
          {deck.playing ? "Playing" : deck.loaded ? "Ready" : "Empty"}
        </span>
      </header>

      <div className="waveform-frame">
        <div className="waveform-ruler" aria-hidden="true">
          <span>00:00</span>
          <span>{formatTime(deck.duration_seconds / 2)}</span>
          <span>{formatTime(deck.duration_seconds)}</span>
        </div>
        <Waveform deck={deck} deckIndex={deckIndex} />
      </div>

      <div className="deck-meta" aria-label={`Deck ${label} audio details`}>
        <span>
          {deck.loaded ? formatTime(deck.duration_seconds) : "—:—"} duration
        </span>
        <span>
          {deck.sample_rate
            ? `${(deck.sample_rate / 1000).toFixed(deck.sample_rate % 1000 ? 1 : 0)} kHz`
            : "— kHz"}
        </span>
        <span>{deck.channels ? `${deck.channels} ch` : "— ch"}</span>
      </div>

      <div className="deck-transport">
        <button
          className={`transport-primary ${color}`}
          type="button"
          disabled={!deck.loaded || loading}
          onClick={() =>
            onCommand("set_playing", {
              deck: deckIndex,
              playing: !deck.playing,
            })
          }
        >
          <MaterialIcon name={deck.playing ? "pause" : "play_arrow"} />
          {deck.playing ? "Pause" : "Play"}
        </button>
        <button
          className="transport-secondary"
          type="button"
          disabled={!deck.loaded || loading}
          onClick={() => onCommand("stop_track", { deck: deckIndex })}
        >
          <MaterialIcon name="stop" />
          Stop
        </button>
        <output
          className="position-readout"
          aria-label={`Deck ${label} playback position`}
        >
          {formatTime(deck.position_seconds)} <span>/</span>{" "}
          {formatTime(deck.duration_seconds)}
        </output>
      </div>

      <label className="seek-control">
        <span className="sr-only">Deck {label} position</span>
        <input
          aria-label={`Deck ${label} position`}
          type="range"
          min="0"
          max={deck.duration_seconds || 1}
          step="0.01"
          value={deck.position_seconds}
          disabled={!deck.loaded || loading}
          onChange={(event) =>
            onCommand("seek_track", {
              deck: deckIndex,
              position: Number(event.target.value),
            })
          }
        />
      </label>

      <div className="deck-controls" aria-label={`Deck ${label} controls`}>
        {controls.map(({ key, label: controlLabel, value, min, max, step }) => (
          <label className="channel-control" key={key}>
            <span>{controlLabel}</span>
            <input
              aria-label={`Deck ${label} ${controlLabel}`}
              type="range"
              min={min}
              max={max}
              step={step}
              value={value}
              disabled={loading}
              onChange={(event) => {
                const next = Number(event.target.value);
                onCommand("set_deck_control", {
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

export function MixerConsole({
  mixer,
  onCommand,
}: {
  mixer: MixerSnapshot;
  onCommand: (command: string, args: Record<string, unknown>) => void;
}) {
  const meter = Math.min(1, Math.max(0, mixer.master_meter));
  return (
    <section className="mixer-console" aria-labelledby="mixer-heading">
      <div className="console-heading">
        <div>
          <p className="eyebrow">MASTER BUS</p>
          <h2 id="mixer-heading">Mixer</h2>
        </div>
        <span className="signal-label">
          <span /> Live engine meter
        </span>
      </div>

      <div className="master-meter-block">
        <div className="meter-caption">
          <span>MASTER PEAK</span>
          <output>{Math.round(meter * 100)}%</output>
        </div>
        <div
          className="master-meter"
          role="meter"
          aria-label="Master output peak level"
          aria-valuemin={0}
          aria-valuemax={100}
          aria-valuenow={Math.round(meter * 100)}
        >
          <span style={{ width: `${meter * 100}%` }} />
        </div>
        <div className="meter-scale" aria-hidden="true">
          <span>−∞</span>
          <span>−12</span>
          <span>−6</span>
          <span>0 dB</span>
        </div>
      </div>

      <label className="mixer-control master-control">
        <span>
          <strong>Master level</strong>
          <output>{Math.round(mixer.master_volume * 100)}%</output>
        </span>
        <input
          aria-label="Master level"
          type="range"
          min="0"
          max="1"
          step="0.01"
          value={mixer.master_volume}
          onChange={(event) =>
            onCommand("set_mixer", {
              master: Number(event.target.value),
              crossfader: null,
            })
          }
        />
      </label>

      <div
        className="channel-presence"
        aria-label="Decks routed to master output"
      >
        <div>
          <span className="deck-badge lime">A</span>
          <span>{mixer.deck_a.loaded ? "Loaded" : "Empty"}</span>
          <span>{mixer.deck_a.playing ? "Playing" : "Stopped"}</span>
        </div>
        <div>
          <span className="deck-badge cyan">B</span>
          <span>{mixer.deck_b.loaded ? "Loaded" : "Empty"}</span>
          <span>{mixer.deck_b.playing ? "Playing" : "Stopped"}</span>
        </div>
      </div>

      <label className="mixer-control crossfader-control">
        <span>
          <strong>Crossfader</strong>
          <output>
            {mixer.crossfader <= 0.02
              ? "A"
              : mixer.crossfader >= 0.98
                ? "B"
                : "Center"}
          </output>
        </span>
        <input
          aria-label="Crossfader"
          type="range"
          min="0"
          max="1"
          step="0.01"
          value={mixer.crossfader}
          onChange={(event) =>
            onCommand("set_mixer", {
              master: null,
              crossfader: Number(event.target.value),
            })
          }
        />
        <span className="fader-ends">
          <span>A</span>
          <span>Center</span>
          <span>B</span>
        </span>
      </label>
      <p className="mixer-note">
        One real master meter is available in this milestone. Per-deck meters
        are not exposed by the engine.
      </p>
    </section>
  );
}

export function QuickCrateBrowser({
  mixer,
  loadingDeck,
  onLoad,
}: {
  mixer: MixerSnapshot;
  loadingDeck: number | null;
  onLoad: (deck: number) => void;
}) {
  const decks = [mixer.deck_a, mixer.deck_b];
  return (
    <section className="quick-crate" aria-labelledby="crate-heading">
      <header className="section-heading">
        <div>
          <p className="eyebrow">LOCAL FILES ONLY</p>
          <h2 id="crate-heading">Quick crate</h2>
        </div>
        <span className="crate-count">
          {decks.filter((deck) => deck.loaded).length} / 2 decks loaded
        </span>
      </header>
      <div className="crate-slots">
        {decks.map((deck, index) => {
          const label = index === 0 ? "A" : "B";
          return (
            <article
              className={`crate-slot crate-slot-${label.toLowerCase()}`}
              key={label}
            >
              <span className={`deck-badge ${label === "A" ? "lime" : "cyan"}`}>
                DECK {label}
              </span>
              <div className="crate-track">
                <strong title={deck.filename || undefined}>
                  {deck.filename || "No WAV selected"}
                </strong>
                <span>
                  {deck.loaded
                    ? `${formatTime(deck.duration_seconds)} · ${deck.sample_rate / 1000} kHz · ${deck.channels} ch`
                    : "Choose a local PCM WAV file"}
                </span>
              </div>
              <button
                type="button"
                className="load-button"
                disabled={loadingDeck === index}
                onClick={() => onLoad(index)}
              >
                {loadingDeck === index
                  ? "Opening…"
                  : deck.loaded
                    ? "Replace WAV"
                    : "Load WAV"}
              </button>
            </article>
          );
        })}
      </div>
      <p className="crate-footnote">
        Files are opened through the native picker. SAYT does not scan folders
        or keep a library index yet.
      </p>
    </section>
  );
}

export function LibraryEmptyState({
  onLoad,
  loading,
}: {
  onLoad: () => void;
  loading: boolean;
}) {
  return (
    <section className="empty-library" aria-labelledby="library-empty-heading">
      <div className="empty-library-icon" aria-hidden="true">
        <MaterialIcon name="library_music" />
      </div>
      <p className="eyebrow">LOCAL MUSIC LIBRARY</p>
      <h2 id="library-empty-heading">Your library is empty</h2>
      <p>
        SAYT does not build or save a library index in M1. Load a WAV directly
        into a deck with the native file picker.
      </p>
      <button
        className="transport-primary lime"
        type="button"
        disabled={loading}
        onClick={onLoad}
      >
        {loading ? "Opening native picker…" : "Load WAV to Deck A"}
      </button>
      <span className="scope-note">
        Library indexing and collection management are planned for a later
        milestone.
      </span>
    </section>
  );
}

export function AudioSettings({
  devices,
  selectedDevice,
  outputError,
  onRefresh,
  onRetry,
}: {
  devices: DeviceInfo[];
  selectedDevice: string | null;
  outputError: boolean;
  onRefresh: () => void;
  onRetry: () => void;
}) {
  const selectedName = devices.find(
    (device) => device.id === selectedDevice,
  )?.name;
  const isSystemDefault = !selectedDevice || selectedDevice === "default";
  return (
    <section className="settings-layout" aria-labelledby="settings-heading">
      <div className="settings-intro">
        <p className="eyebrow">M1 AUDIO ENGINE</p>
        <h2 id="settings-heading">Audio &amp; device settings</h2>
        <p>
          Choose the output used by both decks. Sample rate, buffer size, cue
          routing, and latency controls are not available in this milestone.
        </p>
      </div>
      <div className="settings-card">
        <div className="settings-card-heading">
          <div>
            <span className="settings-icon" aria-hidden="true">
              <MaterialIcon name="volume_up" />
            </span>
            <div>
              <h3>Output device</h3>
              <p>One shared master output</p>
            </div>
          </div>
          <span className={`settings-state ${outputError ? "error" : ""}`}>
            <span />
            {outputError
              ? "Output error"
              : isSystemDefault
                ? "System default"
                : "Selected"}
          </span>
        </div>
        <p className="selected-device-name">
          {selectedName ||
            (isSystemDefault
              ? "System default output"
              : "Current output selection")}
        </p>
        {outputError && (
          <p className="settings-error" role="alert">
            Audio output stopped. Re-select an output device to retry.
          </p>
        )}
        <div className="settings-actions">
          <button
            className="transport-secondary"
            type="button"
            onClick={onRefresh}
          >
            Refresh device list
          </button>
          <button className="load-button" type="button" onClick={onRetry}>
            Retry selected output
          </button>
        </div>
        <p className="settings-help">
          Device names come from the operating system. Output recovery and
          sample-rate negotiation still need physical-device verification.
        </p>
      </div>
      <div className="settings-card capabilities-card">
        <p className="eyebrow">AVAILABLE NOW</p>
        <ul>
          <li>Native output-device enumeration and selection</li>
          <li>Deck A and Deck B routed through one master bus</li>
          <li>Real master peak telemetry from the audio engine</li>
          <li>Retry and refresh actions after an output error</li>
        </ul>
      </div>
    </section>
  );
}

export function AppShell({
  page,
  onNavigate,
  devices,
  selectedDevice,
  onSelectDevice,
  engineConnected,
  children,
}: {
  page: Page;
  onNavigate: (nextPage: Page) => void;
  devices: DeviceInfo[];
  selectedDevice: string | null;
  onSelectDevice: (deviceId: string) => void;
  engineConnected: boolean;
  children: ReactNode;
}) {
  const pages: { id: Page; label: string }[] = [
    { id: "dj", label: "SAYT DJ" },
    { id: "library", label: "Library" },
    { id: "settings", label: "Settings" },
  ];
  const futurePages = ["Bridge", "Preparation", "Studio", "FX & Sampler"];
  const currentDeviceExists = devices.some(
    (device) => device.id === selectedDevice,
  );

  return (
    <div className="app-shell">
      <header className="topbar">
        <a
          className="brand"
          href="#workspace"
          onClick={(event) => {
            event.preventDefault();
            onNavigate("dj");
          }}
          aria-label="SAYT home"
        >
          <span className="brand-mark" aria-hidden="true">
            S
          </span>
          <span>SAYT</span>
        </a>
        <nav className="primary-nav" aria-label="Main navigation">
          {pages.map(({ id, label }) => (
            <button
              key={id}
              type="button"
              className={page === id ? "nav-item active" : "nav-item"}
              aria-current={page === id ? "page" : undefined}
              onClick={() => onNavigate(id)}
            >
              {label}
            </button>
          ))}
          {futurePages.map((label) => (
            <button
              key={label}
              className="nav-item future-nav"
              type="button"
              disabled
              title={`${label} is not implemented in M1`}
            >
              {label}
              <span>Later</span>
            </button>
          ))}
        </nav>
        <div className="topbar-status">
          <span
            className={`engine-indicator ${engineConnected ? "connected" : ""}`}
            aria-hidden="true"
          />
          <span className="engine-status-text">
            {engineConnected ? "Audio engine" : "Connecting"}
          </span>
          <label className="header-device-picker">
            <span className="sr-only">Output device</span>
            <select
              aria-label="Output device"
              value={selectedDevice ?? "default"}
              onChange={(event) => onSelectDevice(event.target.value)}
            >
              <option value="default">System default</option>
              {selectedDevice &&
                selectedDevice !== "default" &&
                !currentDeviceExists && (
                  <option value={selectedDevice}>Current selection</option>
                )}
              {devices.map((device) => (
                <option key={device.id} value={device.id}>
                  {device.name}
                </option>
              ))}
            </select>
          </label>
        </div>
      </header>
      {children}
      <footer className="statusbar">
        <span>
          <i className={engineConnected ? "status-light on" : "status-light"} />{" "}
          {engineConnected
            ? "Engine snapshot connected"
            : "Waiting for audio engine"}
        </span>
        <span>Local audio · No library index · M1</span>
      </footer>
    </div>
  );
}

export function App() {
  const [mixer, setMixer] = useState<MixerSnapshot>(emptyMixer);
  const [devices, setDevices] = useState<DeviceInfo[]>([]);
  const [page, setPage] = useState<Page>("dj");
  const [error, setError] = useState("");
  const [loadingDeck, setLoadingDeck] = useState<number | null>(null);
  const [engineConnected, setEngineConnected] = useState(false);

  useEffect(() => {
    let active = true;
    let timer = 0;

    const pollSnapshot = async () => {
      try {
        const snapshot = await invoke<MixerSnapshot>("engine_snapshot");
        if (active) {
          setMixer(snapshot);
          setEngineConnected(true);
        }
      } catch (cause) {
        if (active) {
          setError(getErrorMessage(cause));
          setEngineConnected(false);
        }
      }
      if (active)
        timer = window.setTimeout(
          () => void pollSnapshot(),
          SNAPSHOT_INTERVAL_MS,
        );
    };

    const initialize = async () => {
      try {
        const outputDevices = await invoke<DeviceInfo[]>("list_devices");
        if (active) setDevices(outputDevices);
      } catch (cause) {
        if (active) setError(getErrorMessage(cause));
      }
      try {
        const snapshot = await invoke<MixerSnapshot>("select_device", {
          deviceId: "default",
        });
        if (active) {
          setMixer(snapshot);
          setEngineConnected(true);
        }
      } catch (cause) {
        if (active) setError(getErrorMessage(cause));
      }
      if (active) void pollSnapshot();
    };

    void initialize();
    return () => {
      active = false;
      window.clearTimeout(timer);
    };
  }, []);

  const runMixerCommand = (command: string, args: Record<string, unknown>) => {
    setError("");
    void invoke<MixerSnapshot>(command, args)
      .then(setMixer)
      .catch((cause) => setError(getErrorMessage(cause)));
  };

  const loadTrack = async (deck: number) => {
    setLoadingDeck(deck);
    setError("");
    try {
      const snapshot = await invokeNativeTrackPicker(deck);
      setMixer(snapshot);
    } catch (cause) {
      setError(getErrorMessage(cause));
    } finally {
      setLoadingDeck(null);
    }
  };

  const refreshDevices = async () => {
    setError("");
    try {
      setDevices(await invoke<DeviceInfo[]>("list_devices"));
    } catch (cause) {
      setError(getErrorMessage(cause));
    }
  };

  const selectDevice = (deviceId: string) =>
    runMixerCommand("select_device", { deviceId });
  const selectedDeviceName = devices.find(
    (device) => device.id === mixer.selected_device,
  )?.name;
  const isSystemDefault =
    !mixer.selected_device || mixer.selected_device === "default";

  return (
    <AppShell
      page={page}
      onNavigate={setPage}
      devices={devices}
      selectedDevice={mixer.selected_device}
      onSelectDevice={selectDevice}
      engineConnected={engineConnected}
    >
      <main className="main-content" id="workspace">
        <div className="page-heading">
          <div>
            <p className="eyebrow">LOCAL-FIRST PERFORMANCE</p>
            <h1>
              {page === "dj"
                ? "DJ Workspace"
                : page === "library"
                  ? "Music Library"
                  : "Audio Settings"}
            </h1>
          </div>
          <div className="page-context">
            <span className="m1-pill">M1 · TWO DECKS</span>
            <span className="output-summary">
              Output:{" "}
              {selectedDeviceName ||
                (isSystemDefault
                  ? "System default output"
                  : "Current selection")}
            </span>
          </div>
        </div>

        {(error || mixer.output_error) && (
          <div className="error-banner" role="alert">
            <span>
              {error ||
                "Audio output stopped. Re-select an output device to retry."}
            </span>
            <button
              type="button"
              onClick={() => selectDevice(mixer.selected_device ?? "default")}
            >
              Retry output
            </button>
            <button type="button" onClick={() => void refreshDevices()}>
              Refresh devices
            </button>
          </div>
        )}

        {page === "dj" && (
          <>
            <div className="dj-grid">
              <DeckPanel
                deck={mixer.deck_a}
                deckIndex={0}
                loading={loadingDeck === 0}
                onCommand={runMixerCommand}
              />
              <MixerConsole mixer={mixer} onCommand={runMixerCommand} />
              <DeckPanel
                deck={mixer.deck_b}
                deckIndex={1}
                loading={loadingDeck === 1}
                onCommand={runMixerCommand}
              />
            </div>
            <QuickCrateBrowser
              mixer={mixer}
              loadingDeck={loadingDeck}
              onLoad={(deck) => void loadTrack(deck)}
            />
          </>
        )}

        {page === "library" && (
          <LibraryEmptyState
            loading={loadingDeck === 0}
            onLoad={() => void loadTrack(0)}
          />
        )}

        {page === "settings" && (
          <AudioSettings
            devices={devices}
            selectedDevice={mixer.selected_device}
            outputError={mixer.output_error}
            onRefresh={() => void refreshDevices()}
            onRetry={() => selectDevice(mixer.selected_device ?? "default")}
          />
        )}
      </main>
    </AppShell>
  );
}

export function invokeNativeTrackPicker(deck: number) {
  return invoke<MixerSnapshot>("pick_track", { deck });
}
