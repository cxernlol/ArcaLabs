# SPEC: Desktop workspace and live audio sources

**Status: Approved audio workflow and offline microphone/application meters.**

## 1. Goal & User Story
Windows streamers need a clear workspace for preparing and controlling a live stream, including a way to select microphone, desktop, and application audio. Arcalabs captures and mixes selected audio only for a live stream; it never records audio locally or asks users to sign in.

## 2. Inputs & Outputs
**Inputs**
- Windows audio capture endpoints and the default render endpoint, discovered through Core Audio.
- Applications with active audio sessions, with display name and process ID held in memory only.
- Selected source IDs, mixer volume and mute state, and stream lifecycle state.
- Audio device, session, and microphone-permission changes or errors.

**Outputs**
- The main workspace has a real, local preview of the active scene, distinct Scenes, Sources, and Audio panels, a disabled Go Live action, and Settings. Capture and preview requirements are defined in [`specs/scenes-capture/SPEC.md`](specs/scenes-capture/SPEC.md).
- Add Mic/Aux opens a picker of connected microphones. Selecting one adds that microphone as a mixer source.
- Add Desktop adds the default output loopback source directly. The source is not duplicated if already present.
- Add App opens a picker of applications with active audio sessions. The list updates as sessions start or end; selecting one adds its process audio as a source.
- Selected audio sources are captured and mixed in memory only while the stream lifecycle is live. The proposed mixer output is 48 kHz, stereo, float32 PCM supplied to the live-stream sink; no audio buffers are written to disk.
- While offline, each added microphone or application source shows a horizontal input-level meter beneath its volume control. A shared-mode Windows audio capture client reads an added microphone; process-loopback capture reads only the selected application's process tree. Both run only while the Audio panel is open and the app window is visible. Rust reduces each received buffer to a peak value and immediately discards the samples. Meters are independent of mixer mute and volume.
- Meter updates are displayed at 20 Hz. When streaming is later implemented, the meter uses the stream's existing microphone samples instead of opening a second capture client. No PCM is persisted or logged.
- Audio capture uses Windows Core Audio through the existing Rust Windows API dependency; no additional audio dependency is introduced.
- Network calls: none except the stream connection.

## 3. Acceptance Criteria
- [ ] Mic/Aux opens a picker containing the currently connected capture endpoints; selecting an endpoint adds it once to the mixer.
- [ ] Desktop adds the current default render endpoint as a loopback source, and a second click does not create a duplicate.
- [ ] App opens a picker containing only applications with active audio sessions; new and ended sessions are reflected while the picker is open.
- [ ] Selecting an application adds process-scoped loopback audio, excluding unrelated applications and system audio.
- [ ] Volume and mute changes affect that source before the in-memory mix; deleting a source stops capturing it.
- [ ] Each added microphone and application source shows a horizontal input meter beneath its volume control while offline; silence settles to zero and sound moves the meter. Each meter is unaffected by that track's mute and volume controls, and application readings exclude other processes.
- [ ] If the endpoint meter cannot be read, show “Level unavailable” and no fabricated level.
- [ ] Offline microphone and process-loopback capture runs only for microphone/application sources added to the mixer and only while the Audio panel is open and the app is visible; it stops when the panel closes, the app is hidden/minimized, or the source is removed.
- [ ] Meter samples stay in memory only, are reduced to a peak and discarded, and are never written to files, logs, or settings or sent over a network.
- [ ] The stream audio pipeline starts only when streaming becomes live and stops when it ends; offline meter capture is separate and never mixes, monitors, or publishes audio.
- [ ] The complete audio feature is enabled only on Windows versions that support process-loopback capture. On unsupported versions, audio source controls explain the OS requirement and do not start partial capture.
- [ ] The mixer supplies synchronized 48 kHz stereo float32 PCM frames to the live-stream sink; it creates no local audio files or monitoring output.
- [ ] Audio endpoint IDs and application names stay local and are not written to logs. The app makes no telemetry or other network calls.
- [ ] Existing stream keys remain in the OS keychain; existing app settings remain local. Go Live remains disabled until the separate publishing feature exists.
- [ ] The UI uses the approved Obsidian Black (#0B0B0D), Icy White (#F2F4F8), and Electric Violet (#7955FF) palette, with honest unavailable, idle, and error states.
- [ ] No local recording, replay buffer, account login, analytics, telemetry, automatic reporting, or marketing-site changes are added.

## 4. Edge Cases & Error States
| Situation | Behavior |
|---|---|
| No microphone is connected | The Mic/Aux picker says no microphone is available and offers a refresh action. |
| No application has an active audio session | The App picker explains that it lists apps when they produce audio. |
| Windows does not support process-loopback capture | All audio source controls show the OS requirement; no source is captured. |
| Microphone access is denied | Show the Windows permission failure and how to enable access; do not start capture. |
| Microphone access is denied or capture cannot start | Keep the microphone source usable, show the Windows permission or device error, and do not show a fabricated level. |
| A microphone is silent | Show a zero input level without changing the selected device or mixer controls. |
| A selected endpoint is unplugged or an app session ends | Mark that source unavailable and stop its capture; do not silently switch to another device or app. |
| Audio initialization or mixing fails | Stop the affected capture safely, show an actionable error, and keep the rest of the app responsive. |
| Stream stops or fails to start | Stop all audio capture and discard in-memory buffers. |
| App closes unexpectedly | No audio file remains because audio is never persisted. |

## 5. Out of Scope
- RTMP publishing and enabling Go Live; the mixer exposes an in-memory sink for the separate publishing feature. Video capture and preview are scoped in [`specs/scenes-capture/SPEC.md`](specs/scenes-capture/SPEC.md).
- Local recording, replay buffer, audio monitoring/playback, plugins, streaming-platform account login, or user accounts.
- Non-Windows audio backends in this phase.
- Website changes, new external dependencies, or outbound network calls beyond a future user-started stream connection.
