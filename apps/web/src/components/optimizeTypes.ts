export type Phase = "idle" | "running" | "done";

export type Row = { label: string; before: string; after: string };

export const OPTIMIZED: Row[] = [
  { label: "Encoder", before: "x264 (CPU)", after: "NVIDIA NVENC" },
  { label: "Resolution", before: "1280×720", after: "1920×1080" },
  { label: "Frame rate", before: "30 fps", after: "60 fps" },
  { label: "Bitrate", before: "4,500 kbps", after: "6,000 kbps" },
];
