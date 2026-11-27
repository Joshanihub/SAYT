export type DeckId = 0 | 1;
export type HotCueSlot = 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7;
export type HotCueAction = "set" | "trigger" | "clear";
export type ManualLoopAction = "setIn" | "setOut" | "setSize" | "release";
export type BeatPerformanceAction =
  "setManualBpm" | "beatJump" | "beginLoopRoll" | "endLoopRoll";
export type HotCueColor =
  "coral" | "orange" | "amber" | "lime" | "cyan" | "blue" | "violet" | "pink";
export type WorkspaceId = "dj" | "library" | "preparation" | "settings";
export type DeckControlId = "gain" | "low" | "mid" | "high";

export const COMMAND_ADJUSTMENTS = {
  seekSeconds: 5,
  crossfader: 0.05,
  masterLevel: 0.05,
  gain: 0.05,
  eqDb: 0.5,
} as const;

export const DECK_CONTROL_LIMITS: Record<
  DeckControlId,
  { min: number; max: number; step: number }
> = {
  gain: { min: 0, max: 1.5, step: COMMAND_ADJUSTMENTS.gain },
  low: { min: -12, max: 12, step: COMMAND_ADJUSTMENTS.eqDb },
  mid: { min: -12, max: 12, step: COMMAND_ADJUSTMENTS.eqDb },
  high: { min: -12, max: 12, step: COMMAND_ADJUSTMENTS.eqDb },
};

export const MIXER_CONTROL_LIMITS = {
  master: { min: 0, max: 1, step: 0.01 },
  crossfader: { min: 0, max: 1, center: 0.5, step: 0.01 },
} as const;

export type CommandPayloadMap = {
  "deck.focus": { deck: DeckId };
  "deck.playPause": { deck: DeckId };
  "deck.stop": { deck: DeckId };
  "deck.loadTrack": { deck: DeckId };
  "deck.seekTo": { deck: DeckId; position: number };
  "deck.seekBy": { deck: DeckId; delta: number };
  "deck.setControl": {
    deck: DeckId;
    control: DeckControlId;
    value: number;
  };
  "deck.adjustControl": {
    deck: DeckId;
    control: DeckControlId;
    delta: number;
  };
  "cue.set": {
    deck: DeckId;
    slot: HotCueSlot;
    sourceTrackId: string | null;
  };
  "cue.trigger": {
    deck: DeckId;
    slot: HotCueSlot;
    sourceTrackId: string | null;
  };
  "cue.clear": {
    deck: DeckId;
    slot: HotCueSlot;
    sourceTrackId: string | null;
  };
  "cue.rename": {
    deck: DeckId;
    slot: HotCueSlot;
    sourceTrackId: string | null;
    name: string;
  };
  "cue.setColor": {
    deck: DeckId;
    slot: HotCueSlot;
    sourceTrackId: string | null;
    color: HotCueColor;
  };
  "loop.setIn": { deck: DeckId; sourceTrackId: string | null };
  "loop.setOut": { deck: DeckId; sourceTrackId: string | null };
  "loop.setSize": {
    deck: DeckId;
    sourceTrackId: string | null;
    sizeSeconds: number;
  };
  "loop.release": { deck: DeckId; sourceTrackId: string | null };
  "performance.setManualBpm": {
    deck: DeckId;
    sourceTrackId: string | null;
    bpm: number;
  };
  "performance.beatJump": {
    deck: DeckId;
    sourceTrackId: string | null;
    beats: number;
  };
  "performance.beginLoopRoll": {
    deck: DeckId;
    sourceTrackId: string | null;
    beats: number;
  };
  "performance.endLoopRoll": {
    deck: DeckId;
    sourceTrackId: string | null;
  };
  "mixer.setMaster": { value: number };
  "mixer.adjustMaster": { delta: number };
  "mixer.setCrossfader": { value: number };
  "mixer.adjustCrossfader": { delta: number };
  "workspace.navigate": { workspace: WorkspaceId };
  "output.select": { deviceId: string };
  "output.refreshDevices": Record<string, never>;
  "output.retry": Record<string, never>;
  "shortcuts.open": Record<string, never>;
};

export type CommandId = keyof CommandPayloadMap;
export type AppCommand = {
  [Id in CommandId]: { id: Id; payload: CommandPayloadMap[Id] };
}[CommandId];
export type CommandDispatcher = (command: AppCommand) => void;

