# SPEC: Settings modal + local crash log

## 1. Goal & User Story

A streamer opens a small Settings popup for Stream / Video / Audio / Hotkeys / General. When the app crashes, a scrubbed local crash log is saved so the product can improve — without storing or sending user secrets or personal data.

## 2. Inputs & Outputs

**Inputs**

- Open: gear beside Offline; Close: **Back**, backdrop, Escape
- Left nav: **Stream**, **Video**, **Audio**, **Hotkeys**, **General**; **Optimize** stub at nav bottom
- Stream / Video: existing local + keychain behavior (no Automatic toggle; GPU hint only)
- Audio: stub default mic + desktop selects → `defaultMicId` / `defaultDesktopId` in app config
- Hotkeys: three read-only placeholder rows (Go Live/Stop, Mute mic, Next scene)
- General: **Start minimized**, **Confirm before Stop stream** toggles; **Copy crash report** button; short note that crash logs stay on device
- Crash capture: Rust panic hook (+ optional frontend `window.onerror` / unhandledrejection → append via Tauri command)
- **Network calls: none except the stream connection** (none in v0.1). No Sentry / telemetry / auto-upload.

**Outputs**

- Settings modal as specified previously
- Crash file: `{app_config_dir}/crash.log` (user profile app data only — never Program Files)
- Each entry includes only: timestamp (UTC), app version, OS descriptor, GPU vendor/name if already known locally, last error message, scrubbed stack
- Scrub always before write: stream key, passwords, `Users\<name>` / home path segments, full ingest URLs (keep host only or redact to `rtmp://[redacted]`), account names
- `copy_crash_report` (or reuse read + clipboard from UI): returns/copies scrubbed file text for the user to paste; app never sends it
- Stream key remains OS keychain only (`arcalabs` / `stream-key/{platform}`)

## 3. Acceptance Criteria

- [ ] Settings modal: nav pages + Optimize stub; Back / backdrop / Esc close
- [ ] Stream + Video behavior as current (keychain; editable video fields; no Automatic card)
- [ ] Audio / Hotkeys / General pages as above; config persists locally
- [ ] Intentional test panic or thrown error appends a scrubbed line/block to `crash.log` under app config dir
- [ ] `crash.log` never contains a stream key or a raw `C:\Users\…` path
- [ ] Copy crash report puts scrubbed text on the clipboard; no network call
- [ ] No crash-reporting / telemetry dependency added
- [ ] Lucide + Inter + `design.md` tokens

## 4. Edge Cases & Error States

| Situation | Behavior |
|---|---|
| Config / crash write fails | Inline error or silent skip for crash hook (never crash-loop); previous config kept |
| Crash file missing | Copy report shows “No crash log yet” |
| Crash file huge | Keep last ~256 KB (trim oldest) |
| Hotkey row clicked | Non-editable in v0.1 |
| Optimize clicked | Disabled / not in v0.1 |

## 5. Out of Scope

- Automatic upload of crash logs to any server or GitHub
- Minidumps / Windows Error Reporting integration
- Real WASAPI enum, OS hotkey registration, live Start-minimized behavior
- Advanced / themes / plugins
- Any network call from the desktop app
