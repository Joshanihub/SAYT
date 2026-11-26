import { describe, expect, it, vi } from "vitest";
import {
  createShortcutCommand,
  detectShortcutPlatform,
  dispatchShortcutKeydown,
  formatShortcut,
  getDefaultShortcutBindings,
  getEffectiveShortcutBindings,
  isEditableTarget,
  isNativeSpaceTarget,
  KEYBOARD_COMMANDS,
  loadShortcutOverrides,
  saveShortcutOverrides,
  searchKeyboardCommands,
  shortcutBindingFromEvent,
  shortcutMatchesEvent,
  SHORTCUT_STORAGE_KEY,
  validateShortcutBinding,
  type ShortcutBinding,
  type ShortcutPlatform,
  type ShortcutStorage,
} from "./commandRegistry";

class MemoryStorage implements ShortcutStorage {
  private readonly values = new Map<string, string>();

  getItem(key: string) {
    return this.values.get(key) ?? null;
  }

  setItem(key: string, value: string) {
    this.values.set(key, value);
  }
}

const binding = (
  code: string,
  modifiers: Partial<Omit<ShortcutBinding, "code">> = {},
): ShortcutBinding => ({
  code,
  ctrl: modifiers.ctrl ?? false,
  alt: modifiers.alt ?? false,
  shift: modifiers.shift ?? false,
  meta: modifiers.meta ?? false,
});

const target = (value: Record<string, unknown>) =>
  value as unknown as EventTarget;

