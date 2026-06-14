@echo off
cd /d "%~dp0"

echo ========================================
echo   CoSurf Development Environment
echo ========================================
echo.

echo Step 1: Building shared types...
call npx pnpm --filter @cosurf/shared build
if errorlevel 1 (
    echo ERROR: Failed to build shared types
    pause
    exit /b 1
)
echo OK
echo.

echo Step 2: Starting Playwright service...
start "Playwright Service" cmd /c "npx pnpm --filter @cosurf/playwright-service dev"
timeout /t 3 /nobreak >nul
echo OK (running in background)
echo.

echo Step 3: Starting Tauri dev server...
echo This will take a moment...
echo.
cargo tauri dev

pause
