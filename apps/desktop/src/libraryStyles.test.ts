import { describe, expect, it } from "vitest";
import libraryComponent from "./MusicLibrary.tsx?raw";

describe("library deck load actions", () => {
  it("uses library-scoped deck classes instead of DJ workspace grid areas", () => {
    expect(libraryComponent).toContain(
      'className="library-deck-load library-deck-a"',
    );
    expect(libraryComponent).toContain(
      'className="library-deck-load library-deck-b"',
    );
    expect(libraryComponent).not.toMatch(
      /className="library-deck-load deck-[ab]"/,
    );
  });

  it("loads the selected track's persisted waveform through the Tauri command", () => {
    expect(libraryComponent).toContain(
      'invoke<unknown>("library_track_waveform"',
    );
    expect(libraryComponent).toContain("validateLibraryWaveform(payload)");
    expect(libraryComponent).toContain("<LibraryWaveformPreview");
  });

  it("provides a focused responsive layout for the Preparation workspace", async () => {
    const fileSystemModuleName = "node:fs";
    const fileSystem = (await import(fileSystemModuleName)) as {
      readFileSync(path: URL, encoding: string): string;
    };
    const libraryStyles = fileSystem.readFileSync(
      new URL("./library.css", import.meta.url),
      "utf8",
    );

    expect(libraryComponent).toContain('workspace === "preparation"');
    expect(libraryStyles).toContain(
      ".music-library--preparation .library-layout",
    );
    expect(libraryStyles).toContain(
      ".music-library--preparation .library-track-waveform",
    );
    expect(libraryStyles).toContain("@media (max-width: 980px)");
    expect(libraryStyles).toContain("@media (max-width: 720px)");
    expect(libraryComponent).toContain("recordTempoTap");
    expect(libraryComponent).toContain("TAP BPM");
    expect(libraryComponent).toContain("onClick={tapBpm}");
    expect(libraryComponent).toContain("performance.now()");
    expect(libraryComponent).toContain('aria-live="polite"');
    expect(libraryComponent).toContain(
      "setBpmCorrectionDraft(result.bpm.toFixed(1))",
    );
    expect(libraryComponent).toContain("Save correction to persist.");
  });
});
