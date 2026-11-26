import { describe, expect, it } from "vitest";
import styles from "./styles.css?inline";

describe("shared interaction motion", () => {
  it("keeps control transitions short and honors reduced motion", () => {
    expect(styles).toContain("--motion-control-duration: 120ms");
    expect(styles).toContain("--motion-control-easing: ease-out");
    expect(styles).toMatch(
      /button,\s*select\s*\{[^}]*color var\(--motion-control-duration\)/,
    );
    expect(styles).toMatch(/@media\s*\(prefers-reduced-motion:\s*reduce\)/);
    expect(styles).toContain("animation-duration: 0ms !important");
    expect(styles).toContain("transition-duration: 0ms !important");
    expect(styles).toContain("scroll-behavior: auto !important");
  });
});

describe("responsive deck workspace placement", () => {
  it("keeps both decks visible before the mixer at compact and narrow widths", () => {
    expect(styles).toMatch(
      /\.dj-grid\s*\{[^}]*grid-template-areas:\s*"deck-a mixer deck-b"/s,
    );
    expect(styles).toMatch(/\.deck-a\s*\{\s*grid-area:\s*deck-a;/);
    expect(styles).toMatch(/\.deck-b\s*\{\s*grid-area:\s*deck-b;/);
    expect(styles).toMatch(/\.mixer-console\s*\{\s*grid-area:\s*mixer;/);
    expect(styles).toMatch(
      /@media\s*\(max-width:\s*1060px\)[\s\S]*?\.dj-grid\s*\{[^}]*grid-template-areas:\s*"deck-a deck-b"\s+"mixer mixer"/s,
    );
    expect(styles).toMatch(
      /@media\s*\(max-width:\s*720px\)[\s\S]*?\.dj-grid\s*\{[^}]*grid-template-areas:\s*"deck-a"\s+"deck-b"\s+"mixer"/s,
    );
  });
});

describe("keyboard command surfaces", () => {
  it("shows focused deck state and styles the searchable shortcut overlay", () => {
    expect(styles).toContain(".deck-panel.keyboard-focused");
    expect(styles).toContain(".deck-b.keyboard-focused");
    expect(styles).toContain(".deck-keyboard-hint");
    expect(styles).toContain(".shortcut-inline-hint");
    expect(styles).toContain(".shortcut-overlay-backdrop");
    expect(styles).toContain(".shortcut-command-list");
    expect(styles).toContain(".shortcut-map-button.recording");
    expect(styles).toMatch(
      /@media\s*\(max-width:\s*480px\)[\s\S]*?\.shortcut-command-row\s*\{[^}]*grid-template-columns:\s*minmax\(0,\s*1fr\)\s+auto/s,
    );
  });
});
