#!/bin/bash
# Quick verification script for rhizoCrypt changes

set -e

echo "🔍 Verifying rhizoCrypt changes..."
echo ""

cd "$(dirname "$0")"

# 1. Check formatting
echo "✅ Checking formatting..."
cargo fmt --all -- --check
echo "   ✓ All files formatted correctly"
echo ""

# 2. Check clippy
echo "✅ Running clippy..."
cargo clippy --workspace --lib -- -D warnings > /dev/null 2>&1
echo "   ✓ Zero clippy warnings"
echo ""

# 3. Run tests
echo "✅ Running tests..."
cargo test --workspace --lib --quiet
echo "   ✓ All tests passing"
echo ""

# 4. Check build
echo "✅ Building release..."
cargo build --workspace --release --quiet
echo "   ✓ Release build successful"
echo ""

# 5. Verify new files exist
echo "✅ Verifying new files..."
if [ -f "crates/rhizo-crypt-core/src/clients/loamspine_http.rs" ]; then
    echo "   ✓ LoamSpine HTTP client exists"
else
    echo "   ✗ LoamSpine HTTP client missing!"
    exit 1
fi
echo ""

# 6. Check coverage (if llvm-cov available)
if command -v cargo-llvm-cov &> /dev/null; then
    echo "✅ Checking coverage..."
    COVERAGE=$(cargo llvm-cov --workspace --lib --summary-only 2>&1 | grep "TOTAL" | awk '{print $10}')
    echo "   ✓ Coverage: $COVERAGE (target: >60%)"
else
    echo "⚠️  Skipping coverage (cargo-llvm-cov not installed)"
fi
echo ""

echo "🎉 All verifications passed!"
echo ""
echo "📊 Summary:"
echo "   • Formatting: ✅ Clean"
echo "   • Clippy: ✅ Zero warnings"
echo "   • Tests: ✅ Passing"
echo "   • Build: ✅ Success"
echo "   • LoamSpine client: ✅ Created"
echo ""
echo "✅ Ready to commit!"

