export const LIBRARY_WAVEFORM_PEAK_COUNT = 160;

export function validateLibraryWaveform(payload: unknown): number[] | null {
  if (payload === null) return null;
  if (
    !Array.isArray(payload) ||
    payload.length !== LIBRARY_WAVEFORM_PEAK_COUNT ||
    payload.some(
      (peak) =>
        typeof peak !== "number" ||
        !Number.isFinite(peak) ||
        peak < 0 ||
        peak > 1,
    )
  ) {
    throw new Error("Saved waveform data is invalid. Analyze the track again.");
  }
  return payload;
}
