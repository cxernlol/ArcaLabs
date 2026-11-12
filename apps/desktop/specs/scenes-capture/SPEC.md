# SPEC: Scenes, Sources, Capture, and Preview

**Status: Proposed native-preview revision; awaiting user review.**

## 1. Goal & User Story

A Windows streamer can organize a live layout into scenes and add real display or game/window sources. They can see the selected scene locally before a later milestone connects the scene output to streaming.

## 2. Inputs & Outputs

**Inputs**

- Scene actions: create, select, rename, reorder, and delete by stable scene ID.
- Source actions: add a display or select a window, rename, reorder, show/hide, and remove by stable source ID.
- Windows Graphics Capture items for windows and DXGI outputs for displays.
- Scene canvas dimensions: 1920×1080. Source bounds are normalized to the canvas and retained locally.
- The scene canvas is 1920×1080. The preview follows the capture source cadence up to 60 fps and presents directly in a native Windows surface positioned over the preview area.
- Network calls: none except a future user-started stream connection. This feature makes no network calls.

**Outputs**

- Rust-owned scene/source collection exposed through Tauri commands; React presents state and forwards user actions.
- Versioned scene data in the local app config directory. Writes replace the previous file safely; invalid existing data is preserved and reported rather than overwritten.
- Display capture through DXGI Desktop Duplication and window/game capture through Windows Graphics Capture's system picker.
- Rust composes visible sources in z-order. Each source preserves its capture aspect ratio, centered inside its scene bounds with black letterboxing as needed. The first source fills the canvas; later sources begin at one-quarter canvas width, anchored bottom-right. The preview has no drag or resize handles.
- A Rust-owned D3D11 composition surface rendered through a native child window and swap chain. Captured frames remain GPU-side where the capture device permits; React sends only the preview area's bounds and visibility, never frame pixels. The renderer retains at most the newest frame and presents on the next available refresh, up to 60 fps.
- Persist display identity. Do not persist HWNDs, process IDs, full executable paths, or window titles. Window sources remain in the scene but must be reselected after restarting Arcalabs; user-edited labels remain local.
- Capture runs only for the selected scene while the workspace is visible. Switching scenes stops the previous scene's preview workers; hiding, minimizing, or closing the workspace stops preview capture.
- Network calls: none except the future stream connection.

## 3. Acceptance Criteria

- [ ] First launch creates one `Main` scene with no sources; scene data remains after closing and reopening the app.
- [ ] Users can create, select, rename, and reorder scenes. The final scene cannot be deleted; deleting another scene selects a valid remaining scene.
- [ ] Users can add a display source from currently available displays and a window source through the Windows system picker. Canceling the picker changes nothing.
- [ ] Users can rename, reorder, show/hide, and remove sources; changes persist after restart.
- [ ] Adding the first source fits it inside the centered 1920×1080 canvas without distorting its aspect ratio; unused space stays black. Later sources appear at one-quarter canvas width in the bottom-right. The preview exposes no drag or resize controls.
- [ ] The selected scene uses 1920×1080 normalized layout coordinates. Preview composition and presentation remain in Rust/D3D11; no JPEG encoding, per-frame Tauri IPC, or frame polling occurs in the WebView.
- [ ] On a source that supplies 60 fps, the preview presents up to the source cadence with no queued stale frames; the latest available frame is shown on the next available display refresh. Measure and report local capture-to-present timing on the development PC; make no zero-delay claim.
- [ ] The native preview surface tracks the preview area's bounds across resize, minimize/restore, display scaling, and screen changes without covering controls outside the preview region.
- [ ] The window remains usable at 900×600; scene and source labels do not overlap their action controls when the workspace narrows.
- [ ] Preview frames do not queue without bound when the WebView is slow. The newest frame replaces older pending preview data, and unchanged frames are not resent to the WebView.
- [ ] Only sources in the selected scene are captured for preview. Switching scenes stops the previous scene's preview workers; hiding, minimizing, or closing the workspace stops all preview workers.
- [ ] A display source reconnects by its saved display identity when available. A window source asks the user to select its target again after app restart without persisting a window title, HWND, or process ID.
- [ ] Window/display device loss stops stale frames and presents an actionable unavailable state; recovery does not silently select a different target.
- [ ] Scene/source labels, identities, and captured frames remain local and are not written to logs or sent over the network. No frame is written to disk.
- [ ] Go Live remains disabled. This feature does not add local recording, audio capture, webcam capture, chat, login, telemetry, or new dependencies.
- [ ] The UI follows the approved desktop design tokens and remains operable by keyboard with visible focus and reduced-motion support.

## 4. Edge Cases & Error States

| Situation | Behavior |
|---|---|
| User cancels the Windows window picker | Close the picker and leave the scene unchanged. |
| Capture API unsupported or access denied | Keep the source unstarted and explain the Windows requirement or permission action. |
| No displays are available | Show an empty display list with a refresh action. |
| Selected window closes or capture item is lost | Stop its worker, retain the scene/source record, mark it unavailable, and let the user choose the target again. |
| Display disconnects or DXGI access is lost | Stop stale frames, re-enumerate and retry once; if still unavailable, show the source state without switching targets. |
| Game returns black/protected frames | Display received frames as-is and provide a capture troubleshooting hint; do not claim to identify protected content. |
| Scene file is missing | Create the default scene. |
| Scene file is invalid or cannot be written | Preserve the existing file and current in-memory state; show an actionable error. |
| Preview consumer slows or stops | Retain at most the latest frame and stop producing preview frames while the workspace is hidden. |
| Native D3D surface cannot initialize or a source uses an incompatible adapter | Keep the app usable, show an actionable preview-unavailable state, and use the existing bounded JPEG fallback only for the unsupported case. |
| Capture or composition fails | Stop the affected worker, preserve editable scene state, and display the error without crashing the rest of the app. |

## 5. Out of Scope

- RTMP publishing, enabling Go Live, encoders, stream metrics, and stream lifecycle.
- WASAPI capture/mixing, webcam capture, recording, replay buffer, monitoring, filters, transitions, plugins, chat, accounts, or platform login.
- Persisting window/process identifiers or titles.
- macOS/Linux capture backends, website changes, analytics, outbound network calls, or new external dependencies.
- 60 fps capture guarantees on sources or hardware that do not supply or sustain that cadence.
