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

export interface StreamDestination {
  platform: Platform;
  ingestUrl: string;
  hasStreamKey: boolean;
}

export interface VideoSettings {
  automatic: boolean;
  encoder: string;
  resolution: string;
  frameRate: number;
  bitrateKbps: number;
  keyframeIntervalSec: number;
}

export interface AudioDefaults {
  defaultMicId: string;
  defaultDesktopId: string;
}

export interface GeneralSettings {
  startMinimized: boolean;
  confirmBeforeStop: boolean;
}

export interface AppConfig {
  platform: Platform;
  ingestUrls: {
    youtube: string;
    kick: string;
    twitch: string;
  };
  video: VideoSettings;
  audio: AudioDefaults;
  general: GeneralSettings;
}

export interface GpuInfo {
  name: string;
  vendor: "nvidia" | "amd" | "intel" | "unknown" | string;
  suggestedEncoder: string;
}

export const PLATFORMS: { id: Platform; label: string }[] = [
  { id: "youtube", label: "YouTube" },
  { id: "kick", label: "Kick" },
  { id: "twitch", label: "Twitch" },
];

export const PLATFORM_HELPER: Record<Platform, string> = {
  youtube:
    "Paste the RTMP ingest URL and stream key from YouTube Studio → Go live → Stream. No Google login in ArcaLabs.",
  kick: "Paste the RTMP URL and stream key from Kick Creator Dashboard → Stream. No Kick login in ArcaLabs.",
  twitch:
    "Paste rtmp://live.twitch.tv/app and your stream key from Twitch Dashboard → Settings → Stream. No Twitch login in ArcaLabs.",
};

export const KEY_MASK = "••••••••••••";

export function platformLabel(platform: Platform): string {
  return PLATFORMS.find((p) => p.id === platform)?.label ?? platform;
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

export function formatSpecLine(video: VideoSettings): string {
  const res = video.resolution.replace("x", "×");
  return `${video.encoder} · ${res} · ${video.frameRate} · ${video.bitrateKbps.toLocaleString("en-US")} kbps`;
}
