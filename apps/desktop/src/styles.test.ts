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
