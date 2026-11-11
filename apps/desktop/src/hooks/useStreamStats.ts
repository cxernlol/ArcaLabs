import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import type { StreamStats } from "../types/stream";

const OFFLINE_STATS: StreamStats = {
  cpuPercent: 4,
  fps: 60,
  kbps: 0,
  dropped: 0,
  elapsedMs: 0,
};

function normalizeStats(payload: Record<string, unknown>): StreamStats {
  const cpu = Number(payload.cpuPercent ?? payload.cpu_percent);
  const fps = Number(payload.fps);
  const kbps = Number(payload.kbps);
  const dropped = Number(payload.dropped);
  const elapsed = Number(payload.elapsedMs ?? payload.elapsed_ms);

  return {
    cpuPercent: Number.isFinite(cpu) ? cpu : OFFLINE_STATS.cpuPercent,
    fps: Number.isFinite(fps) ? fps : OFFLINE_STATS.fps,
    kbps: Number.isFinite(kbps) ? kbps : OFFLINE_STATS.kbps,
    dropped: Number.isFinite(dropped) ? dropped : OFFLINE_STATS.dropped,
    elapsedMs: Number.isFinite(elapsed) ? elapsed : OFFLINE_STATS.elapsedMs,
  };
}

export function useStreamStats() {
  const [stats, setStats] = useState<StreamStats>(OFFLINE_STATS);

  useEffect(() => {
    let unlisten: (() => void) | undefined;

    listen<Record<string, unknown>>("stream:stats", (event) => {
      setStats(normalizeStats(event.payload ?? {}));
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      unlisten?.();
    };
  }, []);

  return stats;
}
