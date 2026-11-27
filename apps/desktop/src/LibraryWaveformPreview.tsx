export type LibraryWaveformPreviewProps = {
  trackTitle: string;
  loading: boolean;
  peaks: number[] | null;
};

export function LibraryWaveformPreview({
  trackTitle,
  loading,
  peaks,
}: LibraryWaveformPreviewProps) {
  return (
    <div className="library-track-waveform-state">
      {loading ? (
        <span role="status">Loading saved waveform…</span>
      ) : peaks ? (
        <div
          className="library-track-waveform"
          role="img"
          aria-label={`Saved waveform for ${trackTitle}`}
        >
          {peaks.map((peak, index) => (
            <span
              key={index}
              aria-hidden="true"
              style={{ height: `${Math.max(3, peak * 100)}%` }}
            />
          ))}
        </div>
      ) : (
        <span>No saved waveform · analyze this PCM WAV to create one</span>
      )}
    </div>
  );
}
