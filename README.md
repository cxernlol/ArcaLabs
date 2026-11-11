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

### Vercel (CD)

Import the repo in Vercel and set **Root Directory** to [`apps/web`](apps/web). See [`apps/web/README.md`](apps/web/README.md).

Set **Production Branch** to `main` so merges to `main` update [arcalabs.vercel.app](https://arcalabs.vercel.app).

### CI (GitHub Actions)

PRs and pushes to `main` run [`.github/workflows/ci.yml`](.github/workflows/ci.yml):

| Job | Runner | Checks |
|-----|--------|--------|
| `web` | Ubuntu | `pnpm lint` + `pnpm build` in `apps/web` |
| `rust` | Windows | `cargo check` in `apps/desktop/src-tauri` |

Maintainers: in GitHub **Settings → Branches**, protect `main` and require both `web` and `rust` status checks. Desktop installer / release builds are not in CI yet.

## Documentation

| Doc | Purpose |
|-----|---------|
| [`SPEC.md`](SPEC.md) | Current feature spec (Contributor CI/CD) |
| [`docs/superpowers/specs/2026-11-12-arcalabs-system-design.md`](docs/superpowers/specs/2026-11-12-arcalabs-system-design.md) | System architecture (draft) |
| [`design.md`](design.md) | UI design system |
| [`AGENTS.md`](AGENTS.md) | Agent workflow, privacy rules, `SPEC.md` template |

## License

GPL-3.0 — see [`LICENSE`](LICENSE).
