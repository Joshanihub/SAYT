import {
  useCallback,
  useEffect,
  useMemo,
  useReducer,
  useRef,
  useState,
} from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import {
  initialLibraryCollectionSelection,
  reduceLibraryCollectionSelection,
} from "./libraryCollectionSelection";
import { effectiveBpm } from "./libraryTempo";
import { validateLibraryWaveform } from "./libraryWaveform";
import { LibraryWaveformPreview } from "./LibraryWaveformPreview";

type TrackFilter =
  "all" | "recent" | "missing" | "unanalyzed" | "favorites" | "unsupported";

type TrackRecord = {
  id: string;
  title: string;
  artist: string | null;
  album: string | null;
  duration_seconds: number | null;
  format: string;
  bpm: number | null;
  bpm_analysis_candidate: number | null;
  bpm_analysis_confidence: number | null;
  bpm_analysis_status: "estimated" | "uncertain" | null;
  bpm_correction: number | null;
  musical_key: string | null;
  path: string;
  file_size: number;
  import_state: string;
  load_supported: boolean;
  load_status: string;
  missing: boolean;
  favorite: boolean;
  duplicate_locations: number;
  artwork_available: boolean;
};

type TrackPage = {
  items: TrackRecord[];
  total: number;
  offset: number;
  limit: number;
};

type Folder = {
  id: number;
  path: string;
  track_count: number;
  missing_count: number;
};

type Collection = {
  id: string;
  name: string;
  kind: "crate" | "playlist";
  item_count: number;
};

type ScanProgress = {
  state: "scanning" | "complete" | "partial" | "cancelled" | "failed";
  roots_total: number;
  roots_done: number;
  partial_roots: number;
  files_seen: number;
  tracks_indexed: number;
  unsupported: number;
  unreadable: number;
  traversal_errors: number;
  current_file: string | null;
  message: string;
};

type BpmAnalysisProgress = {
  trackId: string;
  state:
    | "preparing"
    | "analyzing"
    | "complete"
    | "uncertain"
    | "cancelled"
    | "failed";
  processedFrames: number;
  totalFrames: number | null;
  candidateBpm: number | null;
  confidence: number | null;
  message: string;
};

type BackgroundProgress<T> = {
  active: boolean;
  progress: T | null;
};

type MusicLibraryProps = {
  loadingDeck: number | null;
  onLoadTrack: (trackId: string, deck: 0 | 1) => Promise<boolean>;
};

const PAGE_SIZE = 100;
const MAX_SEARCH_CHARS = 256;
const BACKGROUND_STATUS_POLL_MS = 250;

function formatDuration(seconds: number | null): string {
  if (seconds === null || !Number.isFinite(seconds) || seconds < 0) return "—";
  const wholeSeconds = Math.floor(seconds);
  const minutes = Math.floor(wholeSeconds / 60);
  const remainder = String(wholeSeconds % 60).padStart(2, "0");
  return `${minutes}:${remainder}`;
}

function formatFileSize(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes < 0) return "—";
  const units = ["B", "KB", "MB", "GB"];
  let size = bytes;
  let unit = 0;
  while (size >= 1024 && unit < units.length - 1) {
    size /= 1024;
    unit += 1;
  }
  return `${size.toFixed(unit === 0 ? 0 : 1)} ${units[unit]}`;
}

function errorMessage(cause: unknown): string {
  return cause instanceof Error ? cause.message : String(cause);
}

