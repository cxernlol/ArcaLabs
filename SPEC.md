# SPEC: Strict contributing workflow + professional README

## 1. Goal & User Story

Contributors need a clear, enforceable path from clone to PR so broken or undisciplined changes do not land on `main`. Maintainers and newcomers should open a professional README that explains the product, how to run it, and how to contribute without hunting through chat history.

## 2. Inputs & Outputs

**Inputs**

- Contributor changes under `apps/web` and/or `apps/desktop` (or docs-only)
- Local machine: Node/pnpm (web), Rust + MSVC on Windows (desktop Rust check)
- GitHub PR opened against `main`

**Outputs**

- Rewritten root [`README.md`](README.md): product summary, status, site link, structure, quick start, CI/CD overview, contributing pointer, docs table, license — professional tone, no broken markdown
- [`CONTRIBUTING.md`](CONTRIBUTING.md): branch naming (`feat/`, `fix/`, `docs/`), PR-only to `main`, local checks required before push, commit guidance, privacy reminders (desktop: no telemetry)
- Root check scripts mirroring CI:
  - `scripts/check.sh` (Unix)
  - `scripts/check.ps1` (Windows)
  - Both run: `apps/web` `pnpm install` (if needed) + `pnpm lint` + `pnpm build`, and `cargo check` in `apps/desktop/src-tauri`
- [`.github/PULL_REQUEST_TEMPLATE.md`](.github/PULL_REQUEST_TEMPLATE.md): summary, test plan, checklist (branch name, local `scripts/check`, CI expected green, no desktop telemetry)
- Network calls: none except local tooling (pnpm registry, crates.io) when contributors run checks. No new desktop telemetry.

## 3. Acceptance Criteria

- [ ] `README.md` is restructured and reads as a project landing page (not a scratch pad)
- [ ] `CONTRIBUTING.md` documents: branch from `main`; names `feat/*`, `fix/*`, `docs/*`; no direct pushes to `main`; run local check script before push; open PR to `main`; short commit message guidance
- [ ] `scripts/check.sh` and `scripts/check.ps1` exit non-zero if lint, build, or `cargo check` fails
- [ ] `README.md` and/or `CONTRIBUTING.md` tell contributors to run the check script before opening a PR
- [ ] `.github/PULL_REQUEST_TEMPLATE.md` includes a checklist covering local checks and branch naming
- [ ] Existing `.github/workflows/ci.yml` behavior unchanged (still web + Windows rust on PR/push to `main`)
- [ ] No desktop installer/release CI; no new analytics deps in desktop

## 4. Edge Cases & Error States

| Situation | Behavior |
|---|---|
| Contributor lacks Rust/MSVC | `cargo check` fails with visible error; CONTRIBUTING notes Windows Build Tools requirement |
| Docs-only PR | Still must run full check script (matches CI always-on jobs) |
| `pnpm` / lockfile missing | Script fails at install/lint with clear shell error |
| Someone pushes to `main` | Policy documented; enforcement is GitHub branch protection (out of script scope) |
| Check script run from wrong cwd | Scripts resolve repo root from their location so they work from anywhere |

## 5. Out of Scope

- Enforcing branch protection via GitHub API
- pre-commit hooks / husky / lefthook
- Desktop `tauri build` or release workflows
- Changing CI job matrix or path filters
- Rewriting `apps/web/README.md` beyond a one-line cross-link if needed
