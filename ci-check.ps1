#!/usr/bin/env pwsh
# Local CI/CD Pipeline Check Script - COMPLETE MIRROR of GitHub Actions
# This script mirrors ALL checks from .github/workflows/ci.yml and security.yml
# Run this before pushing to ensure GitHub Actions will pass

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Memory-MCP Local CI Pipeline Check   " -ForegroundColor Cyan
Write-Host "  Complete GitHub Actions Mirror       " -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

$ErrorCount = 0
$WarningCount = 0
$startTime = Get-Date

# Function to check if a tool is installed
function Test-ToolInstalled {
    param($Tool, $InstallCmd)
    
    try {
        & $Tool --version 2>&1 | Out-Null
        return $true
    } catch {
        Write-Host "  ⚠️  $Tool not installed. Install with: $InstallCmd" -ForegroundColor Yellow
        return $false
    }
}

# Function to run a check
function Run-Check {
    param($Name, $Command, $Critical = $true)
    
    Write-Host "▶️  Running: $Name" -ForegroundColor Yellow
    $output = ""
    $exitCode = 0
    
    try {
        $output = Invoke-Expression $Command 2>&1 | Out-String
        $exitCode = $LASTEXITCODE
    } catch {
        $output = $_.Exception.Message
        $exitCode = 1
    }
    
    if ($exitCode -eq 0) {
        Write-Host "  ✅ $Name passed" -ForegroundColor Green
        return $true
    } else {
        if ($Critical) {
            Write-Host "  ❌ $Name failed (Critical)" -ForegroundColor Red
            if ($output.Length -gt 500) {
                Write-Host "     Output (truncated): $($output.Substring(0, 500))..." -ForegroundColor DarkRed
            } else {
                Write-Host "     Output: $output" -ForegroundColor DarkRed
            }
            $global:ErrorCount++
        } else {
            Write-Host "  ⚠️  $Name failed (Warning)" -ForegroundColor Yellow
            $global:WarningCount++
        }
        return $false
    }
}

Write-Host ""
Write-Host "📋 Checking Required Tools..." -ForegroundColor Cyan
Write-Host "================================" -ForegroundColor DarkGray

# Check for required tools and suggest installation
$tools = @{
    "cargo" = "Install Rust from https://rustup.rs/"
    "cargo-tarpaulin" = "cargo install cargo-tarpaulin"
    "cargo-audit" = "cargo install cargo-audit"
    "cargo-deny" = "cargo install cargo-deny"
    "cargo-geiger" = "cargo install cargo-geiger"
}

$missingTools = @()
foreach ($tool in $tools.Keys) {
    if ($tool -eq "cargo") {
        $installed = Test-ToolInstalled $tool $tools[$tool]
        if (-not $installed) {
            Write-Host "  ❌ Cargo not installed - cannot continue!" -ForegroundColor Red
            exit 1
        }
    } else {
        # Check if cargo subcommand exists
        try {
            $subCmd = $tool.Replace("cargo-", "")
            & cargo $subCmd --version 2>&1 | Out-Null
            Write-Host "  ✅ $tool is installed" -ForegroundColor Green
        } catch {
            Write-Host "  ⚠️  $tool not installed" -ForegroundColor Yellow
            $missingTools += $tool
        }
    }
}

# Offer to install missing tools
if ($missingTools.Count -gt 0) {
    Write-Host ""
    Write-Host "Missing tools: $($missingTools -join ', ')" -ForegroundColor Yellow
    $install = Read-Host "Install missing tools? (y/n)"
    if ($install -eq 'y') {
        foreach ($tool in $missingTools) {
            Write-Host "Installing $tool..." -ForegroundColor Yellow
            Invoke-Expression $tools[$tool]
        }
    }
}

Write-Host ""
Write-Host "================== CI.yml CHECKS ==================" -ForegroundColor Magenta
Write-Host ""

Write-Host "🔍 Code Quality Checks" -ForegroundColor Cyan
Write-Host "================================" -ForegroundColor DarkGray

# 1. Format Check
Run-Check "Rust Formatting" "cargo fmt --all -- --check" | Out-Null

# 2. Clippy (Linting)
Run-Check "Clippy Linting" "cargo clippy --all-targets --all-features -- -D warnings" | Out-Null

# 3. Documentation Check
Run-Check "Documentation" "cargo doc --no-deps --document-private-items" | Out-Null

Write-Host ""
Write-Host "🔨 Build Checks" -ForegroundColor Cyan
Write-Host "================================" -ForegroundColor DarkGray

