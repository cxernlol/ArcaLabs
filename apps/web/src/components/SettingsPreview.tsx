import type { Phase } from "./optimizeTypes";
import "./SettingsPreview.css";

export function SettingsPreview({ phase }: { phase: Phase }) {
  const status =
    phase === "running"
      ? "running"
      : phase === "done"
        ? "complete"
        : "pending";

  return (
    <aside className="settings-preview" aria-label="Read-only settings preview">
      <p className="settings-preview-label">read-only preview</p>
      <pre className="settings-preview-code">
        {`# video.automatic
mode: automatic
optimize: ${status}

encoder: ${phase === "done" ? "nvenc" : phase === "running" ? "…" : "x264"}
resolution: ${phase === "done" ? "1920×1080" : "1280×720"}
frame_rate: ${phase === "done" ? "60" : "30"}
bitrate_kbps: ${phase === "done" ? "6000" : "4500"}

# stream.connection (desktop app only)
telemetry: off`}
      </pre>
    </aside>
  );
}
