import { useCallback, useEffect, useState, type MouseEvent as ReactMouseEvent, type ReactNode } from "react";
import { AnimatePresence, motion, useReducedMotion } from "framer-motion";
import {
  AppWindow,
  ChevronDown,
  Eye,
  EyeOff,
  Monitor,
  Pencil,
  Mic,
  MicOff,
  Plus,
  Settings,
  Settings2,
  ChevronUp,
  ChevronDown as ChevronDownIcon,
  Trash2,
  Volume2,
  VolumeX,
} from "lucide-react";
import { Logo } from "./Logo";
import { WindowControls } from "./WindowControls";
import { SettingsShell } from "./SettingsShell";
import {
  addAudioTrack,
  detectGpu,
  getAudioTracks,
  getAudioInputLevels,
  getAudioSources,
  startAudioMeter,
  stopAudioMeter,
  removeAudioTrack,
  setAudioTrackSettings,
  getVideoSettings,
  setVideoSettings,
  getScenes,
  createScene,
  renameScene,
  setActiveScene,
  deleteScene,
  reorderScene,
  getCaptureDisplays,
  addDisplaySource,
  addWindowSource,
  reselectWindowSource,
  renameSceneSource,
  setSceneSourceVisibility,
  reorderSceneSource,
  removeSceneSource,
  getPreviewFrame,
  stopPreview,
} from "../lib/tauriApi";
import type { AudioInputLevel, AudioSource, AudioTrack, CaptureDisplay, SceneCollection } from "../types/stream";
import "./MainWindow.css";

type AudioKind = "mic" | "desktop" | "application";
type WorkspaceContextTarget =
  | { kind: "scene"; id: string }
  | { kind: "source"; id: string; sceneId: string }
  | { kind: "audio"; id: string };
type WorkspaceContextMenu = WorkspaceContextTarget & { x: number; y: number };

let audioMeterQueue = Promise.resolve();

function queueAudioMeterCommand(start: boolean): Promise<void> {
  audioMeterQueue = audioMeterQueue
    .catch(() => {})
    .then(() => start ? startAudioMeter() : stopAudioMeter());
  return audioMeterQueue;
}

function kindLabel(kind: AudioKind): string {
  if (kind === "mic") return "Mic/Aux";
  if (kind === "desktop") return "Desktop audio";
  return "Application audio";
}

