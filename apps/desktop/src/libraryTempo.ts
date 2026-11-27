export type TrackBpmValues = {
  bpm: number | null;
  bpm_analysis_candidate: number | null;
  bpm_analysis_status: "estimated" | "uncertain" | null;
  bpm_correction: number | null;
};

export const BPM_CORRECTION_MIN = 30;
export const BPM_CORRECTION_MAX = 300;

export function scaleBpmCorrection(
  currentBpm: number | null,
  factor: 0.5 | 2,
): number | null {
  if (
    currentBpm === null ||
    !Number.isFinite(currentBpm) ||
    currentBpm < BPM_CORRECTION_MIN ||
    currentBpm > BPM_CORRECTION_MAX
  ) {
    return null;
  }

  const candidate = Math.round(currentBpm * factor * 10) / 10;
  return candidate >= BPM_CORRECTION_MIN && candidate <= BPM_CORRECTION_MAX
    ? candidate
    : null;
}

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
