import { useCallback, useEffect, useState, type ReactNode } from "react";
import { AnimatePresence, motion, useReducedMotion } from "framer-motion";
import {
  AppWindow,
  ChevronDown,
  Eye,
  Mic,
  MicOff,
  Plus,
  Settings,
  Settings2,
  Trash2,
  Volume2,
  VolumeX,
} from "lucide-react";
import { Logo } from "./Logo";
import { WindowControls } from "./WindowControls";
import { SettingsShell } from "./SettingsShell";
import { formatElapsed, formatKbps } from "../types/stream";
import { useStreamStats } from "../hooks/useStreamStats";
import {
  detectGpu,
  getVideoSettings,
  setVideoSettings,
} from "../lib/tauriApi";
import "./MainWindow.css";

const SCENES = ["Main", "Just chatting", "Be right back"];
const SOURCES = [
  { id: "screen", label: "Screen 1", visible: true },
  { id: "webcam", label: "Webcam", visible: true },
];

type AudioKind = "mic" | "desktop" | "application";

type AudioDeviceOption = { id: string; label: string };

/** shortcut: stub device/app lists until WASAPI enum lands */
const MIC_DEVICES: AudioDeviceOption[] = [
  { id: "default-mic", label: "Default microphone" },
  { id: "mic-realtek", label: "Microphone (Realtek Audio)" },
  { id: "mic-usb", label: "USB Microphone" },
];

const DESKTOP_DEVICES: AudioDeviceOption[] = [
  { id: "default-desktop", label: "Default desktop audio" },
  { id: "speakers", label: "Speakers (Realtek Audio)" },
  { id: "headphones", label: "Headphones" },
];

const APP_DEVICES: AudioDeviceOption[] = [
  { id: "app-browser", label: "Browser" },
  { id: "app-discord", label: "Discord" },
  { id: "app-spotify", label: "Spotify" },
  { id: "app-game", label: "Game" },
];

type AudioTrack = {
  id: string;
  kind: AudioKind;
  volume: number;
  muted: boolean;
  level: number;
  deviceId: string;
};

function devicesFor(kind: AudioKind): AudioDeviceOption[] {
  if (kind === "mic") return MIC_DEVICES;
  if (kind === "desktop") return DESKTOP_DEVICES;
  return APP_DEVICES;
}

function trackTitle(track: AudioTrack): string {
  const device = devicesFor(track.kind).find((d) => d.id === track.deviceId);
  if (track.kind === "mic") return device?.label ?? "Mic/Aux";
  if (track.kind === "desktop") return device?.label ?? "Desktop Audio";
  return device?.label ?? "Application";
}

function kindLabel(kind: AudioKind): string {
  if (kind === "mic") return "Mic/Aux";
  if (kind === "desktop") return "Desktop";
  return "Application";
}

const DEFAULT_AUDIO: AudioTrack[] = [
  {
    id: "mic",
    kind: "mic",
    volume: 0.75,
    muted: false,
    level: 0.35,
    deviceId: "default-mic",
  },
  {
    id: "desktop",
    kind: "desktop",
    volume: 0.6,
    muted: false,
    level: 0.12,
    deviceId: "default-desktop",
  },
];

