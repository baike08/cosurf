$ErrorActionPreference = "Stop"

Write-Host "Starting CoSurf development environment..." -ForegroundColor Cyan

Write-Host "[1/3] Building shared types..." -ForegroundColor Yellow
pnpm --filter @cosurf/shared build

Write-Host "[2/3] Starting Playwright service..." -ForegroundColor Yellow
Start-Process pnpm -ArgumentList "--filter", "@cosurf/playwright-service", "dev" -NoNewWindow

Write-Host "[3/3] Starting Tauri dev server..." -ForegroundColor Yellow
pnpm dev:tauri
