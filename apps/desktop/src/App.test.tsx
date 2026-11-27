import { renderToStaticMarkup } from "react-dom/server";
import { isValidElement, type ReactElement, type ReactNode } from "react";
import { describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { M1_MATERIAL_ICON_NAMES, MaterialIcon } from "./MaterialIcon";
import {
  createHotCueCommand,
  createManualLoopCommand,
  createShortcutCommand,
  getDefaultShortcutBindings,
  type CommandDispatcher,
  type ShortcutId,
  type HotCueSlot,
} from "./commands/commandRegistry";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

import {
  App,
  AppShell,
  AudioSettings,
  BeatPerformanceControls,
  CommandErrorBanner,
  DeckPanel,
  HotCueControls,
  ManualLoopControls,
  invokeNativeTrackPicker,
  MixerConsole,
  QuickCrateBrowser,
  Waveform,
} from "./App";
import { MusicLibrary } from "./MusicLibrary";
import { ShortcutOverlay } from "./ShortcutOverlay";

type TestElementProps = {
  children?: ReactNode;
  "aria-label"?: string;
  className?: string;
  disabled?: boolean;
  onBlur?: () => void;
  onChange?: (event: { target: { value: string } }) => void;
  onClick?: () => void;
  onKeyDown?: (event: {
    key: string;
    currentTarget: { blur: () => void };
  }) => void;
  onFocusCapture?: () => void;
  onPointerDownCapture?: () => void;
};

function findElements(node: ReactNode): ReactElement<TestElementProps>[] {
  if (Array.isArray(node)) return node.flatMap(findElements);
  if (!isValidElement(node)) return [];

  const element = node as ReactElement<TestElementProps>;
  return [element, ...findElements(element.props.children)];
}

function findByLabel(
  elements: ReactElement<TestElementProps>[],
  label: string,
) {
  const element = elements.find(
    (candidate) => candidate.props["aria-label"] === label,
  );
  if (!element) throw new Error(`Could not find element labelled ${label}`);
  return element;
}

const emptyDeck = {
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
  loop_in_seconds: null,
  loop_out_seconds: null,
  loop_size_seconds: 1,
  loop_active: false,
  loop_release_pending: false,
  manual_bpm: null,
  loop_roll_active: false,
};

const loadedDeck = {
  ...emptyDeck,
  loaded: true,
  filename: "track.wav",
  source_track_id: `sha256:${"a".repeat(64)}`,
  duration_seconds: 10,
  sample_rate: 48_000,
  channels: 2,
};

const emptyMixer = {
  deck_a: emptyDeck,
  deck_b: emptyDeck,
  master_volume: 0.8,
  crossfader: 0.5,
  master_meter: 0,
  selected_device: null,
  output_error: false,
};

const testShortcutText = (shortcutId: ShortcutId) => shortcutId;

describe("M1 desktop workspace", () => {
  it("renders decorative Material Symbols without hiding transport labels", () => {
    const markup = M1_MATERIAL_ICON_NAMES.map((name) =>
      renderToStaticMarkup(<MaterialIcon name={name} />),
    );

    expect(markup).toHaveLength(5);
    expect(markup.every((icon) => icon.includes('aria-hidden="true"'))).toBe(
      true,
    );
    expect(markup.map((icon) => icon.match(/>([a-z_]+)<\/span>/)?.[1])).toEqual(
      M1_MATERIAL_ICON_NAMES,
    );
  });

  it("renders the implemented navigation and two real deck surfaces", () => {
    const markup = renderToStaticMarkup(<App />);

    expect(markup).toContain("DJ Workspace");
    expect(markup).toContain("Library");
    expect(markup).toContain("Settings");
    expect(markup).toContain("DECK A");
    expect(markup).toContain("DECK B");
    expect(markup).toContain("Quick crate");
    expect(markup).toContain("Crossfader");
    expect(
      markup.match(/aria-label="Deck [AB] performance pads"/g),
    ).toHaveLength(2);
    expect(
      markup.match(/aria-label="Deck [AB] performance mode"/g),
    ).toHaveLength(2);
    expect(markup).toContain('aria-label="Master output peak level"');
    expect(markup).toContain(
      'disabled="" title="Bridge is not implemented in M1"',
    );
    expect(markup).toContain("No WAV loaded");
    expect(markup).toContain("Output: Checking output devices");
    expect(markup).not.toContain("Output: System default output");
  });

  it("exposes Preparation as an implemented workspace route", () => {
    const markup = renderToStaticMarkup(
      <AppShell
        page="preparation"
        onNavigate={() => undefined}
        devices={[]}
        selectedDevice={null}
        onSelectDevice={() => undefined}
        engineConnected={false}
        onShortcuts={() => undefined}
        shortcutHelpHint="Ctrl+/"
      >
        <div />
      </AppShell>,
    );

    expect(markup).toContain('aria-current="page"');
    expect(markup).toMatch(
      /nav-item active" aria-current="page"[^>]*>Preparation<\/button>/,
    );
    expect(markup).not.toContain(
      'title="Preparation is not implemented in M1"',
    );
  });

  it("renders the persistent music-library controls without seeded tracks", () => {
    const markup = renderToStaticMarkup(
      <MusicLibrary loadingDeck={null} onLoadTrack={vi.fn(async () => true)} />,
    );

    expect(markup).toContain("Loading your local catalog");
    expect(markup).toContain("Import folders");
    expect(markup).toContain("Export backup");
    expect(markup).toContain("Create");
    expect(markup).toContain("Search title, artist, album, or folder");
    expect(markup).toContain("Select a track");
    expect(markup).not.toContain("Dreams in Motion");
  });

  it("renders the same catalog as an honest Track Preparation workspace", () => {
    const markup = renderToStaticMarkup(
      <MusicLibrary
        workspace="preparation"
        loadingDeck={null}
        onLoadTrack={vi.fn(async () => true)}
      />,
    );

    expect(markup).toContain('aria-label="Track preparation workspace"');
    expect(markup).toContain("music-library--preparation");
    expect(markup).toContain("Loading your local catalog");
    expect(markup).not.toContain("Dreams in Motion");
  });

  it("calls the native track-picker command with the requested deck", async () => {
    await invokeNativeTrackPicker(1);
    expect(invoke).toHaveBeenCalledWith("pick_track", { deck: 1 });
  });

  it("shows only currently loaded deck slots in the quick crate", () => {
    const load = vi.fn();
    const markup = renderToStaticMarkup(
      <QuickCrateBrowser mixer={emptyMixer} loadingDeck={null} onLoad={load} />,
    );

    expect(markup).toContain("0 / 2 decks loaded");
    expect(markup).toContain("No WAV selected");
    expect(markup).toContain(
      "The library catalog stores file references and does not copy or edit your audio.",
    );
    expect(markup).not.toContain("playlist");
  });

  it("renders waveform peaks from a loaded snapshot and marks the real play position", () => {
    const markup = renderToStaticMarkup(
      <Waveform
        deck={{
          ...emptyDeck,
          loaded: true,
          duration_seconds: 10,
          position_seconds: 5,
          waveform: [0.5, 1],
        }}
        deckIndex={0}
      />,
    );

    expect(markup).toContain(
      'aria-label="Deck A waveform from loaded WAV audio"',
    );
    expect(markup).toContain("height:44%");
    expect(markup).toContain("height:88%");
    expect(markup).toContain("left:50%");
  });

  it("renders eight empty cue pads for a loaded deck and disables them when empty", () => {
    const emptyMarkup = renderToStaticMarkup(
      <HotCueControls
        deck={emptyDeck}
        deckIndex={0}
        loading={false}
        shortcutText={testShortcutText}
        onCommand={() => undefined}
      />,
    );
    const loadedMarkup = renderToStaticMarkup(
      <HotCueControls
        deck={loadedDeck}
        deckIndex={1}
        loading={false}
        shortcutText={testShortcutText}
        onCommand={() => undefined}
      />,
    );

    expect(emptyMarkup).toContain("Load a track to begin");
    expect(
      emptyMarkup.match(
        /aria-label="Set Deck A hot cue \d at the current position"/g,
      ),
    ).toHaveLength(8);
    expect(emptyMarkup.match(/disabled=""/g)).toHaveLength(8);
    expect(loadedMarkup).toContain("Saved with track");
    expect(loadedMarkup).toContain("Hot Cue 8");
  });

  it("renders persisted cue labels, colors, positions, active feedback and editor controls", () => {
    const cues = [...loadedDeck.hot_cues] as (null | {
      name: string;
      color: "coral" | "cyan";
      position_seconds: number;
    })[];
    cues[0] = { name: "Intro", color: "coral", position_seconds: 2.25 };
    cues[4] = { name: "Drop", color: "cyan", position_seconds: 5 };
    const markup = renderToStaticMarkup(
      <HotCueControls
        deck={{
          ...loadedDeck,
          hot_cues: cues,
          active_hot_cue_slot: 4,
        }}
        deckIndex={1}
        loading={false}
        shortcutText={testShortcutText}
        onCommand={() => undefined}
      />,
    );

    expect(markup).toContain("Intro");
    expect(markup).toContain("Drop");
    expect(markup).toContain("hot-cue-color-coral");
    expect(markup).toContain("hot-cue-color-cyan is-assigned is-active");
    expect(markup).toContain("0:02");
    expect(markup).toContain("0:05");
  });

  it("renders honest manual-loop time, state, preset sizes and load gating", () => {
    const emptyMarkup = renderToStaticMarkup(
      <ManualLoopControls
        deck={emptyDeck}
        deckIndex={0}
        loading={false}
        shortcutText={testShortcutText}
        onCommand={() => undefined}
      />,
    );
    const activeMarkup = renderToStaticMarkup(
      <ManualLoopControls
        deck={{
          ...loadedDeck,
          loop_in_seconds: 0.25,
          loop_out_seconds: 0.75,
          loop_size_seconds: 0.5,
          loop_active: true,
          loop_release_pending: true,
        }}
        deckIndex={1}
        loading={false}
        shortcutText={testShortcutText}
        onCommand={() => undefined}
      />,
    );

    expect(emptyMarkup).toContain(
      'aria-label="Set Loop In on Deck A at the current position"',
    );
    expect(emptyMarkup.match(/disabled=""/g)).toHaveLength(9);
    expect(activeMarkup).toContain("0:00.250");
    expect(activeMarkup).toContain("0:00.750");
    expect(activeMarkup).toContain("Releasing at loop boundary");
    expect(activeMarkup).toContain('aria-pressed="true"');
    expect(activeMarkup).toContain(
      "Manual loop sizes use seconds. Beatgrids and quantized loop placement are not implemented.",
    );
  });

  it("renders manual tempo, beat jumps and held Loop Roll with explicit track limits", () => {
    const emptyMarkup = renderToStaticMarkup(
      <BeatPerformanceControls
        deck={emptyDeck}
        deckIndex={0}
        loading={false}
        onCommand={() => undefined}
      />,
    );
    const loadedMarkup = renderToStaticMarkup(
      <BeatPerformanceControls
        deck={{ ...loadedDeck, playing: true, manual_bpm: 124.5 }}
        deckIndex={1}
        loading={false}
        onCommand={() => undefined}
      />,
    );

    expect(emptyMarkup).toContain('aria-label="Deck A manual BPM"');
    expect(emptyMarkup).toContain('min="30" max="300"');
    expect(emptyMarkup).toContain('disabled=""');
    expect(loadedMarkup).toContain('aria-label="Forward 4 beats on Deck B"');
    expect(loadedMarkup).toContain(
      'aria-label="Hold 1/4 beat Loop Roll on Deck B"',
    );
    expect(loadedMarkup).toContain("124.5 BPM · manual");
    expect(loadedMarkup).toContain("no beatgrid is used.");
  });

  it("routes manual-loop pad actions through the typed dispatcher with source identity", () => {
    const dispatch = vi.fn();
    const tree = ManualLoopControls({
      deck: loadedDeck,
      deckIndex: 1,
      loading: false,
      shortcutText: testShortcutText,
      onCommand: dispatch,
    });
    const elements = findElements(tree);

    findByLabel(
      elements,
      "Set 0.25 second loop from the current position on Deck B",
    ).props.onClick?.();
    expect(dispatch).toHaveBeenLastCalledWith(
      createManualLoopCommand("setSize", 1, loadedDeck.source_track_id, 0.25),
    );

    findByLabel(
      elements,
      "Set Loop In on Deck B at the current position",
    ).props.onClick?.();
    expect(dispatch).toHaveBeenLastCalledWith(
      createManualLoopCommand("setIn", 1, loadedDeck.source_track_id),
    );
  });

  it.each<["set" | "trigger" | "clear", HotCueSlot]>([
    ["set", 0],
    ["trigger", 3],
    ["clear", 7],
  ])(
    "creates the typed %s command for the selected deck, slot and track",
    (action, slot) => {
      const sourceTrackId = `sha256:${"b".repeat(64)}`;
      expect(createHotCueCommand(action, 1, slot, sourceTrackId)).toEqual({
        id: `cue.${action}`,
        payload: { deck: 1, slot, sourceTrackId },
      });
    },
  );

  it("limits Audio Settings to currently exposed output controls", () => {
    const markup = renderToStaticMarkup(
      <AudioSettings
        devices={[{ id: "system-out-1", name: "System Output" }]}
        deviceListStatus="loaded"
        selectedDevice="system-out-1"
        outputError={false}
        onRefresh={() => undefined}
        onRetry={() => undefined}
      />,
    );

    expect(markup).toContain("System Output");
    expect(markup).toContain("One shared master output");
    expect(markup).toContain(
      "Sample rate, buffer size, cue routing, and latency controls are not available",
    );
    expect(markup).not.toContain("Test Audio Output");
    expect(markup).not.toContain("Headphone Cue");
  });

  it("keeps empty deck transport and seek controls disabled", () => {
    const tree = DeckPanel({
      deck: emptyDeck,
      deckIndex: 0,
      loading: false,
      focused: true,
      shortcutText: testShortcutText,
      onCommand: () => undefined,
    });
    const controls = findElements(tree);

    expect(
      controls
        .filter((element) => element.type === "button")
        .every((element) => element.props.disabled === true),
    ).toBe(true);
    expect(findByLabel(controls, "Deck A position").props.disabled).toBe(true);
  });

  it("maps Deck B transport commands to deck index 1", () => {
    const onCommand = vi.fn<CommandDispatcher>();
    const tree = DeckPanel({
      deck: loadedDeck,
      deckIndex: 1,
      loading: false,
      focused: false,
      shortcutText: testShortcutText,
      onCommand,
    });
    const buttons = findElements(tree).filter(
      (element) => element.type === "button",
    );

    buttons[0]?.props.onClick?.();
    expect(onCommand).toHaveBeenLastCalledWith({
      id: "deck.playPause",
      payload: { deck: 1 },
    });
    expect(onCommand.mock.lastCall?.[0]).toEqual(
      createShortcutCommand("togglePlayback", { focusedDeck: 1 }),
    );

    buttons[1]?.props.onClick?.();
    expect(onCommand).toHaveBeenLastCalledWith({
      id: "deck.stop",
      payload: { deck: 1 },
    });
  });

  it("selects a deck as the keyboard target when its surface receives focus or pointer input", () => {
    const onCommand = vi.fn<CommandDispatcher>();
    const tree = DeckPanel({
      deck: loadedDeck,
      deckIndex: 1,
      loading: false,
      focused: false,
      shortcutText: testShortcutText,
      onCommand,
    });
    const panel = findElements(tree).find((element) =>
      element.props.className?.includes("deck-panel"),
    );

    panel?.props.onPointerDownCapture?.();
    expect(onCommand).toHaveBeenCalledWith({
      id: "deck.focus",
      payload: { deck: 1 },
    });
  });

  it("maps pause to a stopped state for a playing deck", () => {
    const onCommand = vi.fn<CommandDispatcher>();
    const tree = DeckPanel({
      deck: { ...loadedDeck, playing: true },
      deckIndex: 0,
      loading: false,
      focused: true,
      shortcutText: testShortcutText,
      onCommand,
    });

    findElements(tree)
      .filter((element) => element.type === "button")[0]
      ?.props.onClick?.();
    expect(onCommand).toHaveBeenCalledWith({
      id: "deck.playPause",
      payload: { deck: 0 },
    });
  });

  it("disables loaded deck actions while another picker operation runs", () => {
    const tree = DeckPanel({
      deck: loadedDeck,
      deckIndex: 0,
      loading: true,
      focused: true,
      shortcutText: testShortcutText,
      onCommand: () => undefined,
    });
    const controls = findElements(tree);

    expect(
      controls
        .filter((element) => element.type === "button")
        .every((element) => element.props.disabled === true),
    ).toBe(true);
    expect(findByLabel(controls, "Deck A position").props.disabled).toBe(true);
  });

  it("maps seek input to the selected deck and position", () => {
    const onCommand = vi.fn<CommandDispatcher>();
    const tree = DeckPanel({
      deck: loadedDeck,
      deckIndex: 0,
      loading: false,
      focused: true,
      shortcutText: testShortcutText,
      onCommand,
    });

    findByLabel(findElements(tree), "Deck A position").props.onChange?.({
      target: { value: "3.25" },
    });
    expect(onCommand).toHaveBeenCalledWith({
      id: "deck.seekTo",
      payload: { deck: 0, position: 3.25 },
    });
  });

  it.each<[string, "gain" | "low" | "mid" | "high", number]>([
    ["Deck A Gain", "gain", 1.2],
    ["Deck A Low EQ", "low", -3],
    ["Deck A Mid EQ", "mid", 2.5],
    ["Deck A High EQ", "high", 4],
  ])("maps %s to one independent deck control", (label, control, value) => {
    const onCommand = vi.fn<CommandDispatcher>();
    const tree = DeckPanel({
      deck: loadedDeck,
      deckIndex: 0,
      loading: false,
      focused: true,
      shortcutText: testShortcutText,
      onCommand,
    });

    findByLabel(findElements(tree), label).props.onChange?.({
      target: { value: String(value) },
    });
    expect(onCommand).toHaveBeenCalledWith({
      id: "deck.setControl",
      payload: { deck: 0, control, value },
    });
  });

  it("maps master level and crossfader changes independently", () => {
    const onCommand = vi.fn<CommandDispatcher>();
    const tree = MixerConsole({
      mixer: emptyMixer,
      shortcutText: testShortcutText,
      onCommand,
    });
    const controls = findElements(tree);

    findByLabel(controls, "Master level").props.onChange?.({
      target: { value: "0.65" },
    });
    expect(onCommand).toHaveBeenLastCalledWith({
      id: "mixer.setMaster",
      payload: { value: 0.65 },
    });

    findByLabel(controls, "Crossfader").props.onChange?.({
      target: { value: "0.2" },
    });
    expect(onCommand).toHaveBeenLastCalledWith({
      id: "mixer.setCrossfader",
      payload: { value: 0.2 },
    });
  });

  it("exposes the current output device list and maps device selection", () => {
    const onSelectDevice = vi.fn();
    const tree = AppShell({
      page: "settings",
      onNavigate: () => undefined,
      devices: [
        { id: "system-out-1", name: "System Output" },
        { id: "usb-out-2", name: "USB Interface" },
      ],
      selectedDevice: "system-out-1",
      onSelectDevice,
      engineConnected: true,
      onShortcuts: () => undefined,
      shortcutHelpHint: "F1",
      children: null,
    });
    const controls = findElements(tree);
    const markup = renderToStaticMarkup(tree);

    expect(markup).toContain("System Output");
    expect(markup).toContain("USB Interface");
    findByLabel(controls, "Output device").props.onChange?.({
      target: { value: "usb-out-2" },
    });
    expect(onSelectDevice).toHaveBeenCalledWith("usb-out-2");
  });

  it("renders the searchable, editable shortcut reference with platform mappings", () => {
    const markup = renderToStaticMarkup(
      <ShortcutOverlay
        bindings={getDefaultShortcutBindings("windows")}
        platform="windows"
        warning={null}
        onClose={() => undefined}
        onBind={() => null}
        onReset={() => undefined}
      />,
    );

    expect(markup).toContain('role="dialog"');
    expect(markup).toContain('type="search"');
    expect(markup).toContain("Search commands");
    expect(markup).toContain("Change shortcut for Play / pause focused deck");
    expect(markup).toContain("Reset defaults");
    expect(markup).toContain("Alt+1");
  });

  it("routes each quick-crate load action to its deck", () => {
    const onLoad = vi.fn<(deck: number) => void>();
    const tree = QuickCrateBrowser({
      mixer: emptyMixer,
      loadingDeck: null,
      onLoad,
    });
    const buttons = findElements(tree).filter(
      (element) => element.type === "button",
    );

    buttons[0]?.props.onClick?.();
    buttons[1]?.props.onClick?.();
    expect(onLoad.mock.calls).toEqual([[0], [1]]);
  });

  it("presents output errors and exposes refresh and retry actions", () => {
    const onRefresh = vi.fn();
    const onRetry = vi.fn();
    const tree = AudioSettings({
      devices: [],
      deviceListStatus: "loaded",
      selectedDevice: "removed-device",
      outputError: true,
      onRefresh,
      onRetry,
    });
    const controls = findElements(tree);
    const markup = renderToStaticMarkup(tree);
    const buttons = controls.filter((element) => element.type === "button");

    expect(markup).toContain('role="alert"');
    expect(markup).toContain("No output devices are listed");
    buttons[0]?.props.onClick?.();
    buttons[1]?.props.onClick?.();
    expect(onRefresh).toHaveBeenCalledOnce();
    expect(onRetry).toHaveBeenCalledOnce();
  });

  it("does not offer output recovery actions for a WAV decoder error", () => {
    const markup = renderToStaticMarkup(
      <CommandErrorBanner
        error="Only uncompressed integer PCM WAV files are supported"
        outputError={false}
        outputErrorMessage=""
        onRetry={() => undefined}
        onRefresh={() => undefined}
      />,
    );

    expect(markup).toContain(
      "Only uncompressed integer PCM WAV files are supported",
    );
    expect(markup).not.toContain("Retry output");
    expect(markup).not.toContain("Refresh devices");
  });

  it("keeps output recovery actions attached to output failures", () => {
    const retry = vi.fn();
    const refresh = vi.fn();
    const tree = CommandErrorBanner({
      error: "",
      outputError: false,
      outputErrorMessage: "Could not switch to USB output",
      onRetry: retry,
      onRefresh: refresh,
    });
    const controls = findElements(tree);
    const markup = renderToStaticMarkup(tree);

    expect(markup).toContain("Could not switch to USB output");
    const buttons = controls.filter((element) => element.type === "button");
    buttons[0]?.props.onClick?.();
    buttons[1]?.props.onClick?.();
    expect(retry).toHaveBeenCalledOnce();
    expect(refresh).toHaveBeenCalledOnce();
  });

  it("distinguishes device discovery, no devices, and device-list errors", () => {
    const onRefresh = vi.fn();
    const onRetry = vi.fn();
    const loadingTree = AudioSettings({
      devices: [],
      deviceListStatus: "loading",
      selectedDevice: null,
      outputError: false,
      onRefresh,
      onRetry,
    });
    const loadingControls = findElements(loadingTree);
    const loadingMarkup = renderToStaticMarkup(loadingTree);

    expect(loadingMarkup).toContain("Searching for audio output devices");
    expect(loadingMarkup).not.toContain('role="alert"');
    expect(
      loadingControls.find((element) => element.type === "button")?.props
        .disabled,
    ).toBe(true);

    const emptyTree = AudioSettings({
      devices: [],
      deviceListStatus: "loaded",
      selectedDevice: null,
      outputError: false,
      onRefresh,
      onRetry,
    });
    const emptyMarkup = renderToStaticMarkup(emptyTree);

    expect(emptyMarkup).toContain('role="alert"');
    expect(emptyMarkup).toContain("No audio output devices are listed");
    expect(emptyMarkup).not.toContain("System default output");

    const defaultTree = AudioSettings({
      devices: [],
      deviceListStatus: "loaded",
      selectedDevice: "default",
      outputError: false,
      onRefresh,
      onRetry,
    });
    const defaultMarkup = renderToStaticMarkup(defaultTree);

    expect(defaultMarkup).toContain("System default output");
    expect(defaultMarkup).not.toContain("No audio output devices are listed");

    const failedTree = AudioSettings({
      devices: [],
      deviceListStatus: "error",
      selectedDevice: null,
      outputError: false,
      onRefresh,
      onRetry,
    });
    const failedMarkup = renderToStaticMarkup(failedTree);

    expect(failedMarkup).toContain("Could not refresh the audio device list");
    expect(failedMarkup).not.toContain("No audio output devices are listed");
  });
});