export function MainWindow() {
  const stats = useStreamStats();
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [scenesOpen, setScenesOpen] = useState(false);
  const [sourcesOpen, setSourcesOpen] = useState(false);
  const [audioOpen, setAudioOpen] = useState(false);
  const [activeScene, setActiveScene] = useState("Main");
  const [activeSource, setActiveSource] = useState(SOURCES[0].id);
  const [audioTracks, setAudioTracks] = useState<AudioTrack[]>(DEFAULT_AUDIO);
  const [audioSettingsId, setAudioSettingsId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  function addAudio(kind: AudioKind) {
    const devices = devicesFor(kind);
    const deviceId = devices[0]?.id ?? "default";
    setAudioTracks((list) => [
      ...list,
      {
        id: `${kind}-${Date.now()}`,
        kind,
        volume: kind === "desktop" ? 0.7 : 0.8,
        muted: false,
        level: kind === "application" ? 0.22 : 0.25,
        deviceId,
      },
    ]);
  }

  const closePops = useCallback(() => {
    setScenesOpen(false);
    setSourcesOpen(false);
    setAudioOpen(false);
  }, []);

  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      if (e.key === "Escape") closePops();
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [closePops]);

  const refresh = useCallback(async () => {
    const [vid, gpu] = await Promise.all([
      getVideoSettings(),
      detectGpu().catch(() => null),
    ]);
    if (vid.automatic && gpu && vid.encoder !== gpu.suggestedEncoder) {
      await setVideoSettings({
        ...vid,
        encoder: gpu.suggestedEncoder,
      });
    }
  }, []);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        await refresh();
      } catch (e) {
        if (!cancelled) {
          setError(e instanceof Error ? e.message : String(e));
        }
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [refresh]);

  function closeSettings() {
    setSettingsOpen(false);
    void refresh().catch((e) =>
      setError(e instanceof Error ? e.message : String(e)),
    );
  }

  const activeSourceLabel =
    SOURCES.find((s) => s.id === activeSource)?.label ?? "Source";

  return (
    <div className="app-shell">
      <header className="app-header">
        <div className="app-header-start" data-tauri-drag-region>
          <span className="wordmark" data-tauri-drag-region>
            <span className="wordmark-icon">
              <Logo />
            </span>
            Arcalabs
          </span>
        </div>
        <div className="app-header-status">
          <span className="status-dot status-idle" aria-hidden />
          <span>Offline</span>
          <button
            type="button"
            className="icon-button header-settings"
            aria-label="Settings"
            title="Settings"
            onClick={() => setSettingsOpen(true)}
          >
            <Settings size={16} strokeWidth={1.5} />
          </button>
        </div>
        <div className="app-header-end">
          <WindowControls />
        </div>
      </header>

      <main className="app-main">
        <section className="preview-wrap">
          <div className="preview-letterbox">
            <div className="preview-canvas">
              <span className="preview-label">
                {activeSourceLabel}
                <span aria-hidden> · </span>
                1920×1080
              </span>
            </div>
          </div>
        </section>

        <section className="bottom-row">
          <div className="control-boxes">
            <PopCard
              title="Scenes"
              open={scenesOpen}
              onToggle={() => {
                const next = !scenesOpen;
                closePops();
                setScenesOpen(next);
              }}
            >
              <p className="pop-card-current">{activeScene}</p>
              <ul className="pop-card-list">
                {SCENES.map((scene) => (
                  <li key={scene}>
                    <button
                      type="button"
                      className={`pop-card-item${activeScene === scene ? " selected" : ""}`}
                      onClick={() => {
                        setActiveScene(scene);
                        setScenesOpen(false);
                      }}
                    >
                      {scene}
                    </button>
                  </li>
                ))}
              </ul>
            </PopCard>

            <PopCard
              title="Sources"
              open={sourcesOpen}
              onToggle={() => {
                const next = !sourcesOpen;
                closePops();
                setSourcesOpen(next);
              }}
            >
              <p className="pop-card-current">{activeSourceLabel}</p>
              <ul className="pop-card-list">
                {SOURCES.map((src) => (
                  <li key={src.id}>
                    <button
                      type="button"
                      className={`pop-card-item source-item${activeSource === src.id ? " selected" : ""}`}
                      onClick={() => {
                        setActiveSource(src.id);
                        setSourcesOpen(false);
                      }}
                    >
                      <span>{src.label}</span>
                      <Eye size={16} strokeWidth={1.5} />
                    </button>
                  </li>
                ))}
                <li>
                  <button type="button" className="pop-card-item muted">
                    <span>Add source</span>
                    <Plus size={16} strokeWidth={1.5} />
                  </button>
                </li>
              </ul>
            </PopCard>

            <PopCard
              title="Audio"
              open={audioOpen}
              onToggle={() => {
                const next = !audioOpen;
                closePops();
                setAudioOpen(next);
              }}
            >
              <div className="audio-pop-body">
                {audioTracks.length === 0 ? (
                  <p className="audio-empty">No audio sources</p>
                ) : (
                  audioTracks.map((track) => (
                    <AudioChannel
                      key={track.id}
                      track={track}
                      settingsOpen={audioSettingsId === track.id}
                      onToggleSettings={() =>
                        setAudioSettingsId((id) =>
                          id === track.id ? null : track.id,
                        )
                      }
                      onMute={() =>
                        setAudioTracks((list) =>
                          list.map((t) =>
                            t.id === track.id ? { ...t, muted: !t.muted } : t,
                          ),
                        )
                      }
                      onVolume={(volume) =>
                        setAudioTracks((list) =>
                          list.map((t) =>
                            t.id === track.id ? { ...t, volume } : t,
                          ),
                        )
                      }
                      onDevice={(deviceId) =>
                        setAudioTracks((list) =>
                          list.map((t) =>
                            t.id === track.id ? { ...t, deviceId } : t,
                          ),
                        )
                      }
                      onRemove={() => {
                        setAudioTracks((list) =>
                          list.filter((t) => t.id !== track.id),
                        );
                        setAudioSettingsId((id) =>
                          id === track.id ? null : id,
                        );
                      }}
                    />
                  ))
                )}
                <div className="audio-add-row three">
                  <button
                    type="button"
                    className="audio-add"
                    onClick={() => addAudio("mic")}
                  >
                    <Plus size={14} strokeWidth={1.5} />
                    Mic/Aux
                  </button>
                  <button
                    type="button"
                    className="audio-add"
                    onClick={() => addAudio("desktop")}
                  >
                    <Plus size={14} strokeWidth={1.5} />
                    Desktop
                  </button>
                  <button
                    type="button"
                    className="audio-add"
                    onClick={() => addAudio("application")}
                  >
                    <AppWindow size={14} strokeWidth={1.5} />
                    App
                  </button>
                </div>
              </div>
            </PopCard>
          </div>

          <div className="go-live-slot">
            <button
              type="button"
              className="btn-go-live"
              disabled
              title="Streaming is not available in v0.1"
              aria-label="Go Live (unavailable in v0.1)"
            >
              Go Live
            </button>
            {error ? (
              <p className="form-error go-live-error" role="alert">
                {error}
              </p>
            ) : null}
          </div>
        </section>
      </main>

      <footer className="status-bar">
        <span className="status-bar-left">Not streaming</span>
        <span className="status-bar-right tabular">
          <span>CPU {stats.cpuPercent}%</span>
          <span>{stats.fps} fps</span>
          <span>{formatKbps(stats.kbps)} kbps</span>
          <span>{stats.dropped} drop</span>
          <span>{formatElapsed(stats.elapsedMs)}</span>
        </span>
      </footer>

      {settingsOpen ? (
        <SettingsShell initialPage="stream" onDone={closeSettings} />
      ) : null}
    </div>
  );
}

