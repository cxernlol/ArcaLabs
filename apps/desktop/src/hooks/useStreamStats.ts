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

export function useStreamStats() {
  const [stats, setStats] = useState<StreamStats>(OFFLINE_STATS);

  useEffect(() => {
    let unlisten: (() => void) | undefined;

    listen<{
      cpu_percent: number;
      fps: number;
      kbps: number;
      dropped: number;
      elapsed_ms: number;
    }>("stream:stats", (event) => {
      const p = event.payload;
      setStats({
        cpuPercent: p.cpu_percent,
        fps: p.fps,
        kbps: p.kbps,
        dropped: p.dropped,
        elapsedMs: p.elapsed_ms,
      });
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      unlisten?.();
    };
  }, []);

  return stats;
}
