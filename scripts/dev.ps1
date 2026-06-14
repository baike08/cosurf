$ErrorActionPreference = "Stop"

Write-Host "Starting CoSurf development environment..." -ForegroundColor Cyan

Write-Host "[1/2] Building shared types..." -ForegroundColor Yellow
pnpm --filter @cosurf/shared build

Write-Host "[2/2] Starting Electron dev server..." -ForegroundColor Yellow
npm run dev
