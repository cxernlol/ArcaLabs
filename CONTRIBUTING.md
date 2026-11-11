# Contributing to ArcaLabs

Thanks for helping. ArcaLabs is privacy-first streaming software: the **desktop app collects no user data** and must not phone home. This guide keeps `main` releasable and CI green.

## Quick path

1. Branch from an up-to-date `main` (`feat/`, `fix/`, or `docs/` prefix).
2. Run local checks (below) until they pass.
3. Open a pull request against **`main`** — never push straight to `main`.
4. Wait for GitHub Actions **`web`** + **`rust`** to pass.

## Rules

1. **Never push directly to `main`.** Open a pull request.
2. **Branch from an up-to-date `main`.**
3. **Name branches** with one of these prefixes:
   - `feat/<short-description>` — new functionality
   - `fix/<short-description>` — bug fixes
   - `docs/<short-description>` — documentation only
4. **Run local checks before every push** (required, including docs-only changes).
5. **Do not add telemetry** to the desktop app (`apps/desktop`, Rust crates). No analytics SDKs, crash *upload* services, or automatic reporting to GitHub/any remote. The marketing site may use Vercel Web Analytics only as documented in [`docs/internal/AGENTS.md`](docs/internal/AGENTS.md).
6. **Stream keys** stay in the OS keychain only — never commit keys, dump them in logs, or write them to `settings.json`.
7. **Local crash logs** (scrubbed `crash.log` / Copy crash report) are fine; automatic upload is not.

## Local checks (required before push)

These scripts mirror GitHub Actions CI. They must exit successfully before you push and open a PR.

**Windows (PowerShell):**

```powershell
.\scripts\check.ps1
```

**macOS / Linux:**

```bash
./scripts/check.sh
```

Or from the repo root with pnpm:

```bash
pnpm check
```

What they run (same as CI):

| Check | Where | Command |
|-------|--------|---------|
| Web lint | `apps/web` | `pnpm lint` |
| Web build | `apps/web` | `pnpm build` |
| Rust | `apps/desktop/src-tauri` | `cargo check` |

### Tooling you need

| Check | Requirement |
|-------|-------------|
| Web | Node.js LTS, [pnpm](https://pnpm.io/) |
| Rust | Stable Rust via [rustup](https://rustup.rs/); on Windows, **MSVC Build Tools** (Visual Studio Build Tools with C++ workload) |
| Desktop run (optional) | WebView2 (usually present on Windows 10/11) |

If `cargo check` fails with a linker / `link.exe` error, install MSVC Build Tools and retry.

## CI / CD (what GitHub and Vercel run)

**CI** — [`.github/workflows/ci.yml`](.github/workflows/ci.yml) on every PR and push to `main`:

| Job | Runner | Must pass |
|-----|--------|-----------|
| `web` | Ubuntu | `pnpm install --frozen-lockfile`, `pnpm lint`, `pnpm build` in `apps/web` |
| `rust` | Windows | `cargo check` in `apps/desktop/src-tauri` |

**CD** — Vercel builds `apps/web` for [arcalabs.vercel.app](https://arcalabs.vercel.app) when `main` updates. Desktop installers are not in CI/CD yet.

PRs should stay red until both CI jobs are green.

## Pull requests

1. Push your branch and open a PR against **`main`**.
2. Use the PR template checklist.
3. Wait for CI (`web` + `rust`) to pass.
4. Keep the PR focused; do not expand scope beyond the stated change.

## Commit messages

- Prefer short, imperative subjects (e.g. `Add stream panel stub`, `Fix optimize demo layout`).
- Explain *why* in the body when the change is non-obvious.
- Do not commit secrets, stream keys, or personal paths.

## Spec-first features

Larger features follow [`docs/internal/AGENTS.md`](docs/internal/AGENTS.md): clarify requirements, write/update [`SPEC.md`](SPEC.md), get approval, then implement only what the spec lists. Match [`design.md`](design.md) for UI (Lucide, Inter, nested radius).

## Trying the v0.1 desktop prototype

```bash
pnpm install
pnpm dev:desktop
```

Details and scope limits: root [`README.md`](README.md).

## Questions

Open an issue or discuss on the PR. Do not open PRs that add analytics, automatic crash upload, or other phone-home behavior to the desktop app.
