@echo off
REM Local CI validation script - Run before pushing!
REM Usage: ci-check.bat

echo =======================================
echo Running Complete CI Validation Locally
echo =======================================

set FAILED=0

echo.
echo Checking Code Formatting...
cargo fmt --all -- --check
if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] Formatting issues found! Run: cargo fmt --all
    set FAILED=1
) else (
    echo [OK] Formatting OK
)

echo.
echo Running Clippy with CI settings...
cargo clippy --all-targets --all-features -- -D warnings
if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] Clippy warnings found!
    set FAILED=1
) else (
    echo [OK] Clippy OK
)

echo.
echo Building project...
cargo build --release
if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] Build failed!
    set FAILED=1
) else (
    echo [OK] Build OK
)

echo.
echo Running all tests...
cargo test --all --no-fail-fast
if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] Tests failed!
    set FAILED=1
) else (
    echo [OK] Tests OK
)

echo.
echo Checking documentation...
cargo doc --no-deps --document-private-items
if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] Documentation issues!
    set FAILED=1
) else (
    echo [OK] Documentation OK
)

echo.
echo Running security audit...
cargo audit 2>nul
if %ERRORLEVEL% EQU 0 (
    echo [OK] Security audit OK
) else (
    echo [WARNING] Security audit not available or found issues
)

echo.
echo =======================================
if %FAILED% EQU 1 (
    echo [ERROR] CI checks FAILED - Fix issues before pushing!
    exit /b 1
) else (
    echo [OK] All CI checks PASSED - Safe to push!
    echo.
    echo Tip: Run 'git push' to update your PR
)