export function MainWindow() {
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [scenesOpen, setScenesOpen] = useState(true);
  const [sourcesOpen, setSourcesOpen] = useState(true);
  const [audioOpen, setAudioOpen] = useState(true);
  const [sceneCollection, setSceneCollection] = useState<SceneCollection | null>(null);
  const [sceneLoading, setSceneLoading] = useState(true);
  const [sceneEditor, setSceneEditor] = useState<{ id: string | null; name: string } | null>(null);
  const [sourceEditor, setSourceEditor] = useState<{ id: string; sceneId: string; name: string } | null>(null);
  const [sourceSettings, setSourceSettings] = useState<{ id: string; sceneId: string; name: string } | null>(null);
  const [contextMenu, setContextMenu] = useState<WorkspaceContextMenu | null>(null);
  const [deleteConfirm, setDeleteConfirm] = useState<{ kind: "scene" | "source"; id: string; name: string; sceneId?: string } | null>(null);
  const [sceneError, setSceneError] = useState<string | null>(null);
  const [activeSource, setActiveSource] = useState<string | null>(null);
  const [displayPickerOpen, setDisplayPickerOpen] = useState(false);
  const [captureDisplays, setCaptureDisplays] = useState<CaptureDisplay[]>([]);
  const [displayLoading, setDisplayLoading] = useState(false);
  const [windowLoading, setWindowLoading] = useState(false);
  const [sourceError, setSourceError] = useState<string | null>(null);
  const [audioTracks, setAudioTracks] = useState<AudioTrack[]>([]);
  const [audioLevels, setAudioLevels] = useState<Record<string, AudioInputLevel>>({});
  const [audioSettingsId, setAudioSettingsId] = useState<string | null>(null);
  const [audioPickerKind, setAudioPickerKind] = useState<AudioKind | null>(null);
  const [audioChoices, setAudioChoices] = useState<AudioSource[]>([]);
  const [audioLoading, setAudioLoading] = useState(false);
  const [audioError, setAudioError] = useState<string | null>(null);
  const [audioNotice, setAudioNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [previewUrl, setPreviewUrl] = useState<string | null>(null);
  const [previewError, setPreviewError] = useState<string | null>(null);
  const refreshAudioChoices = useCallback(async (kind: AudioKind, showLoading = true) => {
    if (showLoading) setAudioLoading(true);
    setAudioError(null);
    try {
      setAudioChoices(await getAudioSources(kind));
    } catch (cause) {
      setAudioError(String(cause));
      setAudioChoices([]);
    } finally {
      if (showLoading) setAudioLoading(false);
    }
  }, []);

  useEffect(() => {
    getAudioTracks().then(setAudioTracks).catch((cause) => setAudioError(String(cause)));
    getScenes()
      .then(setSceneCollection)
      .catch((cause) => setError(String(cause)))
      .finally(() => setSceneLoading(false));
  }, []);

  const meteredTrackIds = audioTracks
    .filter((track) => track.kind === "mic" || track.kind === "application")
    .map((track) => track.id)
    .sort()
    .join("\n");

  useEffect(() => {
    let disposed = false;
    let visible = document.visibilityState === "visible";
    let running = false;
    let timer = 0;
    const queueMeterCommand = (start: boolean) =>
      queueAudioMeterCommand(start).catch((cause) => {
        if (!disposed) setAudioError(String(cause));
      });

    const poll = async () => {
      if (disposed || !running) return;
      try {
        const levels = await getAudioInputLevels();
        if (!disposed) setAudioLevels(Object.fromEntries(levels.map((level) => [level.id, level])));
      } catch (cause) {
        if (!disposed) setAudioError(String(cause));
      }
      if (!disposed && running) timer = window.setTimeout(poll, 50);
    };

    const start = async () => {
      if (running || disposed || !visible || !audioOpen || !meteredTrackIds) return;
      running = true;
      await queueMeterCommand(true);
      if (!disposed && running) void poll();
    };

    const stop = () => {
      running = false;
      window.clearTimeout(timer);
      setAudioLevels({});
      if (audioOpen && meteredTrackIds) void queueMeterCommand(false);
    };

    const onVisibilityChange = () => {
      visible = document.visibilityState === "visible";
      if (visible) void start();
      else stop();
    };

    document.addEventListener("visibilitychange", onVisibilityChange);
    if (audioOpen && meteredTrackIds && visible) void start();
    else void queueMeterCommand(false);

    return () => {
      disposed = true;
      running = false;
      window.clearTimeout(timer);
      document.removeEventListener("visibilitychange", onVisibilityChange);
      if (audioOpen && meteredTrackIds) void queueMeterCommand(false);
    };
  }, [audioOpen, meteredTrackIds]);

  async function updateScenes(action: Promise<SceneCollection>) {
    try {
      setSceneCollection(await action);
      setError(null);
      setSceneError(null);
      return true;
    } catch (cause) {
      setSceneError(String(cause));
      return false;
    }
  }

  async function saveScene() {
    const name = sceneEditor?.name.trim();
    if (!sceneEditor || !name) return;
    const saved = await updateScenes(sceneEditor.id ? renameScene(sceneEditor.id, name) : createScene(name));
    if (saved) setSceneEditor(null);
  }

  async function saveSource() {
    const name = sourceEditor?.name.trim();
    if (!sourceEditor || !name) return;
    try {
      setSceneCollection(await renameSceneSource(sourceEditor.sceneId, sourceEditor.id, name));
      setSceneError(null);
      setSourceEditor(null);
    } catch (cause) {
      setSourceError(String(cause));
    }
  }

  async function saveSourceSettings() {
    const name = sourceSettings?.name.trim();
    if (!sourceSettings || !name) return;
    try {
      setSceneCollection(await renameSceneSource(sourceSettings.sceneId, sourceSettings.id, name));
      setSceneError(null);
      setSourceSettings(null);
    } catch (cause) {
      setSourceError(String(cause));
    }
  }

  function openContextMenu(event: ReactMouseEvent, menu: WorkspaceContextTarget) {
    event.preventDefault();
    event.stopPropagation();
    if (menu.kind === "source") setActiveSource(menu.id);
    setContextMenu({ ...menu, x: event.clientX, y: event.clientY });
  }

  async function confirmDelete() {
    if (!deleteConfirm) return;
    const result = deleteConfirm.kind === "scene"
      ? await updateScenes(deleteScene(deleteConfirm.id))
      : deleteConfirm.sceneId
        ? await removeSceneSource(deleteConfirm.sceneId, deleteConfirm.id)
          .then((next) => { setSceneCollection(next); setSceneError(null); return true; })
          .catch((cause) => { setSceneError(String(cause)); return false; })
        : false;
    if (result) {
      if (deleteConfirm.kind === "source") setActiveSource((id) => id === deleteConfirm.id ? null : id);
      setDeleteConfirm(null);
    }
  }

  async function openDisplayPicker() {
    setDisplayPickerOpen(true);
    setDisplayLoading(true);
    setSourceError(null);
    try {
      setCaptureDisplays(await getCaptureDisplays());
    } catch (cause) {
      setSourceError(String(cause));
      setCaptureDisplays([]);
    } finally {
      setDisplayLoading(false);
    }
  }

  async function selectDisplay(display: CaptureDisplay) {
    if (!activeScene) return;
    setDisplayLoading(true);
    setSourceError(null);
    try {
      const next = await addDisplaySource(activeScene.id, display.id);
      setSceneCollection(next);
      const scene = next.scenes.find((item) => item.id === activeScene.id);
      setActiveSource(scene?.sources[scene.sources.length - 1]?.id ?? null);
      setDisplayPickerOpen(false);
    } catch (cause) {
      setSourceError(String(cause));
    } finally {
      setDisplayLoading(false);
    }
  }

  async function addWindowCapture() {
    if (!activeScene) return;
    setWindowLoading(true);
    setSourceError(null);
    try {
      const next = await addWindowSource(activeScene.id);
      if (!next) return;
      setSceneCollection(next);
      const scene = next.scenes.find((item) => item.id === activeScene.id);
      setActiveSource(scene?.sources[scene.sources.length - 1]?.id ?? null);
    } catch (cause) {
      setSourceError(String(cause));
    } finally {
      setWindowLoading(false);
    }
  }

  async function reselectWindow(sceneId: string, sourceId: string) {
    setWindowLoading(true);
    setSourceError(null);
    try {
      const next = await reselectWindowSource(sceneId, sourceId);
      if (next) setSceneCollection(next);
    } catch (cause) {
      setSourceError(String(cause));
    } finally {
      setWindowLoading(false);
    }
  }

  async function openAudioPicker(kind: AudioKind) {
    setAudioNotice(null);
    if (kind === "desktop") {
      setAudioLoading(true);
      setAudioError(null);
      try {
        const [source] = await getAudioSources(kind);
        if (!source) throw new Error("No default desktop audio output is available.");
        setAudioTracks(await addAudioTrack(kind, source.id));
        setAudioNotice("Desktop audio added.");
      } catch (cause) {
        setAudioError(String(cause));
      } finally {
        setAudioLoading(false);
      }
      return;
    }

    setAudioPickerKind(kind);
    setAudioChoices([]);
    await refreshAudioChoices(kind);
  }

  async function addAudioSource(source: AudioSource) {
    setAudioLoading(true);
    setAudioError(null);
    try {
      setAudioTracks(await addAudioTrack(source.kind, source.id));
      setAudioPickerKind(null);
      setAudioChoices([]);
    } catch (cause) {
      setAudioError(String(cause));
    } finally {
      setAudioLoading(false);
    }
  }

  async function updateAudioTrack(track: AudioTrack, changes: Partial<Pick<AudioTrack, "volume" | "muted">>) {
    const next = { ...track, ...changes };
    setAudioTracks((tracks) => tracks.map((item) => item.id === track.id ? next : item));
    try {
      setAudioTracks(await setAudioTrackSettings(next.id, next.volume, next.muted));
    } catch (cause) {
      setAudioError(String(cause));
      getAudioTracks().then(setAudioTracks).catch(() => {});
    }
  }

  async function deleteAudioTrack(id: string) {
    try {
      setAudioTracks(await removeAudioTrack(id));
      setAudioSettingsId((current) => current === id ? null : current);
    } catch (cause) {
      setAudioError(String(cause));
    }
  }

  useEffect(() => {
    if (audioPickerKind !== "application") return;
    let cancelled = false;
    let timer = 0;
    const poll = async () => {
      await refreshAudioChoices("application", false);
      if (!cancelled) timer = window.setTimeout(poll, 2000);
    };
    timer = window.setTimeout(poll, 2000);
    return () => {
      cancelled = true;
      window.clearTimeout(timer);
    };
  }, [audioPickerKind, refreshAudioChoices]);

  useEffect(() => {
    if (!audioPickerKind) return;
    function closeOnEscape(event: KeyboardEvent) {
      if (event.key === "Escape") setAudioPickerKind(null);
    }
    window.addEventListener("keydown", closeOnEscape);
    return () => window.removeEventListener("keydown", closeOnEscape);
  }, [audioPickerKind]);

  useEffect(() => {
    if (!displayPickerOpen) return;
    function closeOnEscape(event: KeyboardEvent) {
      if (event.key === "Escape") setDisplayPickerOpen(false);
    }
    window.addEventListener("keydown", closeOnEscape);
    return () => window.removeEventListener("keydown", closeOnEscape);
  }, [displayPickerOpen]);

  const refresh = useCallback(async () => {
    const [vid, gpu] = await Promise.all([
      getVideoSettings(),
      detectGpu().catch(() => null),
    ]);
    if (vid.automatic && gpu && vid.encoder !== gpu.suggestedEncoder) {
      await setVideoSettings({
        ...vid,
        encoder: gpu.suggestedEncoder,
      });
    }
  }, []);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        await refresh();
      } catch (e) {
        if (!cancelled) {
          setError(e instanceof Error ? e.message : String(e));
        }
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [refresh]);

  function closeSettings() {
    setSettingsOpen(false);
    void refresh().catch((e) =>
      setError(e instanceof Error ? e.message : String(e)),
    );
  }

  const activeScene = sceneCollection?.scenes.find((scene) => scene.id === sceneCollection.activeSceneId);
  const previewSceneId = activeScene?.id;
  const previewSources = activeScene?.sources;
  const selectedSource = activeScene?.sources.find((source) => source.id === activeSource);
  const contextScene = contextMenu?.kind === "scene" ? sceneCollection?.scenes.find((scene) => scene.id === contextMenu.id) : undefined;
  const contextSceneIndex = contextMenu?.kind === "scene" ? sceneCollection?.scenes.findIndex((scene) => scene.id === contextMenu.id) ?? -1 : -1;
  const contextSourceScene = contextMenu?.kind === "source" ? sceneCollection?.scenes.find((scene) => scene.id === contextMenu.sceneId) : undefined;
  const contextSource = contextMenu?.kind === "source" ? contextSourceScene?.sources.find((source) => source.id === contextMenu.id) : undefined;
  const contextSourceIndex = contextMenu?.kind === "source" ? contextSourceScene?.sources.findIndex((source) => source.id === contextMenu.id) ?? -1 : -1;
  const contextAudioTrack = contextMenu?.kind === "audio" ? audioTracks.find((track) => track.id === contextMenu.id) : undefined;
  const sourceSettingsScene = sceneCollection?.scenes.find((scene) => scene.id === sourceSettings?.sceneId);
  const sourceSettingsItem = sourceSettingsScene?.sources.find((source) => source.id === sourceSettings?.id);

  useEffect(() => {
    if (!contextMenu) return;
    const dismiss = (event: Event) => {
      if (event.type === "keydown" && (event as KeyboardEvent).key !== "Escape") return;
      if (event.type === "pointerdown" && event.target instanceof Element && event.target.closest("[data-row-context-menu]")) return;
      setContextMenu(null);
    };
    window.addEventListener("pointerdown", dismiss);
    window.addEventListener("keydown", dismiss);
    window.addEventListener("resize", dismiss);
    return () => {
      window.removeEventListener("pointerdown", dismiss);
      window.removeEventListener("keydown", dismiss);
      window.removeEventListener("resize", dismiss);
    };
  }, [contextMenu]);
  const windowSourceNeedingSelection = previewError?.startsWith("Select the window for")
    ? activeScene?.sources.find((source) => source.kind === "window" && previewError.includes(source.name))
    : undefined;

  useEffect(() => {
    if (!activeScene) return;
    if (!activeScene.sources.some((source) => source.id === activeSource)) {
      const source = [...activeScene.sources].reverse().find((item) => item.visible)
        ?? activeScene.sources[activeScene.sources.length - 1];
      setActiveSource(source?.id ?? null);
    }
  }, [activeScene?.id, activeSource]);

  useEffect(() => {
    let disposed = false;
    let running = false;
    let timer = 0;
    let retryDelay = 66;
    let currentUrl: string | null = null;
    setPreviewUrl(null);
    setPreviewError(null);

    async function pullFrame() {
      if (disposed || running || document.visibilityState !== "visible" || !previewSceneId) return;
      if (!previewSources?.some((source) => source.visible)) {
        setPreviewUrl((previous) => {
          if (previous) URL.revokeObjectURL(previous);
          return null;
        });
        void stopPreview(previewSceneId);
        return;
      }
      running = true;
      try {
        const bytes = await getPreviewFrame(previewSceneId);
        if (bytes.byteLength && !disposed) {
          currentUrl = URL.createObjectURL(new Blob([bytes], { type: "image/jpeg" }));
          setPreviewUrl((previous) => {
            if (previous) URL.revokeObjectURL(previous);
            return currentUrl;
          });
        }
        if (!disposed) {
          retryDelay = 66;
          setPreviewError(null);
        }
      } catch (cause) {
        if (!disposed) {
          retryDelay = 1000;
          setPreviewError(String(cause));
          setPreviewUrl((previous) => {
            if (previous) URL.revokeObjectURL(previous);
            return null;
          });
        }
      } finally {
        running = false;
        if (!disposed) timer = window.setTimeout(pullFrame, retryDelay);
      }
    }

    void pullFrame();
    function onVisibilityChange() {
      if (document.visibilityState === "visible") void pullFrame();
      else void stopPreview(previewSceneId);
    }
    document.addEventListener("visibilitychange", onVisibilityChange);
    return () => {
      disposed = true;
      window.clearTimeout(timer);
      document.removeEventListener("visibilitychange", onVisibilityChange);
      if (currentUrl) URL.revokeObjectURL(currentUrl);
      void stopPreview(previewSceneId);
    };
  }, [previewSceneId, previewSources]);

  return (
    <div className="app-shell" onContextMenu={(event) => event.preventDefault()}>
      <header className="app-header">
        <div className="app-header-start" data-tauri-drag-region>
          <span className="wordmark" data-tauri-drag-region>
            <span className="wordmark-icon">
              <Logo />
            </span>
            Arcalabs
          </span>
        </div>
        <div className="app-header-status">
          <span className="status-dot status-idle" aria-hidden />
          <span>Offline</span>
        </div>
        <div className="app-header-end">
          <WindowControls />
        </div>
      </header>

      <main className="app-main">
        <section className="preview-stage" aria-label="Live scene preview">
          <div className="preview-canvas">
            {previewUrl ? (
              <img className="preview-frame" src={previewUrl} alt={`Live preview of ${activeScene?.name ?? "active scene"}`} />
            ) : (
            <div className="preview-empty" role={previewError ? "alert" : undefined}>
              <strong>{previewError ? "Preview unavailable" : activeScene?.sources.some((source) => source.visible) ? "Starting preview…" : "Add a source to preview this scene"}</strong>
              {previewError ? <span>{previewError}</span> : null}
              {activeScene && windowSourceNeedingSelection ? (
                <button type="button" className="scene-add preview-reselect" onClick={() => void reselectWindow(activeScene.id, windowSourceNeedingSelection.id)} disabled={windowLoading}>
                  <AppWindow size={14} /> {windowLoading ? "Opening picker…" : `Select window for ${windowSourceNeedingSelection.name}`}
                </button>
              ) : null}
            </div>
            )}
          </div>
        </section>
        <section
          className="bottom-row"
          aria-label="Streaming workspace"
        >
          <div className="control-boxes">
            <PopCard
              title="Scenes"
              detail={sceneCollection ? `${sceneCollection.scenes.length} scene${sceneCollection.scenes.length === 1 ? "" : "s"}` : ""}
              open={scenesOpen}
              onToggle={() => setScenesOpen((open) => !open)}
            >
              {sceneLoading ? <p className="dock-note">Loading scenes…</p> : null}
              <ul className="pop-card-list">
                {(sceneCollection?.scenes ?? []).map((scene, index) => (
                  <li key={scene.id} className="scene-row" onContextMenu={(event) => openContextMenu(event, { kind: "scene", id: scene.id })}>
                    {sceneEditor?.id === scene.id ? (
                      <form className="inline-editor" onSubmit={(event) => { event.preventDefault(); void saveScene(); }}>
                        <input aria-label="Scene name" autoFocus maxLength={64} value={sceneEditor.name} onChange={(event) => setSceneEditor({ ...sceneEditor, name: event.target.value })} />
                        <button type="submit" className="inline-action primary" disabled={!sceneEditor.name.trim()}>Save</button>
                        <button type="button" className="inline-action" onClick={() => setSceneEditor(null)}>Cancel</button>
                      </form>
                    ) : deleteConfirm?.kind === "scene" && deleteConfirm.id === scene.id ? (
                      <div className="inline-confirm" role="group" aria-label={`Delete ${scene.name}`}>
                        <span>Delete {scene.name} and its sources?</span>
                        <button type="button" className="inline-action danger" onClick={() => void confirmDelete()}>Delete</button>
                        <button type="button" className="inline-action" onClick={() => setDeleteConfirm(null)}>Cancel</button>
                      </div>
                    ) : (
                      <>
                        <button
                          type="button"
                          className={`pop-card-item${sceneCollection?.activeSceneId === scene.id ? " selected" : ""}`}
                          aria-pressed={sceneCollection?.activeSceneId === scene.id}
                          onClick={() => { setDeleteConfirm(null); void updateScenes(setActiveScene(scene.id)); }}
                        >
                          <span>{scene.name}</span>
                          {sceneCollection?.activeSceneId === scene.id ? <span className="scene-active-label">Active</span> : null}
                        </button>
                        <div className="scene-actions">
                          <button type="button" aria-label={`Move ${scene.name} up`} disabled={index === 0} onClick={() => void updateScenes(reorderScene(scene.id, index - 1))}><ChevronUp size={14} /></button>
                          <button type="button" aria-label={`Move ${scene.name} down`} disabled={index === (sceneCollection?.scenes.length ?? 1) - 1} onClick={() => void updateScenes(reorderScene(scene.id, index + 1))}><ChevronDownIcon size={14} /></button>
                          <button type="button" aria-label={`Rename ${scene.name}`} onClick={() => { setDeleteConfirm(null); setSceneEditor({ id: scene.id, name: scene.name }); }}><Pencil size={13} /></button>
                          <button type="button" aria-label={`Delete ${scene.name}`} disabled={(sceneCollection?.scenes.length ?? 0) <= 1} onClick={() => setDeleteConfirm({ kind: "scene", id: scene.id, name: scene.name })}><Trash2 size={13} /></button>
                        </div>
                      </>
                    )}
                  </li>
                ))}
              </ul>
              {sceneEditor?.id === null ? (
                <form className="inline-editor scene-create" onSubmit={(event) => { event.preventDefault(); void saveScene(); }}>
                  <input aria-label="New scene name" placeholder="Scene name" autoFocus maxLength={64} value={sceneEditor.name} onChange={(event) => setSceneEditor({ ...sceneEditor, name: event.target.value })} />
                  <button type="submit" className="inline-action primary" disabled={!sceneEditor.name.trim()}>Add</button>
                  <button type="button" className="inline-action" onClick={() => setSceneEditor(null)}>Cancel</button>
                </form>
              ) : (
                <button type="button" className="scene-add" onClick={() => { setDeleteConfirm(null); setSceneEditor({ id: null, name: "" }); }}><Plus size={14} /> Add scene</button>
              )}
              {sceneError ? <p className="audio-error" role="alert">{sceneError}</p> : null}
            </PopCard>

            <PopCard
              title="Sources"
              detail={`${activeScene?.sources.length ?? 0} source${activeScene?.sources.length === 1 ? "" : "s"}`}
              open={sourcesOpen}
              onToggle={() => setSourcesOpen((open) => !open)}
            >
              {activeScene && activeScene.sources.length > 0 ? (
                <ul className="pop-card-list source-list">
                {activeScene.sources.map((src, index) => (
                  <li key={src.id} className="source-row" onContextMenu={(event) => openContextMenu(event, { kind: "source", id: src.id, sceneId: activeScene.id })}>
                    {sourceEditor?.id === src.id ? (
                      <form className="inline-editor source-editor" onSubmit={(event) => { event.preventDefault(); void saveSource(); }}>
                        <input aria-label="Source name" autoFocus maxLength={64} value={sourceEditor.name} onChange={(event) => setSourceEditor({ ...sourceEditor, name: event.target.value })} />
                        <button type="submit" className="inline-action primary" disabled={!sourceEditor.name.trim()}>Save</button>
                        <button type="button" className="inline-action" onClick={() => setSourceEditor(null)}>Cancel</button>
                      </form>
                    ) : deleteConfirm?.kind === "source" && deleteConfirm.id === src.id ? (
                      <div className="inline-confirm" role="group" aria-label={`Remove ${src.name}`}>
                        <span>Remove {src.name} from this scene?</span>
                        <button type="button" className="inline-action danger" onClick={() => void confirmDelete()}>Remove</button>
                        <button type="button" className="inline-action" onClick={() => setDeleteConfirm(null)}>Cancel</button>
                      </div>
                    ) : (
                      <>
                        <button
                          type="button"
                          className={`pop-card-item source-item${activeSource === src.id ? " selected" : ""}${src.visible ? "" : " source-hidden"}`}
                          aria-pressed={activeSource === src.id}
                          onClick={() => setActiveSource(src.id)}
                        >
                          <span>{src.kind === "window" ? <AppWindow size={15} /> : <Monitor size={15} />}{src.name}</span>
                        </button>
                        <div className="source-actions">
                          <button type="button" aria-label={src.visible ? `Hide ${src.name}` : `Show ${src.name}`} aria-pressed={src.visible} onClick={() => {
                            void setSceneSourceVisibility(activeScene.id, src.id, !src.visible).then(setSceneCollection).catch((cause) => setSourceError(String(cause)));
                          }}>{src.visible ? <Eye size={14} /> : <EyeOff size={14} />}</button>
                          <button type="button" aria-label={index === 0 ? `Move ${src.name} down` : `Move ${src.name} up`} disabled={activeScene.sources.length < 2} onClick={() => void reorderSceneSource(activeScene.id, src.id, index === 0 ? index + 1 : index - 1).then(setSceneCollection).catch((cause) => setSourceError(String(cause)))}>{index === 0 ? <ChevronDownIcon size={14} /> : <ChevronUp size={14} />}</button>
                          <button type="button" aria-label={`Remove ${src.name}`} onClick={() => setDeleteConfirm({ kind: "source", id: src.id, name: src.name, sceneId: activeScene.id })}><Trash2 size={13} /></button>
                          <button type="button" aria-label={`Settings for ${src.name}`} onClick={() => { setSourceError(null); setSourceSettings({ id: src.id, sceneId: activeScene.id, name: src.name }); }}><Settings2 size={13} /></button>
                        </div>
                      </>
                    )}
                  </li>
                ))}
                </ul>
              ) : <div className="panel-empty"><Monitor size={20} /><strong>No sources yet</strong><span>Add a display or choose a window or game.</span></div>}
              <div className="source-add-row">
                <button type="button" className="scene-add" onClick={() => void openDisplayPicker()} disabled={!activeScene || displayLoading}><Plus size={14} /> Display</button>
                <button type="button" className="scene-add" onClick={() => void addWindowCapture()} disabled={!activeScene || windowLoading}><Plus size={14} /> Window or game</button>
                {selectedSource?.kind === "window" ? <button type="button" className="scene-add" onClick={() => void reselectWindow(activeScene!.id, selectedSource.id)} disabled={windowLoading}><AppWindow size={14} /> Select window</button> : null}
              </div>
              {sourceError ? <p className="audio-error" role="alert">{sourceError}</p> : null}
            </PopCard>

            <PopCard
              title="Audio"
              detail="Mixer"
              open={audioOpen}
              onToggle={() => setAudioOpen((open) => !open)}
            >
              <div className="audio-pop-body">
                <p className="dock-note">Microphone and app levels are read locally. Streaming capture is not connected yet.</p>
                {audioTracks.length === 0 ? (
                  <p className="audio-empty">Add a microphone, desktop audio, or an app.</p>
                ) : (
                  audioTracks.map((track) => (
                    <AudioChannel
                      key={track.id}
                      track={track}
                      inputLevel={audioLevels[track.id]}
                      onContextMenu={(event) => openContextMenu(event, { kind: "audio", id: track.id })}
                      settingsOpen={audioSettingsId === track.id}
                      onToggleSettings={() =>
                        setAudioSettingsId((id) =>
                          id === track.id ? null : track.id,
                        )
                      }
                        onMute={() => void updateAudioTrack(track, { muted: !track.muted })}
                        onVolume={(volume) => void updateAudioTrack(track, { volume })}
                        onRemove={() => void deleteAudioTrack(track.id)}
                    />
                  ))
                )}
                <div className="audio-add-row three">
                  <button
                    type="button"
                    className="audio-add"
                    onClick={() => void openAudioPicker("mic")}
                  >
                    <Plus size={14} strokeWidth={1.5} />
                    Mic/Aux
                  </button>
                  <button
                    type="button"
                    className="audio-add"
                    onClick={() => void openAudioPicker("desktop")}
                  >
                    <Plus size={14} strokeWidth={1.5} />
                    Desktop
                  </button>
                  <button
                    type="button"
                    className="audio-add"
                    onClick={() => void openAudioPicker("application")}
                  >
                    <AppWindow size={14} strokeWidth={1.5} />
                    App
                  </button>
                </div>
                {audioNotice ? <p className="audio-notice" role="status">{audioNotice}</p> : null}
                {audioError && !audioPickerKind ? <p className="audio-error" role="alert">{audioError}</p> : null}
              </div>
            </PopCard>
          </div>

          <div className="go-live-slot">
            <div className="workspace-actions">
              <button
                type="button"
                className="btn-go-live"
                disabled
                title="Streaming is not available in v0.1"
                aria-describedby="go-live-unavailable"
              >
                Go Live
              </button>
              <button
                type="button"
                className="btn-settings"
                onClick={() => setSettingsOpen(true)}
              >
                <Settings size={15} strokeWidth={1.7} />
                Settings
              </button>
            </div>
            <span id="go-live-unavailable" className="sr-only">
              Streaming is not available in this prototype.
            </span>
            {error ? (
              <p className="form-error go-live-error" role="alert">
                {error}
              </p>
            ) : null}
          </div>
        </section>
      </main>

      <footer className="status-bar">
        <span className="status-bar-left">Offline</span>
        <span className="status-bar-right">Live metrics appear when streaming is available.</span>
      </footer>

      {contextMenu && (contextScene || contextSource || contextAudioTrack) ? (
        <div
          className="row-context-menu"
          data-row-context-menu
          role="menu"
          aria-label={`Actions for ${contextScene?.name ?? contextSource?.name ?? contextAudioTrack?.label}`}
          style={{
            left: Math.max(8, Math.min(contextMenu.x, window.innerWidth - 224)),
            top: Math.max(8, Math.min(contextMenu.y, window.innerHeight - 300)),
          }}
        >
          {contextMenu.kind === "scene" && contextScene ? (
            <>
              {sceneCollection?.activeSceneId !== contextScene.id ? <button type="button" role="menuitem" onClick={() => {
                void updateScenes(setActiveScene(contextScene.id));
                setContextMenu(null);
              }}>Make active</button> : null}
              <button type="button" role="menuitem" onClick={() => {
                setSceneEditor({ id: contextScene.id, name: contextScene.name });
                setContextMenu(null);
              }}>Rename</button>
              {contextSceneIndex > 0 ? <button type="button" role="menuitem" onClick={() => {
                void updateScenes(reorderScene(contextScene.id, contextSceneIndex - 1));
                setContextMenu(null);
              }}>Move up</button> : null}
              {sceneCollection && contextSceneIndex < sceneCollection.scenes.length - 1 ? <button type="button" role="menuitem" onClick={() => {
                void updateScenes(reorderScene(contextScene.id, contextSceneIndex + 1));
                setContextMenu(null);
              }}>Move down</button> : null}
              <div className="row-context-divider" />
              <button type="button" role="menuitem" className="danger" disabled={(sceneCollection?.scenes.length ?? 0) <= 1} onClick={() => {
                setDeleteConfirm({ kind: "scene", id: contextScene.id, name: contextScene.name });
                setContextMenu(null);
              }}>Remove</button>
            </>
          ) : null}
          {contextMenu.kind === "source" && contextSource ? (
            <>
              <button type="button" role="menuitem" onClick={() => {
                setContextMenu(null);
                setSourceEditor({ id: contextSource.id, sceneId: contextMenu.sceneId, name: contextSource.name });
              }}>Rename</button>
              <button type="button" role="menuitem" onClick={() => {
                setSourceSettings({ id: contextSource.id, sceneId: contextMenu.sceneId, name: contextSource.name });
                setContextMenu(null);
              }}>Settings</button>
              <div className="row-context-divider" />
              <button type="button" role="menuitem" onClick={() => {
                void setSceneSourceVisibility(contextMenu.sceneId, contextSource.id, !contextSource.visible)
                  .then(setSceneCollection)
                  .catch((cause) => setSourceError(String(cause)));
                setContextMenu(null);
              }}>{contextSource.visible ? "Hide" : "Show"}</button>
              {contextSourceIndex > 0 ? <button type="button" role="menuitem" onClick={() => {
                void reorderSceneSource(contextMenu.sceneId, contextSource.id, contextSourceIndex - 1)
                  .then(setSceneCollection)
                  .catch((cause) => setSourceError(String(cause)));
                setContextMenu(null);
              }}>Move up</button> : null}
              {contextSourceScene && contextSourceIndex >= 0 && contextSourceIndex < contextSourceScene.sources.length - 1 ? <button type="button" role="menuitem" onClick={() => {
                void reorderSceneSource(contextMenu.sceneId, contextSource.id, contextSourceIndex + 1)
                  .then(setSceneCollection)
                  .catch((cause) => setSourceError(String(cause)));
                setContextMenu(null);
              }}>Move down</button> : null}
              {contextSource.kind === "window" ? <button type="button" role="menuitem" onClick={() => {
                void reselectWindow(contextMenu.sceneId, contextSource.id);
                setContextMenu(null);
              }}>Select window</button> : null}
              <div className="row-context-divider" />
              <button type="button" role="menuitem" className="danger" onClick={() => {
                setDeleteConfirm({ kind: "source", id: contextSource.id, name: contextSource.name, sceneId: contextMenu.sceneId });
                setContextMenu(null);
              }}>Remove</button>
            </>
          ) : null}
          {contextMenu.kind === "audio" && contextAudioTrack ? (
            <>
              <button type="button" role="menuitem" onClick={() => {
                void updateAudioTrack(contextAudioTrack, { muted: !contextAudioTrack.muted });
                setContextMenu(null);
              }}>{contextAudioTrack.muted ? "Unmute" : "Mute"}</button>
              <button type="button" role="menuitem" onClick={() => {
                setAudioSettingsId(contextAudioTrack.id);
                setContextMenu(null);
              }}>Settings</button>
              <div className="row-context-divider" />
              <button type="button" role="menuitem" className="danger" onClick={() => {
                void deleteAudioTrack(contextAudioTrack.id);
                setContextMenu(null);
              }}>Remove</button>
            </>
          ) : null}
        </div>
      ) : null}

      {sourceSettings && sourceSettingsItem ? (
        <div className="source-settings-overlay" onMouseDown={(event) => {
          if (event.target === event.currentTarget) setSourceSettings(null);
        }}>
          <section className="source-settings-dialog" role="dialog" aria-modal="true" aria-labelledby="source-settings-title">
            <div className="audio-picker-head">
              <div>
                <h2 id="source-settings-title">Source settings</h2>
                <p>{sourceSettingsItem.kind === "window" ? "Window capture" : "Display capture"}</p>
              </div>
              <button type="button" className="audio-picker-close" onClick={() => setSourceSettings(null)} aria-label="Close">×</button>
            </div>
            <label className="source-settings-name">
              Source name
              <input autoFocus maxLength={64} value={sourceSettings.name} onChange={(event) => setSourceSettings({ ...sourceSettings, name: event.target.value })} />
            </label>
            {sourceSettingsItem.kind === "window" ? (
              <div className="source-settings-target">
                <span>Window target</span>
                <button type="button" className="btn-quiet" onClick={() => void reselectWindow(sourceSettings.sceneId, sourceSettings.id)} disabled={windowLoading}>
                  {windowLoading ? "Opening picker…" : "Select window"}
                </button>
              </div>
            ) : null}
            {sourceError ? <p className="audio-picker-error" role="alert">{sourceError}</p> : null}
            <div className="audio-picker-footer">
              <span>Source settings stay on this PC.</span>
              <div className="source-settings-actions">
                <button type="button" className="btn-quiet" onClick={() => setSourceSettings(null)}>Cancel</button>
                <button type="button" className="inline-action primary" onClick={() => void saveSourceSettings()} disabled={!sourceSettings.name.trim()}>Save</button>
              </div>
            </div>
          </section>
        </div>
      ) : null}

      {settingsOpen ? (
        <SettingsShell initialPage="stream" onDone={closeSettings} />
      ) : null}
      {audioPickerKind ? (
        <div className="audio-picker-overlay" onMouseDown={(event) => {
          if (event.target === event.currentTarget) setAudioPickerKind(null);
        }}>
          <section className="audio-picker" role="dialog" aria-modal="true" aria-labelledby="audio-picker-title">
            <div className="audio-picker-head">
              <div>
                <h2 id="audio-picker-title">{audioPickerKind === "mic" ? "Choose a microphone" : "Choose an app"}</h2>
                <p>{audioPickerKind === "mic" ? "Connected recording devices" : "Apps currently producing audio"}</p>
              </div>
              <button type="button" className="audio-picker-close" onClick={() => setAudioPickerKind(null)} aria-label="Close" autoFocus>×</button>
            </div>
            {audioLoading ? <p className="audio-picker-state" role="status">Scanning audio sources…</p> : null}
            {audioError ? <p className="audio-picker-error" role="alert">{audioError}</p> : null}
            {!audioLoading && !audioError && audioChoices.length === 0 ? (
              <p className="audio-picker-state">
                {audioPickerKind === "mic" ? "No microphone is connected." : "No app is producing audio right now."}
              </p>
            ) : null}
            {audioChoices.length > 0 ? (
              <ul className="audio-picker-list">
                {audioChoices.map((source) => (
                  <li key={source.id}>
                    <button
                      type="button"
                      onClick={() => addAudioSource(source)}
                      disabled={audioTracks.some((track) => track.id === source.id)}
                    >
                      <span>{source.label}</span>
                      <span>{audioTracks.some((track) => track.id === source.id) ? "Added" : "Add"}</span>
                    </button>
                  </li>
                ))}
              </ul>
            ) : null}
            <div className="audio-picker-footer">
              <span>Audio stays on this PC.</span>
              <button type="button" className="btn-quiet" onClick={() => void refreshAudioChoices(audioPickerKind)} disabled={audioLoading}>Refresh</button>
            </div>
          </section>
        </div>
      ) : null}
      {displayPickerOpen ? (
        <div className="audio-picker-overlay" onMouseDown={(event) => {
          if (event.target === event.currentTarget) setDisplayPickerOpen(false);
        }}>
          <section className="audio-picker" role="dialog" aria-modal="true" aria-labelledby="display-picker-title">
            <div className="audio-picker-head">
              <div>
                <h2 id="display-picker-title">Add a display</h2>
                <p>Choose a connected monitor for this scene.</p>
              </div>
              <button type="button" className="audio-picker-close" onClick={() => setDisplayPickerOpen(false)} aria-label="Close">×</button>
            </div>
            {displayLoading ? <p className="audio-picker-state" role="status">Looking for displays…</p> : null}
            {sourceError ? <p className="audio-picker-error" role="alert">{sourceError}</p> : null}
            {!displayLoading && !sourceError && captureDisplays.length === 0 ? <p className="audio-picker-state">No connected displays were found.</p> : null}
            {captureDisplays.length > 0 ? (
              <ul className="audio-picker-list">
                {captureDisplays.map((display) => (
                  <li key={display.id}>
                    <button type="button" onClick={() => void selectDisplay(display)} disabled={displayLoading}>
                      <span>{display.label} · {display.width} × {display.height}</span>
                      <span>Add</span>
                    </button>
                  </li>
                ))}
              </ul>
            ) : null}
            <div className="audio-picker-footer">
              <span>Display names stay on this PC.</span>
              <button type="button" className="btn-quiet" onClick={() => void openDisplayPicker()} disabled={displayLoading}>Refresh</button>
            </div>
          </section>
        </div>
      ) : null}
    </div>
  );
}

