# Arca Labs

Lightweight desktop streaming for **YouTube**, **Kick**, and **Twitch**.

ArcaLabs picks encoder, resolution, frame rate, and bitrate for the user’s PC automatically, with manual controls available underneath. The desktop app is built with **Rust** and **Tauri** for a small footprint and low RAM use. **It collects no user data.**

**Website:** [arcalabs.vercel.app](https://arcalabs.vercel.app) · **Contributing:** [`CONTRIBUTING.md`](CONTRIBUTING.md)

[![CI](https://github.com/cxernlol/ArcaLabs/actions/workflows/ci.yml/badge.svg)](https://github.com/cxernlol/ArcaLabs/actions/workflows/ci.yml)

| | |
|---|---|
| **License** | [GPL-3.0](LICENSE) |
| **Status** | P0 — **v0.1 offline desktop prototype** (no Go Live yet); marketing site live |
| **Platforms** | Windows-first (desktop); web is static |
| **Required checks** | CI `web` + `rust` (see below) |

## Desktop prototype (v0.1)

Local-only shell you can run today. Nothing goes live; there is **no OAuth** and **no outbound network** from the desktop app in this version.

**Try it**

```bash
pnpm install
pnpm dev:desktop
```

Needs Node LTS, pnpm, Rust stable, and **MSVC Build Tools** + WebView2 on Windows.

**What’s in the prototype**

- Main window: preview placeholder, Scenes / Sources / Audio pop panels, disabled **Go Live**
- Header gear → centered **Settings** modal (General · Stream · Video · Audio · Hotkeys)
- Stream: YouTube / Kick / Twitch, ingest URL, stream key in the **OS keychain** only (never in config or logs)
- Video: encoder / resolution / frame rate / bitrate stubs; GPU detect stays on-device
- Local scrubbed `crash.log` + **Copy crash report** (no upload, no telemetry)
- Optimize is a stub for a later local pass

**Not in v0.1:** RTMP publish, real capture / WASAPI, OS hotkey registration, installers, auto-update.

Current feature contract: [`SPEC.md`](SPEC.md). UI tokens: [`design.md`](design.md).

## Repository layout

```
apps/desktop/     Tauri + React desktop app (this prototype)
apps/web/         Marketing site (Vite + React)
docs/             Architecture and design notes
.github/          CI workflows and PR template
scripts/          Local pre-PR checks (mirrors CI)
```

## Quick start

### Prerequisites

- [Node.js](https://nodejs.org/) LTS and [pnpm](https://pnpm.io/)
- Desktop: [Rust](https://rustup.rs/) (stable) and **MSVC Build Tools** on Windows
- Desktop: WebView2 (usually present on Windows 10/11)

From the **repo root** (`Arcalabs/`):

```bash
pnpm install
pnpm dev:desktop   # Tauri prototype (needs MSVC Build Tools on Windows)
pnpm dev:web       # marketing site
```

Or per app:

```bash
cd apps/desktop
pnpm install
pnpm tauri dev

cd apps/web
pnpm install
pnpm dev
```

See [`apps/web/README.md`](apps/web/README.md) for Vercel deploy notes.

## CI / CD checks

Every pull request and push to `main` must pass [`.github/workflows/ci.yml`](.github/workflows/ci.yml):

| Check | Job | Runner | What it runs |
|-------|-----|--------|--------------|
| Marketing site | `web` | Ubuntu | `pnpm install --frozen-lockfile` → `pnpm lint` → `pnpm build` in `apps/web` |
| Desktop Rust | `rust` | Windows | `cargo check` in `apps/desktop/src-tauri` |

**Run the same checks locally before push:**

```powershell
# Windows
.\scripts\check.ps1
```

```bash
# macOS / Linux
./scripts/check.sh

# or
pnpm check
```

**CD** — Vercel builds `apps/web` when `main` updates ([arcalabs.vercel.app](https://arcalabs.vercel.app)). Root Directory: `apps/web`. Desktop installers / auto-update are not in CI/CD yet.

Protect `main` and require the `web` + `rust` status checks.

## Contributing

Start with **[`CONTRIBUTING.md`](CONTRIBUTING.md)** (branching, privacy rules, PR checklist, CI expectations).

Short version:

1. Branch from `main` with `feat/`, `fix/`, or `docs/` prefixes — no direct commits to `main`.
2. Run `.\scripts\check.ps1` / `./scripts/check.sh` (or `pnpm check`) until green.
3. Open a PR and wait for CI `web` + `rust`.

## Documentation

| Document | Purpose |
|----------|---------|
| [`CONTRIBUTING.md`](CONTRIBUTING.md) | Branching, local checks, PR rules |
| [`SPEC.md`](SPEC.md) | Current approved feature spec |
| [`design.md`](design.md) | Desktop UI design system |
| [`AGENTS.md`](AGENTS.md) | Agent workflow and privacy rules |
| [`docs/superpowers/specs/2026-11-12-arcalabs-system-design.md`](docs/superpowers/specs/2026-11-12-arcalabs-system-design.md) | System architecture (draft) |

## License

This project is licensed under the [GNU General Public License v3.0](LICENSE).