export function MusicLibrary({ loadingDeck, onLoadTrack }: MusicLibraryProps) {
  const [tracks, setTracks] = useState<TrackPage>({
    items: [],
    total: 0,
    offset: 0,
    limit: PAGE_SIZE,
  });
  const [folders, setFolders] = useState<Folder[]>([]);
  const [collections, setCollections] = useState<Collection[]>([]);
  const [filter, setFilter] = useState<TrackFilter>("all");
  const [folderId, setFolderId] = useState<number | null>(null);
  const [collectionSelection, dispatchCollectionSelection] = useReducer(
    reduceLibraryCollectionSelection,
    initialLibraryCollectionSelection,
  );
  const { viewCollectionId: collectionId, addTargetCollectionId } =
    collectionSelection;
  const [searchDraft, setSearchDraft] = useState("");
  const [search, setSearch] = useState("");
  const [sort, setSort] = useState("title");
  const [descending, setDescending] = useState(false);
  const [offset, setOffset] = useState(0);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [artworkDataUrl, setArtworkDataUrl] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [scanning, setScanning] = useState(false);
  const [scanProgress, setScanProgress] = useState<ScanProgress | null>(null);
  const [error, setError] = useState("");
  const [statusMessage, setStatusMessage] = useState("");
  const [refreshNumber, setRefreshNumber] = useState(0);
  const [collectionName, setCollectionName] = useState("");
  const [collectionKind, setCollectionKind] = useState<"crate" | "playlist">(
    "crate",
  );
  const [renameDraft, setRenameDraft] = useState("");
  const [editingMetadata, setEditingMetadata] = useState(false);
  const [metadataDraft, setMetadataDraft] = useState({
    title: "",
    artist: "",
    album: "",
  });
  const [actionBusy, setActionBusy] = useState(false);
  const [bpmCorrectionDraft, setBpmCorrectionDraft] = useState("");
  const [bpmCorrectionBusy, setBpmCorrectionBusy] = useState(false);
  const [bpmAnalysisProgress, setBpmAnalysisProgress] =
    useState<BpmAnalysisProgress | null>(null);
  const [trackWaveform, setTrackWaveform] = useState<{
    trackId: string;
    peaks: number[] | null;
    loading: boolean;
  } | null>(null);
  const scanCompletionHandled = useRef(false);
  const bpmCompletionHandled = useRef(false);

  const activeCollection = collections.find(
    (collection) => collection.id === collectionId,
  );
  const addTargetCollection = collections.find(
    (collection) => collection.id === addTargetCollectionId,
  );
  const selectedTrack = tracks.items.find((track) => track.id === selectedId);
  const selectedAnalysisProgress =
    selectedTrack && bpmAnalysisProgress?.trackId === selectedTrack.id
      ? bpmAnalysisProgress
      : null;
  const bpmAnalysisBusy =
    bpmAnalysisProgress?.state === "preparing" ||
    bpmAnalysisProgress?.state === "analyzing";
  const parsedBpmCorrection = Number(bpmCorrectionDraft);
  const bpmCorrectionValid =
    bpmCorrectionDraft.trim().length > 0 &&
    Number.isFinite(parsedBpmCorrection) &&
    parsedBpmCorrection >= 30 &&
    parsedBpmCorrection <= 300;
  const pageCount = Math.max(1, Math.ceil(tracks.total / PAGE_SIZE));
  const currentPage = Math.floor(offset / PAGE_SIZE) + 1;
  const collectionGroups = useMemo(
    () => ({
      crates: collections.filter((collection) => collection.kind === "crate"),
      playlists: collections.filter(
        (collection) => collection.kind === "playlist",
      ),
    }),
    [collections],
  );

  const applyScanProgress = useCallback((progress: ScanProgress) => {
    setScanProgress(progress);
    setScanning(progress.state === "scanning");
    if (progress.state === "complete") {
      if (!scanCompletionHandled.current) {
        scanCompletionHandled.current = true;
        setRefreshNumber((value) => value + 1);
      }
      setStatusMessage(
        `Scanned ${progress.roots_done} folder${progress.roots_done === 1 ? "" : "s"}: ${progress.tracks_indexed} tracks indexed, ${progress.unsupported} not loadable, ${progress.unreadable} unreadable.`,
      );
    } else if (progress.state === "partial") {
      if (!scanCompletionHandled.current) {
        scanCompletionHandled.current = true;
        setRefreshNumber((value) => value + 1);
      }
      setStatusMessage(
        `Partial scan: ${progress.partial_roots} folder${progress.partial_roots === 1 ? " was" : "s were"} not fully readable. Existing missing-file state was preserved. ${progress.tracks_indexed} tracks indexed; ${progress.unreadable} unreadable.`,
      );
    } else if (progress.state === "cancelled") {
      if (!scanCompletionHandled.current) {
        scanCompletionHandled.current = true;
        setRefreshNumber((value) => value + 1);
      }
      setStatusMessage(progress.message);
    } else if (progress.state === "failed") {
      scanCompletionHandled.current = true;
      setError(progress.message);
    }
  }, []);

  const applyBpmProgress = useCallback((progress: BpmAnalysisProgress) => {
    setBpmAnalysisProgress(progress);
    if (progress.state === "complete") {
      if (!bpmCompletionHandled.current) {
        bpmCompletionHandled.current = true;
        setRefreshNumber((value) => value + 1);
      }
      setStatusMessage(
        `BPM estimate saved${progress.candidateBpm === null ? "" : `: ${progress.candidateBpm.toFixed(1)} BPM`}`,
      );
    } else if (progress.state === "uncertain") {
      if (!bpmCompletionHandled.current) {
        bpmCompletionHandled.current = true;
        setRefreshNumber((value) => value + 1);
      }
      setStatusMessage(
        progress.candidateBpm === null
          ? "Analysis could not find a reliable tempo. No BPM was invented."
          : `Tempo is uncertain; candidate ${progress.candidateBpm.toFixed(1)} BPM needs review.`,
      );
    } else if (progress.state === "cancelled") {
      bpmCompletionHandled.current = true;
      setStatusMessage(progress.message);
    } else if (progress.state === "failed") {
      bpmCompletionHandled.current = true;
      setError(progress.message);
    }
  }, []);

  useEffect(() => {
    const debounce = window.setTimeout(() => {
      setSearch(searchDraft.trim());
      setOffset(0);
    }, 160);
    return () => window.clearTimeout(debounce);
  }, [searchDraft]);

  useEffect(() => {
    let active = true;
    setLoading(true);
    setError("");
    const query = {
      search,
      filter,
      collectionId,
      folderId,
      sort,
      descending,
      offset,
      limit: PAGE_SIZE,
    };
    void Promise.all([
      invoke<TrackPage>("library_tracks", { query }),
      invoke<Folder[]>("library_folders"),
      invoke<Collection[]>("library_collections"),
    ])
      .then(([nextTracks, nextFolders, nextCollections]) => {
        if (!active) return;
        setTracks(nextTracks);
        setFolders(nextFolders);
        setCollections(nextCollections);
      })
      .catch((cause: unknown) => {
        if (active) setError(errorMessage(cause));
      })
      .finally(() => {
        if (active) setLoading(false);
      });
    return () => {
      active = false;
    };
  }, [
    collectionId,
    descending,
    filter,
    folderId,
    offset,
    refreshNumber,
    search,
    sort,
  ]);

  useEffect(() => {
    let active = true;
    let unlisten: UnlistenFn | undefined;
    void listen<ScanProgress>("library-scan-progress", (event) => {
      if (active) applyScanProgress(event.payload);
    }).then((unsubscribe) => {
      if (active) unlisten = unsubscribe;
      else unsubscribe();
    });
    return () => {
      active = false;
      unlisten?.();
    };
  }, [applyScanProgress]);

  useEffect(() => {
    if (!scanning) return;
    let active = true;
    let requestInFlight = false;
    const pollScanStatus = async () => {
      if (requestInFlight) return;
      requestInFlight = true;
      try {
        const snapshot = await invoke<BackgroundProgress<ScanProgress>>(
          "library_scan_status",
        );
        if (!active) return;
        if (snapshot.progress) applyScanProgress(snapshot.progress);
        else if (!snapshot.active) {
          scanCompletionHandled.current = true;
          setScanning(false);
          setStatusMessage("Folder scan finished. The local catalog is ready.");
          setRefreshNumber((value) => value + 1);
        }
      } catch (cause) {
        if (active) setError(errorMessage(cause));
      } finally {
        requestInFlight = false;
      }
    };
    void pollScanStatus();
    const timer = window.setInterval(
      () => void pollScanStatus(),
      BACKGROUND_STATUS_POLL_MS,
    );
    return () => {
      active = false;
      window.clearInterval(timer);
    };
  }, [applyScanProgress, scanning]);

  useEffect(() => {
    let active = true;
    let unlisten: UnlistenFn | undefined;
    void listen<BpmAnalysisProgress>(
      "library-bpm-analysis-progress",
      (event) => {
        if (active) applyBpmProgress(event.payload);
      },
    ).then((unsubscribe) => {
      if (active) unlisten = unsubscribe;
      else unsubscribe();
    });
    return () => {
      active = false;
      unlisten?.();
      void invoke<boolean>("library_cancel_bpm_analysis").catch(
        () => undefined,
      );
    };
  }, [applyBpmProgress]);

  useEffect(() => {
    if (!bpmAnalysisBusy) return;
    let active = true;
    let requestInFlight = false;
    const pollBpmAnalysisStatus = async () => {
      if (requestInFlight) return;
      requestInFlight = true;
      try {
        const snapshot = await invoke<BackgroundProgress<BpmAnalysisProgress>>(
          "library_bpm_analysis_status",
        );
        if (!active) return;
        if (snapshot.progress) applyBpmProgress(snapshot.progress);
      } catch (cause) {
        if (active) setError(errorMessage(cause));
      } finally {
        requestInFlight = false;
      }
    };
    void pollBpmAnalysisStatus();
    const timer = window.setInterval(
      () => void pollBpmAnalysisStatus(),
      BACKGROUND_STATUS_POLL_MS,
    );
    return () => {
      active = false;
      window.clearInterval(timer);
    };
  }, [applyBpmProgress, bpmAnalysisBusy]);

  useEffect(() => {
    setArtworkDataUrl(null);
    setEditingMetadata(false);
    if (!selectedTrack) return;
    setMetadataDraft({
      title: selectedTrack.title,
      artist: selectedTrack.artist ?? "",
      album: selectedTrack.album ?? "",
    });
    setBpmCorrectionDraft(selectedTrack.bpm_correction?.toString() ?? "");
    let active = true;
    void invoke<string | null>("library_track_artwork", {
      trackId: selectedTrack.id,
    })
      .then((dataUrl) => {
        if (active) setArtworkDataUrl(dataUrl);
      })
      .catch(() => {
        if (active) setArtworkDataUrl(null);
      });
    return () => {
      active = false;
    };
  }, [selectedTrack?.id]);

  useEffect(() => {
    let active = true;
    if (!selectedTrack) {
      setTrackWaveform(null);
      return;
    }

    setTrackWaveform({
      trackId: selectedTrack.id,
      peaks: null,
      loading: true,
    });
    void invoke<unknown>("library_track_waveform", {
      trackId: selectedTrack.id,
    })
      .then((payload) => {
        if (!active) return;
        setTrackWaveform({
          trackId: selectedTrack.id,
          peaks: validateLibraryWaveform(payload),
          loading: false,
        });
      })
      .catch((cause: unknown) => {
        if (!active) return;
        setError(errorMessage(cause));
        setTrackWaveform({
          trackId: selectedTrack.id,
          peaks: null,
          loading: false,
        });
      });
    return () => {
      active = false;
    };
  }, [refreshNumber, selectedTrack?.id]);

  const refresh = () => setRefreshNumber((value) => value + 1);

  const selectView = (nextFilter: TrackFilter) => {
    setFilter(nextFilter);
    setFolderId(null);
    dispatchCollectionSelection({ type: "browse", collectionId: null });
    setOffset(0);
  };

  const importFolders = async () => {
    setError("");
    setStatusMessage("");
    setScanProgress(null);
    try {
      const result = await invoke<{ started: boolean; message: string }>(
        "library_start_scan",
      );
      scanCompletionHandled.current = !result.started;
      setScanProgress(null);
      setStatusMessage(result.message);
      setScanning(result.started);
    } catch (cause) {
      setError(errorMessage(cause));
    }
  };

  const cancelScan = async () => {
    try {
      const cancelled = await invoke<boolean>("library_cancel_scan");
      if (!cancelled) setStatusMessage("No scan is currently running");
      else setStatusMessage("Cancelling after the current file…");
    } catch (cause) {
      setError(errorMessage(cause));
    }
  };

  const analyzeSelectedBpm = async () => {
    if (!selectedTrack) return;
    setError("");
    setBpmAnalysisProgress({
      trackId: selectedTrack.id,
      state: "preparing",
      processedFrames: 0,
      totalFrames: null,
      candidateBpm: null,
      confidence: null,
      message: "Preparing local analysis",
    });
    bpmCompletionHandled.current = false;
    try {
      const result = await invoke<{ started: boolean; message: string }>(
        "library_start_bpm_analysis",
        { trackId: selectedTrack.id },
      );
      setStatusMessage(result.message);
    } catch (cause) {
      setBpmAnalysisProgress({
        trackId: selectedTrack.id,
        state: "failed",
        processedFrames: 0,
        totalFrames: null,
        candidateBpm: null,
        confidence: null,
        message: errorMessage(cause),
      });
      setError(errorMessage(cause));
    }
  };

  const cancelBpmAnalysis = async () => {
    try {
      const cancelled = await invoke<boolean>("library_cancel_bpm_analysis");
      setStatusMessage(
        cancelled ? "Cancelling BPM analysis…" : "No BPM analysis is running",
      );
    } catch (cause) {
      setError(errorMessage(cause));
    }
  };

  const saveBpmCorrection = async (bpm: number | null) => {
    if (!selectedTrack) return;
    setBpmCorrectionBusy(true);
    setError("");
    try {
      await invoke("library_set_bpm_correction", {
        trackId: selectedTrack.id,
        bpm,
      });
      setStatusMessage(
        bpm === null
          ? "Manual BPM correction cleared"
          : `Manual BPM correction saved at ${bpm.toFixed(1)} BPM`,
      );
      setRefreshNumber((value) => value + 1);
    } catch (cause) {
      setError(errorMessage(cause));
    } finally {
      setBpmCorrectionBusy(false);
    }
  };

  const exportBackup = async () => {
    setError("");
    try {
      const path = await invoke<string | null>("library_export_backup");
      if (path) setStatusMessage(`Library backup saved to ${path}`);
    } catch (cause) {
      setError(errorMessage(cause));
    }
  };

  const createCollection = async () => {
    if (!collectionName.trim()) return;
    setActionBusy(true);
    setError("");
    try {
      const collection = await invoke<Collection>("library_create_collection", {
        name: collectionName,
        kind: collectionKind,
      });
      setCollectionName("");
      dispatchCollectionSelection({
        type: "browse",
        collectionId: collection.id,
      });
      setFolderId(null);
      setFilter("all");
      refresh();
      setStatusMessage(
        `${collection.kind === "crate" ? "Crate" : "Playlist"} created`,
      );
    } catch (cause) {
      setError(errorMessage(cause));
    } finally {
      setActionBusy(false);
    }
  };

  const renameCollection = async () => {
    if (!activeCollection || !renameDraft.trim()) return;
    setActionBusy(true);
    setError("");
    try {
      await invoke("library_rename_collection", {
        collectionId: activeCollection.id,
        name: renameDraft,
      });
      setRenameDraft("");
      refresh();
      setStatusMessage("Collection renamed");
    } catch (cause) {
      setError(errorMessage(cause));
    } finally {
      setActionBusy(false);
    }
  };

  const deleteCollection = async () => {
    if (!activeCollection) return;
    setActionBusy(true);
    setError("");
    try {
      await invoke("library_delete_collection", {
        collectionId: activeCollection.id,
      });
      dispatchCollectionSelection({ type: "browse", collectionId: null });
      setFilter("all");
      refresh();
      setStatusMessage(
        "Collection deleted. Source audio files were not changed.",
      );
    } catch (cause) {
      setError(errorMessage(cause));
    } finally {
      setActionBusy(false);
    }
  };

  const addSelectedToCollection = async () => {
    if (!selectedTrack || !addTargetCollectionId) return;
    setActionBusy(true);
    setError("");
    try {
      await invoke("library_add_track_to_collection", {
        collectionId: addTargetCollectionId,
        trackId: selectedTrack.id,
      });
      refresh();
      setStatusMessage(
        `Added “${selectedTrack.title}” to ${addTargetCollection?.name ?? "collection"}`,
      );
    } catch (cause) {
      setError(errorMessage(cause));
    } finally {
      setActionBusy(false);
    }
  };

  const removeFromCollection = async (trackId: string) => {
    if (!collectionId) return;
    try {
      await invoke("library_remove_track_from_collection", {
        collectionId,
        trackId,
      });
      refresh();
    } catch (cause) {
      setError(errorMessage(cause));
    }
  };

  const movePlaylistTrack = async (trackId: string, delta: -1 | 1) => {
    if (!collectionId) return;
    try {
      await invoke("library_move_playlist_track", {
        collectionId,
        trackId,
        delta,
      });
      refresh();
    } catch (cause) {
      setError(errorMessage(cause));
    }
  };

  const toggleFavorite = async () => {
    if (!selectedTrack) return;
    try {
      await invoke("library_set_favorite", {
        trackId: selectedTrack.id,
        favorite: !selectedTrack.favorite,
      });
      refresh();
    } catch (cause) {
      setError(errorMessage(cause));
    }
  };

  const saveMetadata = async () => {
    if (!selectedTrack) return;
    try {
      await invoke("library_update_track_metadata", {
        trackId: selectedTrack.id,
        title: metadataDraft.title,
        artist: metadataDraft.artist,
        album: metadataDraft.album,
      });
      setEditingMetadata(false);
      refresh();
      setStatusMessage(
        "SAYT-only track details saved. The source file was not changed.",
      );
    } catch (cause) {
      setError(errorMessage(cause));
    }
  };

  const loadSelected = async (deck: 0 | 1) => {
    if (
      !selectedTrack ||
      !selectedTrack.load_supported ||
      selectedTrack.missing
    )
      return;
    await onLoadTrack(selectedTrack.id, deck);
  };

  const changeSort = (field: string) => {
    if (sort === field) setDescending((value) => !value);
    else {
      setSort(field);
      setDescending(false);
    }
    setOffset(0);
  };

  const chooseCollection = (collection: Collection) => {
    dispatchCollectionSelection({
      type: "browse",
      collectionId: collection.id,
    });
    setFolderId(null);
    setFilter("all");
    setOffset(0);
  };

  const renderCollectionList = (items: Collection[]) =>
    items.map((collection) => (
      <button
        className={`library-side-item ${collectionId === collection.id ? "selected" : ""}`}
        key={collection.id}
        type="button"
        aria-current={collectionId === collection.id ? "page" : undefined}
        onClick={() => chooseCollection(collection)}
      >
        <span className="library-side-item-name">{collection.name}</span>
        <span className="library-side-count">{collection.item_count}</span>
      </button>
    ));

  return (
    <section className="music-library" aria-label="Music library manager">
      <header className="library-toolbar">
        <label className="library-search">
          <span className="library-search-glyph" aria-hidden="true">
            ⌕
          </span>
          <span className="sr-only">Search tracks</span>
          <input
            type="search"
            value={searchDraft}
            maxLength={MAX_SEARCH_CHARS}
            onChange={(event) => setSearchDraft(event.target.value)}
            placeholder="Search title, artist, album, or folder…"
          />
          <kbd>Ctrl K</kbd>
        </label>
        <div className="library-toolbar-actions">
          <button
            className="library-button secondary"
            type="button"
            onClick={exportBackup}
          >
            Export backup
          </button>
          <button
            className="library-button primary"
            type="button"
            onClick={importFolders}
            disabled={scanning}
          >
            {scanning ? "Scanning folders…" : "Import folders"}
          </button>
        </div>
      </header>

      {(error || statusMessage) && (
        <div
          className={error ? "library-message error" : "library-message"}
          role={error ? "alert" : "status"}
        >
          <span>{error || statusMessage}</span>
          <button
            type="button"
            aria-label="Dismiss library message"
            onClick={() => {
              setError("");
              setStatusMessage("");
            }}
          >
            ×
          </button>
        </div>
      )}

      {scanning && scanProgress && (
        <div className="library-scan-status" role="status" aria-live="polite">
          <div className="library-scan-copy">
            <strong>Scanning local music</strong>
            <span>
              {scanProgress.files_seen} audio files checked ·{" "}
              {scanProgress.tracks_indexed} indexed · {scanProgress.unsupported}{" "}
              not loadable
            </span>
          </div>
          <progress aria-label="Audio files scanned so far" />
          <button
            className="library-button secondary compact"
            type="button"
            onClick={cancelScan}
          >
            Cancel scan
          </button>
        </div>
      )}

      <div className="library-layout">
        <aside className="library-sidebar" aria-label="Library navigation">
          <section className="library-nav-group">
            <h2>QUICK ACCESS</h2>
            {(
              [
                ["All Tracks", "all"],
                ["Recently Added", "recent"],
                ["Missing Files", "missing"],
                ["Unanalyzed", "unanalyzed"],
                ["Favorites", "favorites"],
                ["Unsupported", "unsupported"],
              ] as const
            ).map(([label, value]) => (
              <button
                className={`library-side-item ${filter === value && !collectionId && folderId === null ? "selected" : ""}`}
                key={value}
                title={
                  value === "unanalyzed"
                    ? "No BPM or key tags are present. Analyze a supported WAV for tempo; key detection is not implemented."
                    : undefined
                }
                type="button"
                aria-current={
                  filter === value && !collectionId && folderId === null
                    ? "page"
                    : undefined
                }
                onClick={() => selectView(value)}
              >
                <span className="library-side-item-name">{label}</span>
                {value === "all" && (
                  <span className="library-side-count">{tracks.total}</span>
                )}
              </button>
            ))}
          </section>

          <section className="library-nav-group">
            <div className="library-sidebar-heading">
              <h2>CRATES</h2>
              <span>{collectionGroups.crates.length}</span>
            </div>
            {renderCollectionList(collectionGroups.crates)}
            {collectionGroups.crates.length === 0 && (
              <p className="library-side-empty">No crates yet</p>
            )}
          </section>

          <section className="library-nav-group">
            <div className="library-sidebar-heading">
              <h2>PLAYLISTS</h2>
              <span>{collectionGroups.playlists.length}</span>
            </div>
            {renderCollectionList(collectionGroups.playlists)}
            {collectionGroups.playlists.length === 0 && (
              <p className="library-side-empty">No playlists yet</p>
            )}
          </section>

          <section className="library-nav-group library-folders">
            <div className="library-sidebar-heading">
              <h2>FOLDERS</h2>
              <button
                type="button"
                aria-label="Add music folder"
                onClick={importFolders}
              >
                +
              </button>
            </div>
            {folders.map((folder) => (
              <button
                className={`library-side-item folder-item ${folderId === folder.id ? "selected" : ""}`}
                key={folder.id}
                type="button"
                title={folder.path}
                aria-current={folderId === folder.id ? "page" : undefined}
                onClick={() => {
                  setFolderId(folder.id);
                  dispatchCollectionSelection({
                    type: "browse",
                    collectionId: null,
                  });
                  setFilter("all");
                  setOffset(0);
                }}
              >
                <span className="folder-dot" aria-hidden="true" />
                <span className="library-side-item-name">
                  {folder.path.split(/[\\/]/).filter(Boolean).at(-1) ??
                    folder.path}
                </span>
                <span className="library-side-count">{folder.track_count}</span>
              </button>
            ))}
            {folders.length === 0 && (
              <p className="library-side-empty">Import a folder to begin</p>
            )}
          </section>

          <section className="library-storage-note">
            <span className="library-storage-led" aria-hidden="true" />
            <div>
              <strong>LOCAL CATALOG</strong>
              <span>Source files stay in place</span>
            </div>
          </section>
        </aside>

        <section className="library-track-browser" aria-label="Track list">
          <header className="library-browser-heading">
            <div>
              <p className="library-eyebrow">
                {activeCollection
                  ? activeCollection.kind.toUpperCase()
                  : folderId !== null
                    ? "FOLDER"
                    : "LOCAL LIBRARY"}
              </p>
              <h2>
                {activeCollection?.name ??
                  (folderId !== null
                    ? folders
                        .find((folder) => folder.id === folderId)
                        ?.path.split(/[\\/]/)
                        .filter(Boolean)
                        .at(-1)
                    : null) ??
                  filterLabel(filter)}
              </h2>
              <span>
                {loading
                  ? "Loading tracks…"
                  : `${tracks.total.toLocaleString()} track${tracks.total === 1 ? "" : "s"}`}
              </span>
            </div>
            {activeCollection && (
              <div className="library-collection-actions">
                <button
                  className="library-button secondary compact"
                  type="button"
                  disabled={actionBusy}
                  onClick={() => setRenameDraft(activeCollection.name)}
                >
                  Rename
                </button>
                <button
                  className="library-button secondary compact danger"
                  type="button"
                  disabled={actionBusy}
                  onClick={() => void deleteCollection()}
                >
                  Delete
                </button>
              </div>
            )}
          </header>

          {renameDraft && activeCollection && (
            <form
              className="library-inline-form"
              onSubmit={(event) => {
                event.preventDefault();
                void renameCollection();
              }}
            >
              <label>
                Rename {activeCollection.kind}
                <input
                  value={renameDraft}
                  maxLength={64}
                  onChange={(event) => setRenameDraft(event.target.value)}
                />
              </label>
              <button
                className="library-button primary compact"
                type="submit"
                disabled={actionBusy}
              >
                Save name
              </button>
              <button
                className="library-button secondary compact"
                type="button"
                onClick={() => setRenameDraft("")}
              >
                Cancel
              </button>
            </form>
          )}

          <form
            className="library-create-row"
            onSubmit={(event) => {
              event.preventDefault();
              void createCollection();
            }}
          >
            <label className="sr-only" htmlFor="library-collection-name">
              New crate or playlist name
            </label>
            <input
              id="library-collection-name"
              value={collectionName}
              maxLength={64}
              onChange={(event) => setCollectionName(event.target.value)}
              placeholder="Name a crate or playlist…"
            />
            <select
              aria-label="New collection type"
              value={collectionKind}
              onChange={(event) =>
                setCollectionKind(event.target.value as "crate" | "playlist")
              }
            >
              <option value="crate">Crate</option>
              <option value="playlist">Playlist</option>
            </select>
            <button
              className="library-button secondary compact"
              type="submit"
              disabled={actionBusy || !collectionName.trim()}
            >
              Create
            </button>
            {selectedTrack && collections.length > 0 && (
              <>
                <select
                  aria-label="Choose collection for selected track"
                  value={addTargetCollectionId ?? ""}
                  onChange={(event) =>
                    dispatchCollectionSelection({
                      type: "choose-add-target",
                      collectionId: event.target.value || null,
                    })
                  }
                >
                  <option value="">Choose collection</option>
                  {collections.map((collection) => (
                    <option value={collection.id} key={collection.id}>
                      {collection.name}
                    </option>
                  ))}
                </select>
                <button
                  className="library-button secondary compact"
                  type="button"
                  disabled={actionBusy || !addTargetCollectionId}
                  onClick={() => void addSelectedToCollection()}
                >
                  Add selected
                </button>
              </>
            )}
          </form>

          <div className="library-table-scroll">
            <table className="library-table">
              <thead>
                <tr>
                  <th className="library-index-column">#</th>
                  <th>
                    <button type="button" onClick={() => changeSort("title")}>
                      TITLE{sort === "title" ? (descending ? " ↓" : " ↑") : ""}
                    </button>
                  </th>
                  <th>
                    <button type="button" onClick={() => changeSort("artist")}>
                      ARTIST
                      {sort === "artist" ? (descending ? " ↓" : " ↑") : ""}
                    </button>
                  </th>
                  <th>
                    <button type="button" onClick={() => changeSort("album")}>
                      ALBUM{sort === "album" ? (descending ? " ↓" : " ↑") : ""}
                    </button>
                  </th>
                  <th>
                    <button
                      type="button"
                      onClick={() => changeSort("duration")}
                    >
                      TIME
                      {sort === "duration" ? (descending ? " ↓" : " ↑") : ""}
                    </button>
                  </th>
                  <th>
                    <button type="button" onClick={() => changeSort("bpm")}>
                      BPM{sort === "bpm" ? (descending ? " ↓" : " ↑") : ""}
                    </button>
                  </th>
                  <th>KEY</th>
                  <th>
                    <button type="button" onClick={() => changeSort("format")}>
                      FORMAT
                      {sort === "format" ? (descending ? " ↓" : " ↑") : ""}
                    </button>
                  </th>
                  {activeCollection?.kind === "playlist" && <th>ORDER</th>}
                  {activeCollection && <th>COLLECTION</th>}
                </tr>
              </thead>
              <tbody>
                {tracks.items.map((track, index) => (
                  <tr
                    className={selectedId === track.id ? "selected" : ""}
                    key={track.id}
                  >
                    <td className="library-index-column">
                      {offset + index + 1}
                    </td>
                    <td className="library-title-cell">
                      <button
                        type="button"
                        className="library-track-select"
                        aria-pressed={selectedId === track.id}
                        onClick={() => setSelectedId(track.id)}
                      >
                        <span
                          className="track-status-led"
                          data-state={
                            track.missing
                              ? "missing"
                              : track.load_supported
                                ? "ready"
                                : "unsupported"
                          }
                          aria-hidden="true"
                        />
                        <span className="track-title-copy">
                          <strong title={track.title}>{track.title}</strong>
                          <small>
                            {track.missing
                              ? "MISSING FILE"
                              : track.load_supported
                                ? "PCM WAV CANDIDATE"
                                : track.import_state === "unreadable"
                                  ? "METADATA UNREADABLE"
                                  : "METADATA ONLY"}
                          </small>
                        </span>
                      </button>
                    </td>
                    <td title={track.artist ?? "No artist tag"}>
                      {track.artist ?? "—"}
                    </td>
                    <td title={track.album ?? "No album tag"}>
                      {track.album ?? "—"}
                    </td>
                    <td className="library-time-cell">
                      {formatDuration(track.duration_seconds)}
                    </td>
                    <td className="library-mono-cell">
                      {effectiveBpm(track)?.toFixed(1) ?? "—"}
                    </td>
                    <td className="library-mono-cell">
                      {track.musical_key ?? "—"}
                    </td>
                    <td>
                      <span className="library-format-pill">
                        {track.format}
                      </span>
                    </td>
                    {activeCollection?.kind === "playlist" && (
                      <td className="playlist-order-cell">
                        <button
                          type="button"
                          aria-label={`Move ${track.title} up`}
                          disabled={index === 0}
                          onClick={() => void movePlaylistTrack(track.id, -1)}
                        >
                          ↑
                        </button>
                        <button
                          type="button"
                          aria-label={`Move ${track.title} down`}
                          disabled={
                            index === tracks.items.length - 1 &&
                            offset + index === tracks.total - 1
                          }
                          onClick={() => void movePlaylistTrack(track.id, 1)}
                        >
                          ↓
                        </button>
                      </td>
                    )}
                    {activeCollection && (
                      <td>
                        <button
                          className="library-remove-item"
                          type="button"
                          onClick={() => void removeFromCollection(track.id)}
                        >
                          Remove
                        </button>
                      </td>
                    )}
                  </tr>
                ))}
              </tbody>
            </table>
            {!loading && tracks.items.length === 0 && (
              <div className="library-empty-state">
                <div className="library-empty-mark" aria-hidden="true">
                  ♫
                </div>
                <h3>
                  {search
                    ? "No matching tracks"
                    : folders.length === 0
                      ? "Your library is empty"
                      : "No tracks in this view"}
                </h3>
                <p>
                  {search
                    ? "Try a shorter search or clear the filter."
                    : folders.length === 0
                      ? "Select a local music folder to build your catalog. SAYT indexes your files in place."
                      : "Choose another library view or import another folder."}
                </p>
                {folders.length === 0 && (
                  <button
                    className="library-button primary"
                    type="button"
                    onClick={importFolders}
                  >
                    Choose music folders
                  </button>
                )}
              </div>
            )}
            {loading && (
              <div className="library-loading-state" role="status">
                Loading your local catalog…
              </div>
            )}
          </div>

          <footer className="library-pagination">
            <span>
              {tracks.total === 0
                ? "0 tracks"
                : `${offset + 1}–${Math.min(offset + tracks.items.length, tracks.total)} of ${tracks.total.toLocaleString()}`}
            </span>
            <div>
              <button
                className="library-button secondary compact"
                type="button"
                disabled={currentPage <= 1 || loading}
                onClick={() => setOffset(Math.max(0, offset - PAGE_SIZE))}
              >
                Previous
              </button>
              <span>
                Page {currentPage} of {pageCount}
              </span>
              <button
                className="library-button secondary compact"
                type="button"
                disabled={currentPage >= pageCount || loading}
                onClick={() => setOffset(offset + PAGE_SIZE)}
              >
                Next
              </button>
            </div>
          </footer>
        </section>

        <aside
          className="library-inspector"
          aria-label="Selected track details"
        >
          {selectedTrack ? (
            <>
              <header className="library-inspector-heading">
                <p className="library-eyebrow">TRACK INSPECTOR</p>
                <button
                  className={`library-favorite ${selectedTrack.favorite ? "active" : ""}`}
                  type="button"
                  aria-pressed={selectedTrack.favorite}
                  onClick={() => void toggleFavorite()}
                >
                  {selectedTrack.favorite ? "★" : "☆"}
                  <span className="sr-only">
                    {selectedTrack.favorite
                      ? "Remove favorite"
                      : "Add favorite"}
                  </span>
                </button>
              </header>
              {artworkDataUrl ? (
                <img
                  className="library-artwork"
                  src={artworkDataUrl}
                  alt={`Embedded artwork for ${selectedTrack.album ?? selectedTrack.title}`}
                />
              ) : (
                <div
                  className="library-artwork-placeholder"
                  aria-label="No supported embedded artwork"
                >
                  <span>
                    {selectedTrack.album?.trim().slice(0, 1).toUpperCase() ||
                      selectedTrack.title.slice(0, 1).toUpperCase() ||
                      "♫"}
                  </span>
                  <small>NO ARTWORK PREVIEW</small>
                </div>
              )}

              <div className="library-inspector-title">
                {editingMetadata ? (
                  <div className="library-metadata-form">
                    <label>
                      Title
                      <input
                        value={metadataDraft.title}
                        maxLength={200}
                        onChange={(event) =>
                          setMetadataDraft({
                            ...metadataDraft,
                            title: event.target.value,
                          })
                        }
                      />
                    </label>
                    <label>
                      Artist
                      <input
                        value={metadataDraft.artist}
                        maxLength={200}
                        onChange={(event) =>
                          setMetadataDraft({
                            ...metadataDraft,
                            artist: event.target.value,
                          })
                        }
                      />
                    </label>
                    <label>
                      Album
                      <input
                        value={metadataDraft.album}
                        maxLength={200}
                        onChange={(event) =>
                          setMetadataDraft({
                            ...metadataDraft,
                            album: event.target.value,
                          })
                        }
                      />
                    </label>
                    <div className="library-inspector-actions">
                      <button
                        className="library-button primary compact"
                        type="button"
                        onClick={() => void saveMetadata()}
                      >
                        Save details
                      </button>
                      <button
                        className="library-button secondary compact"
                        type="button"
                        onClick={() => setEditingMetadata(false)}
                      >
                        Cancel
                      </button>
                    </div>
                  </div>
                ) : (
                  <>
                    <h3 title={selectedTrack.title}>{selectedTrack.title}</h3>
                    <p>{selectedTrack.artist ?? "Unknown artist"}</p>
                    <span>{selectedTrack.album ?? "No album tag"}</span>
                    <button
                      className="library-text-button"
                      type="button"
                      onClick={() => setEditingMetadata(true)}
                    >
                      Edit SAYT details
                    </button>
                  </>
                )}
              </div>

              <dl className="library-metadata-grid">
                <div>
                  <dt>DURATION</dt>
                  <dd>{formatDuration(selectedTrack.duration_seconds)}</dd>
                </div>
                <div>
                  <dt>FORMAT</dt>
                  <dd>{selectedTrack.format}</dd>
                </div>
                <div>
                  <dt>BPM TAG</dt>
                  <dd>{selectedTrack.bpm?.toFixed(1) ?? "Not tagged"}</dd>
                </div>
                <div>
                  <dt>BPM IN USE</dt>
                  <dd>
                    {effectiveBpm(selectedTrack)?.toFixed(1) ?? "Not set"}
                  </dd>
                </div>
                <div>
                  <dt>KEY TAG</dt>
                  <dd>{selectedTrack.musical_key ?? "Not tagged"}</dd>
                </div>
                <div>
                  <dt>FILE SIZE</dt>
                  <dd>{formatFileSize(selectedTrack.file_size)}</dd>
                </div>
                <div>
                  <dt>DUPLICATE COPIES</dt>
                  <dd>
                    {selectedTrack.duplicate_locations > 1
                      ? selectedTrack.duplicate_locations
                      : "None"}
                  </dd>
                </div>
                <div className="metadata-full-width">
                  <dt>FILE LOCATION</dt>
                  <dd title={selectedTrack.path}>{selectedTrack.path}</dd>
                </div>
              </dl>

              <section
                className="library-analysis-panel"
                aria-label="Track BPM analysis"
              >
                <div className="library-analysis-heading">
                  <p className="library-eyebrow">LOCAL BPM ANALYSIS</p>
                  {selectedAnalysisProgress?.state === "preparing" ||
                  selectedAnalysisProgress?.state === "analyzing" ? (
                    <span>Working</span>
                  ) : null}
                </div>
                <p className="library-analysis-result" aria-live="polite">
                  {selectedAnalysisProgress?.state === "preparing" ||
                  selectedAnalysisProgress?.state === "analyzing"
                    ? selectedAnalysisProgress.message
                    : selectedTrack.bpm_analysis_status === "estimated"
                      ? `Estimated ${selectedTrack.bpm_analysis_candidate?.toFixed(1)} BPM · ${(100 * (selectedTrack.bpm_analysis_confidence ?? 0)).toFixed(0)}% periodicity score (heuristic)`
                      : selectedTrack.bpm_analysis_status === "uncertain"
                        ? selectedTrack.bpm_analysis_candidate === null
                          ? "Uncertain · no stable tempo candidate found"
                          : `Uncertain · ${selectedTrack.bpm_analysis_candidate.toFixed(1)} BPM candidate · ${(100 * (selectedTrack.bpm_analysis_confidence ?? 0)).toFixed(0)}% periodicity score (heuristic)`
                        : selectedTrack.bpm
                          ? `File tag: ${selectedTrack.bpm.toFixed(1)} BPM · not analyzed`
                          : "Not analyzed · results come from local integer PCM WAV audio"}
                </p>
                <LibraryWaveformPreview
                  trackTitle={selectedTrack.title}
                  loading={
                    trackWaveform === null ||
                    trackWaveform.trackId !== selectedTrack.id ||
                    trackWaveform.loading
                  }
                  peaks={
                    trackWaveform?.trackId === selectedTrack.id
                      ? trackWaveform.peaks
                      : null
                  }
                />
                {selectedAnalysisProgress?.state === "analyzing" &&
                selectedAnalysisProgress.totalFrames !== null &&
                selectedAnalysisProgress.totalFrames > 0 ? (
                  <progress
                    className="library-analysis-progress"
                    aria-label="BPM analysis progress"
                    max={selectedAnalysisProgress.totalFrames}
                    value={Math.min(
                      selectedAnalysisProgress.processedFrames,
                      selectedAnalysisProgress.totalFrames,
                    )}
                  />
                ) : null}
                <div className="library-analysis-actions">
                  <button
                    className="library-button secondary compact"
                    type="button"
                    disabled={
                      bpmAnalysisBusy ||
                      !selectedTrack.load_supported ||
                      selectedTrack.missing
                    }
                    onClick={() => void analyzeSelectedBpm()}
                  >
                    Analyze local WAV
                  </button>
                  {bpmAnalysisBusy ? (
                    <button
                      className="library-button secondary compact"
                      type="button"
                      onClick={() => void cancelBpmAnalysis()}
                    >
                      Cancel
                    </button>
                  ) : null}
                </div>
                <label className="library-bpm-correction">
                  Manual BPM correction
                  <input
                    type="number"
                    min={30}
                    max={300}
                    step={0.1}
                    value={bpmCorrectionDraft}
                    onChange={(event) =>
                      setBpmCorrectionDraft(event.target.value)
                    }
                    aria-label="Manual BPM correction"
                  />
                </label>
                <div className="library-analysis-actions">
                  <button
                    className="library-button primary compact"
                    type="button"
                    disabled={bpmCorrectionBusy || !bpmCorrectionValid}
                    onClick={() => void saveBpmCorrection(parsedBpmCorrection)}
                  >
                    Save correction
                  </button>
                  <button
                    className="library-button secondary compact"
                    type="button"
                    disabled={
                      bpmCorrectionBusy || selectedTrack.bpm_correction === null
                    }
                    onClick={() => {
                      setBpmCorrectionDraft("");
                      void saveBpmCorrection(null);
                    }}
                  >
                    Clear
                  </button>
                </div>
                <small>
                  Corrections are stored in SAYT and never written into the
                  audio file. Analysis is cached by track identity and algorithm
                  version.
                </small>
              </section>

              <div className="library-load-actions">
                <p className="library-eyebrow">LOAD TO DECK</p>
                <div>
                  <button
                    className="library-deck-load library-deck-a"
                    type="button"
                    disabled={
                      !selectedTrack.load_supported ||
                      selectedTrack.missing ||
                      loadingDeck === 0
                    }
                    onClick={() => void loadSelected(0)}
                  >
                    {loadingDeck === 0 ? "Loading A…" : "Deck A"}
                  </button>
                  <button
                    className="library-deck-load library-deck-b"
                    type="button"
                    disabled={
                      !selectedTrack.load_supported ||
                      selectedTrack.missing ||
                      loadingDeck === 1
                    }
                    onClick={() => void loadSelected(1)}
                  >
                    {loadingDeck === 1 ? "Loading B…" : "Deck B"}
                  </button>
                </div>
                <small
                  className={
                    selectedTrack.load_supported && !selectedTrack.missing
                      ? "load-hint ready"
                      : "load-hint"
                  }
                >
                  {selectedTrack.missing
                    ? "File missing. Re-scan its folder after restoring it."
                    : selectedTrack.load_status}
                </small>
              </div>
              <p className="library-source-note">
                SAYT stores this track’s catalog data locally. Editing the
                details or removing a collection never changes the music file.
              </p>
            </>
          ) : (
            <div className="library-inspector-empty">
              <span aria-hidden="true">♫</span>
              <h3>Select a track</h3>
              <p>Track tags, file details, and deck actions appear here.</p>
            </div>
          )}
        </aside>
      </div>
    </section>
  );
}

function filterLabel(filter: TrackFilter): string {
  switch (filter) {
    case "recent":
      return "Recently Added";
    case "missing":
      return "Missing Files";
    case "unanalyzed":
      return "Unanalyzed";
    case "favorites":
      return "Favorites";
    case "unsupported":
      return "Unsupported Formats";
    default:
      return "All Tracks";
  }
}
