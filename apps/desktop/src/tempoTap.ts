import { BPM_CORRECTION_MAX, BPM_CORRECTION_MIN } from "./libraryTempo";

export const TEMPO_TAP_MIN_INTERVAL_MS = 200;
export const TEMPO_TAP_MAX_INTERVAL_MS = 2_000;
export const TEMPO_TAP_WINDOW_INTERVALS = 4;
export const TEMPO_TAP_MIN_SAMPLES = 3;

export type TempoTapResult = {
  timestampsMs: number[];
  bpm: number | null;
  restarted: boolean;
};

export function recordTempoTap(
  previousTimestampsMs: readonly number[],
  tappedAtMs: number,
): TempoTapResult {
  const previous = previousTimestampsMs.filter(Number.isFinite);
  const retainedPrevious = previous.slice(-(TEMPO_TAP_WINDOW_INTERVALS + 1));

  if (!Number.isFinite(tappedAtMs)) {
    return { timestampsMs: retainedPrevious, bpm: null, restarted: false };
  }

  const lastTapMs = retainedPrevious.at(-1);
  if (lastTapMs === undefined) {
    return {
      timestampsMs: [tappedAtMs],
      bpm: null,
      restarted: false,
    };
  }

  const intervalMs = tappedAtMs - lastTapMs;
  if (
    intervalMs < TEMPO_TAP_MIN_INTERVAL_MS ||
    intervalMs > TEMPO_TAP_MAX_INTERVAL_MS
  ) {
    return { timestampsMs: [tappedAtMs], bpm: null, restarted: true };
  }

  const timestampsMs = [...retainedPrevious, tappedAtMs].slice(
    -(TEMPO_TAP_WINDOW_INTERVALS + 1),
  );
  if (timestampsMs.length < TEMPO_TAP_MIN_SAMPLES) {
    return { timestampsMs, bpm: null, restarted: false };
  }

  const intervals = timestampsMs.slice(1).map((timestamp, index) => {
    return timestamp - timestampsMs[index];
  });
  const meanIntervalMs =
    intervals.reduce((total, interval) => total + interval, 0) /
    intervals.length;
  const candidateBpm = Math.round((60_000 / meanIntervalMs) * 10) / 10;
  const bpm =
    candidateBpm >= BPM_CORRECTION_MIN && candidateBpm <= BPM_CORRECTION_MAX
      ? candidateBpm
      : null;

  return { timestampsMs, bpm, restarted: false };
}
