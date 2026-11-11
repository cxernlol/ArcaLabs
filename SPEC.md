# SPEC: v0.1 — Offline shell (privacy-first, no live)

## 1. Goal & User Story

A streamer opening ArcaLabs should see a polished, offline desktop shell that matches `design.md` and makes the RTMP-only, no-login model obvious. They can pick YouTube / Kick / Twitch, store an ingest URL hint and stream key locally (key in OS keychain), and open Settings — but they cannot go live yet.

## 2. Inputs & Outputs

**Inputs**

- Platform selection: `youtube` | `kick` | `twitch` (segmented control order per `design.md`)
- Optional ingest URL string (plain text, local config)
- Optional stream key string (written only to OS keychain; UI shows masked value after save)
- Settings toggles/fields for Video Automatic vs Manual UI (values stubbed / local only; no real optimize)
- Tauri commands (names locked in implementation to match this list):
  - `get_app_state` → `{ state: "idle", live: false }`
  - `get_stream_destination` / `set_stream_destination` (platform + ingest URL; never returns raw key)
  - `set_stream_key` / `has_stream_key` / `clear_stream_key` (keychain; `has_stream_key` → boolean only)
  - `get_video_settings` / `set_video_settings` (local config stubs)

**Outputs**

- Main window: custom title bar, Offline status, letterboxed preview placeholder, Scenes / Sources / Audio / Stream panels, status bar offline metrics (stub OK)
- Stream panel: platform selector; helper text per platform pointing to dashboard stream settings (RTMP, not OAuth); ingest URL field; masked stream key field; **Optimize for this PC** outlined (may be present but non-functional or disabled with “Coming in a later version”); **Go Live** visible but **disabled** with helper: streaming not available in v0.1
- Settings shell: nav + Video and Stream pages; Automatic/Manual UI layout; Done dismisses settings
- Platform + ingest URL + video stubs persist in app config directory; stream key only in keychain (`arcalabs/stream-key/{platform}`)
- **Network calls: none.** No OAuth, no RTMP publish, no telemetry, no analytics, no crash reporting, no remote logging in the desktop app.

## 3. Acceptance Criteria

- [ ] App launches offline (`pnpm tauri dev` on Windows) with main window matching `design.md` layout tokens (colors, radii, Inter, Lucide only)
- [ ] Header shows grey Offline dot + “Offline”; status bar shows not-streaming / stub metrics
- [ ] Platform segmented control YouTube | Kick | Twitch; selection persists across restart
- [ ] Ingest URL persists per platform in local config; stream key save uses OS keychain only; UI never displays the full key after blur/save (masked); key never appears in logs or config files
- [ ] Helper copy under key field names the selected platform and RTMP/dashboard path (no “Log in with …” wording)
- [ ] Go Live is disabled; clicking does nothing; accessible name/title explains unavailable in v0.1
- [ ] Settings opens with Video + Stream pages; Automatic/Manual UI renders; no outbound network when changing settings
- [ ] No OAuth buttons or flows; no capture/encode/RTMP publish code paths invoked from UI
- [ ] No desktop telemetry / analytics / crash-reporting dependencies added
- [ ] CI still passes (`web` + `rust`); local `scripts/check` still mirrors CI

## 4. Edge Cases & Error States

| Situation | Behavior |
|---|---|
| Empty stream key | Field empty; Go Live stays disabled (already disabled in v0.1) |
| Keychain write fails | Inline error on Stream panel/Settings; previous key unchanged; no crash |
| Switch platform | Shows that platform’s ingest URL + masked key presence; empty if none |
| Settings open while… | Always allowed in v0.1 (never live) |
| Optimize clicked | Disabled or no-op with short “Not in v0.1” — no hardware probe |
| User pastes key then clears | `clear_stream_key` removes keychain entry for that platform |
| Logs enabled locally | Scrub: never write stream key, account names, or user home paths |

## 5. Out of Scope

- Go Live / Stop stream / RTMP publish / live stats / elapsed live timer
- OAuth or any “log in” to YouTube, Kick, or Twitch
- Real Optimize / Undo / hardware detection
- Onboarding wizard
- DXGI / camera capture / real preview frames
- Audio device capture (meters may stay decorative stubs)
- Desktop installer / auto-update
- Any network call from the desktop app