function PopCard({
  title,
  open,
  onToggle,
  children,
}: {
  title: string;
  open: boolean;
  onToggle: () => void;
  children: ReactNode;
}) {
  const reduceMotion = useReducedMotion();
  const spring = reduceMotion
    ? { duration: 0 }
    : { type: "spring" as const, stiffness: 380, damping: 32 };

  return (
    <div className={`pop-card-slot${open ? " open" : ""}`}>
      <motion.div className={`pop-card${open ? " open" : ""}`} transition={spring}>
        <button
          type="button"
          className="pop-card-bar"
          aria-expanded={open}
          onClick={onToggle}
        >
          <span className="pop-card-title">{title}</span>
          <motion.span
            className="pop-card-chevron"
            animate={{ rotate: open ? 180 : 0 }}
            transition={spring}
          >
            <ChevronDown size={18} strokeWidth={1.5} />
          </motion.span>
        </button>

        <AnimatePresence initial={false}>
          {open ? (
            <motion.div
              key="body"
              className="pop-card-panel"
              initial={{ height: 0, opacity: 0 }}
              animate={{ height: "auto", opacity: 1 }}
              exit={{ height: 0, opacity: 0 }}
              transition={spring}
            >
              <div className="pop-card-panel-inner">{children}</div>
            </motion.div>
          ) : null}
        </AnimatePresence>
      </motion.div>
    </div>
  );
}

