import { invoke } from "@tauri-apps/api/core";
import type {
  AppConfig,
  AppState,
  AudioSource,
  AudioInputLevel,
  AudioTrack,
  AudioDefaults,
  GeneralSettings,
  GpuInfo,
  Platform,
  StreamDestination,
  VideoSettings,
  SceneCollection,
  CaptureDisplay,
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

export function getScenes(): Promise<SceneCollection> {
  return invoke("get_scenes");
}

export function getCaptureDisplays(): Promise<CaptureDisplay[]> {
  return invoke("get_capture_displays");
}

export function addDisplaySource(sceneId: string, displayId: string): Promise<SceneCollection> {
  return invoke("add_display_source", { sceneId, displayId });
}

export function addWindowSource(sceneId: string): Promise<SceneCollection | null> {
  return invoke("add_window_source", { sceneId });
}

export function reselectWindowSource(sceneId: string, sourceId: string): Promise<SceneCollection | null> {
  return invoke("reselect_window_source", { sceneId, sourceId });
}

export function createScene(name: string): Promise<SceneCollection> {
  return invoke("create_scene", { name });
}

export function renameScene(sceneId: string, name: string): Promise<SceneCollection> {
  return invoke("rename_scene", { sceneId, name });
}

export function setActiveScene(sceneId: string): Promise<SceneCollection> {
  return invoke("set_active_scene", { sceneId });
}

export function deleteScene(sceneId: string): Promise<SceneCollection> {
  return invoke("delete_scene", { sceneId });
}

export function reorderScene(sceneId: string, destination: number): Promise<SceneCollection> {
  return invoke("reorder_scene", { sceneId, destination });
}

export function renameSceneSource(sceneId: string, sourceId: string, name: string): Promise<SceneCollection> {
  return invoke("rename_source", { sceneId, sourceId, name });
}

export function setSceneSourceVisibility(sceneId: string, sourceId: string, visible: boolean): Promise<SceneCollection> {
  return invoke("set_source_visibility", { sceneId, sourceId, visible });
}

export function setSceneSourceTransform(
  sceneId: string,
  sourceId: string,
  x: number,
  y: number,
  width: number,
  height: number,
): Promise<SceneCollection> {
  return invoke("set_source_transform", { sceneId, sourceId, x, y, width, height });
}

export function reorderSceneSource(sceneId: string, sourceId: string, destination: number): Promise<SceneCollection> {
  return invoke("reorder_source", { sceneId, sourceId, destination });
}

export function removeSceneSource(sceneId: string, sourceId: string): Promise<SceneCollection> {
  return invoke("remove_source", { sceneId, sourceId });
}

export function getPreviewFrame(sceneId: string): Promise<ArrayBuffer> {
  return invoke("get_preview_frame", { sceneId });
}

export function stopPreview(sceneId?: string): Promise<void> {
  return invoke("stop_preview", { sceneId: sceneId ?? null });
}

export function getAudioSources(
  kind: AudioSource["kind"],
): Promise<AudioSource[]> {
  return invoke("get_audio_sources", { kind });
}

export function getAudioTracks(): Promise<AudioTrack[]> {
  return invoke("get_audio_tracks");
}

export function addAudioTrack(
  kind: AudioSource["kind"],
  id: string,
): Promise<AudioTrack[]> {
  return invoke("add_audio_track", { kind, id });
}

export function setAudioTrackSettings(
  id: string,
  volume: number,
  muted: boolean,
): Promise<AudioTrack[]> {
  return invoke("set_audio_track_settings", { id, volume, muted });
}

export function removeAudioTrack(id: string): Promise<AudioTrack[]> {
  return invoke("remove_audio_track", { id });
}

export function startAudioMeter(): Promise<void> {
  return invoke("start_audio_meter");
}

export function stopAudioMeter(): Promise<void> {
  return invoke("stop_audio_meter");
}

export function getAudioInputLevels(): Promise<AudioInputLevel[]> {
  return invoke("get_audio_input_levels");
}
