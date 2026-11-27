import { describe, expect, it } from "vitest";
import {
  LIBRARY_WAVEFORM_PEAK_COUNT,
  validateLibraryWaveform,
} from "./libraryWaveform";

describe("validateLibraryWaveform", () => {
  it("preserves an absent waveform as the not-analyzed state", () => {
    expect(validateLibraryWaveform(null)).toBeNull();
  });

  it("accepts the bounded normalized peak vector", () => {
    const peaks = Array.from(
      { length: LIBRARY_WAVEFORM_PEAK_COUNT },
      () => 0.5,
    );
    expect(validateLibraryWaveform(peaks)).toEqual(peaks);
  });

  it.each([
    [],
    Array.from({ length: LIBRARY_WAVEFORM_PEAK_COUNT - 1 }, () => 0.5),
    Array.from({ length: LIBRARY_WAVEFORM_PEAK_COUNT + 1 }, () => 0.5),
    Array.from({ length: LIBRARY_WAVEFORM_PEAK_COUNT }, (_, index) =>
      index === 17 ? Number.NaN : 0.5,
    ),
    Array.from({ length: LIBRARY_WAVEFORM_PEAK_COUNT }, (_, index) =>
      index === 17 ? 1.1 : 0.5,
    ),
  ])("rejects malformed peak data", (peaks) => {
    expect(() => validateLibraryWaveform(peaks)).toThrow(
      "Saved waveform data is invalid. Analyze the track again.",
    );
  });
});
