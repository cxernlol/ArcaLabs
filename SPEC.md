# SPEC: Contributor CI/CD

## 1. Goal & User Story

Contributors and maintainers need automated checks so broken web or Rust desktop changes fail before merge. Production marketing site updates when `main` changes, via existing Vercel Git integration.

## 2. Inputs & Outputs

**Inputs**

- Push / pull_request targeting `main` (and PRs from forks against `main`)
- Paths: whole repo (no path filters in v1 — both jobs always run)

**Outputs**

- GitHub Actions workflow with two jobs:
  - **web:** `apps/web` → `pnpm install` → `pnpm lint` → `pnpm build`
  - **rust:** `apps/desktop/src-tauri` on `windows-latest` → `cargo check`
- Docs: root README describing how CI works and that Vercel Production Branch = `main`
- Network calls: GitHub Actions runners only (pnpm registry, crates.io). No telemetry added to the desktop app. Site deploy remains Vercel’s existing Git hook (already allowed for `apps/web`).

## 3. Acceptance Criteria

- [ ] `.github/workflows/ci.yml` exists and runs on `pull_request` and `push` to `main`
- [ ] Web job fails the workflow if lint or build fails
- [ ] Rust job fails the workflow if `cargo check` fails on Windows
- [ ] README documents CI jobs and that production site tracks `main` via Vercel
- [ ] Desktop installer / GitHub Releases workflow is not added
- [ ] No new analytics or phone-home deps in desktop

## 4. Edge Cases & Error States

| Situation | Behavior |
|---|---|
| PR only touches docs | Both jobs still run (simple v1) |
| `cargo check` needs Windows tooling | Use `windows-latest`; fail with visible Rust/linker errors if env broken |
| Fork PR | Same CI; no secrets required for these jobs |
| Vercel Production Branch not `main` | Document dashboard fix; CI does not change Vercel config via API |

## 5. Out of Scope

- Tauri installer / release builds (later)
- Desktop frontend `pnpm build` / full `tauri build`
- Branch protection API setup (document only: enable required checks in GitHub settings)
- Path-filtered jobs
- Changing GitHub About / domain redirects
