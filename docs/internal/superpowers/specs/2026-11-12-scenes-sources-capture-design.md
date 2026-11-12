# Arcalabs v1 — Scenes, Sources, Capture, and Preview

| Field | Value |
|---|---|
| Status | **Proposed native-preview revision; awaiting user review** |
| Scope | First implementation milestone toward desktop v1.0 |
| Platform | Windows 10/11 desktop |
| Related | [`apps/desktop/SPEC.md`](../../../../apps/desktop/SPEC.md), [`2026-11-12-arcalabs-system-design.md`](2026-11-12-arcalabs-system-design.md) |

## 1. Goal

Turn Scenes and Sources into a usable workspace for arranging scenes and selecting display or window/game targets. Keep capture, scene state, and composition in Rust; show a real local preview before a later milestone connects the scene output to streaming.

This is the first vertical slice toward v1.0. It does not start a stream or create a local recording.

## 2. Current state and constraints

- The current scene names and screen/webcam source rows in `MainWindow.tsx` are sample UI state, not persisted objects.
- The audio work has local device/session discovery and in-memory mixer controls; it does not yet capture PCM or feed a stream.
- Go Live is disabled. The desktop app makes no network calls and collects no user data.
- `apps/desktop/SPEC.md` currently excludes video capture. This design and its feature spec were approved before product implementation.
- Reuse the existing `windows` crate and Tauri APIs. Do not add dependencies in this milestone.
- Preserve existing local settings and OS-keychain storage for stream keys. Scene/source data must never contain a stream key.

## 3. Design decisions

### 3.1 Rust-owned scene model

Rust owns the authoritative collection and exposes narrow Tauri commands for listing, creating, renaming, selecting, deleting, and ordering scenes and sources. The React layer renders responses and sends user actions; it does not become the persistence or media-state owner.

Each scene has a stable ID, name, and ordered source list. A source has a stable ID, type, display label, visibility, and canvas transform (position, size, and z-order). The initial scene uses a 1920×1080 canvas. The first source fills the canvas by default; added sources can be moved and resized in the preview. Scenes and source metadata are stored locally in the app config directory using a versioned JSON format and safe replacement writes.

Persist display identity so a display source can be reconnected. Do not persist transient HWNDs, process IDs, or full executable paths. A window source keeps its scene entry and user-facing label but requires the user to select its target again after app restart. No window titles or process names are written to logs.

### 3.2 Capture target selection

- **Window/game source:** Windows Graphics Capture. The user starts selection through the Windows system picker; retain the selected capture item only in runtime memory.
- **Display source:** DXGI Desktop Duplication. Enumerate active displays, capture the selected output, and recreate the duplication session after `DXGI_ERROR_ACCESS_LOST`.
- Keep capture on dedicated Rust workers and stop workers when their source is removed, their target closes, or the app exits.