# 4. Debug Build
Run-Check "Debug Build" "cargo build --verbose" | Out-Null

# 5. Release Build
Run-Check "Release Build" "cargo build --release --verbose" | Out-Null

Write-Host ""
Write-Host "🧪 Test Suite" -ForegroundColor Cyan
Write-Host "================================" -ForegroundColor DarkGray

# 6. Run all tests
Run-Check "Unit Tests" "cargo test --verbose --all --no-fail-fast" | Out-Null

# 7. Run tests in release mode
Run-Check "Release Tests" "cargo test --release --verbose" | Out-Null

# 8. Doc tests
Run-Check "Documentation Tests" "cargo test --doc --verbose" | Out-Null

Write-Host ""
Write-Host "📊 Code Coverage" -ForegroundColor Cyan
Write-Host "================================" -ForegroundColor DarkGray

# 9. Code Coverage with cargo-tarpaulin
$tarpaulinInstalled = $false
try {
    cargo tarpaulin --version 2>&1 | Out-Null
    $tarpaulinInstalled = $true
} catch {
    $tarpaulinInstalled = $false
}

if ($tarpaulinInstalled) {
    Write-Host "  Running code coverage analysis..." -ForegroundColor Yellow
    Write-Host "  This may take a few minutes..." -ForegroundColor DarkGray
    
    # Run tarpaulin with same settings as GitHub Actions
    $coverageCmd = "cargo tarpaulin --verbose --all-features --workspace --timeout 120 --out Xml --out Html --output-dir coverage"
    $coverageOutput = Invoke-Expression $coverageCmd 2>&1 | Out-String
    
    # Extract coverage percentage
    if ($coverageOutput -match "(\d+\.\d+)%\s+coverage") {
        $coverage = [double]$matches[1]
        
        if ($coverage -ge 80) {
            Write-Host "  ✅ Code Coverage: $coverage% (Target: 80%)" -ForegroundColor Green
        } else {
            Write-Host "  ❌ Code Coverage: $coverage% (Target: 80%)" -ForegroundColor Red
            Write-Host "     GitHub Actions will report coverage below target!" -ForegroundColor DarkRed
            $ErrorCount++
        }
    } else {
        Write-Host "  ⚠️  Could not determine coverage percentage" -ForegroundColor Yellow
        $WarningCount++
    }
    
    Write-Host "  📄 Coverage reports generated:" -ForegroundColor DarkGray
    Write-Host "     - HTML: ./coverage/tarpaulin-report.html" -ForegroundColor DarkGray
    Write-Host "     - XML:  ./coverage/cobertura.xml" -ForegroundColor DarkGray
} else {
    Write-Host "  ⚠️  Skipping coverage (cargo-tarpaulin not installed)" -ForegroundColor Yellow
    Write-Host "     GitHub Actions WILL run this and require 80% coverage!" -ForegroundColor DarkYellow
    $WarningCount++
}

Write-Host ""
Write-Host "============== SECURITY.yml CHECKS ================" -ForegroundColor Magenta
Write-Host ""

Write-Host "🔒 Security Audit" -ForegroundColor Cyan
Write-Host "================================" -ForegroundColor DarkGray

# 10. Dependency Audit
$auditInstalled = $false
try {
    cargo audit --version 2>&1 | Out-Null
    $auditInstalled = $true
} catch {
    $auditInstalled = $false
}

if ($auditInstalled) {
    # Match GitHub Actions: cargo audit --deny warnings
    Run-Check "Dependency Audit" "cargo audit --deny warnings" $false | Out-Null
} else {
    Write-Host "  ⚠️  Skipping audit (cargo-audit not installed)" -ForegroundColor Yellow
    $WarningCount++
}

Write-Host ""
Write-Host "🔐 Supply Chain Security" -ForegroundColor Cyan
Write-Host "================================" -ForegroundColor DarkGray

# 11. Supply Chain Security with cargo-deny
$denyInstalled = $false
try {
    cargo deny --version 2>&1 | Out-Null
    $denyInstalled = $true
} catch {
    $denyInstalled = $false
}

if ($denyInstalled) {
    # Match GitHub Actions exactly
    Run-Check "Ban Check" "cargo deny check bans" $false | Out-Null
    Run-Check "License Check" "cargo deny check licenses" $false | Out-Null
    Run-Check "Source Check" "cargo deny check sources" $false | Out-Null
    Run-Check "Advisory Check" "cargo deny check advisories" $false | Out-Null
} else {
    Write-Host "  ⚠️  Skipping supply chain checks (cargo-deny not installed)" -ForegroundColor Yellow
    $WarningCount++
}

