export type TrackBpmValues = {
  bpm: number | null;
  bpm_analysis_candidate: number | null;
  bpm_analysis_status: "estimated" | "uncertain" | null;
  bpm_correction: number | null;
};

export function effectiveBpm(track: TrackBpmValues): number | null {
  if (track.bpm_correction !== null) return track.bpm_correction;
  if (
    track.bpm_analysis_status === "estimated" &&
    track.bpm_analysis_candidate !== null
  ) {
    return track.bpm_analysis_candidate;
  }
  return track.bpm;
}
