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

Write-Host "==> Rust: cargo check" -ForegroundColor Cyan
Push-Location (Join-Path $Root "apps\desktop\src-tauri")
try {
  cargo check
  if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}
finally {
  Pop-Location
}

Write-Host "==> All local checks passed" -ForegroundColor Green
exit 0
