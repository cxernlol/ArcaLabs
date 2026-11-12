import { useEffect, useState } from "react";
import { ArrowLeft, Zap } from "lucide-react";
import type {
  AudioDefaults,
  GeneralSettings,
  GpuInfo,
  Platform,
  StreamDestination,
  VideoSettings,
} from "../types/stream";
import {
  KEY_MASK,
  PLATFORM_HELPER,
  PLATFORMS,
  platformLabel,
} from "../types/stream";
import {
  clearStreamKey,
  detectGpu,
  getAppConfig,
  getStreamDestination,
  getVideoSettings,
  readCrashLog,
  setAudioDefaults,
  setGeneralSettings,
  setStreamDestination,
  setStreamKey,
  setVideoSettings,
} from "../lib/tauriApi";
import "./SettingsShell.css";

type SettingsPage = "stream" | "video" | "audio" | "hotkeys" | "general";

const NAV: { id: SettingsPage; label: string }[] = [
  { id: "general", label: "General" },
  { id: "stream", label: "Stream" },
  { id: "video", label: "Video" },
  { id: "audio", label: "Audio" },
  { id: "hotkeys", label: "Hotkeys" },
];

const RESOLUTIONS = ["1920x1080", "1280x720", "854x480"];
const FRAME_RATES = [60, 30];
const ENCODERS = ["NVENC", "AMF", "QuickSync", "x264"];

const MIC_OPTIONS = [
  { id: "default-mic", label: "Default microphone" },
  { id: "mic-realtek", label: "Microphone (Realtek Audio)" },
  { id: "mic-usb", label: "USB Microphone" },
];

const DESKTOP_OPTIONS = [
  { id: "default-desktop", label: "Default desktop audio" },
  { id: "speakers", label: "Speakers (Realtek Audio)" },
  { id: "headphones", label: "Headphones" },
];

const HOTKEYS = [
  { id: "go-live", label: "Go Live / Stop", chord: "Ctrl+Shift+G" },
  { id: "mute-mic", label: "Mute mic", chord: "Ctrl+Shift+M" },
  { id: "next-scene", label: "Next scene", chord: "Ctrl+Shift+S" },
];