export type ShortcutPlatform = "windows" | "mac" | "linux" | "other";

export type ShortcutBinding = {
  code: string;
  ctrl: boolean;
  alt: boolean;
  shift: boolean;
  meta: boolean;
};

type ShortcutSpec = {
  code: string;
  primary?: boolean;
  ctrl?: boolean;
  alt?: boolean;
  shift?: boolean;
  meta?: boolean;
};

export type ShortcutContext = {
  focusedDeck: DeckId;
  focusedTrackId?: string | null;
};

export type ShortcutDefinition = {
  id: string;
  label: string;
  description: string;
  category: string;
  context: "Global" | "Focused deck";
  defaultShortcut: ShortcutSpec;
  repeatable: boolean;
  createCommand: (context: ShortcutContext) => AppCommand;
};

const BASE_KEYBOARD_COMMANDS = [
  {
    id: "focusDeckA",
    label: "Focus Deck A",
    description: "Select Deck A as the target for focused-deck commands.",
    category: "Deck focus",
    context: "Global",
    defaultShortcut: { code: "Digit1", alt: true },
    repeatable: false,
    createCommand: () => ({ id: "deck.focus", payload: { deck: 0 } }),
  },
  {
    id: "focusDeckB",
    label: "Focus Deck B",
    description: "Select Deck B as the target for focused-deck commands.",
    category: "Deck focus",
    context: "Global",
    defaultShortcut: { code: "Digit2", alt: true },
    repeatable: false,
    createCommand: () => ({ id: "deck.focus", payload: { deck: 1 } }),
  },
  {
    id: "togglePlayback",
    label: "Play / pause focused deck",
    description: "Toggle transport for the currently focused deck.",
    category: "Transport",
    context: "Focused deck",
    defaultShortcut: { code: "Space" },
    repeatable: false,
    createCommand: ({ focusedDeck }) => ({
      id: "deck.playPause",
      payload: { deck: focusedDeck },
    }),
  },
  {
    id: "stopFocusedDeck",
    label: "Stop focused deck",
    description: "Stop the currently focused deck.",
    category: "Transport",
    context: "Focused deck",
    defaultShortcut: { code: "Space", shift: true },
    repeatable: false,
    createCommand: ({ focusedDeck }) => ({
      id: "deck.stop",
      payload: { deck: focusedDeck },
    }),
  },
  {
    id: "loadFocusedDeck",
    label: "Load WAV into focused deck",
    description: "Open the native file picker for the focused deck.",
    category: "Track loading",
    context: "Focused deck",
    defaultShortcut: { code: "KeyL", alt: true, shift: true },
    repeatable: false,
    createCommand: ({ focusedDeck }) => ({
      id: "deck.loadTrack",
      payload: { deck: focusedDeck },
    }),
  },
  {
    id: "seekBackward",
    label: "Seek backward",
    description: "Move the focused deck backward by five seconds.",
    category: "Transport",
    context: "Focused deck",
    defaultShortcut: { code: "ArrowLeft", alt: true },
    repeatable: true,
    createCommand: ({ focusedDeck }) => ({
      id: "deck.seekBy",
      payload: { deck: focusedDeck, delta: -COMMAND_ADJUSTMENTS.seekSeconds },
    }),
  },
  {
    id: "seekForward",
    label: "Seek forward",
    description: "Move the focused deck forward by five seconds.",
    category: "Transport",
    context: "Focused deck",
    defaultShortcut: { code: "ArrowRight", alt: true },
    repeatable: true,
    createCommand: ({ focusedDeck }) => ({
      id: "deck.seekBy",
      payload: { deck: focusedDeck, delta: COMMAND_ADJUSTMENTS.seekSeconds },
    }),
  },
  {
    id: "gainDown",
    label: "Reduce focused deck gain",
    description: "Lower focused-deck gain by one keyboard step.",
    category: "Deck controls",
    context: "Focused deck",
    defaultShortcut: { code: "KeyG", alt: true },
    repeatable: true,
    createCommand: ({ focusedDeck }) => ({
      id: "deck.adjustControl",
      payload: {
        deck: focusedDeck,
        control: "gain",
        delta: -COMMAND_ADJUSTMENTS.gain,
      },
    }),
  },
  {
    id: "gainUp",
    label: "Raise focused deck gain",
    description: "Raise focused-deck gain by one keyboard step.",
    category: "Deck controls",
    context: "Focused deck",
    defaultShortcut: { code: "KeyG", alt: true, shift: true },
    repeatable: true,
    createCommand: ({ focusedDeck }) => ({
      id: "deck.adjustControl",
      payload: {
        deck: focusedDeck,
        control: "gain",
        delta: COMMAND_ADJUSTMENTS.gain,
      },
    }),
  },
  {
    id: "lowEqDown",
    label: "Reduce focused deck low EQ",
    description: "Lower the focused deck's low EQ by one keyboard step.",
    category: "Deck controls",
    context: "Focused deck",
    defaultShortcut: { code: "KeyQ", alt: true },
    repeatable: true,
    createCommand: ({ focusedDeck }) => ({
      id: "deck.adjustControl",
      payload: {
        deck: focusedDeck,
        control: "low",
        delta: -COMMAND_ADJUSTMENTS.eqDb,
      },
    }),
  },
  {
    id: "lowEqUp",
    label: "Raise focused deck low EQ",
    description: "Raise the focused deck's low EQ by one keyboard step.",
    category: "Deck controls",
    context: "Focused deck",
    defaultShortcut: { code: "KeyQ", alt: true, shift: true },
    repeatable: true,
    createCommand: ({ focusedDeck }) => ({
      id: "deck.adjustControl",
      payload: {
        deck: focusedDeck,
        control: "low",
        delta: COMMAND_ADJUSTMENTS.eqDb,
      },
    }),
  },
  {
    id: "midEqDown",
    label: "Reduce focused deck mid EQ",
    description: "Lower the focused deck's mid EQ by one keyboard step.",
    category: "Deck controls",
    context: "Focused deck",
    defaultShortcut: { code: "KeyW", alt: true },
    repeatable: true,
    createCommand: ({ focusedDeck }) => ({
      id: "deck.adjustControl",
      payload: {
        deck: focusedDeck,
        control: "mid",
        delta: -COMMAND_ADJUSTMENTS.eqDb,
      },
    }),
  },
  {
    id: "midEqUp",
    label: "Raise focused deck mid EQ",
    description: "Raise the focused deck's mid EQ by one keyboard step.",
    category: "Deck controls",
    context: "Focused deck",
    defaultShortcut: { code: "KeyW", alt: true, shift: true },
    repeatable: true,
    createCommand: ({ focusedDeck }) => ({
      id: "deck.adjustControl",
      payload: {
        deck: focusedDeck,
        control: "mid",
        delta: COMMAND_ADJUSTMENTS.eqDb,
      },
    }),
  },
  {
    id: "highEqDown",
    label: "Reduce focused deck high EQ",
    description: "Lower the focused deck's high EQ by one keyboard step.",
    category: "Deck controls",
    context: "Focused deck",
    defaultShortcut: { code: "KeyE", alt: true },
    repeatable: true,
    createCommand: ({ focusedDeck }) => ({
      id: "deck.adjustControl",
      payload: {
        deck: focusedDeck,
        control: "high",
        delta: -COMMAND_ADJUSTMENTS.eqDb,
      },
    }),
  },
  {
    id: "highEqUp",
    label: "Raise focused deck high EQ",
    description: "Raise the focused deck's high EQ by one keyboard step.",
    category: "Deck controls",
    context: "Focused deck",
    defaultShortcut: { code: "KeyE", alt: true, shift: true },
    repeatable: true,
    createCommand: ({ focusedDeck }) => ({
      id: "deck.adjustControl",
      payload: {
        deck: focusedDeck,
        control: "high",
        delta: COMMAND_ADJUSTMENTS.eqDb,
      },
    }),
  },
  {
    id: "setLoopIn",
    label: "Set loop in on focused deck",
    description: "Set the manual loop start at the focused deck's position.",
    category: "Manual loops",
    context: "Focused deck",
    defaultShortcut: { code: "KeyI", alt: true },
    repeatable: false,
    createCommand: ({ focusedDeck, focusedTrackId }) => ({
      id: "loop.setIn",
      payload: { deck: focusedDeck, sourceTrackId: focusedTrackId ?? null },
    }),
  },
  {
    id: "setLoopOut",
    label: "Set loop out on focused deck",
    description: "Set the manual loop end at the focused deck's position.",
    category: "Manual loops",
    context: "Focused deck",
    defaultShortcut: { code: "KeyO", alt: true },
    repeatable: false,
    createCommand: ({ focusedDeck, focusedTrackId }) => ({
      id: "loop.setOut",
      payload: { deck: focusedDeck, sourceTrackId: focusedTrackId ?? null },
    }),
  },
  {
    id: "releaseLoop",
    label: "Release focused deck loop",
    description: "Exit the active loop at its next boundary while playing.",
    category: "Manual loops",
    context: "Focused deck",
    defaultShortcut: { code: "KeyO", alt: true, shift: true },
    repeatable: false,
    createCommand: ({ focusedDeck, focusedTrackId }) => ({
      id: "loop.release",
      payload: { deck: focusedDeck, sourceTrackId: focusedTrackId ?? null },
    }),
  },
  {
    id: "crossfaderLeft",
    label: "Move crossfader toward Deck A",
    description: "Move the crossfader left by one keyboard step.",
    category: "Mixer",
    context: "Global",
    defaultShortcut: {
      code: "ArrowLeft",
      primary: true,
      shift: true,
    },
    repeatable: true,
    createCommand: () => ({
      id: "mixer.adjustCrossfader",
      payload: { delta: -COMMAND_ADJUSTMENTS.crossfader },
    }),
  },
  {
    id: "crossfaderRight",
    label: "Move crossfader toward Deck B",
    description: "Move the crossfader right by one keyboard step.",
    category: "Mixer",
    context: "Global",
    defaultShortcut: {
      code: "ArrowRight",
      primary: true,
      shift: true,
    },
    repeatable: true,
    createCommand: () => ({
      id: "mixer.adjustCrossfader",
      payload: { delta: COMMAND_ADJUSTMENTS.crossfader },
    }),
  },
  {
    id: "crossfaderCenter",
    label: "Center crossfader",
    description: "Set the crossfader to its center position.",
    category: "Mixer",
    context: "Global",
    defaultShortcut: {
      code: "ArrowDown",
      primary: true,
      shift: true,
    },
    repeatable: false,
    createCommand: () => ({
      id: "mixer.setCrossfader",
      payload: { value: MIXER_CONTROL_LIMITS.crossfader.center },
    }),
  },
  {
    id: "masterLevelDown",
    label: "Lower master level",
    description: "Lower master output by one keyboard step.",
    category: "Mixer",
    context: "Global",
    defaultShortcut: {
      code: "ArrowDown",
      primary: true,
      alt: true,
    },
    repeatable: true,
    createCommand: () => ({
      id: "mixer.adjustMaster",
      payload: { delta: -COMMAND_ADJUSTMENTS.masterLevel },
    }),
  },
  {
    id: "masterLevelUp",
    label: "Raise master level",
    description: "Raise master output by one keyboard step.",
    category: "Mixer",
    context: "Global",
    defaultShortcut: {
      code: "ArrowUp",
      primary: true,
      alt: true,
    },
    repeatable: true,
    createCommand: () => ({
      id: "mixer.adjustMaster",
      payload: { delta: COMMAND_ADJUSTMENTS.masterLevel },
    }),
  },
  {
    id: "showDjWorkspace",
    label: "Show DJ workspace",
    description: "Open the SAYT DJ workspace.",
    category: "Workspaces",
    context: "Global",
    defaultShortcut: { code: "Digit1", primary: true, alt: true },
    repeatable: false,
    createCommand: () => ({
      id: "workspace.navigate",
      payload: { workspace: "dj" },
    }),
  },
  {
    id: "showLibraryWorkspace",
    label: "Show Library workspace",
    description: "Open the Library workspace.",
    category: "Workspaces",
    context: "Global",
    defaultShortcut: { code: "Digit2", primary: true, alt: true },
    repeatable: false,
    createCommand: () => ({
      id: "workspace.navigate",
      payload: { workspace: "library" },
    }),
  },
  {
    id: "showSettingsWorkspace",
    label: "Show Settings workspace",
    description: "Open Audio Settings.",
    category: "Workspaces",
    context: "Global",
    defaultShortcut: { code: "Digit3", primary: true, alt: true },
    repeatable: false,
    createCommand: () => ({
      id: "workspace.navigate",
      payload: { workspace: "settings" },
    }),
  },
  {
    id: "refreshAudioDevices",
    label: "Refresh audio devices",
    description: "Re-enumerate the operating-system audio outputs.",
    category: "Audio devices",
    context: "Global",
    defaultShortcut: { code: "KeyR", primary: true, alt: true, shift: true },
    repeatable: false,
    createCommand: () => ({
      id: "output.refreshDevices",
      payload: {},
    }),
  },
  {
    id: "retryAudioOutput",
    label: "Retry audio output",
    description: "Reopen the currently selected output device.",
    category: "Audio devices",
    context: "Global",
    defaultShortcut: { code: "KeyR", primary: true, alt: true },
    repeatable: false,
    createCommand: () => ({ id: "output.retry", payload: {} }),
  },
  {
    id: "openShortcutHelp",
    label: "Open keyboard shortcuts",
    description: "Search, remap or reset SAYT keyboard commands.",
    category: "Help",
    context: "Global",
    defaultShortcut: { code: "F1" },
    repeatable: false,
    createCommand: () => ({ id: "shortcuts.open", payload: {} }),
  },
] as const satisfies readonly ShortcutDefinition[];

