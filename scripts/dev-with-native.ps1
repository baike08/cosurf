# CoSurf Development Script with Auto Native Rebuild
# Builds native module and starts dev server

$ErrorActionPreference = "Stop"

Write-Host ""
Write-Host "  ========================================" -ForegroundColor Cyan
Write-Host "    CoSurf Development Mode" -ForegroundColor Cyan
Write-Host "  ========================================" -ForegroundColor Cyan
Write-Host ""

$rootDir = Split-Path -Parent $PSScriptRoot

# Check if native module needs rebuild
$nodeFile = Join-Path $rootDir "native\cosurf-native.node"
$dllFile = Join-Path $rootDir "target\release\cosurf_native.dll"

$needsRebuild = $false

if (-not (Test-Path $nodeFile)) {
    Write-Host "[INFO] Native module not found, will build..." -ForegroundColor Yellow
    $needsRebuild = $true
} elseif (Test-Path $dllFile) {
    $nodeInfo = Get-Item $nodeFile
    $dllInfo = Get-Item $dllFile
    
    if ($dllInfo.LastWriteTime -gt $nodeInfo.LastWriteTime) {
        Write-Host "[INFO] DLL is newer than .node file, will rebuild..." -ForegroundColor Yellow
        $needsRebuild = $true
    }
}

if ($needsRebuild) {
    Write-Host ""
    Write-Host "[1/2] Building native module..." -ForegroundColor Yellow
    & "$PSScriptRoot\build-native.ps1"
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Native build failed!" -ForegroundColor Red
        exit 1
    }
    Write-Host ""
} else {
    Write-Host "[INFO] Native module is up to date" -ForegroundColor Green
}

Write-Host "[2/2] Starting development server..." -ForegroundColor Yellow
Write-Host ""

# Start dev server
Push-Location $rootDir
pnpm dev
Pop-Location
