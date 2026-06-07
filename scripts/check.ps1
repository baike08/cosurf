$ErrorActionPreference = "Stop"

Write-Host "Running all checks..." -ForegroundColor Cyan

Write-Host "[1/3] TypeScript type check..." -ForegroundColor Yellow
pnpm --filter @cosurf/web typecheck

Write-Host "[2/3] ESLint..." -ForegroundColor Yellow
pnpm --filter @cosurf/web lint

Write-Host "[3/3] Rust clippy..." -ForegroundColor Yellow
Push-Location src-tauri
cargo clippy -- -D warnings
Pop-Location

Write-Host "All checks passed!" -ForegroundColor Green
