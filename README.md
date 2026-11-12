# ArcaLabs

Privacy-first desktop streaming software for YouTube, Kick, and Twitch. ArcaLabs is built with Rust and Tauri, with a Windows-first focus and a small-footprint goal.

**Website:** [arcalabs.vercel.app](https://arcalabs.vercel.app) · **Contributing:** [CONTRIBUTING.md](CONTRIBUTING.md)

[![CI](https://github.com/cxernlol/ArcaLabs/actions/workflows/ci.yml/badge.svg)](https://github.com/cxernlol/ArcaLabs/actions/workflows/ci.yml)

> **Current status:** Early desktop prototype. Scene management, capture sources, preview, and local audio meters are implemented and still being refined. RTMP publishing and the live audio mix are not implemented yet, so **Go Live is unavailable**. The app does not record locally.

## What the desktop prototype includes

- Scene management with display and window capture sources.
- Live preview for the selected scene.
- Microphone, desktop, and application audio source selection. Microphone and application input levels are measured locally; live audio mixing is not available yet.
- Stream destination settings for YouTube, Kick, and Twitch using RTMP details. Stream keys are stored in the operating system keychain.
- Video and general settings, with unavailable features clearly marked.
- Local crash reports that users choose whether to copy and share.

The desktop app does not upload logs, collect telemetry, or make analytics calls. Audio is not saved as a local recording. See [`apps/desktop/SPEC.md`](apps/desktop/SPEC.md) and the focused [capture specification](apps/desktop/specs/scenes-capture/SPEC.md) for current scope and limitations.

## Run locally

### Requirements

- Node.js LTS and pnpm
- Rust stable
- Windows: MSVC Build Tools with the C++ workload and WebView2

From the repository root:

```bash
pnpm install
pnpm dev:desktop
```

Run the marketing site separately:

```bash
pnpm dev:web
```

See [`apps/web/README.md`](apps/web/README.md) for Vercel setup.

## CI and deployment

Pull requests and pushes to `main` run [`.github/workflows/ci.yml`](.github/workflows/ci.yml):

| Check | Runner | What it runs |
|---|---|---|
| `web` | Ubuntu | Frozen pnpm install, lint, and production build for `apps/web` |
| `rust` | Windows | `cargo check` for `apps/desktop/src-tauri` |

Run the same checks locally before pushing:

```powershell
.\scripts\check.ps1
```

Vercel deploys the marketing site from `apps/web` when changes land on `main`. Desktop installer and auto-update builds are not part of CI yet.

## Contributing

Open a focused pull request against `main`; do not push directly to `main`. Use `feat/`, `fix/`, or `docs/` branch prefixes, and run CI checks before each push. Read [`CONTRIBUTING.md`](CONTRIBUTING.md) for the full workflow and privacy requirements.

## License

ArcaLabs is licensed under [GPL-3.0](LICENSE).