describe("central keyboard command registry", () => {
  it.each<ShortcutPlatform>(["windows", "mac", "linux", "other"])(
    "provides a unique built-in mapping set on %s",
    (platform) => {
      const defaults = Object.values(getDefaultShortcutBindings(platform));
      const signatures = defaults.map((value) =>
        [value.code, value.ctrl, value.alt, value.shift, value.meta].join("/"),
      );

      expect(defaults).toHaveLength(KEYBOARD_COMMANDS.length);
      expect(new Set(signatures).size).toBe(signatures.length);
    },
  );

  it.each<ShortcutPlatform>(["windows", "mac", "linux", "other"])(
    "keeps every built-in mapping out of reserved keys on %s",
    (platform) => {
      const defaults = getDefaultShortcutBindings(platform);
      for (const command of KEYBOARD_COMMANDS) {
        expect(
          validateShortcutBinding(
            command.id,
            defaults[command.id],
            defaults,
            platform,
          ),
          command.id,
        ).toBeNull();
      }
    },
  );

  it("uses the platform primary modifier for default commands and display", () => {
    const windows = getDefaultShortcutBindings("windows").showLibraryWorkspace;
    const mac = getDefaultShortcutBindings("mac").showLibraryWorkspace;

    expect(windows.ctrl).toBe(true);
    expect(windows.meta).toBe(false);
    expect(mac.ctrl).toBe(false);
    expect(mac.meta).toBe(true);
    expect(formatShortcut(mac, "mac")).toBe("⌘+Option+2");
    expect(detectShortcutPlatform("Mozilla Macintosh Mac OS X")).toBe("mac");
    expect(detectShortcutPlatform("Windows NT 10.0")).toBe("windows");
  });

  it("matches every modifier exactly and captures physical keys", () => {
    const target = binding("KeyA", { ctrl: true, shift: true });
    const matchingEvent = {
      code: "KeyA",
      ctrlKey: true,
      altKey: false,
      shiftKey: true,
      metaKey: false,
    };

    expect(shortcutMatchesEvent(target, matchingEvent)).toBe(true);
    expect(
      shortcutMatchesEvent(target, { ...matchingEvent, shiftKey: false }),
    ).toBe(false);
    expect(shortcutBindingFromEvent(matchingEvent)).toEqual(target);
    expect(
      shortcutBindingFromEvent({ ...matchingEvent, code: "ShiftLeft" }),
    ).toBeNull();
  });

  it("routes a real matching keydown through the typed command dispatcher", () => {
    const event = {
      code: "F1",
      ctrlKey: false,
      altKey: false,
      shiftKey: false,
      metaKey: false,
      repeat: false,
      target: target({ tagName: "MAIN" }),
      preventDefault: vi.fn(),
    };
    const dispatch = vi.fn();

    expect(
      dispatchShortcutKeydown(event, {
        bindings: getDefaultShortcutBindings("windows"),
        context: { focusedDeck: 1 },
        overlayOpen: false,
        dispatch,
      }),
    ).toBe(true);
    expect(event.preventDefault).toHaveBeenCalledOnce();
    expect(dispatch).toHaveBeenCalledWith({
      id: "shortcuts.open",
      payload: {},
    });
  });

  it("ignores editable targets, native Space buttons, open overlays and non-repeatable repeats", () => {
    const bindings = getDefaultShortcutBindings("windows");
    const dispatch = vi.fn();
    const createEvent = (
      code: string,
      eventTarget: EventTarget,
      repeat = false,
    ) => ({
      code,
      ctrlKey: false,
      altKey: false,
      shiftKey: false,
      metaKey: false,
      repeat,
      target: eventTarget,
      preventDefault: vi.fn(),
    });
    const ignoredCases = [
      {
        event: createEvent("F1", target({ tagName: "INPUT" })),
        overlayOpen: false,
      },
      {
        event: createEvent("Space", target({ tagName: "BUTTON" })),
        overlayOpen: false,
      },
      {
        event: createEvent("F1", target({ tagName: "MAIN" })),
        overlayOpen: true,
      },
      {
        event: createEvent("F1", target({ tagName: "MAIN" }), true),
        overlayOpen: false,
      },
    ];

    for (const { event, overlayOpen } of ignoredCases) {
      expect(
        dispatchShortcutKeydown(event, {
          bindings,
          context: { focusedDeck: 0 },
          overlayOpen,
          dispatch,
        }),
      ).toBe(false);
      expect(event.preventDefault).not.toHaveBeenCalled();
    }
    expect(dispatch).not.toHaveBeenCalled();
  });

  it("does not treat text editors or native space buttons as command surfaces", () => {
    expect(isEditableTarget(target({ tagName: "INPUT" }))).toBe(true);
    expect(isEditableTarget(target({ tagName: "TEXTAREA" }))).toBe(true);
    expect(isEditableTarget(target({ isContentEditable: true }))).toBe(true);
    expect(isEditableTarget(target({ tagName: "BUTTON" }))).toBe(false);
    expect(isNativeSpaceTarget(target({ tagName: "BUTTON" }))).toBe(true);
    expect(isNativeSpaceTarget(target({ tagName: "SECTION" }))).toBe(false);
  });

  it("rejects operating-system-reserved combinations and duplicate assignments", () => {
    const defaults = getDefaultShortcutBindings("windows");
    const conflict = validateShortcutBinding(
      "gainDown",
      defaults.togglePlayback,
      defaults,
      "windows",
    );
    const reserved = validateShortcutBinding(
      "gainDown",
      binding("F4", { alt: true }),
      defaults,
      "windows",
    );
    const modifierOnly = validateShortcutBinding(
      "gainDown",
      binding("ControlLeft", { ctrl: true }),
      defaults,
      "windows",
    );

    expect(conflict).toContain("Play / pause focused deck");
    expect(reserved).toContain("reserved");
    expect(modifierOnly).toContain("non-modifier key");
    expect(
      validateShortcutBinding(
        "gainDown",
        binding("Delete", { ctrl: true, alt: true }),
        defaults,
        "windows",
      ),
    ).toContain("reserved");
    expect(
      validateShortcutBinding(
        "gainDown",
        binding("KeyW", { ctrl: true }),
        defaults,
        "windows",
      ),
    ).toContain("reserved");
    expect(
      validateShortcutBinding("gainDown", binding("Tab"), defaults, "windows"),
    ).toContain("reserved");
    expect(
      validateShortcutBinding("gainDown", binding("F12"), defaults, "windows"),
    ).toContain("reserved");
  });

  it("resolves focused-deck commands to the same typed actions as interface controls", () => {
    expect(createShortcutCommand("togglePlayback", { focusedDeck: 1 })).toEqual(
      { id: "deck.playPause", payload: { deck: 1 } },
    );
    expect(
      createShortcutCommand("crossfaderRight", { focusedDeck: 0 }),
    ).toEqual({
      id: "mixer.adjustCrossfader",
      payload: { delta: 0.05 },
    });
  });

  it("persists and reloads valid local overrides", () => {
    const storage = new MemoryStorage();
    const overrides = { gainUp: binding("KeyX", { ctrl: true }) };

    expect(saveShortcutOverrides(storage, overrides)).toBeNull();
    const loaded = loadShortcutOverrides(storage, "windows");

    expect(storage.getItem(SHORTCUT_STORAGE_KEY)).toContain('"version":1');
    expect(loaded.warning).toBeNull();
    expect(loaded.overrides).toEqual(overrides);
    expect(
      getEffectiveShortcutBindings(loaded.overrides, "windows").gainUp,
    ).toEqual(overrides.gainUp);
  });

  it("falls back cleanly for malformed, unsupported, or conflicting saved data", () => {
    const storage = new MemoryStorage();
    storage.setItem(SHORTCUT_STORAGE_KEY, "{");
    expect(loadShortcutOverrides(storage, "windows").warning).toContain(
      "damaged",
    );

    storage.setItem(
      SHORTCUT_STORAGE_KEY,
      JSON.stringify({ version: 2, shortcuts: {} }),
    );
    expect(loadShortcutOverrides(storage, "windows").warning).toContain(
      "unsupported",
    );

    storage.setItem(
      SHORTCUT_STORAGE_KEY,
      JSON.stringify({
        version: 1,
        shortcuts: {
          gainUp: binding("Space"),
          unknownFutureCommand: binding("KeyZ"),
        },
      }),
    );
    const result = loadShortcutOverrides(storage, "windows");
    expect(result.warning).toContain("conflict or are invalid");
    expect(result.overrides).toEqual({});
  });

  it("reports storage access failures without claiming the preference was saved", () => {
    const failingStorage: ShortcutStorage = {
      getItem: () => {
        throw new Error("blocked");
      },
      setItem: () => {
        throw new Error("quota");
      },
    };

    expect(loadShortcutOverrides(failingStorage, "mac").warning).toContain(
      "could not be read",
    );
    expect(
      String(saveShortcutOverrides(failingStorage, {})).toLowerCase(),
    ).toContain("could not be saved");
  });

  it("searches command labels, descriptions, categories, and contexts", () => {
    expect(
      searchKeyboardCommands("  mixer ").map((command) => command.id),
    ).toEqual(
      expect.arrayContaining([
        "crossfaderLeft",
        "masterLevelDown",
        "masterLevelUp",
      ]),
    );
    expect(searchKeyboardCommands("focused deck").length).toBeGreaterThan(0);
    expect(searchKeyboardCommands("no such command")).toEqual([]);
  });

  it("clears saved overrides when reset writes the empty mapping", () => {
    const storage = new MemoryStorage();
    expect(
      saveShortcutOverrides(storage, { gainUp: binding("KeyX") }),
    ).toBeNull();
    expect(saveShortcutOverrides(storage, {})).toBeNull();
    expect(loadShortcutOverrides(storage, "windows").overrides).toEqual({});
  });
});
