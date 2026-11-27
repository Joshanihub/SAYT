import { describe, expect, it } from "vitest";
import { effectiveBpm } from "./libraryTempo";

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