type HotCueSlotNumber = 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8;
export type HotCueShortcutId =
  `${Capitalize<HotCueAction>}HotCue${HotCueSlotNumber}`;
type HotCueShortcutDefinition = Omit<ShortcutDefinition, "id"> & {
  id: HotCueShortcutId;
};

const HOT_CUE_SLOT_IDS: readonly HotCueSlot[] = [0, 1, 2, 3, 4, 5, 6, 7];
export function createHotCueCommand(
  action: HotCueAction,
  deck: DeckId,
  slot: HotCueSlot,
  sourceTrackId: string | null,
): AppCommand {
  const payload = { deck, slot, sourceTrackId };
  switch (action) {
    case "set":
      return { id: "cue.set", payload };
    case "trigger":
      return { id: "cue.trigger", payload };
    case "clear":
      return { id: "cue.clear", payload };
  }
}

export function createManualLoopCommand(
  action: ManualLoopAction,
  deck: DeckId,
  sourceTrackId: string | null,
  sizeSeconds?: number,
): AppCommand {
  if (action === "setIn") {
    return { id: "loop.setIn", payload: { deck, sourceTrackId } };
  }
  if (action === "setOut") {
    return { id: "loop.setOut", payload: { deck, sourceTrackId } };
  }
  if (action === "release") {
    return { id: "loop.release", payload: { deck, sourceTrackId } };
  }
  if (
    sizeSeconds === undefined ||
    !Number.isFinite(sizeSeconds) ||
    sizeSeconds <= 0
  ) {
    throw new RangeError("Loop size commands require positive finite seconds");
  }
  return {
    id: "loop.setSize",
    payload: { deck, sourceTrackId, sizeSeconds },
  };
}

