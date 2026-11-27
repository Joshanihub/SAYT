import { describe, expect, it } from "vitest";
import { effectiveBpm, scaleBpmCorrection } from "./libraryTempo";

describe("effective library BPM", () => {
  it("prefers a user correction over analysis and source tags", () => {
    expect(
      effectiveBpm({
        bpm: 118,
        bpm_analysis_candidate: 120,
        bpm_analysis_status: "estimated",
        bpm_correction: 121.5,
      }),
    ).toBe(121.5);
  });

  it("uses a confident analysis result before the source tag", () => {
    expect(
      effectiveBpm({
        bpm: 118,
        bpm_analysis_candidate: 120,
        bpm_analysis_status: "estimated",
        bpm_correction: null,
      }),
    ).toBe(120);
  });

  it("does not promote an uncertain candidate over a source tag", () => {
    expect(
      effectiveBpm({
        bpm: 118,
        bpm_analysis_candidate: 120,
        bpm_analysis_status: "uncertain",
        bpm_correction: null,
      }),
    ).toBe(118);
  });

  it("returns no BPM when none of the sources has a usable value", () => {
    expect(
      effectiveBpm({
        bpm: null,
        bpm_analysis_candidate: null,
        bpm_analysis_status: null,
        bpm_correction: null,
      }),
    ).toBeNull();
  });
});

describe("manual BPM correction presets", () => {
  it("scales an in-range tempo and rounds to one decimal place", () => {
    expect(scaleBpmCorrection(123.46, 2)).toBe(246.9);
    expect(scaleBpmCorrection(123.46, 0.5)).toBe(61.7);
  });

  it("accepts inclusive correction boundaries and rejects out-of-range results", () => {
    expect(scaleBpmCorrection(60, 0.5)).toBe(30);
    expect(scaleBpmCorrection(30, 0.5)).toBeNull();
    expect(scaleBpmCorrection(150, 2)).toBe(300);
    expect(scaleBpmCorrection(151, 2)).toBeNull();
  });

  it("rejects missing, non-finite, and out-of-range source tempos", () => {
    expect(scaleBpmCorrection(null, 2)).toBeNull();
    expect(scaleBpmCorrection(Number.NaN, 2)).toBeNull();
    expect(scaleBpmCorrection(29, 2)).toBeNull();
    expect(scaleBpmCorrection(301, 0.5)).toBeNull();
  });
});
