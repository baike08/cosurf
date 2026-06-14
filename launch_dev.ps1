# CoSurf Development Launcher
Write-Host "Starting CoSurf development environment..." -ForegroundColor Cyan
Write-Host ""

# Check if native module exists
$nativeDll = "d:\coding-harness\CoSurf\target\release\cosurf_native.dll"
if (Test-Path $nativeDll) {
    Write-Host "[OK] Native module found" -ForegroundColor Green
} else {
    Write-Host "[ERROR] Native module not found. Building..." -ForegroundColor Red
    Set-Location "d:\coding-harness\CoSurf\native"
    cargo build --release
    Set-Location "d:\coding-harness\CoSurf"
}

# Check if frontend dist exists
$frontendDist = "d:\coding-harness\CoSurf\src-web\dist"
if (Test-Path $frontendDist) {
    Write-Host "[OK] Frontend dist found" -ForegroundColor Green
} else {
    Write-Host "[WARN] Frontend dist not found. Will be built by Tauri." -ForegroundColor Yellow
}

Write-Host ""
Write-Host "Starting Tauri dev server..." -ForegroundColor Cyan
Write-Host "Press Ctrl+C to stop" -ForegroundColor Gray
Write-Host ""

# Start Tauri dev
Set-Location "d:\coding-harness\CoSurf"
cargo tauri dev
