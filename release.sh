#!/bin/bash

# Release script for ai_code_buddy
# Usage: ./release.sh <version>

set -e

if [ $# -eq 0 ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 0.1.0"
    exit 1
fi

VERSION=$1

echo "🚀 Preparing release $VERSION..."

# Update version in Cargo.toml
sed -i '' "s/^version = .*/version = \"$VERSION\"/" Cargo.toml

# Run tests
echo "🧪 Running tests..."
cargo test

# Build release
echo "🔨 Building release..."
cargo build --release

# Run final check
echo "✅ Running final check..."
cargo check

echo "📦 Ready to tag and push!"
echo "Run the following commands to complete the release:"
echo "  git add ."
echo "  git commit -m \"Release v$VERSION\""
echo "  git tag v$VERSION"
echo "  git push origin main"
echo "  git push origin v$VERSION"

echo "🎉 Release $VERSION is ready!"
