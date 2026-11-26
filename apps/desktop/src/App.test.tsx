import { renderToStaticMarkup } from "react-dom/server";
import { isValidElement, type ReactElement, type ReactNode } from "react";
import { describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { M1_MATERIAL_ICON_NAMES, MaterialIcon } from "./MaterialIcon";
import {
  createShortcutCommand,
  getDefaultShortcutBindings,
  type CommandDispatcher,
  type ShortcutId,
} from "./commands/commandRegistry";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

import {
  App,
  AppShell,
  AudioSettings,
  DeckPanel,
  invokeNativeTrackPicker,
  LibraryEmptyState,
  MixerConsole,
  QuickCrateBrowser,
  Waveform,
} from "./App";
import { ShortcutOverlay } from "./ShortcutOverlay";

type TestElementProps = {
  children?: ReactNode;
  "aria-label"?: string;
  className?: string;
  disabled?: boolean;
  onChange?: (event: { target: { value: string } }) => void;
  onClick?: () => void;
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
};

const loadedDeck = {
  ...emptyDeck,
  loaded: true,
  filename: "track.wav",
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
    expect(markup).toContain('aria-label="Master output peak level"');
    expect(markup).toContain(
      'disabled="" title="Bridge is not implemented in M1"',
    );
    expect(markup).toContain("No WAV loaded");
  });

  it("renders an honest library empty state with a native-picker action", () => {
    const load = vi.fn();
    const markup = renderToStaticMarkup(
      <LibraryEmptyState loading={false} onLoad={load} />,
    );

    expect(markup).toContain("Your library is empty");
    expect(markup).toContain("does not build or save a library index in M1");
    expect(markup).toContain("Load WAV to Deck A");
    expect(load).not.toHaveBeenCalled();
  });

  it("calls the native track-picker command with the requested deck", async () => {
    await invokeNativeTrackPicker(1);
    expect(invoke).toHaveBeenCalledWith("pick_track", { deck: 1 });
  });

  it("disables the library picker action while loading", () => {
    const markup = renderToStaticMarkup(
      <LibraryEmptyState loading onLoad={() => undefined} />,
    );

    expect(markup).toContain('disabled=""');
    expect(markup).toContain("Opening native picker…");
  });

  it("shows only currently loaded deck slots in the quick crate", () => {
    const load = vi.fn();
    const markup = renderToStaticMarkup(
      <QuickCrateBrowser mixer={emptyMixer} loadingDeck={null} onLoad={load} />,
    );

    expect(markup).toContain("0 / 2 decks loaded");
    expect(markup).toContain("No WAV selected");
    expect(markup).toContain(
      "SAYT does not scan folders or keep a library index yet.",
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

  it("limits Audio Settings to currently exposed output controls", () => {
    const markup = renderToStaticMarkup(
      <AudioSettings
        devices={[{ id: "system-out-1", name: "System Output" }]}
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
      selectedDevice: "removed-device",
      outputError: true,
      onRefresh,
      onRetry,
    });
    const controls = findElements(tree);
    const markup = renderToStaticMarkup(tree);
    const buttons = controls.filter((element) => element.type === "button");

    expect(markup).toContain('role="alert"');
    expect(markup).toContain("Audio output stopped");
    buttons[0]?.props.onClick?.();
    buttons[1]?.props.onClick?.();
    expect(onRefresh).toHaveBeenCalledOnce();
    expect(onRetry).toHaveBeenCalledOnce();
  });
});
