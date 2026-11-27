import { describe, expect, it } from "vitest";
import {
  recordTempoTap,
  TEMPO_TAP_MAX_INTERVAL_MS,
  TEMPO_TAP_MIN_INTERVAL_MS,
} from "./tempoTap";

describe("manual tempo tapping", () => {
  it("waits for three monotonic taps and stages their average tempo", () => {
    const first = recordTempoTap([], 1_000);
    const second = recordTempoTap(first.timestampsMs, 1_500);
    const third = recordTempoTap(second.timestampsMs, 2_000);

    expect(first.bpm).toBeNull();
    expect(second.bpm).toBeNull();
    expect(third.bpm).toBe(120);
  });

  it("uses a bounded rolling interval window for continued taps", () => {
    const taps = [0, 250, 750, 1_250, 1_750, 2_250].reduce<
      ReturnType<typeof recordTempoTap>
    >((result, tappedAtMs) => recordTempoTap(result.timestampsMs, tappedAtMs), {
      timestampsMs: [],
      bpm: null,
      restarted: false,
    });

    expect(taps.timestampsMs).toHaveLength(5);
    expect(taps.bpm).toBe(120);
  });

  it("accepts the 30 to 300 BPM interval boundaries", () => {
    const slow = [
      0,
      TEMPO_TAP_MAX_INTERVAL_MS,
      TEMPO_TAP_MAX_INTERVAL_MS * 2,
    ].reduce<ReturnType<typeof recordTempoTap>>(
      (result, tappedAtMs) => recordTempoTap(result.timestampsMs, tappedAtMs),
      { timestampsMs: [], bpm: null, restarted: false },
    );
    const fast = [
      0,
      TEMPO_TAP_MIN_INTERVAL_MS,
      TEMPO_TAP_MIN_INTERVAL_MS * 2,
    ].reduce<ReturnType<typeof recordTempoTap>>(
      (result, tappedAtMs) => recordTempoTap(result.timestampsMs, tappedAtMs),
      { timestampsMs: [], bpm: null, restarted: false },
    );

    expect(slow.bpm).toBe(30);
    expect(fast.bpm).toBe(300);
  });

  it("restarts on an out-of-range or non-monotonic interval", () => {
    const tooSlow = recordTempoTap(
      [1_000],
      1_000 + TEMPO_TAP_MAX_INTERVAL_MS + 1,
    );
    const tooFast = recordTempoTap(
      [1_000],
      1_000 + TEMPO_TAP_MIN_INTERVAL_MS - 1,
    );
    const backwards = recordTempoTap([1_000], 999);

    expect(tooSlow).toMatchObject({ bpm: null, restarted: true });
    expect(tooFast).toMatchObject({ bpm: null, restarted: true });
    expect(backwards).toMatchObject({ bpm: null, restarted: true });
  });

  it("ignores non-finite clock values without promoting a tempo", () => {
    expect(recordTempoTap([], Number.NaN)).toEqual({
      timestampsMs: [],
      bpm: null,
      restarted: false,
    });
  });
});