export function createBeatPerformanceCommand(
  action: BeatPerformanceAction,
  deck: DeckId,
  sourceTrackId: string | null,
  value?: number,
): AppCommand {
  if (action === "endLoopRoll") {
    return {
      id: "performance.endLoopRoll",
      payload: { deck, sourceTrackId },
    };
  }
  if (value === undefined || !Number.isFinite(value)) {
    throw new RangeError(`${action} commands require a finite value`);
  }
  if (action === "setManualBpm") {
    return {
      id: "performance.setManualBpm",
      payload: { deck, sourceTrackId, bpm: value },
    };
  }
  if (action === "beatJump") {
    return {
      id: "performance.beatJump",
      payload: { deck, sourceTrackId, beats: value },
    };
  }
  if (value <= 0) {
    throw new RangeError("Loop Roll commands require positive beat counts");
  }
  return {
    id: "performance.beginLoopRoll",
    payload: { deck, sourceTrackId, beats: value },
  };
}

function createHotCueShortcut(
  action: HotCueAction,
  slot: HotCueSlot,
): HotCueShortcutDefinition {
  const slotNumber = (slot + 1) as HotCueSlotNumber;
  const actionName = action[0].toUpperCase() + action.slice(1);
  const defaultShortcut: ShortcutSpec =
    action === "trigger"
      ? { code: `Digit${slotNumber}`, alt: true, shift: true }
      : action === "set"
        ? { code: `Digit${slotNumber}`, primary: true, shift: true }
        : {
            code: `Digit${slotNumber}`,
            primary: true,
            alt: true,
            shift: true,
          };

  return {
    id: `${actionName}HotCue${slotNumber}` as HotCueShortcutId,
    label: `${actionName} focused deck Hot Cue ${slotNumber}`,
    description: `${actionName} Hot Cue ${slotNumber} on the focused deck.`,
    category: "Hot cues",
    context: "Focused deck",
    defaultShortcut,
    repeatable: false,
    createCommand: ({ focusedDeck, focusedTrackId }) =>
      createHotCueCommand(action, focusedDeck, slot, focusedTrackId ?? null),
  };
}