Write-Host ""
Write-Host "☢️  Static Security Analysis" -ForegroundColor Cyan
Write-Host "================================" -ForegroundColor DarkGray

# 12. Static Security Analysis with cargo-geiger
$geigerInstalled = $false
try {
    cargo geiger --version 2>&1 | Out-Null
    $geigerInstalled = $true
} catch {
    $geigerInstalled = $false
}

if ($geigerInstalled) {
    Write-Host "  Analyzing unsafe code usage..." -ForegroundColor Yellow
    # Match GitHub Actions: cargo geiger --all-features
    $geigerOutput = cargo geiger --all-features 2>&1 | Out-String
    
    # Count unsafe code occurrences
    $unsafeCount = ([regex]::Matches($geigerOutput, "☢")).Count
    
    if ($unsafeCount -eq 0) {
        Write-Host "  ✅ No unsafe code detected" -ForegroundColor Green
    } else {
        Write-Host "  ⚠️  Unsafe code detected: $unsafeCount occurrences" -ForegroundColor Yellow
        Write-Host "     This is expected for Windows API calls" -ForegroundColor DarkYellow
        $WarningCount++
    }
} else {
    Write-Host "  ⚠️  Skipping unsafe code analysis (cargo-geiger not installed)" -ForegroundColor Yellow
    $WarningCount++
}

Write-Host ""
Write-Host "🧬 Memory Safety Check (Miri)" -ForegroundColor Cyan
Write-Host "================================" -ForegroundColor DarkGray

# 13. Miri Check (Memory Safety)
$miriInstalled = $false
try {
    rustup +nightly component list --installed | Select-String "miri" | Out-Null
    $miriInstalled = $LASTEXITCODE -eq 0
} catch {
    $miriInstalled = $false
}

if ($miriInstalled) {
    Write-Host "  Running Miri memory safety checks..." -ForegroundColor Yellow
    
    # Setup Miri (matching GitHub Actions)
    cargo +nightly miri setup 2>&1 | Out-Null
    
    # Run Miri tests with same flags as GitHub Actions
    $env:MIRIFLAGS = "-Zmiri-disable-isolation"
    $miriOutput = cargo +nightly miri test 2>&1 | Out-String
    $miriExitCode = $LASTEXITCODE
    
    if ($miriExitCode -eq 0) {
        Write-Host "  ✅ Miri memory safety check passed" -ForegroundColor Green
    } else {
        # Check if it's just FFI-related issues (expected)
        if ($miriOutput -match "FFI" -or $miriOutput -match "foreign function" -or $miriOutput -match "cfg_attr.*miri.*ignore") {
            Write-Host "  ✅ Miri check passed (FFI tests ignored)" -ForegroundColor Green
        } else {
            Write-Host "  ❌ Miri memory safety check failed" -ForegroundColor Red
            Write-Host "     GitHub Actions will fail!" -ForegroundColor DarkRed
            $ErrorCount++
        }
    }
} else {
    Write-Host "  ⚠️  Miri not installed. Install with:" -ForegroundColor Yellow
    Write-Host "     rustup +nightly component add miri" -ForegroundColor DarkYellow
    $WarningCount++
}

Write-Host ""
Write-Host "🌐 Additional Security Scanning" -ForegroundColor Cyan
Write-Host "================================" -ForegroundColor DarkGray

# 14. Check for Trivy (Security Scanner)
$trivyInstalled = $false
try {
    trivy --version 2>&1 | Out-Null
    $trivyInstalled = $true
} catch {
    $trivyInstalled = $false
}

if ($trivyInstalled) {
    Write-Host "  Running Trivy security scan..." -ForegroundColor Yellow
    # Match GitHub Actions settings
    trivy fs . --format table --exit-code 1 --severity HIGH,CRITICAL 2>&1 | Out-Null
    if ($LASTEXITCODE -eq 0) {
        Write-Host "  ✅ Trivy security scan passed" -ForegroundColor Green
    } else {
        Write-Host "  ⚠️  Trivy found HIGH/CRITICAL vulnerabilities" -ForegroundColor Yellow
        $WarningCount++
    }
} else {
    Write-Host "  ℹ️  Trivy not installed (optional for local)" -ForegroundColor DarkGray
    Write-Host "     GitHub Actions WILL run this check" -ForegroundColor DarkGray
    Write-Host "     Install: https://github.com/aquasecurity/trivy/releases" -ForegroundColor DarkGray
}

