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
});