const HOT_CUE_SHORTCUTS = HOT_CUE_SLOT_IDS.flatMap((slot) => [
  createHotCueShortcut("set", slot),
  createHotCueShortcut("trigger", slot),
  createHotCueShortcut("clear", slot),
]);

export const KEYBOARD_COMMANDS = [
  ...BASE_KEYBOARD_COMMANDS,
  ...HOT_CUE_SHORTCUTS,
] as const satisfies readonly ShortcutDefinition[];

export type ShortcutId = (typeof KEYBOARD_COMMANDS)[number]["id"];
export type ShortcutBindings = Record<ShortcutId, ShortcutBinding>;
export type ShortcutOverrides = Partial<ShortcutBindings>;

export function getHotCueShortcutId(
  action: HotCueAction,
  slot: HotCueSlot,
): HotCueShortcutId {
  const actionName = action[0].toUpperCase() + action.slice(1);
  return `${actionName}HotCue${slot + 1}` as HotCueShortcutId;
}

export const SHORTCUT_STORAGE_KEY = "sayt.keyboard-shortcuts.v1";
const SHORTCUT_STORAGE_VERSION = 1;
const MODIFIER_CODES = new Set([
  "AltLeft",
  "AltRight",
  "ControlLeft",
  "ControlRight",
  "MetaLeft",
  "MetaRight",
  "ShiftLeft",
  "ShiftRight",
]);

