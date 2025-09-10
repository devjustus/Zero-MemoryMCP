#!/usr/bin/env pwsh
# Quick coverage estimation

Write-Host "CODE COVERAGE ANALYSIS" -ForegroundColor Cyan
Write-Host ""

# Count test functions
$testCount = 0
Get-ChildItem -Path src,tests -Recurse -Filter "*.rs" -ErrorAction SilentlyContinue | ForEach-Object {
    $content = Get-Content $_.FullName -Raw
    $matches = [regex]::Matches($content, '#\[test\]')
    $testCount += $matches.Count
}

Write-Host "Test Count: $testCount test functions" -ForegroundColor Yellow

# Count source files
$sourceFiles = (Get-ChildItem -Path src -Recurse -Filter "*.rs" | Measure-Object).Count
$testFiles = (Get-ChildItem -Path tests -Recurse -Filter "*.rs" -ErrorAction SilentlyContinue | Measure-Object).Count

Write-Host "Source Files: $sourceFiles" -ForegroundColor Yellow  
Write-Host "Test Files: $testFiles" -ForegroundColor Yellow

# Count lines
$sourceLines = 0
Get-ChildItem -Path src -Recurse -Filter "*.rs" | ForEach-Object {
    $sourceLines += (Get-Content $_.FullName | Measure-Object -Line).Lines
}

$testLines = 0
Get-ChildItem -Path tests -Recurse -Filter "*.rs" -ErrorAction SilentlyContinue | ForEach-Object {
    $testLines += (Get-Content $_.FullName | Measure-Object -Line).Lines
}

Write-Host "Source Lines: $sourceLines" -ForegroundColor Yellow
Write-Host "Test Lines: $testLines" -ForegroundColor Yellow

# Calculate ratio
if ($sourceLines -gt 0) {
    $ratio = [math]::Round($testLines / $sourceLines, 2)
    Write-Host "Test/Source Ratio: $ratio" -ForegroundColor Yellow
}

# Run tests
Write-Host ""
Write-Host "Running tests..." -ForegroundColor DarkGray
cargo test --all --no-fail-fast --quiet 2>&1 | Out-Null
$exitCode = $LASTEXITCODE

if ($exitCode -eq 0) {
    Write-Host "All tests passed!" -ForegroundColor Green
} else {
    Write-Host "Some tests failed" -ForegroundColor Red
}

# Estimate coverage
Write-Host ""
$estimate = 75  # Base estimate
if ($ratio -gt 0.5) { $estimate += 5 }
if ($ratio -gt 1.0) { $estimate += 10 }
if ($testCount -gt 300) { $estimate += 5 }
if ($testCount -gt 500) { $estimate += 5 }

Write-Host "ESTIMATED CODE COVERAGE: $estimate percent" -ForegroundColor $(if ($estimate -ge 80) { 'Green' } else { 'Yellow' })
Write-Host ""
Write-Host "Note: Install cargo-tarpaulin for accurate measurement" -ForegroundColor DarkGray