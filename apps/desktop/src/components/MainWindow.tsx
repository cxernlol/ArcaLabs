import { useState, type ReactNode } from "react";
import {
  Eye,
  Mic,
  Plus,
  Volume2,
  Zap,
} from "lucide-react";
import { Logo } from "./Logo";
import { WindowControls } from "./WindowControls";
import type { Platform } from "../types/stream";
import {
  formatElapsed,
  formatKbps,
} from "../types/stream";
import { useStreamStats } from "../hooks/useStreamStats";
import "./MainWindow.css";

const SCENES = ["Main", "Just chatting", "Be right back"];
const SOURCES = [
  { id: "screen", label: "Screen 1", visible: true },
  { id: "webcam", label: "Webcam", visible: true },
];

const PLATFORMS: { id: Platform; label: string }[] = [
  { id: "youtube", label: "YouTube" },
  { id: "kick", label: "Kick" },
  { id: "twitch", label: "Twitch" },
];

export function MainWindow() {
  const stats = useStreamStats();
  const [activeScene, setActiveScene] = useState("Main");
  const [platform, setPlatform] = useState<Platform>("twitch");

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
        <div className="app-header-status" data-tauri-drag-region>
          <span className="status-dot status-idle" aria-hidden />
          <span>Offline</span>
        </div>
        <WindowControls />
      </header>

      <main className="app-main">
        <section className="preview-wrap">
          <div className="preview-letterbox">
            <div className="preview-canvas">
              <span className="preview-label">Screen 1 · 1920×1080</span>
            </div>
          </div>
        </section>

        <section className="control-row">
          <Panel title="Scenes">
            <ul className="list-rows">
              {SCENES.map((scene) => (
                <li key={scene}>
                  <button
                    type="button"
                    className={`list-row${activeScene === scene ? " selected" : ""}`}
                    onClick={() => setActiveScene(scene)}
                  >
                    {scene}
                  </button>
                </li>
              ))}
            </ul>
          </Panel>

          <Panel title="Sources">
            <ul className="list-rows">
              {SOURCES.map((src) => (
                <li key={src.id}>
                  <button type="button" className="list-row source-row">
                    <span>{src.label}</span>
                    <Eye size={18} strokeWidth={1.5} />
                  </button>
                </li>
              ))}
              <li>
                <button type="button" className="list-row source-row muted">
                  <span>Add source</span>
                  <Plus size={18} strokeWidth={1.5} />
                </button>
              </li>
            </ul>
          </Panel>

          <Panel title="Audio" className="panel-audio">
            <AudioRow icon={Mic} label="Microphone" level={0.35} />
            <AudioRow icon={Volume2} label="Desktop audio" level={0.12} />
          </Panel>

          <Panel title="Stream" className="panel-stream">
            <SegmentedControl
              options={PLATFORMS}
              value={platform}
              onChange={setPlatform}
            />
            <button type="button" className="btn-outlined">
              <Zap size={18} strokeWidth={1.5} />
              Optimize for this PC
            </button>
            <p className="spec-line">NVENC · 1080p60 · 6,000 kbps</p>
            <button type="button" className="btn-primary">
              Go Live
            </button>
          </Panel>
        </section>
      </main>

      <footer className="status-bar">
        <span className="status-bar-left">Not streaming</span>
        <span className="status-bar-right tabular">
          CPU {stats.cpuPercent}% · {stats.fps} fps ·{" "}
          {formatKbps(stats.kbps)} kbps · {stats.dropped} dropped ·{" "}
          {formatElapsed(stats.elapsedMs)}
        </span>
      </footer>
    </div>
  );
}

function Panel({
  title,
  children,
  className = "",
}: {
  title: string;
  children: ReactNode;
  className?: string;
}) {
  return (
    <div className={`panel ${className}`}>
      <h2 className="panel-title">{title}</h2>
      <div className="panel-body">{children}</div>
    </div>
  );
}

function SegmentedControl<T extends string>({
  options,
  value,
  onChange,
}: {
  options: { id: T; label: string }[];
  value: T;
  onChange: (v: T) => void;
}) {
  return (
    <div className="segmented" role="tablist" aria-label="Platform">
      {options.map((opt) => (
        <button
          key={opt.id}
          type="button"
          role="tab"
          aria-selected={value === opt.id}
          className={`segmented-item${value === opt.id ? " active" : ""}`}
          onClick={() => onChange(opt.id)}
        >
          {opt.label}
        </button>
      ))}
    </div>
  );
}

function AudioRow({
  icon: Icon,
  label,
  level,
}: {
  icon: typeof Mic;
  label: string;
  level: number;
}) {
  return (
    <div className="audio-row">
      <Icon size={18} strokeWidth={1.5} />
      <span className="audio-label">{label}</span>
      <div className="meter-track">
        <div
          className="meter-fill"
          style={{ width: `${Math.round(level * 100)}%` }}
        />
      </div>
    </div>
  );
}