export function detectShortcutPlatform(userAgent: string): ShortcutPlatform {
  if (/Macintosh|Mac OS|iPhone|iPad/i.test(userAgent)) return "mac";
  if (/Windows/i.test(userAgent)) return "windows";
  if (/Linux|X11/i.test(userAgent)) return "linux";
  return "other";
}

function resolveShortcutSpec(
  spec: ShortcutSpec,
  platform: ShortcutPlatform,
): ShortcutBinding {
  const primary = spec.primary ?? false;
  return {
    code: spec.code,
    ctrl: spec.ctrl ?? (primary && platform !== "mac"),
    alt: spec.alt ?? false,
    shift: spec.shift ?? false,
    meta: spec.meta ?? (primary && platform === "mac"),
  };
}

export function getDefaultShortcutBindings(
  platform: ShortcutPlatform,
): ShortcutBindings {
  return Object.fromEntries(
    KEYBOARD_COMMANDS.map((command) => [
      command.id,
      resolveShortcutSpec(command.defaultShortcut, platform),
    ]),
  ) as ShortcutBindings;
}

export function getEffectiveShortcutBindings(
  overrides: ShortcutOverrides,
  platform: ShortcutPlatform,
): ShortcutBindings {
  return { ...getDefaultShortcutBindings(platform), ...overrides };
}

export function searchKeyboardCommands(
  query: string,
): readonly (typeof KEYBOARD_COMMANDS)[number][] {
  const normalizedQuery = query.trim().toLowerCase();
  if (!normalizedQuery) return KEYBOARD_COMMANDS;
  return KEYBOARD_COMMANDS.filter((command) =>
    [command.label, command.description, command.category, command.context]
      .join(" ")
      .toLowerCase()
      .includes(normalizedQuery),
  );
}

export function shortcutBindingFromEvent(
  event: Pick<
    KeyboardEvent,
    "code" | "ctrlKey" | "altKey" | "shiftKey" | "metaKey"
  >,
): ShortcutBinding | null {
  if (!event.code || MODIFIER_CODES.has(event.code)) return null;
  return {
    code: event.code,
    ctrl: event.ctrlKey,
    alt: event.altKey,
    shift: event.shiftKey,
    meta: event.metaKey,
  };
}

export function shortcutMatchesEvent(
  binding: ShortcutBinding,
  event: Pick<
    KeyboardEvent,
    "code" | "ctrlKey" | "altKey" | "shiftKey" | "metaKey"
  >,
): boolean {
  return (
    event.code === binding.code &&
    event.ctrlKey === binding.ctrl &&
    event.altKey === binding.alt &&
    event.shiftKey === binding.shift &&
    event.metaKey === binding.meta
  );
}

