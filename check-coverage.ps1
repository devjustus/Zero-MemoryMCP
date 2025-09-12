#!/usr/bin/env pwsh
# Simple coverage check script

Write-Host "==================== CODE METRICS ====================" -ForegroundColor Cyan
Write-Host ""

# Count Rust source files
$sourceFiles = Get-ChildItem -Path src -Recurse -Filter "*.rs" | Measure-Object
$testFiles = Get-ChildItem -Path tests -Recurse -Filter "*.rs" | Measure-Object

Write-Host "📊 File Count:" -ForegroundColor Yellow
Write-Host "  Source files: $($sourceFiles.Count)"
Write-Host "  Test files: $($testFiles.Count)"
Write-Host ""

# Count lines of code
$sourceLines = 0
$testLines = 0

Get-ChildItem -Path src -Recurse -Filter "*.rs" | ForEach-Object {
    $sourceLines += (Get-Content $_.FullName | Measure-Object -Line).Lines
}

Get-ChildItem -Path tests -Recurse -Filter "*.rs" | ForEach-Object {
    $testLines += (Get-Content $_.FullName | Measure-Object -Line).Lines
}

Write-Host "📏 Lines of Code:" -ForegroundColor Yellow
Write-Host "  Source code: $sourceLines lines"
Write-Host "  Test code: $testLines lines"
$ratio = [math]::Round($testLines / $sourceLines, 2)
Write-Host "  Test/Source ratio: ${ratio}:1"
Write-Host ""

# Count test functions
$testCount = 0
Get-ChildItem -Path src,tests -Recurse -Filter "*.rs" | ForEach-Object {
    $content = Get-Content $_.FullName -Raw
    $matches = [regex]::Matches($content, '#\[test\]|#\[cfg\(test\)\]')
    $testCount += $matches.Count
}

Write-Host "🧪 Test Count:" -ForegroundColor Yellow
Write-Host "  Total test functions: $testCount"
Write-Host ""

# Run tests and get results
Write-Host "Running tests..." -ForegroundColor DarkGray
$testOutput = cargo test --all --no-fail-fast 2>&1 | Out-String

# Extract test results
if ($testOutput -match "(\d+) passed.*?(\d+) failed.*?(\d+) ignored") {
    $passed = $matches[1]
    $failed = $matches[2]
    $ignored = $matches[3]
    
    Write-Host "✅ Test Results:" -ForegroundColor Green
    Write-Host "  Passed: $passed"
    Write-Host "  Failed: $failed"
    Write-Host "  Ignored: $ignored"
    
    $successRate = [math]::Round(($passed / ($passed + $failed)) * 100, 2)
    Write-Host "  Success rate: $successRate%"
} else {
    Write-Host "  Could not parse test results" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "==================== ESTIMATED COVERAGE ====================" -ForegroundColor Magenta
Write-Host ""

# Estimate coverage based on file analysis
$coveredFiles = @()
$totalFiles = @()

Get-ChildItem -Path src -Recurse -Filter "*.rs" | ForEach-Object {
    $fileName = $_.Name
    $baseName = $fileName -replace '\.rs$', ''
    $totalFiles += $fileName
    
    # Check if there's a corresponding test file
    $hasTests = $false
    
    # Check in src for #[cfg(test)] modules
    $content = Get-Content $_.FullName -Raw
    if ($content -match '#\[cfg\(test\)\]') {
        $hasTests = $true
    }
    
    # Check in tests directory
    if (Test-Path "tests/*$baseName*") {
        $hasTests = $true
    }
    
    if ($hasTests) {
        $coveredFiles += $fileName
    }
}

$fileCoverage = [math]::Round(($coveredFiles.Count / $totalFiles.Count) * 100, 2)

Write-Host "📈 Estimated Coverage Metrics:" -ForegroundColor Yellow
$coverageText = "  Files with tests: $($coveredFiles.Count)/$($totalFiles.Count) (${fileCoverage}`%)"
Write-Host $coverageText
Write-Host "  Test/Source line ratio: ${ratio}:1"
Write-Host ""

# Basic coverage estimation
$estimatedCoverage = [math]::Round(($fileCoverage + ($ratio * 100)) / 2, 2)
if ($estimatedCoverage -gt 100) { $estimatedCoverage = 95 }

$coverageMsg = "🎯 Estimated Code Coverage: ~${estimatedCoverage}`%"
$coverageColor = if ($estimatedCoverage -ge 80) { 'Green' } else { 'Red' }
Write-Host $coverageMsg -ForegroundColor $coverageColor
Write-Host ""
Write-Host "Note: This is a rough estimate. Install cargo-tarpaulin for accurate coverage:" -ForegroundColor DarkGray
Write-Host '  cargo install cargo-tarpaulin' -ForegroundColor DarkGray
Write-Host '  cargo tarpaulin --all-features --workspace --timeout 120' -ForegroundColor DarkGray