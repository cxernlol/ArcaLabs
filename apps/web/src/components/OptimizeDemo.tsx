import { useCallback, useState } from "react";
import "./OptimizeDemo.css";

type Row = { label: string; before: string; after: string };

const OPTIMIZED: Row[] = [
  { label: "Encoder", before: "x264 (CPU)", after: "NVIDIA NVENC" },
  { label: "Resolution", before: "1280×720", after: "1920×1080" },
  { label: "Frame rate", before: "30 fps", after: "60 fps" },
  { label: "Bitrate", before: "4,500 kbps", after: "6,000 kbps" },
];

type Phase = "idle" | "running" | "done";

export function OptimizeDemo() {
  const [phase, setPhase] = useState<Phase>("idle");

  const runOptimize = useCallback(() => {
    if (phase === "running") return;
    setPhase("running");
    const delay = window.matchMedia("(prefers-reduced-motion: reduce)").matches
      ? 200
      : 1600;
    window.setTimeout(() => setPhase("done"), delay);
  }, [phase]);

  const undo = useCallback(() => setPhase("idle"), []);

  const specLine =
    phase === "done" ? "NVENC · 1080p60 · 6,000 kbps" : "Automatic · not run yet";

  return (
    <section id="optimize-demo" className="demo" aria-labelledby="demo-title">
      <div className="demo-card">
        <header className="demo-card-head">
          <div>
            <h2 id="demo-title">Optimize for this PC</h2>
            <p className="demo-lead">
              Same flow as the app: we pick settings, you see what changed, you can
              undo. Runs entirely in your browser. Nothing is sent anywhere.
            </p>
          </div>
        </header>

        <div className="demo-automatic">
          <div>
            <p className="demo-row-title">Automatic</p>
            <p className="demo-row-hint">
              Picks encoder, resolution, frame rate and bitrate for this machine.
            </p>
          </div>
          <div className="demo-automatic-actions">
            {phase === "done" && (
              <button type="button" className="btn-ghost" onClick={runOptimize}>
                Re-run
              </button>
            )}
            <button
              type="button"
              className="btn-accent"
              onClick={runOptimize}
              disabled={phase === "running"}
              aria-busy={phase === "running"}
            >
              {phase === "running" && <span className="spinner" aria-hidden />}
              Optimize for this PC
            </button>
          </div>
        </div>

        <p className="demo-spec tabular-nums">{specLine}</p>

        {phase === "done" && (
          <div className="demo-result">
            <div className="demo-result-head">
              <span className="demo-result-label">
                <span className="demo-check" aria-hidden>
                  ✓
                </span>
                Optimized just now
              </span>
              <button type="button" className="btn-ghost" onClick={undo}>
                Undo
              </button>
            </div>
            <ul className="demo-diff">
              {OPTIMIZED.map((row) => (
                <li key={row.label}>
                  <span className="demo-diff-label">{row.label}</span>
                  <span className="demo-diff-values">
                    <span className="demo-old">{row.before}</span>
                    <span aria-hidden>→</span>
                    <span className="demo-new">{row.after}</span>
                  </span>
                </li>
              ))}
            </ul>
          </div>
        )}

        {phase === "idle" && (
          <p className="demo-idle-hint">
            Tap optimize to preview the result card from the desktop mockup.
          </p>
        )}
      </div>
    </section>
  );
}