export function SettingsShell({
  onDone,
  initialPage = "stream",
}: {
  onDone: () => void;
  initialPage?: SettingsPage;
}) {
  const [page, setPage] = useState<SettingsPage>(initialPage);
  const [video, setVideo] = useState<VideoSettings | null>(null);
  const [destination, setDestination] = useState<StreamDestination | null>(
    null,
  );
  const [audio, setAudio] = useState<AudioDefaults | null>(null);
  const [general, setGeneral] = useState<GeneralSettings | null>(null);
  const [gpu, setGpu] = useState<GpuInfo | null>(null);
  const [keyDraft, setKeyDraft] = useState("");
  const [keyEditing, setKeyEditing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [copyNote, setCopyNote] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const [v, d, g, config] = await Promise.all([
          getVideoSettings(),
          getStreamDestination(),
          detectGpu().catch(() => null),
          getAppConfig(),
        ]);
        if (cancelled) return;
        setDestination(d);
        setGpu(g);
        setAudio(config.audio);
        setGeneral(config.general);
        if (v.automatic && g) {
          const next = { ...v, encoder: g.suggestedEncoder };
          const saved = await setVideoSettings(next);
          if (!cancelled) setVideo(saved);
        } else if (!cancelled) {
          setVideo(v);
        }
      } catch (e) {
        if (!cancelled) {
          setError(e instanceof Error ? e.message : String(e));
        }
      }
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      if (e.key === "Escape") {
        e.stopPropagation();
        onDone();
      }
    }
    window.addEventListener("keydown", onKey, true);
    return () => window.removeEventListener("keydown", onKey, true);
  }, [onDone]);

  async function persistVideo(next: VideoSettings) {
    setBusy(true);
    setError(null);
    try {
      const saved = await setVideoSettings(next);
      setVideo(saved);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function persistDestination(platform: Platform, ingestUrl: string) {
    setBusy(true);
    setError(null);
    try {
      const saved = await setStreamDestination(platform, ingestUrl);
      setDestination(saved);
      setKeyDraft("");
      setKeyEditing(false);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function persistAudio(next: AudioDefaults) {
    setBusy(true);
    setError(null);
    try {
      const saved = await setAudioDefaults(next);
      setAudio(saved);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function persistGeneral(next: GeneralSettings) {
    setBusy(true);
    setError(null);
    try {
      const saved = await setGeneralSettings(next);
      setGeneral(saved);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function saveKey() {
    if (!destination || !keyDraft.trim()) return;
    setBusy(true);
    setError(null);
    try {
      await setStreamKey(destination.platform, keyDraft);
      setDestination({ ...destination, hasStreamKey: true });
      setKeyDraft("");
      setKeyEditing(false);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function onClearKey() {
    if (!destination) return;
    setBusy(true);
    setError(null);
    try {
      await clearStreamKey(destination.platform);
      setDestination({ ...destination, hasStreamKey: false });
      setKeyDraft("");
      setKeyEditing(false);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function onCopyCrashReport() {
    setCopyNote(null);
    setError(null);
    try {
      const text = await readCrashLog();
      if (!text.trim()) {
        setCopyNote("No crash log yet.");
        return;
      }
      await navigator.clipboard.writeText(text);
      setCopyNote("Crash report copied. Nothing was uploaded.");
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }

  const loaded = video && destination && audio && general;

  return (
    <div
      className="settings-overlay"
      role="presentation"
      onMouseDown={(e) => {
        if (e.target === e.currentTarget) onDone();
      }}
    >
      <div
        className="settings-modal"
        role="dialog"
        aria-modal="true"
        aria-labelledby="settings-title"
      >
        <header className="settings-modal-header">
          <button
            type="button"
            className="settings-back"
            aria-label="Back to main window"
            title="Back"
            onClick={onDone}
          >
            <ArrowLeft size={15} strokeWidth={1.5} />
            Back
          </button>
          <h1 id="settings-title" className="settings-modal-title">
            Settings
          </h1>
        </header>

        <div className="settings-body">
          <nav className="settings-nav" aria-label="Settings">
            <div className="settings-nav-list">
              {NAV.map((item) => (
                <button
                  key={item.id}
                  type="button"
                  className={`settings-nav-item${page === item.id ? " selected" : ""}`}
                  onClick={() => setPage(item.id)}
                >
                  {item.label}
                </button>
              ))}
            </div>
            <button
              type="button"
              className="settings-optimize"
              disabled
              title="Not in v0.1 — runs locally later"
              aria-label="Optimize for this PC (not available in v0.1)"
            >
              <Zap size={15} strokeWidth={1.5} />
              Optimize
            </button>
          </nav>

          <section className="settings-content">
            {error ? <p className="form-error" role="alert">{error}</p> : null}

            {!loaded ? (
              <p className="settings-loading">Loading settings…</p>
            ) : null}

            {loaded && page === "stream" && destination ? (
              <StreamPage
                destination={destination}
                keyDraft={keyDraft}
                keyEditing={keyEditing}
                busy={busy}
                onPlatform={(platform) => {
                  void (async () => {
                    try {
                      const config = await getAppConfig();
                      await persistDestination(
                        platform,
                        config.ingestUrls[platform] ?? "",
                      );
                    } catch (e) {
                      setError(e instanceof Error ? e.message : String(e));
                    }
                  })();
                }}
                onIngestBlur={(ingestUrl) => {
                  void persistDestination(destination.platform, ingestUrl);
                }}
                onKeyDraft={setKeyDraft}
                onKeyEditing={setKeyEditing}
                onSaveKey={() => void saveKey()}
                onClearKey={() => void onClearKey()}
              />
            ) : null}

            {loaded && page === "video" && video ? (
              <VideoPage
                video={video}
                gpu={gpu}
                busy={busy}
                onChange={(next) => {
                  setVideo(next);
                  void persistVideo(next);
                }}
              />
            ) : null}

            {loaded && page === "audio" && audio ? (
              <AudioPage
                audio={audio}
                busy={busy}
                onChange={(next) => {
                  setAudio(next);
                  void persistAudio(next);
                }}
              />
            ) : null}

            {loaded && page === "hotkeys" ? <HotkeysPage /> : null}

            {loaded && page === "general" && general ? (
              <GeneralPage
                general={general}
                busy={busy}
                copyNote={copyNote}
                onChange={(next) => {
                  setGeneral(next);
                  void persistGeneral(next);
                }}
                onCopyCrash={() => void onCopyCrashReport()}
              />
            ) : null}
          </section>
        </div>
      </div>
    </div>
  );
}

function VideoPage({
  video,
  gpu,
  busy,
  onChange,
}: {
  video: VideoSettings;
  gpu: GpuInfo | null;
  busy: boolean;
  onChange: (next: VideoSettings) => void;
}) {
  const detectionLine = gpu
    ? `Detected ${gpu.name} → ${gpu.suggestedEncoder}. Use Optimize to apply.`
    : "GPU detection stays on this PC. Use Optimize to apply later.";

  return (
    <div className="settings-page">
      <h2 className="settings-page-title">Video</h2>
      <p className="settings-lede">{detectionLine}</p>

      <div className="optimization-note">
        <div>
          <strong>Optimize for this PC</strong>
          <p>One-click optimization is not available in this prototype.</p>
        </div>
        <button type="button" className="btn-quiet" disabled>
          Optimize
        </button>
      </div>

      <div className="manual-rows">
        <label className="manual-row">
          <span>Encoder</span>
          <select
            value={video.encoder}
            disabled={busy}
            onChange={(e) =>
              onChange({ ...video, automatic: false, encoder: e.target.value })
            }
          >
            {ENCODERS.map((enc) => (
              <option key={enc} value={enc}>
                {enc}
              </option>
            ))}
          </select>
        </label>
        <label className="manual-row">
          <span>Resolution</span>
          <select
            value={video.resolution}
            disabled={busy}
            onChange={(e) =>
              onChange({
                ...video,
                automatic: false,
                resolution: e.target.value,
              })
            }
          >
            {RESOLUTIONS.map((res) => (
              <option key={res} value={res}>
                {res.replace("x", "×")}
              </option>
            ))}
          </select>
        </label>
        <label className="manual-row">
          <span>Frame rate</span>
          <select
            value={video.frameRate}
            disabled={busy}
            onChange={(e) =>
              onChange({
                ...video,
                automatic: false,
                frameRate: Number(e.target.value),
              })
            }
          >
            {FRAME_RATES.map((fps) => (
              <option key={fps} value={fps}>
                {fps} fps
              </option>
            ))}
          </select>
        </label>
        <label className="manual-row">
          <span>Bitrate</span>
          <select
            value={video.bitrateKbps}
            disabled={busy}
            onChange={(e) =>
              onChange({
                ...video,
                automatic: false,
                bitrateKbps: Number(e.target.value),
              })
            }
          >
            {[2500, 3500, 4500, 6000, 8000, 10000, 12000].map((kbps) => (
              <option key={kbps} value={kbps}>
                {kbps.toLocaleString("en-US")} kbps
              </option>
            ))}
          </select>
        </label>
      </div>
    </div>
  );
}

function StreamPage({
  destination,
  keyDraft,
  keyEditing,
  busy,
  onPlatform,
  onIngestBlur,
  onKeyDraft,
  onKeyEditing,
  onSaveKey,
  onClearKey,
}: {
  destination: StreamDestination;
  keyDraft: string;
  keyEditing: boolean;
  busy: boolean;
  onPlatform: (platform: Platform) => void;
  onIngestBlur: (ingestUrl: string) => void;
  onKeyDraft: (value: string) => void;
  onKeyEditing: (value: boolean) => void;
  onSaveKey: () => void;
  onClearKey: () => void;
}) {
  const [ingestDraft, setIngestDraft] = useState(destination.ingestUrl);

  useEffect(() => {
    setIngestDraft(destination.ingestUrl);
  }, [destination.platform, destination.ingestUrl]);

  const showMask = destination.hasStreamKey && !keyEditing;

  return (
    <div className="settings-page">
      <h2 className="settings-page-title">Stream</h2>
      <p className="settings-lede">
        Connect with an RTMP ingest URL and stream key only. ArcaLabs never logs
        in to {platformLabel(destination.platform)}.
      </p>

      <div className="segmented" role="tablist" aria-label="Platform">
        {PLATFORMS.map((opt) => (
          <button
            key={opt.id}
            type="button"
            role="tab"
            aria-selected={destination.platform === opt.id}
            className={`segmented-item${destination.platform === opt.id ? " active" : ""}`}
            disabled={busy}
            onClick={() => onPlatform(opt.id)}
          >
            {opt.label}
          </button>
        ))}
      </div>

      <label className="field">
        <span className="field-label">Ingest URL</span>
        <input
          className="text-input"
          value={ingestDraft}
          disabled={busy}
          placeholder="rtmp://…"
          onChange={(e) => setIngestDraft(e.target.value)}
          onBlur={() => onIngestBlur(ingestDraft)}
        />
      </label>

      <label className="field">
        <span className="field-label">Stream key</span>
        <input
          className="text-input"
          type="password"
          autoComplete="off"
          spellCheck={false}
          disabled={busy}
          placeholder={showMask ? KEY_MASK : "Paste stream key"}
          value={showMask ? KEY_MASK : keyDraft}
          onFocus={() => {
            if (showMask) {
              onKeyEditing(true);
              onKeyDraft("");
            }
          }}
          onChange={(e) => {
            onKeyEditing(true);
            onKeyDraft(e.target.value);
          }}
          onBlur={() => {
            if (keyDraft.trim()) {
              onSaveKey();
            } else if (!destination.hasStreamKey) {
              onKeyEditing(false);
            } else if (!keyDraft.trim() && keyEditing) {
              onKeyEditing(false);
            }
          }}
        />
        <span className="field-help">{PLATFORM_HELPER[destination.platform]}</span>
      </label>

      <div className="field-actions">
        {destination.hasStreamKey ? (
          <button
            type="button"
            className="btn-text"
            disabled={busy}
            onClick={onClearKey}
          >
            Clear saved key
          </button>
        ) : (
          <span className="field-meta">
            Key is stored in the OS keychain only.
          </span>
        )}
      </div>
    </div>
  );
}

function AudioPage({
  audio,
  busy,
  onChange,
}: {
  audio: AudioDefaults;
  busy: boolean;
  onChange: (next: AudioDefaults) => void;
}) {
  return (
    <div className="settings-page">
      <h2 className="settings-page-title">Audio</h2>
      <p className="settings-lede">
        Default devices for new mixer tracks. Lists are local stubs until capture.
      </p>
      <div className="manual-rows">
        <label className="manual-row">
          <span>Microphone</span>
          <select
            value={audio.defaultMicId}
            disabled={busy}
            onChange={(e) =>
              onChange({ ...audio, defaultMicId: e.target.value })
            }
          >
            {MIC_OPTIONS.map((opt) => (
              <option key={opt.id} value={opt.id}>
                {opt.label}
              </option>
            ))}
          </select>
        </label>
        <label className="manual-row">
          <span>Desktop</span>
          <select
            value={audio.defaultDesktopId}
            disabled={busy}
            onChange={(e) =>
              onChange({ ...audio, defaultDesktopId: e.target.value })
            }
          >
            {DESKTOP_OPTIONS.map((opt) => (
              <option key={opt.id} value={opt.id}>
                {opt.label}
              </option>
            ))}
          </select>
        </label>
      </div>
    </div>
  );
}

function HotkeysPage() {
  return (
    <div className="settings-page">
      <h2 className="settings-page-title">Hotkeys</h2>
      <p className="settings-lede">
        Example bindings only. No shortcuts are registered in this prototype.
      </p>
      <div className="manual-rows">
        {HOTKEYS.map((row) => (
          <div key={row.id} className="manual-row hotkey-row">
            <span>{row.label}</span>
            <kbd className="hotkey-chord">{row.chord}</kbd>
          </div>
        ))}
      </div>
    </div>
  );
}

function GeneralPage({
  general,
  busy,
  copyNote,
  onChange,
  onCopyCrash,
}: {
  general: GeneralSettings;
  busy: boolean;
  copyNote: string | null;
  onChange: (next: GeneralSettings) => void;
  onCopyCrash: () => void;
}) {
  return (
    <div className="settings-page">
      <h2 className="settings-page-title">General</h2>
      <p className="settings-lede">
        Preferences stay on this PC. Crash logs never upload automatically.
      </p>

      <div className="manual-rows">
        <div className="manual-row">
          <span>Start minimized</span>
          <button
            type="button"
            className={`toggle${general.startMinimized ? " on" : ""}`}
            role="switch"
            aria-checked={general.startMinimized}
            disabled={busy}
            onClick={() =>
              onChange({
                ...general,
                startMinimized: !general.startMinimized,
              })
            }
          >
            <span className="toggle-knob" />
          </button>
        </div>
        <div className="manual-row">
          <span>Confirm before Stop</span>
          <button
            type="button"
            className={`toggle${general.confirmBeforeStop ? " on" : ""}`}
            role="switch"
            aria-checked={general.confirmBeforeStop}
            disabled={busy}
            onClick={() =>
              onChange({
                ...general,
                confirmBeforeStop: !general.confirmBeforeStop,
              })
            }
          >
            <span className="toggle-knob" />
          </button>
        </div>
      </div>

      <div className="crash-block">
        <p className="field-help">
          Scrubbed crash log in your app data folder (no stream key, no home
          paths). You choose if and where to share it.
        </p>
        <button
          type="button"
          className="btn-quiet"
          disabled={busy}
          onClick={onCopyCrash}
        >
          Copy crash report
        </button>
        {copyNote ? <p className="field-meta">{copyNote}</p> : null}
      </div>
    </div>
  );
}
