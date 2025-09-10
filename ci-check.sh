#!/bin/bash
# Local CI validation script - Run before pushing!
# Usage: ./ci-check.sh

echo -e "\033[36m🔍 Running Complete CI Validation Locally\033[0m"
echo -e "\033[36m========================================\033[0m"

failed=false

# 1. Format Check
echo -e "\n\033[33m📝 Checking Code Formatting...\033[0m"
if cargo fmt --all -- --check; then
    echo -e "\033[32m✅ Formatting OK\033[0m"
else
    echo -e "\033[31m❌ Formatting issues found! Run: cargo fmt --all\033[0m"
    failed=true
fi

# 2. Clippy (with CI flags)
echo -e "\n\033[33m🔍 Running Clippy with CI settings...\033[0m"
if cargo clippy --all-targets --all-features -- -D warnings; then
    echo -e "\033[32m✅ Clippy OK\033[0m"
else
    echo -e "\033[31m❌ Clippy warnings found!\033[0m"
    failed=true
fi

# 3. Build Check
echo -e "\n\033[33m🔨 Building project...\033[0m"
if cargo build --release; then
    echo -e "\033[32m✅ Build OK\033[0m"
else
    echo -e "\033[31m❌ Build failed!\033[0m"
    failed=true
fi

# 4. Test Suite
echo -e "\n\033[33m🧪 Running all tests...\033[0m"
if cargo test --all --no-fail-fast; then
    echo -e "\033[32m✅ Tests OK\033[0m"
else
    echo -e "\033[31m❌ Tests failed!\033[0m"
    failed=true
fi

# 5. Doc Check
echo -e "\n\033[33m📚 Checking documentation...\033[0m"
if cargo doc --no-deps --document-private-items; then
    echo -e "\033[32m✅ Documentation OK\033[0m"
else
    echo -e "\033[31m❌ Documentation issues!\033[0m"
    failed=true
fi

# 6. Security Audit (if available)
echo -e "\n\033[33m🔒 Running security audit...\033[0m"
if cargo audit 2>/dev/null; then
    echo -e "\033[32m✅ Security audit OK\033[0m"
else
    echo -e "\033[33m⚠️  Security audit not available or found issues\033[0m"
fi

# 7. Check for common issues
echo -e "\n\033[33m🔎 Checking for common CI issues...\033[0m"

# Check for println! in non-test code
if grep -r "println!" src/ --include="*.rs" 2>/dev/null; then
    echo -e "\033[33m⚠️  Found println! in source code (should use log crate)\033[0m"
fi

# Check for unwrap() in non-test code
if grep -r "\.unwrap()" src/ --include="*.rs" 2>/dev/null; then
    echo -e "\033[33m⚠️  Found unwrap() in source code (use ? or expect())\033[0m"
fi

# Check for TODO/FIXME
if grep -r "TODO\|FIXME\|HACK" src/ --include="*.rs" 2>/dev/null; then
    echo -e "\033[33m⚠️  Found TODO/FIXME/HACK comments\033[0m"
fi

echo -e "\n\033[36m========================================\033[0m"
if [ "$failed" = true ]; then
    echo -e "\033[31m❌ CI checks FAILED - Fix issues before pushing!\033[0m"
    exit 1
else
    echo -e "\033[32m✅ All CI checks PASSED - Safe to push!\033[0m"
    echo -e "\n\033[36mTip: Run 'git push' to update your PR\033[0m"
fi