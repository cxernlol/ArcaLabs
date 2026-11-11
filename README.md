# ArcaLabs

Desktop streaming for **YouTube, Kick, and Twitch** — automatic encoder settings first, manual controls when you need them. Built with **Rust + Tauri** for a small footprint and low RAM use.

**Site:** [arcalabs.vercel.app](https://arcalabs.vercel.app)

**Status:** P0 desktop scaffold — run from `apps/desktop`.

```bash
# Desktop (Tauri — requires MSVC Build Tools on Windows)
cd apps/desktop
pnpm install
pnpm tauri dev

# Marketing site (static, client-side optimize demo)
cd apps/web
pnpm install
pnpm dev
```

### Vercel

Import the repo in Vercel and set **Root Directory** to [`apps/web`](apps/web). See [`apps/web/README.md`](apps/web/README.md).
```

## Documentation

| Doc | Purpose |
|-----|---------|
| [`docs/superpowers/specs/2026-11-12-arcalabs-system-design.md`](docs/superpowers/specs/2026-11-12-arcalabs-system-design.md) | System architecture (draft) |
| [`design.md`](design.md) | UI design system |
| [`AGENTS.md`](AGENTS.md) | Agent workflow, privacy rules, `SPEC.md` template |

## License

GPL-3.0 — see [`LICENSE`](LICENSE).
