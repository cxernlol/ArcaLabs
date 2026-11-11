# Contributing to ArcaLabs

Thanks for helping. This project uses a **strict** workflow so `main` stays releasable and CI stays green.

## Rules

1. **Never push directly to `main`.** Open a pull request.
2. **Branch from an up-to-date `main`.**
3. **Name branches** with one of these prefixes:
   - `feat/<short-description>` — new functionality
   - `fix/<short-description>` — bug fixes
   - `docs/<short-description>` — documentation only
4. **Run local checks before every push** (required, including docs-only changes).
5. **Do not add telemetry** to the desktop app (`apps/desktop`, Rust crates). The marketing site may use Vercel Web Analytics only as documented in `AGENTS.md`.

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

What they run:

1. `pnpm install` (if needed), `pnpm lint`, and `pnpm build` in `apps/web`
2. `cargo check` in `apps/desktop/src-tauri`

### Tooling you need

| Check | Requirement |
|-------|-------------|
| Web | Node.js LTS, pnpm |
| Rust | Stable Rust via rustup; on Windows, **MSVC Build Tools** (Visual Studio Build Tools with C++ workload) |

If `cargo check` fails with a linker / `link.exe` error, install MSVC Build Tools and retry.

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

Larger features follow [`AGENTS.md`](AGENTS.md): clarify requirements, write/update [`SPEC.md`](SPEC.md), get approval, then implement only what the spec lists.

## Questions

Open an issue or discuss on the PR. Do not open PRs that add analytics, crash reporting, or other phone-home behavior to the desktop app.
