# SPEC: Native Rust streaming desktop

## 1. Goal & User Story

Streamers use one Windows desktop app to compose a scene, capture video and audio, encode with supported hardware, and publish to YouTube, Kick, or Twitch without configuring a WebView or local recording workflow. The app replaces Tauri/WebView2 with a native Rust UI and keeps the full-rate media path independent from UI rendering.

## 2. Inputs & Outputs

**Inputs**

- Scene and source settings, capture target, output resolution, frame rate, and bitrate.
- Selected microphone, desktop loopback, and active application audio sources.
- Platform, RTMPS ingest URL, and stream key from the OS keychain.
- Local hardware and Media Foundation encoder capabilities.
- User actions to configure, start, and stop a stream.

**Outputs**

- Native `egui/eframe` desktop UI, with GPU-resident D3D11 preview presented in a native child window.
- Rust workspace crates: `arca-core`, `arca-capture`, `arca-compose`, `arca-encode`, `arca-audio`, `arca-publish`, and the desktop app.
- Full-rate capture, GPU composition, Media Foundation hardware H.264 encoding, WASAPI audio capture/mix and AAC encoding, FLV mux, and RTMPS publish.
- Local status and `tracing` metrics for capture/encode rates and dropped frames; no desktop telemetry or automatic reporting.
- A temporary MP4 only for developer pipeline validation; tests remove it, and the app has no local recording feature.
- Network calls: none except the stream connection started by the user.

## 3. Acceptance Criteria

- [ ] Windows build 20348+ can capture windows/displays via Windows Graphics Capture and compose sources on the GPU.
- [ ] The preview is presented by a native child window without per-frame CPU readback, and preview work does not throttle stream output.
- [ ] Media Foundation hardware H.264 encoder capability is detected; Go Live is blocked with an actionable error when none is available. No silent software-encoder fallback exists.
- [ ] Capture and encode can sustain a configured 60 fps in the developer pipeline validation; its temporary MP4 is removed after the check.
- [ ] Microphone, desktop loopback, and active-process audio can be captured, mixed, AAC-encoded, and published with video.
- [ ] User-started publishing and stopping work over RTMPS for configured YouTube, Kick, and Twitch destinations; FLV muxing is handled in Rust.
- [ ] Capture, encode, audio, and network work run independently of the UI through bounded queues; stale video frames are discarded under backpressure.
- [ ] One configured frame rate controls capture cadence, encoder time-base, and output cadence.
- [ ] Existing scenes/settings and stream-key behavior are retained or migrated; keys remain in the OS keychain, and local crash logs remain scrubbed.
- [ ] The native UI follows the established ArcaLabs direction and the selected Obsidian Black / Icy White / Acid Lime palette; interactive controls expose clear loading, empty, disabled, live, and error states.
- [ ] The marketing site remains unchanged and builds through its existing CI job; Windows CI builds/checks the Rust workspace.
- [ ] After native feature parity, the shipped desktop no longer depends on Tauri, React, or WebView2.

## 4. Edge Cases & Error States

| Situation | Behavior |
|---|---|
| Windows is below build 20348 or process-loopback is unavailable | Explain that the required Windows audio capability is unavailable; do not claim full audio support. |
| No supported hardware H.264 encoder or driver failure | Keep Go Live disabled or stop startup with a clear actionable error; never silently use software encoding. |
| Capture target closes, becomes inaccessible, or changes size | Stop or reinitialize that source safely, show its state, and keep the app responsive. |
| Capture/encode queue is full | Discard stale video frames and report dropped-frame metrics locally. |
| RTMPS connection, authentication, or ingest fails | Show the failure, stop publishing cleanly, and never log the stream key. |
| Audio device disconnects or access is denied | Mark the affected source unavailable and let the user recover without terminating the app. |
| Native D3D11 preview integration cannot be initialized | Show a clear preview error; do not fall back to CPU frame readback. |
| Config or crash-log write fails | Preserve the previous valid config and continue without crashing; crash logs remain local and scrubbed. |

## 5. Out of Scope

- Local recording, replay buffers, or MP4 export in the product UI.
- Software video-encoder fallback.
- WebView-based preview or full-rate frame delivery to the UI.
- Telemetry, automatic crash uploads, update checks, or any network call other than a user-started stream.
- macOS/Linux desktop support, plugins, or unrelated marketing-site redesign.
