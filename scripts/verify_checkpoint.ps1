# CoSurf Checkpoint Verification Script
# Usage: powershell -ExecutionPolicy Bypass -File .\scripts\verify_checkpoint.ps1

Write-Host "Checkpoint Mechanism Verification" -ForegroundColor Cyan
Write-Host "================================`n"

# 1. Check database files
Write-Host "1. Checking checkpoint databases..." -ForegroundColor Yellow
$dbDir = Join-Path $env:APPDATA "cosurf\cosurf-data"
$databases = Get-ChildItem -Path $dbDir -Filter "checkpoint_*.db" -ErrorAction SilentlyContinue

if ($databases.Count -eq 0) {
    Write-Host "   [NOT FOUND] No checkpoint database files" -ForegroundColor Red
    Write-Host "   Tip: Run CoSurf and execute AI conversation (iteration >= 3)" -ForegroundColor Gray
} else {
    Write-Host "   [OK] Found $($databases.Count) checkpoint databases" -ForegroundColor Green
    
    foreach ($db in $databases) {
        $sizeKB = [math]::Round($db.Length / 1KB, 2)
        Write-Host "   - $($db.Name) ($sizeKB KB)" -ForegroundColor Gray
    }
}

Write-Host ""

# 2. Check backup directory
Write-Host "2. Checking backup directory..." -ForegroundColor Yellow
$backupDir = Join-Path $env:TEMP "cosurf-checkpoint-backups"

if (Test-Path $backupDir) {
    $backups = Get-ChildItem -Path $backupDir -Filter "*.bak" -ErrorAction SilentlyContinue
    if ($backups.Count -eq 0) {
        Write-Host "   [INFO] Backup directory exists, no backup files yet" -ForegroundColor Yellow
    } else {
        Write-Host "   [OK] Found $($backups.Count) backup files" -ForegroundColor Green
        foreach ($bak in $backups | Select-Object -First 5) {
            $sizeKB = [math]::Round($bak.Length / 1KB, 2)
            Write-Host "     - $($bak.Name) ($sizeKB KB)" -ForegroundColor Gray
        }
        if ($backups.Count -gt 5) {
            Write-Host "     ... and $($backups.Count - 5) more files" -ForegroundColor Gray
        }
    }
} else {
    Write-Host "   [INFO] Backup directory not found (no file modifications yet)" -ForegroundColor Yellow
}

Write-Host ""

# 3. Check logs
Write-Host "3. Checking recent checkpoint logs..." -ForegroundColor Yellow
$logFile = Join-Path $env:APPDATA "cosurf\logs\cosurf.log"

if (Test-Path $logFile) {
    $logContent = Get-Content $logFile -Tail 1000 -ErrorAction SilentlyContinue
    
    # Search for key logs
    $checkpointCreated = ($logContent | Select-String "Created checkpoint" | Measure-Object).Count
    $fileBackedUp = ($logContent | Select-String "Backed up file" | Measure-Object).Count
    $checkpointCleaned = ($logContent | Select-String "Cleaned up.*checkpoint" | Measure-Object).Count
    $rollbackTriggered = ($logContent | Select-String "Rolling back" | Measure-Object).Count
    
    Write-Host "   Statistics:" -ForegroundColor Cyan
    Write-Host "      - Checkpoints created: $checkpointCreated" -ForegroundColor $(if ($checkpointCreated -gt 0) { "Green" } else { "Yellow" })
    Write-Host "      - Files backed up: $fileBackedUp" -ForegroundColor $(if ($fileBackedUp -gt 0) { "Green" } else { "Yellow" })
    Write-Host "      - Checkpoints cleaned: $checkpointCleaned" -ForegroundColor $(if ($checkpointCleaned -gt 0) { "Green" } else { "Gray" })
    Write-Host "      - Rollbacks triggered: $rollbackTriggered" -ForegroundColor $(if ($rollbackTriggered -gt 0) { "Red" } else { "Gray" })
    
    if ($checkpointCreated -eq 0 -and $fileBackedUp -eq 0) {
        Write-Host "`n   [WARNING] No checkpoint activity found" -ForegroundColor Yellow
        Write-Host "   Possible reasons:" -ForegroundColor Gray
        Write-Host "      1. App just started, not enough iterations (need >= 3)" -ForegroundColor Gray
        Write-Host "      2. Agent Loop didn't trigger tool calls" -ForegroundColor Gray
        Write-Host "      3. Different log file path" -ForegroundColor Gray
    }
    
    # Show recent logs
    Write-Host "`n   Recent checkpoint logs:" -ForegroundColor Cyan
    $recentLogs = $logContent | Select-String "Created checkpoint|Backed up file|Cleaned up|Rolling back" | Select-Object -Last 10
    if ($recentLogs) {
        foreach ($log in $recentLogs) {
            Write-Host "      $log" -ForegroundColor Gray
        }
    } else {
        Write-Host "      (No related logs)" -ForegroundColor Gray
    }
} else {
    Write-Host "   [ERROR] Log file not found: $logFile" -ForegroundColor Red
}

Write-Host ""

# 4. Test suggestions
Write-Host "4. How to trigger checkpoint test?" -ForegroundColor Yellow
Write-Host "   Steps:" -ForegroundColor Cyan
Write-Host "   1. Open CoSurf application" -ForegroundColor Gray
Write-Host "   2. Send a request requiring multiple iterations:" -ForegroundColor Gray
Write-Host "      'Create a Python calculator with add/subtract/multiply/divide'" -ForegroundColor White
Write-Host "   3. Watch for these logs:" -ForegroundColor Gray
Write-Host "      - Created checkpoint (iteration=3)" -ForegroundColor Green
Write-Host "      - Backed up file before modification" -ForegroundColor Green
Write-Host "      - Cleaned up old checkpoints (on session end)" -ForegroundColor Green
Write-Host ""

# 5. Summary
Write-Host "=================================" -ForegroundColor Cyan
Write-Host "Verification Complete" -ForegroundColor Green
Write-Host ""
Write-Host "Quick Diagnosis:" -ForegroundColor Cyan
if ($databases.Count -gt 0) {
    Write-Host "  [PASS] Database files exist" -ForegroundColor Green
} else {
    Write-Host "  [FAIL] Database files missing (need AI conversation)" -ForegroundColor Red
}

if (Test-Path $backupDir) {
    Write-Host "  [PASS] Backup directory exists" -ForegroundColor Green
} else {
    Write-Host "  [INFO] Backup directory missing (normal, no file changes yet)" -ForegroundColor Yellow
}

if (Test-Path $logFile) {
    Write-Host "  [PASS] Log file accessible" -ForegroundColor Green
} else {
    Write-Host "  [FAIL] Log file not accessible" -ForegroundColor Red
}

Write-Host ""
Write-Host "For detailed docs: docs/CHECKPOINT_VERIFICATION.md" -ForegroundColor Cyan