function PopCard({
  title,
  detail,
  open,
  onToggle,
  children,
}: {
  title: string;
  detail: string;
  open: boolean;
  onToggle: () => void;
  children: ReactNode;
}) {
  const reduceMotion = useReducedMotion();
  const spring = reduceMotion
    ? { duration: 0 }
    : { type: "spring" as const, stiffness: 380, damping: 32 };

  return (
    <div className={`pop-card-slot${open ? " open" : ""}`}>
      <motion.div className={`pop-card${open ? " open" : ""}`} transition={spring}>
        <button
          type="button"
          className="pop-card-bar"
          aria-expanded={open}
          onClick={onToggle}
        >
          <span className="pop-card-title">{title}</span>
          <span className="pop-card-detail">{detail}</span>
          <motion.span
            className="pop-card-chevron"
            animate={{ rotate: open ? 180 : 0 }}
            transition={spring}
          >
            <ChevronDown size={18} strokeWidth={1.5} />
          </motion.span>
        </button>

        <AnimatePresence initial={false}>
          {open ? (
            <motion.div
              key="body"
              className="pop-card-panel"
              initial={{ height: 0, opacity: 0 }}
              animate={{ height: "auto", opacity: 1 }}
              exit={{ height: 0, opacity: 0 }}
              transition={spring}
            >
              <div className="pop-card-panel-inner">{children}</div>
            </motion.div>
          ) : null}
        </AnimatePresence>
      </motion.div>
    </div>
  );
}

