import { useEffect, useMemo, useRef, useState } from "react";
import {
  formatShortcut,
  searchKeyboardCommands,
  shortcutBindingFromEvent,
  type ShortcutBinding,
  type ShortcutBindings,
  type ShortcutId,
  type ShortcutPlatform,
} from "./commands/commandRegistry";

export function ShortcutOverlay({
  bindings,
  platform,
  warning,
  onClose,
  onBind,
  onReset,
}: {
  bindings: ShortcutBindings;
  platform: ShortcutPlatform;
  warning: string | null;
  onClose: () => void;
  onBind: (shortcutId: ShortcutId, binding: ShortcutBinding) => string | null;
  onReset: () => void;
}) {
  const [query, setQuery] = useState("");
  const [recordingId, setRecordingId] = useState<ShortcutId | null>(null);
  const [feedback, setFeedback] = useState("");
  const dialogRef = useRef<HTMLDivElement>(null);
  const searchRef = useRef<HTMLInputElement>(null);
  const previousFocus = useRef<HTMLElement | null>(null);

  const visibleCommands = useMemo(() => {
    return searchKeyboardCommands(query);
  }, [query]);

  useEffect(() => {
    previousFocus.current =
      document.activeElement instanceof HTMLElement
        ? document.activeElement
        : null;
    searchRef.current?.focus();
    return () => previousFocus.current?.focus();
  }, []);

  const handleKeyDown = (event: React.KeyboardEvent<HTMLDivElement>) => {
    if (recordingId) {
      if (event.code === "Escape") {
        event.preventDefault();
        setRecordingId(null);
        setFeedback("Shortcut capture cancelled.");
        return;
      }
      const binding = shortcutBindingFromEvent(event.nativeEvent);
      event.preventDefault();
      if (!binding) {
        setFeedback("Press a non-modifier key with optional modifiers.");
        return;
      }
      const error = onBind(recordingId, binding);
      setFeedback(error ?? "Shortcut saved.");
      if (!error) setRecordingId(null);
      return;
    }

    if (event.code === "Escape") {
      event.preventDefault();
      onClose();
      return;
    }
    if (event.code !== "Tab") return;

    const focusable = Array.from(
      dialogRef.current?.querySelectorAll<HTMLElement>(
        "button:not(:disabled), input:not(:disabled)",
      ) ?? [],
    );
    if (focusable.length === 0) return;
    const first = focusable[0];
    const last = focusable[focusable.length - 1];
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last?.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first?.focus();
    }
  };

  return (
    <div className="shortcut-overlay-backdrop" onMouseDown={onClose}>
      <div
        ref={dialogRef}
        className="shortcut-dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby="shortcut-dialog-title"
        aria-describedby="shortcut-dialog-description"
        onMouseDown={(event) => event.stopPropagation()}
        onKeyDown={handleKeyDown}
      >
        <header className="shortcut-dialog-header">
          <div>
            <p className="eyebrow">SAYT COMMANDS</p>
            <h2 id="shortcut-dialog-title">Keyboard shortcuts</h2>
          </div>
          <button
            className="shortcut-close-button"
            type="button"
            onClick={onClose}
            aria-label="Close keyboard shortcuts"
          >
            ×
          </button>
        </header>

        <p
          id="shortcut-dialog-description"
          className="shortcut-dialog-description"
        >
          These commands control the same deck, mixer, device and workspace
          actions as the on-screen controls. Shortcuts are ignored while you
          type or operate a native range control.
        </p>

        <label className="shortcut-search">
          <span className="sr-only">Search keyboard commands</span>
          <input
            ref={searchRef}
            type="search"
            aria-label="Search keyboard commands"
            placeholder="Search commands"
            value={query}
            onChange={(event) => setQuery(event.target.value)}
          />
        </label>

        {(warning || feedback) && (
          <p className="shortcut-feedback" role="status" aria-live="polite">
            {feedback || warning}
          </p>
        )}

        <div className="shortcut-command-list" role="list">
          {visibleCommands.length === 0 ? (
            <p className="shortcut-no-results">No matching commands.</p>
          ) : (
            visibleCommands.map((command) => (
              <article
                className="shortcut-command-row"
                role="listitem"
                key={command.id}
              >
                <div className="shortcut-command-copy">
                  <strong>{command.label}</strong>
                  <span>{command.description}</span>
                </div>
                <code className="shortcut-binding">
                  {formatShortcut(bindings[command.id], platform)}
                </code>
                <button
                  className={
                    recordingId === command.id
                      ? "shortcut-map-button recording"
                      : "shortcut-map-button"
                  }
                  type="button"
                  aria-label={`Change shortcut for ${command.label}`}
                  onClick={() => {
                    setFeedback("");
                    setRecordingId(command.id);
                  }}
                >
                  {recordingId === command.id ? "Press keys…" : "Change"}
                </button>
              </article>
            ))
          )}
        </div>

        <footer className="shortcut-dialog-footer">
          <span>
            {visibleCommands.length} commands ·{" "}
            {platform === "mac" ? "macOS" : "Windows/Linux"} keys
          </span>
          <div>
            <button
              className="transport-secondary"
              type="button"
              onClick={() => {
                setRecordingId(null);
                setFeedback("");
                onReset();
                setFeedback("Built-in shortcuts restored.");
              }}
            >
              Reset defaults
            </button>
            <button className="load-button" type="button" onClick={onClose}>
              Done
            </button>
          </div>
        </footer>
      </div>
    </div>
  );
}
