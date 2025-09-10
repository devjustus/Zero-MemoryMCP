#!/usr/bin/env pwsh
# CI/CD Pipeline Local Test Script - Mirrors GitHub Actions Security Workflow
# CRITICAL: This includes Miri tests which have been failing

Write-Host "=== CI/CD Pipeline Local Validation ===" -ForegroundColor Cyan
Write-Host "Mirroring GitHub Actions Security Workflow" -ForegroundColor Gray
Write-Host ""

$failed = $false
$startTime = Get-Date

# 1. Format Check
Write-Host "[1/7] Checking Code Formatting..." -ForegroundColor Yellow
cargo fmt --all -- --check
if ($LASTEXITCODE -ne 0) {
    Write-Host "  ❌ Format check failed! Run: cargo fmt --all" -ForegroundColor Red
    $failed = $true
} else {
    Write-Host "  ✅ Format check passed" -ForegroundColor Green
}

# 2. Clippy Check
Write-Host "`n[2/7] Running Clippy..." -ForegroundColor Yellow
cargo clippy --all-targets --all-features -- -D warnings
if ($LASTEXITCODE -ne 0) {
    Write-Host "  ❌ Clippy check failed!" -ForegroundColor Red
    $failed = $true
} else {
    Write-Host "  ✅ Clippy check passed" -ForegroundColor Green
}

# 3. Build Check
Write-Host "`n[3/7] Building Release..." -ForegroundColor Yellow
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Host "  ❌ Build failed!" -ForegroundColor Red
    $failed = $true
} else {
    Write-Host "  ✅ Build successful" -ForegroundColor Green
}

# 4. Test Suite
Write-Host "`n[4/7] Running Tests..." -ForegroundColor Yellow
cargo test --all --no-fail-fast
if ($LASTEXITCODE -ne 0) {
    Write-Host "  ❌ Tests failed!" -ForegroundColor Red
    $failed = $true
} else {
    Write-Host "  ✅ All tests passed" -ForegroundColor Green
}

# 5. Documentation
Write-Host "`n[5/7] Building Documentation..." -ForegroundColor Yellow
cargo doc --no-deps --document-private-items
if ($LASTEXITCODE -ne 0) {
    Write-Host "  ❌ Documentation build failed!" -ForegroundColor Red
    $failed = $true
} else {
    Write-Host "  ✅ Documentation built" -ForegroundColor Green
}

# 6. CRITICAL: Miri Memory Safety Check
Write-Host "`n[6/7] Running Miri Memory Safety Check..." -ForegroundColor Yellow
Write-Host "  ⚠️  THIS IS THE TEST FAILING IN CI!" -ForegroundColor Magenta

# Install Miri if needed
$miriCheck = cargo +nightly miri --version 2>$null
if ($LASTEXITCODE -ne 0) {
    Write-Host "  Installing Miri..." -ForegroundColor Gray
    rustup +nightly component add miri
}

# Run Miri and capture output
$miriOutput = cargo +nightly miri test 2>&1 | Out-String
$miriExitCode = $LASTEXITCODE

# Check for actual errors (not test names containing "error")
if ($miriExitCode -ne 0) {
    Write-Host "  ❌ Miri test failed!" -ForegroundColor Red
    
    # Extract actual error lines
    $errorLines = $miriOutput -split "`n" | Where-Object { 
        $_ -match "^error:" -or 
        $_ -match "abnormal termination" -or 
        $_ -match "test failed, to rerun"
    }
    
    if ($errorLines.Count -gt 0) {
        Write-Host "  Errors:" -ForegroundColor Red
        foreach ($line in $errorLines) {
            Write-Host "    $line" -ForegroundColor Red
        }
    }
    $failed = $true
} else {
    Write-Host "  ✅ Miri memory safety check passed" -ForegroundColor Green
}

# 7. Optional Security Audit
Write-Host "`n[7/7] Security Audit..." -ForegroundColor Yellow
$auditCheck = cargo audit --version 2>$null
if ($LASTEXITCODE -eq 0) {
    cargo audit 2>$null
    if ($LASTEXITCODE -ne 0) {
        Write-Host "  ⚠️  Security vulnerabilities found (non-blocking)" -ForegroundColor Yellow
    } else {
        Write-Host "  ✅ No vulnerabilities" -ForegroundColor Green
    }
} else {
    Write-Host "  ⏭️  Skipped (cargo-audit not installed)" -ForegroundColor Gray
}

# Summary
Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
$duration = (Get-Date) - $startTime
Write-Host "Duration: $($duration.ToString('mm\:ss'))" -ForegroundColor Gray

if ($failed) {
    Write-Host "❌ PIPELINE FAILED!" -ForegroundColor Red
    Write-Host "GitHub Actions will fail if you push now." -ForegroundColor Yellow
    Write-Host ""
    Write-Host "To debug Miri issues specifically:" -ForegroundColor Cyan
    Write-Host "  cargo +nightly miri test --lib 2>&1 | Select-String -Pattern 'error:' -Context 5" -ForegroundColor Gray
    exit 1
} else {
    Write-Host "✅ ALL CHECKS PASSED!" -ForegroundColor Green
    Write-Host "Safe to push to GitHub." -ForegroundColor Green
    exit 0
}