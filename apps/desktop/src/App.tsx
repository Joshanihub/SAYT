import { invoke } from "@tauri-apps/api/core";
import { useEffect, useMemo, useRef, useState, type ReactNode } from "react";
import { MaterialIcon } from "./MaterialIcon";
import { ShortcutOverlay } from "./ShortcutOverlay";
import {
  DECK_CONTROL_LIMITS,
  createHotCueCommand,
  detectShortcutPlatform,
  dispatchShortcutKeydown,
  formatShortcut,
  getHotCueShortcutId,
  getEffectiveShortcutBindings,
  loadShortcutOverrides,
  MIXER_CONTROL_LIMITS,
  saveShortcutOverrides,
  validateShortcutBinding,
  type AppCommand,
  type CommandDispatcher,
  type DeckControlId,
  type DeckId,
  type HotCueAction,
  type HotCueColor,
  type HotCueSlot,
  type ShortcutBindings,
  type ShortcutBinding,
  type ShortcutContext,
  type ShortcutId,
  type ShortcutOverrides,
  type ShortcutPlatform,
  type WorkspaceId,
} from "./commands/commandRegistry";

type DeckSnapshot = {
  loaded: boolean;
  filename: string;
  source_track_id: string | null;
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
  hot_cues: (HotCueSnapshot | null)[];
  hot_cue_warning: string | null;
  active_hot_cue_slot: number | null;
};