function AudioChannel({
  track,
  inputLevel,
  settingsOpen,
  onContextMenu,
  onToggleSettings,
  onMute,
  onVolume,
  onRemove,
}: {
  track: AudioTrack;
  inputLevel?: AudioInputLevel;
  settingsOpen: boolean;
  onContextMenu: (event: ReactMouseEvent<HTMLDivElement>) => void;
  onToggleSettings: () => void;
  onMute: () => void;
  onVolume: (value: number) => void;
  onRemove: () => void;
}) {
  const title = track.label;
  const MuteIcon =
    track.kind === "application"
      ? track.muted
        ? VolumeX
        : AppWindow
      : track.kind === "desktop"
        ? track.muted
          ? VolumeX
          : Volume2
        : track.muted
          ? MicOff
          : Mic;

  return (
    <div className={`audio-channel${track.muted ? " is-muted" : ""}`} onContextMenu={onContextMenu}>
      <button
        type="button"
        className={`audio-mute${track.muted ? " active" : ""}`}
        aria-pressed={track.muted}
        aria-label={track.muted ? `Unmute ${title}` : `Mute ${title}`}
        title={track.muted ? "Unmute" : "Mute"}
        onClick={onMute}
      >
        <MuteIcon size={15} strokeWidth={1.5} />
      </button>
      <div className="audio-channel-main">
        <div className="audio-channel-meta">
          <span className="audio-label">
            <span className="audio-kind">{kindLabel(track.kind)}</span>
            {title}
          </span>
          <span className="audio-volume-value tabular">
            {Math.round(track.volume * 100)}%
          </span>
          <span className="audio-actions">
            <button
              type="button"
              className={`audio-gear${settingsOpen ? " active" : ""}`}
              aria-expanded={settingsOpen}
              aria-label={`Settings for ${title}`}
              title="Audio settings"
              onClick={onToggleSettings}
            >
              <Settings2 size={13} strokeWidth={1.5} />
            </button>
            <button
              type="button"
              className="audio-remove"
              aria-label={`Delete ${title}`}
              title="Delete"
              onClick={onRemove}
            >
              <Trash2 size={13} strokeWidth={1.5} />
            </button>
          </span>
        </div>
        {track.kind === "mic" || track.kind === "application" ? (
          inputLevel?.error ? (
            <p className="audio-meter-error" role="status">Level unavailable: {inputLevel.error}</p>
          ) : (
            <div
              className={`audio-meter${inputLevel?.peak == null ? " is-waiting" : ""}`}
              role="meter"
              aria-label={`${track.kind === "application" ? "Application" : "Microphone"} input level for ${title}`}
              aria-valuemin={0}
              aria-valuemax={100}
              aria-valuenow={inputLevel?.peak == null ? undefined : Math.round(inputLevel.peak * 100)}
              aria-valuetext={inputLevel?.peak == null ? "Waiting for microphone input" : `${Math.round(inputLevel.peak * 100)} percent`}
            >
              <span style={{ width: `${Math.round((inputLevel?.peak ?? 0) * 100)}%` }} />
            </div>
          )
        ) : null}
        <label className="audio-combo">
          <span className="sr-only">Volume for {title}</span>
          <span className="audio-combo-track" aria-hidden>
            <span
              className="audio-combo-volume"
              style={{ width: `${Math.round(track.volume * 100)}%` }}
            />
          </span>
          <input
            type="range"
            min={0}
            max={100}
            value={Math.round(track.volume * 100)}
            disabled={track.muted}
            onChange={(e) => onVolume(Number(e.target.value) / 100)}
          />
        </label>
        {settingsOpen ? (
          <p className="audio-device-note">Input level is read locally while Audio is open.</p>
        ) : null}
      </div>
    </div>
  );
}
