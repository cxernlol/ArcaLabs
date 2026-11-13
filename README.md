# ArcaLabs

Privacy-first desktop streaming software for YouTube, Kick, and Twitch. ArcaLabs is moving to a Windows-native Rust UI and media pipeline, with a small-footprint goal.

**Website:** [arcalabs.vercel.app](https://arcalabs.vercel.app) · **Contributing:** [CONTRIBUTING.md](CONTRIBUTING.md)

[![CI](https://github.com/cxernlol/ArcaLabs/actions/workflows/ci.yml/badge.svg)](https://github.com/cxernlol/ArcaLabs/actions/workflows/ci.yml)

> **Current status:** Native Rust migration is underway. The native egui shell and local scene/settings foundation are being built first; capture presentation, encoding, live audio mixing, and RTMPS publishing are not yet connected, so **Go Live is unavailable**. The existing Tauri prototype is retained during migration. ArcaLabs does not record locally.

## Desktop migration

- `apps/desktop` is the native `egui/eframe` app.
- `crates/arca-core`, `arca-capture`, `arca-compose`, `arca-encode`, `arca-audio`, and `arca-publish` contain the Rust app and media foundations.
- The existing `apps/desktop/src-tauri` Tauri prototype remains available until native feature parity is reached.
- The stream key remains in the OS keychain. Settings, scenes, and scrubbed crash logs stay local.

The desktop app makes no analytics or telemetry calls. Its only intended network connection is a stream the user starts. See [`apps/desktop/SPEC.md`](apps/desktop/SPEC.md) for the approved migration scope.

## Run locally

### Requirements

- Rust stable
- Windows: MSVC Build Tools with the C++ workload
- Node.js LTS and pnpm for the website and retained Tauri prototype

From the repository root:

```bash
cargo run -p arcalabs
```

The retained Tauri prototype can still be run with `pnpm install` and `pnpm dev:desktop`.

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
| `rust` | Windows | Rust formatting, workspace check, and workspace tests |

Run the same checks locally before pushing:

```powershell
.\scripts\check.ps1
```

Vercel deploys the marketing site from `apps/web` when changes land on `main`. Native installer builds are not part of CI yet.

## Contributing

Open a focused pull request against `main`; do not push directly to `main`. Use `feat/`, `fix/`, or `docs/` branch prefixes, and run CI checks before each push. Read [`CONTRIBUTING.md`](CONTRIBUTING.md) for the full workflow and privacy requirements.

## License

ArcaLabs is licensed under [GPL-3.0](LICENSE).