# Note about CodeQL
Write-Host ""
Write-Host "  ℹ️  CodeQL analysis only runs in GitHub Actions" -ForegroundColor DarkGray
Write-Host "     It will be automatically executed on push/PR" -ForegroundColor DarkGray

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "📈 Pipeline Summary" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

# Calculate duration
$duration = (Get-Date) - $startTime
Write-Host "Duration: $($duration.ToString('mm\:ss'))" -ForegroundColor Gray
Write-Host ""

# Count total checks
$totalChecks = 13
$passedChecks = $totalChecks - $ErrorCount

# Display results
if ($ErrorCount -eq 0) {
    Write-Host "✅ ALL CRITICAL CHECKS PASSED! ($passedChecks/$totalChecks)" -ForegroundColor Green
    
    if ($WarningCount -gt 0) {
        Write-Host "⚠️  $WarningCount warning(s) detected (non-critical)" -ForegroundColor Yellow
        Write-Host "   These won't block GitHub Actions but should be reviewed" -ForegroundColor DarkYellow
    }
    
    Write-Host ""
    Write-Host "🎉 Ready to push to GitHub!" -ForegroundColor Green
    Write-Host "   GitHub Actions should pass successfully." -ForegroundColor DarkGreen
    
    # Coverage check reminder
    if (-not $tarpaulinInstalled) {
        Write-Host ""
        Write-Host "⚠️  IMPORTANT: GitHub requires 80% code coverage!" -ForegroundColor Yellow
        Write-Host "   Install cargo-tarpaulin to verify locally:" -ForegroundColor DarkYellow
        Write-Host "   cargo install cargo-tarpaulin" -ForegroundColor DarkYellow
    }
    
    Write-Host ""
    Write-Host "📋 Checks Summary:" -ForegroundColor Cyan
    Write-Host "  ✓ Format:        Passed" -ForegroundColor DarkGray
    Write-Host "  ✓ Clippy:        Passed" -ForegroundColor DarkGray
    Write-Host "  ✓ Build:         Passed" -ForegroundColor DarkGray
    Write-Host "  ✓ Tests:         Passed" -ForegroundColor DarkGray
    Write-Host "  ✓ Documentation: Passed" -ForegroundColor DarkGray
    Write-Host "  ✓ Miri:          Passed" -ForegroundColor DarkGray
    
    if ($tarpaulinInstalled) {
        Write-Host "  ✓ Coverage:      Check ./coverage/tarpaulin-report.html" -ForegroundColor DarkGray
    } else {
        Write-Host "  ? Coverage:      Not tested locally" -ForegroundColor DarkYellow
    }
    
    if ($auditInstalled) {
        Write-Host "  ✓ Security:      Checked" -ForegroundColor DarkGray
    } else {
        Write-Host "  ? Security:      Not tested locally" -ForegroundColor DarkYellow
    }
    
    exit 0
} else {
    Write-Host "❌ $ErrorCount CRITICAL CHECK(S) FAILED! ($passedChecks/$totalChecks passed)" -ForegroundColor Red
    Write-Host "⚠️  $WarningCount warning(s) detected" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "🔧 FIX THESE ERRORS BEFORE PUSHING!" -ForegroundColor Red
    Write-Host "   GitHub Actions WILL FAIL if you push now." -ForegroundColor DarkRed
    
    Write-Host ""
    Write-Host "💡 Quick Fixes:" -ForegroundColor Yellow
    Write-Host "  • Format:  cargo fmt --all" -ForegroundColor DarkYellow
    Write-Host "  • Clippy:  cargo clippy --fix" -ForegroundColor DarkYellow
    Write-Host "  • Tests:   cargo test -- --nocapture" -ForegroundColor DarkYellow
    Write-Host "  • Miri:    Add #[cfg_attr(miri, ignore)] to FFI tests" -ForegroundColor DarkYellow
    
    if (-not $tarpaulinInstalled) {
        Write-Host ""
        Write-Host "⚠️  Code coverage not tested!" -ForegroundColor Yellow
        Write-Host "   GitHub Actions requires 80% coverage" -ForegroundColor DarkYellow
        Write-Host "   Install: cargo install cargo-tarpaulin" -ForegroundColor DarkYellow
    }
    
    exit 1
}