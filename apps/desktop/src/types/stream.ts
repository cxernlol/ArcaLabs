export type Platform = "youtube" | "kick" | "twitch";

export interface AppState {
  state: string;
  live: boolean;
}

export interface StreamStats {
  cpuPercent: number;
  fps: number;
  kbps: number;
  dropped: number;
  elapsedMs: number;
}

export function formatElapsed(ms: number): string {
  const totalSec = Math.floor(ms / 1000);
  const h = Math.floor(totalSec / 3600);
  const m = Math.floor((totalSec % 3600) / 60);
  const s = totalSec % 60;
  return [h, m, s].map((n) => String(n).padStart(2, "0")).join(":");
}

export function formatKbps(kbps: number): string {
  return kbps.toLocaleString("en-US");
}