export function isEditableTarget(target: EventTarget | null): boolean {
  if (!target || typeof target !== "object") return false;
  const element = target as {
    tagName?: string;
    isContentEditable?: boolean;
    closest?: (selector: string) => unknown;
  };
  if (element.isContentEditable) return true;
  const tagName = element.tagName?.toLowerCase();
  if (tagName && ["input", "textarea", "select"].includes(tagName)) {
    return true;
  }
  return Boolean(
    element.closest?.('[contenteditable="true"], [role="textbox"]'),
  );
}

export function isNativeSpaceTarget(target: EventTarget | null): boolean {
  if (!target || typeof target !== "object") return false;
  const element = target as {
    tagName?: string;
    closest?: (selector: string) => unknown;
  };
  const tagName = element.tagName?.toLowerCase();
  if (tagName === "button" || tagName === "a") return true;
  return Boolean(element.closest?.("button, a, [role=button]"));
}

export type ShortcutKeyEvent = Pick<
  KeyboardEvent,
  | "code"
  | "ctrlKey"
  | "altKey"
  | "shiftKey"
  | "metaKey"
  | "repeat"
  | "target"
  | "preventDefault"
>;

export function dispatchShortcutKeydown(
  event: ShortcutKeyEvent,
  options: {
    bindings: ShortcutBindings;
    context: ShortcutContext;
    overlayOpen: boolean;
    dispatch: CommandDispatcher;
  },
): boolean {
  if (
    options.overlayOpen ||
    isEditableTarget(event.target) ||
    (event.code === "Space" && isNativeSpaceTarget(event.target))
  ) {
    return false;
  }

  const definition = KEYBOARD_COMMANDS.find((command) =>
    shortcutMatchesEvent(options.bindings[command.id], event),
  );
  if (!definition || (event.repeat && !definition.repeatable)) return false;

  event.preventDefault();
  options.dispatch(definition.createCommand(options.context));
  return true;
}

function isValidBinding(value: unknown): value is ShortcutBinding {
  if (!value || typeof value !== "object") return false;
  const binding = value as Partial<ShortcutBinding>;
  return (
    typeof binding.code === "string" &&
    /^[A-Za-z][A-Za-z0-9]{0,31}$/.test(binding.code) &&
    !MODIFIER_CODES.has(binding.code) &&
    typeof binding.ctrl === "boolean" &&
    typeof binding.alt === "boolean" &&
    typeof binding.shift === "boolean" &&
    typeof binding.meta === "boolean"
  );
}

function sameBinding(a: ShortcutBinding, b: ShortcutBinding): boolean {
  return (
    a.code === b.code &&
    a.ctrl === b.ctrl &&
    a.alt === b.alt &&
    a.shift === b.shift &&
    a.meta === b.meta
  );
}

function isReservedShortcut(
  binding: ShortcutBinding,
  platform: ShortcutPlatform,
): boolean {
  const noModifiers =
    !binding.ctrl && !binding.alt && !binding.shift && !binding.meta;
  if (["Tab", "Enter", "Escape"].includes(binding.code)) return true;
  if (["F5", "F11", "F12"].includes(binding.code)) return true;
  if (binding.alt && binding.code === "F4") {
    return true;
  }
  if (binding.ctrl && binding.alt && binding.code === "Delete") return true;
  if ((binding.alt || binding.meta) && binding.code === "Tab") return true;
  if (noModifiers && binding.code === "F5") return true;

  const primary = platform === "mac" ? binding.meta : binding.ctrl;
  if (!primary || binding.alt) return false;
  if (
    [
      "KeyF",
      "KeyL",
      "KeyN",
      "KeyO",
      "KeyP",
      "KeyQ",
      "KeyR",
      "KeyS",
      "KeyT",
      "KeyW",
      "Tab",
    ].includes(binding.code)
  ) {
    return true;
  }
  if (
    ["ArrowLeft", "ArrowRight", "PageUp", "PageDown"].includes(binding.code) &&
    !binding.shift
  ) {
    return true;
  }
  if (binding.code === "Space") return true;
  if (binding.shift && ["KeyI", "KeyJ"].includes(binding.code)) return true;
  return false;
}

