@echo off
echo Starting CoSurf development environment...
echo.

echo [1/3] Building native module...
cd native
cargo build --release
if errorlevel 1 (
    echo Failed to build native module
    exit /b 1
)
cd ..

echo.
echo [2/3] Building shared types...
call pnpm --filter @cosurf/shared build
if errorlevel 1 (
    echo Failed to build shared types
    exit /b 1
)

echo.
echo [3/3] Starting Tauri dev server...
call cargo tauri dev