function AudioChannel({
  track,
  settingsOpen,
  onToggleSettings,
  onMute,
  onVolume,
  onDevice,
  onRemove,
}: {
  track: AudioTrack;
  settingsOpen: boolean;
  onToggleSettings: () => void;
  onMute: () => void;
  onVolume: (value: number) => void;
  onDevice: (deviceId: string) => void;
  onRemove: () => void;
}) {
  const title = trackTitle(track);
  const shownLevel = track.muted ? 0 : track.level * track.volume;
  const devices = devicesFor(track.kind);
  const MuteIcon =
    track.kind === "application"
      ? track.muted
        ? VolumeX
        : AppWindow
      : track.kind === "desktop"
        ? track.muted
          ? VolumeX
          : Volume2
        : track.muted
          ? MicOff
          : Mic;

  return (
    <div className={`audio-channel${track.muted ? " is-muted" : ""}`}>
      <button
        type="button"
        className={`audio-mute${track.muted ? " active" : ""}`}
        aria-pressed={track.muted}
        aria-label={track.muted ? `Unmute ${title}` : `Mute ${title}`}
        title={track.muted ? "Unmute" : "Mute"}
        onClick={onMute}
      >
        <MuteIcon size={15} strokeWidth={1.5} />
      </button>
      <div className="audio-channel-main">
        <div className="audio-channel-meta">
          <span className="audio-label">
            <span className="audio-kind">{kindLabel(track.kind)}</span>
            {title}
          </span>
          <span className="audio-volume-value tabular">
            {Math.round(track.volume * 100)}%
          </span>
          <span className="audio-actions">
            <button
              type="button"
              className={`audio-gear${settingsOpen ? " active" : ""}`}
              aria-expanded={settingsOpen}
              aria-label={`Settings for ${title}`}
              title="Audio settings"
              onClick={onToggleSettings}
            >
              <Settings2 size={13} strokeWidth={1.5} />
            </button>
            <button
              type="button"
              className="audio-remove"
              aria-label={`Delete ${title}`}
              title="Delete"
              onClick={onRemove}
            >
              <Trash2 size={13} strokeWidth={1.5} />
            </button>
          </span>
        </div>
        <label className="audio-combo">
          <span className="sr-only">Volume for {title}</span>
          <span className="audio-combo-track" aria-hidden>
            <span
              className="audio-combo-level"
              style={{ width: `${Math.round(shownLevel * 100)}%` }}
            />
            <span
              className="audio-combo-volume"
              style={{ width: `${Math.round(track.volume * 100)}%` }}
            />
          </span>
          <input
            type="range"
            min={0}
            max={100}
            value={Math.round(track.volume * 100)}
            disabled={track.muted}
            onChange={(e) => onVolume(Number(e.target.value) / 100)}
          />
        </label>
        {settingsOpen ? (
          <select
            className="audio-device-select"
            value={track.deviceId}
            aria-label={`Device for ${title}`}
            onChange={(e) => onDevice(e.target.value)}
          >
            {devices.map((device) => (
              <option key={device.id} value={device.id}>
                {device.label}
              </option>
            ))}
          </select>
        ) : null}
      </div>
    </div>
  );
}
