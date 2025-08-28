#!/bin/bash

# update_coverage.sh - Update coverage badges and reports
# This script runs coverage analysis and updates the README badges

set -e

echo "🧪 Running code coverage analysis..."

# Run coverage analysis
cargo tarpaulin --verbose --timeout 120 --out Html --out Json --no-default-features

# Check if coverage report was generated
if [ ! -f "tarpaulin-report.json" ]; then
    echo "❌ Coverage report not generated"
    exit 1
fi

# Extract coverage percentage from JSON
if command -v jq &> /dev/null; then
    # Use jq if available (more robust)
    COVERAGE=$(jq -r '.coverage' tarpaulin-report.json 2>/dev/null || echo "0")
else
    # Fallback to grep/cut
    COVERAGE=$(grep -o '"coverage":[0-9.]*' tarpaulin-report.json | cut -d':' -f2 | head -1 | tr -d ' ')
fi

# Remove any trailing decimals for badge (keep 2 decimal places)
COVERAGE_ROUNDED=$(printf "%.2f" "$COVERAGE" 2>/dev/null || echo "0.00")

echo "📊 Current coverage: $COVERAGE_ROUNDED%"

# Determine badge color based on coverage
if (( $(echo "$COVERAGE_ROUNDED >= 80" | bc -l 2>/dev/null || echo "0") )); then
    COLOR="brightgreen"
elif (( $(echo "$COVERAGE_ROUNDED >= 60" | bc -l 2>/dev/null || echo "0") )); then
    COLOR="yellow"
elif (( $(echo "$COVERAGE_ROUNDED >= 40" | bc -l 2>/dev/null || echo "0") )); then
    COLOR="orange"
else
    COLOR="red"
fi

echo "🎨 Badge color: $COLOR"

# Update README.md coverage badge
if [ -f "README.md" ]; then
    # Create backup
    cp README.md README.md.bak
    
    # Update coverage badge
    sed -i.tmp "s/coverage-[0-9.]*%25-[a-z]*/coverage-${COVERAGE_ROUNDED}%25-${COLOR}/g" README.md
    
    # Clean up temp file
    rm -f README.md.tmp
    
    echo "✅ Updated coverage badge in README.md"
else
    echo "⚠️  README.md not found"
fi

# Count total tests
TEST_COUNT=$(cargo test --no-default-features 2>&1 | grep "test result:" | awk '{sum += $4} END {print sum}' || echo "0")

# Update test count badge if we can determine it
if [ "$TEST_COUNT" != "0" ]; then
    sed -i.tmp "s/tests-[0-9]*%20passing/tests-${TEST_COUNT}%20passing/g" README.md
    rm -f README.md.tmp
    echo "✅ Updated test count badge: $TEST_COUNT tests"
fi

echo ""
echo "📋 Coverage Summary:"
echo "   Coverage: $COVERAGE_ROUNDED%"
echo "   Color: $COLOR"
echo "   Tests: $TEST_COUNT"
echo ""
echo "📁 Generated files:"
echo "   - tarpaulin-report.html (detailed coverage report)"
echo "   - tarpaulin-report.json (machine-readable data)"
echo ""
echo "🚀 To view the detailed report:"
echo "   open tarpaulin-report.html"
echo ""
echo "✨ Coverage update complete!"
