import { useCallback, useState } from "react";
import { OPTIMIZED, type Phase } from "./optimizeTypes";
import { SettingsPreview } from "./SettingsPreview";
import "./OptimizeDemo.css";

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
      <header className="demo-section-head">
        <h2 id="demo-title">Before & after: optimize</h2>
        <p>
          Same flow as the desktop app: one tap, a visible diff, undo if you want.
          Runs in your browser only.
        </p>
      </header>

      <div className="demo-card">
        <SettingsPreview phase={phase} />

        <div className="compare-grid" aria-live="polite">
          <div className="compare-col">
            <h3 className="compare-col-title">Before</h3>
            <ul className="compare-list">
              {OPTIMIZED.map((row) => (
                <li key={row.label}>
                  <span>{row.label}</span>
                  <span className="compare-value">{row.before}</span>
                </li>
              ))}
            </ul>
          </div>
          <div className="compare-col compare-col-after">
            <h3 className="compare-col-title">After</h3>
            {phase === "done" ? (
              <ul className="compare-list">
                {OPTIMIZED.map((row) => (
                  <li key={row.label}>
                    <span>{row.label}</span>
                    <span className="compare-value compare-value-new">
                      {row.after}
                    </span>
                  </li>
                ))}
              </ul>
            ) : (
              <p className="compare-placeholder">
                {phase === "running"
                  ? "Detecting hardware…"
                  : "Run optimize to fill this column."}
              </p>
            )}
          </div>
        </div>

        <div className="demo-automatic">
          <div>
            <p className="demo-row-title">Automatic</p>
            <p className="demo-row-hint">
              Picks encoder, resolution, frame rate and bitrate on your machine.
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
          <div className="demo-result-actions">
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
        )}
      </div>
    </section>
  );
}
