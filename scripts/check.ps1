#Requires -Version 5.1
$ErrorActionPreference = "Stop"

$Root = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $Root

Write-Host "==> Web: install / lint / build" -ForegroundColor Cyan
Push-Location (Join-Path $Root "apps\web")
try {
  if (-not (Test-Path "node_modules")) {
    pnpm install --frozen-lockfile
  }
  pnpm lint
  if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
  pnpm build
  if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}
finally {
  Pop-Location
}

Write-Host "==> Desktop frontend: build" -ForegroundColor Cyan
Push-Location (Join-Path $Root "apps\desktop")
try {
  pnpm build
  if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}
finally {
  Pop-Location
}

Write-Host "==> Rust: format, workspace check, and tests" -ForegroundColor Cyan
cargo fmt --all -- --check
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
cargo check --workspace --all-targets
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
cargo test --workspace
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "==> All local checks passed" -ForegroundColor Green
exit 0
