#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "==> Web: install / lint / build"
cd "$ROOT/apps/web"
if [[ ! -d node_modules ]]; then
  pnpm install --frozen-lockfile
fi
pnpm lint
pnpm build

echo "==> Rust: cargo check"
cd "$ROOT/apps/desktop/src-tauri"
cargo check

echo "==> All local checks passed"
