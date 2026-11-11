import { invoke } from "@tauri-apps/api/core";
import type {
  AppConfig,
  AppState,
  AudioDefaults,
  GeneralSettings,
  GpuInfo,
  Platform,
  StreamDestination,
  VideoSettings,
} from "../types/stream";

export function getAppState(): Promise<AppState> {
  return invoke("get_app_state");
}

export function getStreamDestination(): Promise<StreamDestination> {
  return invoke("get_stream_destination");
}

export function setStreamDestination(
  platform: Platform,
  ingestUrl: string,
): Promise<StreamDestination> {
  return invoke("set_stream_destination", {
    platform,
    ingestUrl,
  });
}

export function setStreamKey(platform: Platform, key: string): Promise<boolean> {
  return invoke("set_stream_key", { platform, key });
}

export function hasStreamKey(platform: Platform): Promise<boolean> {
  return invoke("has_stream_key", { platform });
}

export function clearStreamKey(platform: Platform): Promise<boolean> {
  return invoke("clear_stream_key", { platform });
}

export function getVideoSettings(): Promise<VideoSettings> {
  return invoke("get_video_settings");
}

export function setVideoSettings(
  settings: VideoSettings,
): Promise<VideoSettings> {
  return invoke("set_video_settings", { settings });
}

export function getAppConfig(): Promise<AppConfig> {
  return invoke("get_app_config");
}

export function setAudioDefaults(
  audio: AudioDefaults,
): Promise<AudioDefaults> {
  return invoke("set_audio_defaults", { audio });
}

export function setGeneralSettings(
  general: GeneralSettings,
): Promise<GeneralSettings> {
  return invoke("set_general_settings", { general });
}

export function appendCrashLog(message: string): Promise<boolean> {
  return invoke("append_crash_log", { message });
}

export function readCrashLog(): Promise<string> {
  return invoke("read_crash_log");
}

export function detectGpu(): Promise<GpuInfo> {
  return invoke("detect_gpu");
}