These Windows APIs provide frame capture from windows/displays and GPU-backed desktop surfaces: [Windows Graphics Capture](https://learn.microsoft.com/en-us/windows/uwp/audio-video-camera/screen-capture), [DXGI Desktop Duplication](https://learn.microsoft.com/en-us/windows/win32/direct3ddxgi/desktop-dup-api).

### 3.3 Compositor and preview

Rust composites visible sources in z-order on the scene canvas. Source transforms are applied in Rust so the same scene model can later feed the encoder without duplicating layout rules in TypeScript.

The primary preview uses a native Windows child surface and a D3D11 swap chain, composed and presented by Rust in the same process as capture. The WebView sends only the preview area's physical bounds, DPI scale, and visibility; image pixels never pass through React, Tauri IPC, JPEG encoding, or WebView canvas rendering. A `ResizeObserver` keeps the native surface aligned with the preview area as the window or display scale changes.

Capture textures are kept GPU-side and composited on the capture device when compatible. The native surface presents the newest completed composition on the next available refresh, up to 60 fps, with at most one pending frame. Source aspect ratio is preserved and centered inside its scene bounds; unused space remains black. The 1920×1080 scene coordinate system remains independent of the preview's physical display size.

If the native surface or GPU path is unsupported, retain the current in-memory JPEG preview as a fallback capped at 1280×720 and 15 fps. The fallback keeps only the newest frame and re-encodes only when a capture source produces a new frame. The preview is not stream input; the future encoder receives its own full-resolution scene output.

### 3.4 User-visible behavior

- Scenes: create, select, rename, reorder, and delete. The final remaining scene cannot be deleted.
- Sources: add a display or choose a window/game, reorder, hide/show, and remove.
- The selected scene is restored on launch. Its display sources reconnect when possible; window sources prompt for target selection again.
- Capture runs only for sources in the selected scene while the preview workspace is visible. Switching scenes stops the old scene capture before starting the new one.
- Go Live remains disabled. No local recording, audio capture, chat, or user login is added here.

## 4. Data flow

This is the local capture path; it makes no network connection.

```text
Scenes/Sources controls
        │ Tauri commands
        ▼
Rust SceneStore ──────► local versioned scene config
        │
        ├── Window source ─► Windows Graphics Capture ─┐
        └── Display source ─► DXGI duplication ────────┤
                                                      ▼
                                            Rust D3D11 compositor
                                               │           │
                                  full-resolution GPU   latest scene texture
                                  output for future      bounded to one frame
                                  encoder milestone          │
                                                           ▼
                                               Native child surface + swap chain
                                               WebView sends bounds/visibility only
```

No frame is written to disk. No scene, source, device, application, or preview data leaves the device. The only planned future network connection is a user-started stream to the selected service.

## 5. Failure handling

| Condition | Behavior |
|---|---|
| User cancels the Windows window picker | Keep the scene unchanged; close the picker without an error. |
| Capture API unsupported or denied | Show a clear message and keep the source unstarted. |
| Window closes or its runtime capture item is lost | Stop that worker and mark the source unavailable; let the user reselect it. |
| Display disconnects or DXGI access is lost | Stop stale frames, retry display duplication once after re-enumeration, then show the source as unavailable if it still fails. |
| Protected content or game capture returns black frames | Show the received preview as-is; provide a capture troubleshooting hint without claiming to detect protected content. |
| Config is missing or invalid | Start with a default scene without overwriting the invalid file; surface a recoverable settings error. |
| Preview consumer slows or stops | Replace the latest frame; never queue unbounded frames. |
| Native D3D surface or GPU interop is unavailable | Show an actionable status and use the bounded JPEG fallback; do not stall scene controls. |
| Preview region moves, resizes, or crosses displays with different DPI | Update the child surface bounds using physical pixels and the current scale factor. |
| Capture or compositor fails | Stop the affected worker, retain editable scene state, and show an actionable UI error. |

## 6. Milestones to v1.0

1. **Scenes, sources, capture, and preview** — this milestone; no streaming.
3. **Audio capture and mix** — WASAPI microphone, desktop loopback, and active app loopback; in-memory only and coordinated with stream state.
4. **Encoding** — Rust-owned video/audio encode adapters, hardware encoder selection, software fallback, timestamps, bounded queues, and measured resource use.
5. **Stream lifecycle and RTMP** — start/stop/error state, platform ingest URL and OS-keychain key, real bitrate/drop metrics, and teardown safety.
6. **v1 readiness** — optimizer behavior, hotkeys, onboarding, crash-log quality, installer/update decision, accessibility, release checks, and profiling against measured targets.

No release date or unmeasured performance claim is implied. The existing architecture note's preview-transport and compositor open items should be updated only after this design is accepted and validated against a working build.

## 7. Review

The feature-level acceptance criteria are in [`apps/desktop/specs/scenes-capture/SPEC.md`](../../../../apps/desktop/specs/scenes-capture/SPEC.md). This native-preview revision and its performance criteria require user review before implementation planning. Keep all work local until the user approves a push.