export function validateShortcutBinding(
  shortcutId: ShortcutId,
  candidate: unknown,
  activeBindings: Partial<ShortcutBindings>,
  platform: ShortcutPlatform,
): string | null {
  if (!isValidBinding(candidate)) {
    return "Choose one non-modifier key with optional modifiers.";
  }
  if (isReservedShortcut(candidate, platform)) {
    return "That key combination is reserved by the operating system or application.";
  }
  const conflict = KEYBOARD_COMMANDS.find(
    (command) =>
      command.id !== shortcutId &&
      activeBindings[command.id] &&
      sameBinding(candidate, activeBindings[command.id]!),
  );
  if (conflict) return `Already assigned to ${conflict.label}.`;
  return null;
}

function displayKey(code: string): string {
  if (code === "Space") return "Space";
  if (code.startsWith("Key")) return code.slice(3);
  if (code.startsWith("Digit")) return code.slice(5);
  const arrows: Record<string, string> = {
    ArrowLeft: "←",
    ArrowRight: "→",
    ArrowUp: "↑",
    ArrowDown: "↓",
  };
  return arrows[code] ?? code;
}

export function formatShortcut(
  binding: ShortcutBinding,
  platform: ShortcutPlatform,
): string {
  const modifiers: string[] = [];
  if (platform === "mac" && binding.meta) modifiers.push("⌘");
  if (binding.ctrl) modifiers.push(platform === "mac" ? "Control" : "Ctrl");
  if (binding.alt) modifiers.push(platform === "mac" ? "Option" : "Alt");
  if (binding.shift) modifiers.push(platform === "mac" ? "⇧" : "Shift");
  if (binding.meta && platform !== "mac") modifiers.push("Meta");
  return [...modifiers, displayKey(binding.code)].join("+");
}

export function createShortcutCommand(
  shortcutId: ShortcutId,
  context: ShortcutContext,
): AppCommand {
  const definition = KEYBOARD_COMMANDS.find(
    (command) => command.id === shortcutId,
  );
  if (!definition) throw new Error(`Unknown keyboard command: ${shortcutId}`);
  return definition.createCommand(context);
}

export type ShortcutStorage = {
  getItem: (key: string) => string | null;
  setItem: (key: string, value: string) => void;
};

export type ShortcutLoadResult = {
  overrides: ShortcutOverrides;
  warning: string | null;
};

function isRecord(value: unknown): value is Record<string, unknown> {
  return Boolean(value && typeof value === "object" && !Array.isArray(value));
}

export function loadShortcutOverrides(
  storage: ShortcutStorage,
  platform: ShortcutPlatform,
): ShortcutLoadResult {
  let serialized: string | null;
  try {
    serialized = storage.getItem(SHORTCUT_STORAGE_KEY);
  } catch {
    return {
      overrides: {},
      warning:
        "Shortcut settings could not be read; built-in shortcuts are active.",
    };
  }
  if (!serialized) return { overrides: {}, warning: null };

  let decoded: unknown;
  try {
    decoded = JSON.parse(serialized);
  } catch {
    return {
      overrides: {},
      warning: "Saved shortcuts are damaged; built-in shortcuts are active.",
    };
  }
  if (
    !isRecord(decoded) ||
    decoded.version !== SHORTCUT_STORAGE_VERSION ||
    !isRecord(decoded.shortcuts)
  ) {
    return {
      overrides: {},
      warning:
        "Saved shortcuts use an unsupported format; built-in shortcuts are active.",
    };
  }

  const overrides: ShortcutOverrides = {};
  const activeBindings: Partial<ShortcutBindings> = {
    ...getDefaultShortcutBindings(platform),
  };
  let rejectedEntry = false;
  for (const command of KEYBOARD_COMMANDS) {
    const value = decoded.shortcuts[command.id];
    if (value === undefined) continue;
    const warning = validateShortcutBinding(
      command.id,
      value,
      activeBindings,
      platform,
    );
    if (warning || !isValidBinding(value)) {
      rejectedEntry = true;
      continue;
    }
    overrides[command.id] = value;
    activeBindings[command.id] = value;
  }
  return {
    overrides,
    warning: rejectedEntry
      ? "Some saved shortcuts conflict or are invalid; defaults are active for those commands."
      : null,
  };
}

export function saveShortcutOverrides(
  storage: ShortcutStorage,
  overrides: ShortcutOverrides,
): string | null {
  try {
    storage.setItem(
      SHORTCUT_STORAGE_KEY,
      JSON.stringify({
        version: SHORTCUT_STORAGE_VERSION,
        shortcuts: overrides,
      }),
    );
    return null;
  } catch {
    return "Shortcut settings could not be saved. Check local app storage and retry.";
  }
}