type HotCueSnapshot = {
  name: string;
  color: HotCueColor;
  position_seconds: number;
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
type DeviceListStatus = "loading" | "loaded" | "error";
type Page = WorkspaceId;

const SNAPSHOT_INTERVAL_MS = 100;

const emptyDeck = (): DeckSnapshot => ({
  loaded: false,
  filename: "",
  source_track_id: null,
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
  hot_cues: Array.from({ length: 8 }, () => null),
  hot_cue_warning: null,
  active_hot_cue_slot: null,
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

function clampValue(value: number, minimum: number, maximum: number) {
  return Math.max(minimum, Math.min(maximum, value));
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

const HOT_CUE_COLORS: readonly HotCueColor[] = [
  "coral",
  "orange",
  "amber",
  "lime",
  "cyan",
  "blue",
  "violet",
  "pink",
];

const HOT_CUE_SLOT_IDS: readonly HotCueSlot[] = [0, 1, 2, 3, 4, 5, 6, 7];

export function HotCueControls({
  deck,
  deckIndex,
  loading,
  shortcutText,
  onCommand,
}: {
  deck: DeckSnapshot;
  deckIndex: DeckId;
  loading: boolean;
  shortcutText: (shortcutId: ShortcutId) => string;
  onCommand: CommandDispatcher;
}) {
  const deckLabel = deckIndex === 0 ? "A" : "B";
  const [selectedSlot, setSelectedSlot] = useState<HotCueSlot | null>(null);
  const selectedCue =
    selectedSlot === null ? null : (deck.hot_cues[selectedSlot] ?? null);
  const [nameDraft, setNameDraft] = useState(selectedCue?.name ?? "");

  useEffect(() => {
    setNameDraft(selectedCue?.name ?? "");
  }, [deck.source_track_id, selectedSlot, selectedCue?.name]);

  const dispatchCue = (action: HotCueAction, slot: HotCueSlot) => {
    onCommand(
      createHotCueCommand(action, deckIndex, slot, deck.source_track_id),
    );
  };

  const saveCueName = () => {
    if (
      selectedSlot === null ||
      !selectedCue ||
      !deck.source_track_id ||
      nameDraft.trim() === selectedCue.name
    ) {
      return;
    }
    onCommand({
      id: "cue.rename",
      payload: {
        deck: deckIndex,
        slot: selectedSlot,
        sourceTrackId: deck.source_track_id,
        name: nameDraft,
      },
    });
  };

  return (
    <section
      className="hot-cue-section"
      aria-label={`Deck ${deckLabel} hot cues`}
      aria-describedby={`deck-${deckLabel}-hot-cue-description`}
    >
      <div className="hot-cue-heading">
        <div>
          <h3>Hot Cues</h3>
          <p id={`deck-${deckLabel}-hot-cue-description`}>
            8 per-track positions · click an empty pad to set, an assigned pad
            to trigger
          </p>
        </div>
        <span className="hot-cue-track-state">
          {deck.loaded ? "Saved with track" : "Load a track to begin"}
        </span>
      </div>

      {deck.hot_cue_warning && (
        <p className="hot-cue-warning" role="status">
          {deck.hot_cue_warning}
        </p>
      )}

      <div className="hot-cue-grid">
        {HOT_CUE_SLOT_IDS.map((slot) => {
          const cue = deck.hot_cues[slot] ?? null;
          const slotNumber = slot + 1;
          const active = deck.active_hot_cue_slot === slot;
          const padAction = cue ? "trigger" : "set";
          const padShortcut = getHotCueShortcutId(padAction, slot);
          return (
            <button
              key={slot}
              type="button"
              className={`hot-cue-pad${cue ? ` hot-cue-color-${cue.color} is-assigned` : " is-empty"}${active ? " is-active" : ""}${selectedSlot === slot ? " is-selected" : ""}`}
              disabled={!deck.loaded || loading}
              aria-label={
                cue
                  ? `Trigger Deck ${deckLabel} hot cue ${slotNumber}, ${cue.name}, at ${formatTime(cue.position_seconds)}`
                  : `Set Deck ${deckLabel} hot cue ${slotNumber} at the current position`
              }
              aria-pressed={active}
              title={
                cue
                  ? `Trigger ${cue.name} at ${formatTime(cue.position_seconds)} · ${shortcutText(padShortcut)}`
                  : `Set Hot Cue ${slotNumber} at the current playback position · ${shortcutText(padShortcut)}`
              }
              onClick={() => {
                setSelectedSlot(slot);
                dispatchCue(cue ? "trigger" : "set", slot);
              }}
            >
              <span className="hot-cue-slot-number">{slotNumber}</span>
              <strong>{cue?.name ?? "EMPTY"}</strong>
              <small>
                {cue ? formatTime(cue.position_seconds) : "SET POSITION"}
                <kbd>{shortcutText(padShortcut)}</kbd>
              </small>
            </button>
          );
        })}
      </div>

      {selectedSlot !== null && (
        <div className="hot-cue-editor" aria-label="Selected hot cue controls">
          <span className="hot-cue-editor-title">Cue {selectedSlot + 1}</span>
          {selectedCue ? (
            <>
              <label>
                <span className="sr-only">
                  Deck {deckLabel} cue {selectedSlot + 1} name
                </span>
                <input
                  type="text"
                  aria-label={`Deck ${deckLabel} hot cue ${selectedSlot + 1} name`}
                  value={nameDraft}
                  maxLength={64}
                  disabled={loading}
                  onChange={(event) =>
                    setNameDraft(
                      Array.from(event.target.value).slice(0, 32).join(""),
                    )
                  }
                  onBlur={saveCueName}
                  onKeyDown={(event) => {
                    if (event.key === "Enter") event.currentTarget.blur();
                    if (event.key === "Escape") setNameDraft(selectedCue.name);
                  }}
                />
              </label>
              <label className="hot-cue-color-control">
                <span className="sr-only">
                  Deck {deckLabel} cue {selectedSlot + 1} color
                </span>
                <select
                  aria-label={`Deck ${deckLabel} hot cue ${selectedSlot + 1} color`}
                  value={selectedCue.color}
                  disabled={loading}
                  onChange={(event) =>
                    onCommand({
                      id: "cue.setColor",
                      payload: {
                        deck: deckIndex,
                        slot: selectedSlot,
                        sourceTrackId: deck.source_track_id,
                        color: event.target.value as HotCueColor,
                      },
                    })
                  }
                >
                  {HOT_CUE_COLORS.map((color) => (
                    <option key={color} value={color}>
                      {color[0]?.toUpperCase()}
                      {color.slice(1)}
                    </option>
                  ))}
                </select>
              </label>
              <button
                type="button"
                disabled={!deck.loaded || loading}
                title={`Reassign at current position · ${shortcutText(getHotCueShortcutId("set", selectedSlot))}`}
                onClick={() => dispatchCue("set", selectedSlot)}
              >
                Reassign{" "}
                <kbd>
                  {shortcutText(getHotCueShortcutId("set", selectedSlot))}
                </kbd>
              </button>
              <button
                type="button"
                disabled={loading}
                title={`Clear this saved cue · ${shortcutText(getHotCueShortcutId("clear", selectedSlot))}`}
                onClick={() => dispatchCue("clear", selectedSlot)}
              >
                Clear{" "}
                <kbd>
                  {shortcutText(getHotCueShortcutId("clear", selectedSlot))}
                </kbd>
              </button>
            </>
          ) : (
            <span className="hot-cue-empty-selection">
              Empty · setting uses the real engine position
            </span>
          )}
        </div>
      )}
    </section>
  );
}

export function DeckPanel({
  deck,
  deckIndex,
  loading,
  focused,
  shortcutText,
  onCommand,
}: {
  deck: DeckSnapshot;
  deckIndex: DeckId;
  loading: boolean;
  focused: boolean;
  shortcutText: (shortcutId: ShortcutId) => string;
  onCommand: CommandDispatcher;
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
  const controlShortcuts: Record<
    DeckControlId,
    { decrease: ShortcutId; increase: ShortcutId }
  > = {
    gain: { decrease: "gainDown", increase: "gainUp" },
    low: { decrease: "lowEqDown", increase: "lowEqUp" },
    mid: { decrease: "midEqDown", increase: "midEqUp" },
    high: { decrease: "highEqDown", increase: "highEqUp" },
  };

  return (
    <section
      className={`deck-panel deck-${label.toLowerCase()}${focused ? " keyboard-focused" : ""}`}
      aria-labelledby={`deck-${label}-heading`}
      onFocusCapture={() =>
        onCommand({ id: "deck.focus", payload: { deck: deckIndex } })
      }
      onPointerDownCapture={() =>
        onCommand({ id: "deck.focus", payload: { deck: deckIndex } })
      }
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
            <p className="deck-keyboard-hint">
              Focus{" "}
              <kbd>
                {shortcutText(deckIndex === 0 ? "focusDeckA" : "focusDeckB")}
              </kbd>
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
            onCommand({ id: "deck.playPause", payload: { deck: deckIndex } })
          }
        >
          <MaterialIcon name={deck.playing ? "pause" : "play_arrow"} />
          {deck.playing ? "Pause" : "Play"}
          <kbd>{shortcutText("togglePlayback")}</kbd>
        </button>
        <button
          className="transport-secondary"
          type="button"
          disabled={!deck.loaded || loading}
          onClick={() =>
            onCommand({ id: "deck.stop", payload: { deck: deckIndex } })
          }
        >
          <MaterialIcon name="stop" />
          Stop
          <kbd>{shortcutText("stopFocusedDeck")}</kbd>
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
            onCommand({
              id: "deck.seekTo",
              payload: {
                deck: deckIndex,
                position: Number(event.target.value),
              },
            })
          }
        />
      </label>

      <HotCueControls
        key={deck.source_track_id ?? `deck-${deckIndex}-empty`}
        deck={deck}
        deckIndex={deckIndex}
        loading={loading}
        shortcutText={shortcutText}
        onCommand={onCommand}
      />

      <div className="deck-controls" aria-label={`Deck ${label} controls`}>
        {controls.map(({ key, label: controlLabel, value, min, max, step }) => (
          <label className="channel-control" key={key}>
            <span>
              {controlLabel}
              <small className="shortcut-inline-hint">
                <kbd>{shortcutText(controlShortcuts[key].decrease)}</kbd>
                <span aria-hidden="true"> / </span>
                <kbd>{shortcutText(controlShortcuts[key].increase)}</kbd>
              </small>
            </span>
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
                onCommand({
                  id: "deck.setControl",
                  payload: { deck: deckIndex, control: key, value: next },
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
  shortcutText,
  onCommand,
}: {
  mixer: MixerSnapshot;
  shortcutText: (shortcutId: ShortcutId) => string;
  onCommand: CommandDispatcher;
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
        <small className="shortcut-inline-hint">
          <kbd>{shortcutText("masterLevelDown")}</kbd>
          <span aria-hidden="true"> / </span>
          <kbd>{shortcutText("masterLevelUp")}</kbd>
        </small>
        <input
          aria-label="Master level"
          type="range"
          min={MIXER_CONTROL_LIMITS.master.min}
          max={MIXER_CONTROL_LIMITS.master.max}
          step={MIXER_CONTROL_LIMITS.master.step}
          value={mixer.master_volume}
          onChange={(event) =>
            onCommand({
              id: "mixer.setMaster",
              payload: { value: Number(event.target.value) },
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
        <small className="shortcut-inline-hint">
          <kbd>{shortcutText("crossfaderLeft")}</kbd>
          <span aria-hidden="true"> / </span>
          <kbd>{shortcutText("crossfaderRight")}</kbd>
          <span aria-hidden="true"> · </span>
          <kbd>{shortcutText("crossfaderCenter")}</kbd>
        </small>
        <input
          aria-label="Crossfader"
          type="range"
          min={MIXER_CONTROL_LIMITS.crossfader.min}
          max={MIXER_CONTROL_LIMITS.crossfader.max}
          step={MIXER_CONTROL_LIMITS.crossfader.step}
          value={mixer.crossfader}
          onChange={(event) =>
            onCommand({
              id: "mixer.setCrossfader",
              payload: { value: Number(event.target.value) },
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
  onLoad: (deck: DeckId) => void;
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
          const deckId: DeckId = index === 0 ? 0 : 1;
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
                onClick={() => onLoad(deckId)}
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
  deviceListStatus,
  selectedDevice,
  outputError,
  onRefresh,
  onRetry,
}: {
  devices: DeviceInfo[];
  deviceListStatus: DeviceListStatus;
  selectedDevice: string | null;
  outputError: boolean;
  onRefresh: () => void;
  onRetry: () => void;
}) {
  const selectedName = devices.find(
    (device) => device.id === selectedDevice,
  )?.name;
  const isSystemDefault = selectedDevice === "default";
  const noOutputDevices =
    deviceListStatus === "loaded" &&
    devices.length === 0 &&
    (!selectedDevice || outputError);
  const deviceListFailed = deviceListStatus === "error";
  const outputMessage = noOutputDevices
    ? outputError
      ? "No output devices are listed, and audio output is in an error state. Connect or enable an output, then refresh the device list."
      : "No audio output devices are listed. Connect or enable an output, then refresh the device list."
    : deviceListFailed
      ? `Could not refresh the audio device list.${devices.length > 0 ? " Listed outputs may be out of date." : ""} Refresh to try again.`
      : outputError
        ? "Audio output stopped. Re-select an output device to retry."
        : null;
  const selectedOutputName =
    deviceListStatus === "loading" && devices.length === 0
      ? "Checking for audio output devices…"
      : noOutputDevices
        ? "No audio output devices found"
        : deviceListFailed && devices.length === 0
          ? "Audio device list unavailable"
          : selectedName ||
            (isSystemDefault
              ? "System default output"
              : selectedDevice === null
                ? "No output selected"
                : "Current output selection");
  const statusLabel =
    outputError && !noOutputDevices
      ? "Output error"
      : deviceListStatus === "loading"
        ? "Checking devices"
        : deviceListFailed
          ? "List error"
          : noOutputDevices
            ? "No devices"
            : selectedDevice === null
              ? "No output selected"
              : isSystemDefault
                ? "System default"
                : "Selected";
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
          <span
            className={`settings-state ${outputError || noOutputDevices || deviceListFailed ? "error" : ""}`}
          >
            <span />
            {statusLabel}
          </span>
        </div>
        <p className="selected-device-name">{selectedOutputName}</p>
        {outputMessage && (
          <p className="settings-error" role="alert">
            {outputMessage}
          </p>
        )}
        {deviceListStatus === "loading" && devices.length === 0 && (
          <p className="settings-help" role="status" aria-live="polite">
            Searching for audio output devices…
          </p>
        )}
        <div className="settings-actions">
          <button
            className="transport-secondary"
            type="button"
            onClick={onRefresh}
            disabled={deviceListStatus === "loading"}
          >
            {deviceListStatus === "loading"
              ? "Checking devices…"
              : "Refresh device list"}
          </button>
          <button className="load-button" type="button" onClick={onRetry}>
            {isSystemDefault || selectedDevice === null
              ? "Retry system default"
              : "Retry selected output"}
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
  onShortcuts,
  shortcutHelpHint,
  children,
}: {
  page: Page;
  onNavigate: (nextPage: Page) => void;
  devices: DeviceInfo[];
  selectedDevice: string | null;
  onSelectDevice: (deviceId: string) => void;
  engineConnected: boolean;
  onShortcuts: () => void;
  shortcutHelpHint: string;
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
        <button
          className="shortcut-help-button"
          type="button"
          onClick={onShortcuts}
        >
          Keyboard shortcuts <kbd>{shortcutHelpHint}</kbd>
        </button>
      </footer>
    </div>
  );
}

export function App() {
  const [mixer, setMixer] = useState<MixerSnapshot>(emptyMixer);
  const [devices, setDevices] = useState<DeviceInfo[]>([]);
  const [deviceListStatus, setDeviceListStatus] =
    useState<DeviceListStatus>("loading");
  const [page, setPage] = useState<Page>("dj");
  const [error, setError] = useState("");
  const [loadingDeck, setLoadingDeck] = useState<number | null>(null);
  const [engineConnected, setEngineConnected] = useState(false);
  const [focusedDeck, setFocusedDeck] = useState<DeckId>(0);
  const [shortcutOverlayOpen, setShortcutOverlayOpen] = useState(false);
  const [shortcutPlatform, setShortcutPlatform] =
    useState<ShortcutPlatform>("other");
  const [shortcutOverrides, setShortcutOverrides] = useState<ShortcutOverrides>(
    {},
  );
  const [shortcutPreferenceMessage, setShortcutPreferenceMessage] = useState<
    string | null
  >(null);
  const effectiveShortcutBindings = useMemo(
    () => getEffectiveShortcutBindings(shortcutOverrides, shortcutPlatform),
    [shortcutOverrides, shortcutPlatform],
  );
  const shortcutText = (shortcutId: ShortcutId) =>
    formatShortcut(effectiveShortcutBindings[shortcutId], shortcutPlatform);
  const shortcutBindingsRef = useRef<ShortcutBindings>(
    effectiveShortcutBindings,
  );
  const initialFocusedTrackId =
    focusedDeck === 0
      ? mixer.deck_a.source_track_id
      : mixer.deck_b.source_track_id;
  const shortcutContextRef = useRef<ShortcutContext>({
    focusedDeck,
    focusedTrackId: initialFocusedTrackId,
  });
  const shortcutOverlayOpenRef = useRef(shortcutOverlayOpen);
  const commandDispatcherRef = useRef<CommandDispatcher>(() => undefined);
  shortcutBindingsRef.current = effectiveShortcutBindings;
  shortcutContextRef.current = {
    focusedDeck,
    focusedTrackId:
      focusedDeck === 0
        ? mixer.deck_a.source_track_id
        : mixer.deck_b.source_track_id,
  };
  shortcutOverlayOpenRef.current = shortcutOverlayOpen;

  useEffect(() => {
    const platform = detectShortcutPlatform(navigator.userAgent);
    setShortcutPlatform(platform);
    try {
      const loaded = loadShortcutOverrides(window.localStorage, platform);
      setShortcutOverrides(loaded.overrides);
      setShortcutPreferenceMessage(loaded.warning);
    } catch {
      setShortcutPreferenceMessage(
        "Shortcut settings could not be read; built-in shortcuts are active.",
      );
    }
  }, []);

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
        if (active) {
          setDevices(outputDevices);
          setDeviceListStatus("loaded");
        }
      } catch (cause) {
        if (active) {
          setError(getErrorMessage(cause));
          setDeviceListStatus("error");
        }
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

  const loadTrack = async (deck: DeckId) => {
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
    setDeviceListStatus("loading");
    setError("");
    try {
      setDevices(await invoke<DeviceInfo[]>("list_devices"));
      setDeviceListStatus("loaded");
    } catch (cause) {
      setError(getErrorMessage(cause));
      setDeviceListStatus("error");
    }
  };

  const setDeckControl = (
    deckIndex: DeckId,
    control: DeckControlId,
    requestedValue: number,
  ) => {
    const limits = DECK_CONTROL_LIMITS[control];
    const value = Math.max(limits.min, Math.min(limits.max, requestedValue));
    runMixerCommand("set_deck_control", {
      deck: deckIndex,
      gain: control === "gain" ? value : null,
      low: control === "low" ? value : null,
      mid: control === "mid" ? value : null,
      high: control === "high" ? value : null,
    });
  };

  const dispatchCommand: CommandDispatcher = (command: AppCommand) => {
    switch (command.id) {
      case "deck.focus":
        setFocusedDeck(command.payload.deck);
        break;
      case "deck.playPause": {
        const deck = command.payload.deck === 0 ? mixer.deck_a : mixer.deck_b;
        if (!deck.loaded || loadingDeck === command.payload.deck) {
          setError(
            `Load a WAV into Deck ${command.payload.deck === 0 ? "A" : "B"} before using transport shortcuts.`,
          );
          return;
        }
        runMixerCommand("set_playing", {
          deck: command.payload.deck,
          playing: !deck.playing,
        });
        break;
      }
      case "deck.stop": {
        const deck = command.payload.deck === 0 ? mixer.deck_a : mixer.deck_b;
        if (!deck.loaded || loadingDeck === command.payload.deck) {
          setError(
            `Load a WAV into Deck ${command.payload.deck === 0 ? "A" : "B"} before stopping it.`,
          );
          return;
        }
        runMixerCommand("stop_track", { deck: command.payload.deck });
        break;
      }
      case "deck.loadTrack":
        setFocusedDeck(command.payload.deck);
        void loadTrack(command.payload.deck);
        break;
      case "deck.seekTo":
        runMixerCommand("seek_track", {
          deck: command.payload.deck,
          position: command.payload.position,
        });
        break;
      case "deck.seekBy": {
        const deck = command.payload.deck === 0 ? mixer.deck_a : mixer.deck_b;
        if (!deck.loaded) {
          setError(
            `Load a WAV into Deck ${command.payload.deck === 0 ? "A" : "B"} before seeking.`,
          );
          return;
        }
        const position = Math.max(
          0,
          Math.min(
            deck.duration_seconds,
            deck.position_seconds + command.payload.delta,
          ),
        );
        runMixerCommand("seek_track", {
          deck: command.payload.deck,
          position,
        });
        break;
      }
      case "deck.setControl": {
        const {
          deck: deckIndex,
          control,
          value: requestedValue,
        } = command.payload;
        setDeckControl(deckIndex, control, requestedValue);
        break;
      }
      case "deck.adjustControl": {
        const { deck: deckIndex, control, delta } = command.payload;
        const deck = deckIndex === 0 ? mixer.deck_a : mixer.deck_b;
        setDeckControl(deckIndex, control, deck[control] + delta);
        break;
      }
      case "cue.set":
      case "cue.trigger":
      case "cue.clear":
      case "cue.rename":
      case "cue.setColor": {
        const { deck, slot, sourceTrackId } = command.payload;
        if (!sourceTrackId) {
          setError(
            `Load a WAV into Deck ${deck === 0 ? "A" : "B"} before using hot cues.`,
          );
          return;
        }
        const nativeCommand = {
          "cue.set": "set_hot_cue",
          "cue.trigger": "trigger_hot_cue",
          "cue.clear": "clear_hot_cue",
          "cue.rename": "rename_hot_cue",
          "cue.setColor": "set_hot_cue_color",
        }[command.id];
        const cueArgs: Record<string, unknown> = {
          deck,
          slot,
          sourceTrackId,
        };
        if (command.id === "cue.rename") cueArgs.name = command.payload.name;
        if (command.id === "cue.setColor")
          cueArgs.color = command.payload.color;
        runMixerCommand(nativeCommand, cueArgs);
        break;
      }
      case "mixer.setMaster":
        runMixerCommand("set_mixer", {
          master: clampValue(
            command.payload.value,
            MIXER_CONTROL_LIMITS.master.min,
            MIXER_CONTROL_LIMITS.master.max,
          ),
          crossfader: null,
        });
        break;
      case "mixer.adjustMaster":
        runMixerCommand("set_mixer", {
          master: clampValue(
            mixer.master_volume + command.payload.delta,
            MIXER_CONTROL_LIMITS.master.min,
            MIXER_CONTROL_LIMITS.master.max,
          ),
          crossfader: null,
        });
        break;
      case "mixer.setCrossfader":
        runMixerCommand("set_mixer", {
          master: null,
          crossfader: clampValue(
            command.payload.value,
            MIXER_CONTROL_LIMITS.crossfader.min,
            MIXER_CONTROL_LIMITS.crossfader.max,
          ),
        });
        break;
      case "mixer.adjustCrossfader":
        runMixerCommand("set_mixer", {
          master: null,
          crossfader: clampValue(
            mixer.crossfader + command.payload.delta,
            MIXER_CONTROL_LIMITS.crossfader.min,
            MIXER_CONTROL_LIMITS.crossfader.max,
          ),
        });
        break;
      case "workspace.navigate":
        setPage(command.payload.workspace);
        break;
      case "output.select":
        runMixerCommand("select_device", {
          deviceId: command.payload.deviceId,
        });
        break;
      case "output.refreshDevices":
        void refreshDevices();
        break;
      case "output.retry":
        runMixerCommand("select_device", {
          deviceId: mixer.selected_device ?? "default",
        });
        break;
      case "shortcuts.open":
        setShortcutOverlayOpen(true);
        break;
      default: {
        const unreachable: never = command;
        return unreachable;
      }
    }
  };
  commandDispatcherRef.current = dispatchCommand;

  useEffect(() => {
    const handleShortcut = (event: KeyboardEvent) => {
      dispatchShortcutKeydown(event, {
        bindings: shortcutBindingsRef.current,
        context: shortcutContextRef.current,
        overlayOpen: shortcutOverlayOpenRef.current,
        dispatch: commandDispatcherRef.current,
      });
    };
    window.addEventListener("keydown", handleShortcut);
    return () => window.removeEventListener("keydown", handleShortcut);
  }, []);

  const bindShortcut = (shortcutId: ShortcutId, binding: ShortcutBinding) => {
    const validation = validateShortcutBinding(
      shortcutId,
      binding,
      effectiveShortcutBindings,
      shortcutPlatform,
    );
    if (validation) {
      setShortcutPreferenceMessage(validation);
      return validation;
    }
    const nextOverrides = { ...shortcutOverrides, [shortcutId]: binding };
    try {
      const saveError = saveShortcutOverrides(
        window.localStorage,
        nextOverrides,
      );
      if (saveError) {
        setShortcutPreferenceMessage(saveError);
        return saveError;
      }
    } catch {
      const saveError =
        "Shortcut settings could not be saved. Check local app storage and retry.";
      setShortcutPreferenceMessage(saveError);
      return saveError;
    }
    setShortcutOverrides(nextOverrides);
    setShortcutPreferenceMessage(null);
    return null;
  };

  const resetShortcuts = () => {
    try {
      const saveError = saveShortcutOverrides(window.localStorage, {});
      if (saveError) {
        setShortcutPreferenceMessage(saveError);
        return;
      }
    } catch {
      setShortcutPreferenceMessage(
        "Shortcut settings could not be saved. Check local app storage and retry.",
      );
      return;
    }
    setShortcutOverrides({});
    setShortcutPreferenceMessage(null);
  };

  const selectedDeviceName = devices.find(
    (device) => device.id === mixer.selected_device,
  )?.name;
  const outputSummary =
    selectedDeviceName ||
    (mixer.selected_device === "default"
      ? "System default output"
      : mixer.selected_device
        ? "Current selection"
        : deviceListStatus === "loading"
          ? "Checking output devices…"
          : deviceListStatus === "error"
            ? "Output device list unavailable"
            : devices.length === 0
              ? "No output device listed"
              : "No output selected");

  return (
    <>
      <AppShell
        page={page}
        onNavigate={(workspace) =>
          dispatchCommand({ id: "workspace.navigate", payload: { workspace } })
        }
        devices={devices}
        selectedDevice={mixer.selected_device}
        onSelectDevice={(deviceId) =>
          dispatchCommand({ id: "output.select", payload: { deviceId } })
        }
        engineConnected={engineConnected}
        onShortcuts={() =>
          dispatchCommand({ id: "shortcuts.open", payload: {} })
        }
        shortcutHelpHint={shortcutText("openShortcutHelp")}
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
              <span className="output-summary">Output: {outputSummary}</span>
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
                onClick={() =>
                  dispatchCommand({ id: "output.retry", payload: {} })
                }
              >
                Retry output
              </button>
              <button
                type="button"
                onClick={() =>
                  dispatchCommand({ id: "output.refreshDevices", payload: {} })
                }
              >
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
                  focused={focusedDeck === 0}
                  shortcutText={shortcutText}
                  onCommand={dispatchCommand}
                />
                <MixerConsole
                  mixer={mixer}
                  shortcutText={shortcutText}
                  onCommand={dispatchCommand}
                />
                <DeckPanel
                  deck={mixer.deck_b}
                  deckIndex={1}
                  loading={loadingDeck === 1}
                  focused={focusedDeck === 1}
                  shortcutText={shortcutText}
                  onCommand={dispatchCommand}
                />
              </div>
              <QuickCrateBrowser
                mixer={mixer}
                loadingDeck={loadingDeck}
                onLoad={(deck) =>
                  dispatchCommand({ id: "deck.loadTrack", payload: { deck } })
                }
              />
            </>
          )}

          {page === "library" && (
            <LibraryEmptyState
              loading={loadingDeck === 0}
              onLoad={() =>
                dispatchCommand({ id: "deck.loadTrack", payload: { deck: 0 } })
              }
            />
          )}

          {page === "settings" && (
            <AudioSettings
              devices={devices}
              deviceListStatus={deviceListStatus}
              selectedDevice={mixer.selected_device}
              outputError={mixer.output_error}
              onRefresh={() =>
                dispatchCommand({ id: "output.refreshDevices", payload: {} })
              }
              onRetry={() =>
                dispatchCommand({ id: "output.retry", payload: {} })
              }
            />
          )}
        </main>
      </AppShell>
      {shortcutOverlayOpen && (
        <ShortcutOverlay
          bindings={effectiveShortcutBindings}
          platform={shortcutPlatform}
          warning={shortcutPreferenceMessage}
          onClose={() => setShortcutOverlayOpen(false)}
          onBind={bindShortcut}
          onReset={resetShortcuts}
        />
      )}
    </>
  );
}

export function invokeNativeTrackPicker(deck: number) {
  return invoke<MixerSnapshot>("pick_track", { deck });
}
