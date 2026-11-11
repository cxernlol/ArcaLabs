# SPEC: P0 — Desktop scaffold

## 1. Goal & User Story

Developers and early testers need a runnable Arcalabs shell that matches the offline main-window mockup so later streaming features have a stable UI host. A casual streamer opening the app should see the familiar layout (preview, four panels, Go Live) even before real capture exists.

## 2. Inputs & Outputs

**Inputs**

- Tauri window creation; no user streaming inputs in P0.

**Outputs**

- Desktop window: custom title bar (~58px), min size ~1100×700, dark `bg.app`, decorations off.
- Main view: preview placeholder, Scenes / Sources / Audio / Stream panels, status bar with stub metrics.
- Tauri command `get_app_state` → `{ state: "idle", live: false }`.
- Event `stream:stats` emitted ~1 Hz with static offline metrics (CPU 4%, 60 fps preview placeholder, 0 kbps, 0 dropped).
- Network calls: none except the stream connection (none in P0).

## 3. Acceptance Criteria

- [ ] `pnpm tauri dev` launches on Windows without CDN asset loads (Inter bundled locally, Lucide via npm).
- [ ] Window matches `design.md` tokens (colors, radii, typography scale).
- [ ] Layout matches mockup structure: header, letterboxed preview, four-panel row, status bar.
- [ ] Header shows wordmark + grey offline dot + "Offline".
- [ ] Stream panel: YouTube | Kick | Twitch segmented control, outlined Optimize (enabled), spec line, purple **Go Live**.
- [ ] `data-tauri-drag-region` on header; window controls work on Windows.
- [ ] No telemetry dependencies added.

## 4. Edge Cases & Error States

| Situation | Behavior |
|-----------|----------|
| Window below 1100px wide | Enforced minimum size; layout not required to reflow to dropdown tabs in P0 |
| Tauri invoke fails | UI shows offline defaults |

## 5. Out of Scope

- Settings routes, onboarding, real capture, optimize logic, keychain, go_live/stop_stream behavior.
- macOS/Linux title bar variants (Windows focus).
- Vitest setup (optional follow-up).
