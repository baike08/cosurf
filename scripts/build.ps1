$ErrorActionPreference = "Stop"

Write-Host ""
Write-Host "  ========================================" -ForegroundColor Cyan
Write-Host "    CoSurf (伴游) - Production Build" -ForegroundColor Cyan
Write-Host "  ========================================" -ForegroundColor Cyan
Write-Host ""

$rootDir = Split-Path -Parent $PSScriptRoot

Write-Host "[1/5] Checking prerequisites..." -ForegroundColor Yellow
$missing = @()
if (-not (Get-Command node -ErrorAction SilentlyContinue)) { $missing += "Node.js" }
if (-not (Get-Command pnpm -ErrorAction SilentlyContinue)) { $missing += "pnpm" }
if (-not (Get-Command rustc -ErrorAction SilentlyContinue)) { $missing += "Rust (rustc)" }
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) { $missing += "Rust (cargo)" }

if ($missing.Count -gt 0) {
    Write-Host "  Missing: $($missing -join ', ')" -ForegroundColor Red
    Write-Host "  Please install the missing tools and try again." -ForegroundColor Red
    Write-Host ""
    Write-Host "  - Node.js: https://nodejs.org" -ForegroundColor Gray
    Write-Host "  - pnpm:    npm install -g pnpm" -ForegroundColor Gray
    Write-Host "  - Rust:    https://rustup.rs" -ForegroundColor Gray
    exit 1
}

$nodeVer = node --version
$rustVer = rustc --version
Write-Host "  Node.js: $nodeVer" -ForegroundColor Gray
Write-Host "  Rust:    $rustVer" -ForegroundColor Gray
Write-Host "  OK" -ForegroundColor Green

Write-Host ""
Write-Host "[2/5] Installing dependencies..." -ForegroundColor Yellow
Push-Location $rootDir
pnpm install --frozen-lockfile 2>$null
if ($LASTEXITCODE -ne 0) { pnpm install }
Pop-Location
Write-Host "  OK" -ForegroundColor Green

Write-Host ""
Write-Host "[3/5] Building shared types..." -ForegroundColor Yellow
Push-Location $rootDir
pnpm --filter @cosurf/shared build
Pop-Location
Write-Host "  OK" -ForegroundColor Green

Write-Host ""
Write-Host "[4/5] Building frontend..." -ForegroundColor Yellow
Push-Location $rootDir
pnpm --filter @cosurf/web build
Pop-Location
Write-Host "  OK" -ForegroundColor Green

Write-Host ""
Write-Host "[5/5] Building Tauri desktop application..." -ForegroundColor Yellow
Push-Location $rootDir
npx tauri build
Pop-Location

if ($LASTEXITCODE -eq 0) {
    Write-Host ""
    Write-Host "  ========================================" -ForegroundColor Green
    Write-Host "    Build complete!" -ForegroundColor Green
    Write-Host "  ========================================" -ForegroundColor Green
    Write-Host ""
    Write-Host "  Output: src-tauri/target/release/bundle/" -ForegroundColor Gray
    Write-Host "  - .msi  installer (Windows)" -ForegroundColor Gray
    Write-Host "  - .exe  standalone executable" -ForegroundColor Gray
} else {
    Write-Host "  Build failed. Check the error messages above." -ForegroundColor Red
    exit 1
}
