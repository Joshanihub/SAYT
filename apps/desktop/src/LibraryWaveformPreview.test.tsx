import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { LibraryWaveformPreview } from "./LibraryWaveformPreview";

describe("LibraryWaveformPreview", () => {
  it("exposes loading and not-yet-analyzed states", () => {
    const loadingMarkup = renderToStaticMarkup(
      <LibraryWaveformPreview trackTitle="Track" loading peaks={null} />,
    );
    const emptyMarkup = renderToStaticMarkup(
      <LibraryWaveformPreview
        trackTitle="Track"
        loading={false}
        peaks={null}
      />,
    );

    expect(loadingMarkup).toContain('role="status"');
    expect(loadingMarkup).toContain("Loading saved waveform");
    expect(emptyMarkup).toContain("No saved waveform");
  });

  it("renders an accessible waveform from the normalized saved peaks", () => {
    const markup = renderToStaticMarkup(
      <LibraryWaveformPreview
        trackTitle="Glass Lines"
        loading={false}
        peaks={[0, 0.5, 1]}
      />,
    );

    expect(markup).toContain('role="img"');
    expect(markup).toContain('aria-label="Saved waveform for Glass Lines"');
    expect(markup.match(/aria-hidden="true"/g)).toHaveLength(3);
    expect(markup).toContain("height:3%");
    expect(markup).toContain("height:50%");
    expect(markup).toContain("height:100%");
  });
});
