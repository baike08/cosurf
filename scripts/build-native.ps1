# CoSurf Native Module Build Script
# Automatically builds Rust code and copies .dll to .node file

$ErrorActionPreference = "Stop"

Write-Host ""
Write-Host "  ========================================" -ForegroundColor Cyan
Write-Host "    CoSurf Native Module Builder" -ForegroundColor Cyan
Write-Host "  ========================================" -ForegroundColor Cyan
Write-Host ""

$rootDir = Split-Path -Parent $PSScriptRoot
$nativeDir = Join-Path $rootDir "native"
$targetReleaseDir = Join-Path $rootDir "target\release"
$outputNodeFile = Join-Path $nativeDir "cosurf-native.node"

Write-Host "[1/3] Building Rust native module..." -ForegroundColor Yellow
Push-Location $rootDir

# Build in release mode
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Host "  Build failed!" -ForegroundColor Red
    Pop-Location
    exit 1
}

Pop-Location
Write-Host "  OK" -ForegroundColor Green

Write-Host ""
Write-Host "[2/3] Locating compiled DLL..." -ForegroundColor Yellow

$dllFile = Join-Path $targetReleaseDir "cosurf_native.dll"
if (-not (Test-Path $dllFile)) {
    Write-Host "  Error: cosurf_native.dll not found at $dllFile" -ForegroundColor Red
    exit 1
}

$dllInfo = Get-Item $dllFile
Write-Host "  Found: $($dllInfo.Name)" -ForegroundColor Gray
Write-Host "  Size: $([math]::Round($dllInfo.Length / 1MB, 2)) MB" -ForegroundColor Gray
Write-Host "  Modified: $($dllInfo.LastWriteTime)" -ForegroundColor Gray
Write-Host "  OK" -ForegroundColor Green

Write-Host ""
Write-Host "[3/3] Copying DLL to .node file..." -ForegroundColor Yellow

# Check if Electron is running
$electronProcesses = Get-Process -Name "electron" -ErrorAction SilentlyContinue
if ($electronProcesses) {
    Write-Host "  Warning: Electron is currently running." -ForegroundColor Yellow
    Write-Host "  The .node file may be locked. Stopping Electron..." -ForegroundColor Yellow
    
    try {
        Stop-Process -Name "electron" -Force -ErrorAction Stop
        Start-Sleep -Seconds 2
        Write-Host "  Electron stopped." -ForegroundColor Green
    } catch {
        Write-Host "  Failed to stop Electron. Please close it manually and retry." -ForegroundColor Red
        exit 1
    }
}

# Copy and rename
try {
    Copy-Item -Path $dllFile -Destination $outputNodeFile -Force
    $nodeInfo = Get-Item $outputNodeFile
    Write-Host "  Copied: cosurf-native.node" -ForegroundColor Green
    Write-Host "  Location: $($nodeInfo.FullName)" -ForegroundColor Gray
    Write-Host "  Size: $([math]::Round($nodeInfo.Length / 1MB, 2)) MB" -ForegroundColor Gray
    Write-Host "  Modified: $($nodeInfo.LastWriteTime)" -ForegroundColor Gray
} catch {
    Write-Host "  Error: Failed to copy DLL to .node file" -ForegroundColor Red
    Write-Host "  Details: $_" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "  ========================================" -ForegroundColor Green
Write-Host "    Build complete!" -ForegroundColor Green
Write-Host "  ========================================" -ForegroundColor Green
Write-Host ""
Write-Host "  Output: native/cosurf-native.node" -ForegroundColor Gray
Write-Host ""
Write-Host "  Next steps:" -ForegroundColor Cyan
Write-Host "  - Run 'pnpm dev' to start the application" -ForegroundColor Gray
Write-Host "  - The new .node file will be loaded automatically" -ForegroundColor Gray
Write-Host ""
