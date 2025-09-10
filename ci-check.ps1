#!/usr/bin/env pwsh
# Local CI validation script - Run before pushing!
# Usage: .\ci-check.ps1

Write-Host "🔍 Running Complete CI Validation Locally" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

$failed = $false

# 1. Format Check
Write-Host "`n📝 Checking Code Formatting..." -ForegroundColor Yellow
cargo fmt --all -- --check
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Formatting issues found! Run: cargo fmt --all" -ForegroundColor Red
    $failed = $true
} else {
    Write-Host "✅ Formatting OK" -ForegroundColor Green
}

# 2. Clippy (with CI flags)
Write-Host "`n🔍 Running Clippy with CI settings..." -ForegroundColor Yellow
cargo clippy --all-targets --all-features -- -D warnings
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Clippy warnings found!" -ForegroundColor Red
    $failed = $true
} else {
    Write-Host "✅ Clippy OK" -ForegroundColor Green
}

# 3. Build Check
Write-Host "`n🔨 Building project..." -ForegroundColor Yellow
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Build failed!" -ForegroundColor Red
    $failed = $true
} else {
    Write-Host "✅ Build OK" -ForegroundColor Green
}

# 4. Test Suite
Write-Host "`n🧪 Running all tests..." -ForegroundColor Yellow
cargo test --all --no-fail-fast
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Tests failed!" -ForegroundColor Red
    $failed = $true
} else {
    Write-Host "✅ Tests OK" -ForegroundColor Green
}

# 5. Doc Check
Write-Host "`n📚 Checking documentation..." -ForegroundColor Yellow
cargo doc --no-deps --document-private-items
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Documentation issues!" -ForegroundColor Red
    $failed = $true
} else {
    Write-Host "✅ Documentation OK" -ForegroundColor Green
}

# 6. Security Audit (if available)
Write-Host "`n🔒 Running security audit..." -ForegroundColor Yellow
cargo audit 2>$null
if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Security audit OK" -ForegroundColor Green
} else {
    Write-Host "Warning: Security audit not available or found issues" -ForegroundColor Yellow
}

# 7. Check for common issues
Write-Host "`n🔎 Checking for common CI issues..." -ForegroundColor Yellow

# Check for println! in non-test code
$printlns = Get-ChildItem -Path src -Recurse -Filter "*.rs" | Select-String -Pattern "println!" -SimpleMatch
if ($printlns) {
    Write-Host "Warning: Found println! in source code (should use log crate)" -ForegroundColor Yellow
    $printlns | ForEach-Object { Write-Host "   $_" }
}

# Check for unwrap() in non-test code
$unwraps = Get-ChildItem -Path src -Recurse -Filter "*.rs" | Select-String -Pattern ".unwrap" -SimpleMatch
if ($unwraps) {
    Write-Host "Warning: Found unwrap in source code - use '?' or 'expect' instead" -ForegroundColor Yellow
    $unwraps | ForEach-Object { Write-Host "   $_" }
}

# Check for TODO/FIXME
$todos = Get-ChildItem -Path src -Recurse -Filter "*.rs" | Select-String -Pattern "TODO|FIXME|HACK" -SimpleMatch
if ($todos) {
    Write-Host "Warning: Found TODO/FIXME/HACK comments:" -ForegroundColor Yellow
    $todos | ForEach-Object { Write-Host "   $_" }
}

Write-Host "`n========================================" -ForegroundColor Cyan
if ($failed) {
    Write-Host "❌ CI checks FAILED - Fix issues before pushing!" -ForegroundColor Red
    exit 1
} else {
    Write-Host "✅ All CI checks PASSED - Safe to push!" -ForegroundColor Green
    Write-Host ""
    Write-Host "Tip: Run git push to update your PR" -ForegroundColor Cyan
}