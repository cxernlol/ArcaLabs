# ArcaLabs

Lightweight desktop streaming for **YouTube**, **Kick**, and **Twitch**.

ArcaLabs picks encoder, resolution, frame rate, and bitrate for the user’s PC automatically, with manual controls available underneath. The desktop app is built with **Rust** and **Tauri** for a small footprint and low RAM use. **It collects no user data.**

**Website:** [arcalabs.vercel.app](https://arcalabs.vercel.app)

| | |
|---|---|
| **License** | [GPL-3.0](LICENSE) |
| **Status** | P0 — desktop shell scaffold; marketing site live |
| **Platforms** | Windows-first (desktop); web is static |

## Repository layout

```
apps/desktop/     Tauri + React desktop app
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

### Desktop app

```bash
cd apps/desktop
pnpm install
pnpm tauri dev
```

### Marketing site

```bash
cd apps/web
pnpm install
pnpm dev
```

See [`apps/web/README.md`](apps/web/README.md) for Vercel deploy notes.

## Continuous integration and deploy

**CI** — PRs and pushes to `main` run [`.github/workflows/ci.yml`](.github/workflows/ci.yml):

| Job | Runner | Checks |
|-----|--------|--------|
| `web` | Ubuntu | `pnpm lint` + `pnpm build` in `apps/web` |
| `rust` | Windows | `cargo check` in `apps/desktop/src-tauri` |

**CD** — Vercel builds `apps/web`. Set **Root Directory** to `apps/web` and **Production Branch** to `main` so merges update [arcalabs.vercel.app](https://arcalabs.vercel.app).

Desktop installer / release builds are not in CI yet.

## Contributing

Contributions are welcome under a **strict workflow**:

1. Branch from `main` using `feat/`, `fix/`, or `docs/` prefixes.
2. Run local checks **before** pushing (same as CI):

   ```powershell
   # Windows
   .\scripts\check.ps1
   ```

   ```bash
   # macOS / Linux
   ./scripts/check.sh
   ```

3. Open a pull request against `main` (no direct commits to `main`).

Full policy, commit guidance, and privacy rules: **[`CONTRIBUTING.md`](CONTRIBUTING.md)**.

Maintainers should protect `main` in GitHub and require the `web` and `rust` status checks.

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
