#!/bin/bash
# filepath: /Volumes/U34 Bolt/Documents/github/ai_code_buddy/bump_version.sh

# Run clippy to check for warnings before proceeding
echo "🔍 Running cargo clippy to check for warnings..."
if ! cargo clippy -- -D warnings; then
  echo "❌ Clippy found warnings or errors. Please fix them before bumping version."
  exit 1
fi
echo "✅ Clippy check passed!"

# Check if git working directory is clean
if ! git diff --quiet || ! git diff --staged --quiet; then
  echo "❌ Git working directory is not clean. Please commit or stash changes before bumping version."
  exit 1
fi
echo "✅ Git working directory is clean!"

# If no version provided, auto-increment patch version
if [ -z "$1" ]; then
  # Extract current version from Cargo.toml
  CURRENT_VERSION=$(grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
  
  if [ -z "$CURRENT_VERSION" ]; then
    echo "Error: Could not find current version in Cargo.toml"
    exit 1
  fi
  
  # Parse version and increment patch
  IFS='.' read -r MAJOR MINOR PATCH <<< "$CURRENT_VERSION"
  NEW_PATCH=$((PATCH + 1))
  NEW_VERSION="$MAJOR.$MINOR.$NEW_PATCH"
  
  echo "Auto-incrementing version from $CURRENT_VERSION to $NEW_VERSION"
else
  NEW_VERSION=$1
fi

BRANCH_NAME="bump-version-$NEW_VERSION"

# Update version in Cargo.toml (only the package version, not dependencies)
echo "📝 Updating version in Cargo.toml to $NEW_VERSION..."
if [ -f "Cargo.toml" ]; then
  sed -i.bak 's/^version = ".*"/version = "'"$NEW_VERSION"'"/' Cargo.toml
  rm Cargo.toml.bak  # Remove backup file
  echo "✅ Cargo.toml updated successfully!"
else
  echo "Error: Cargo.toml not found. Adjust the script for your version file."
  exit 1
fi

# Update Cargo.lock to reflect the new version
echo "🔄 Updating Cargo.lock..."
cargo update --workspace
echo "✅ Cargo.lock updated successfully!"

# Create and switch to new branch
echo "🌿 Creating new branch: $BRANCH_NAME..."
git checkout -b "$BRANCH_NAME"
echo "✅ Branch created and checked out!"

# Add both Cargo.toml and Cargo.lock
echo "📦 Staging version files..."
git add Cargo.toml Cargo.lock
echo "✅ Files staged!"

echo "💾 Committing version bump..."
git commit -m "chore: bump version to $NEW_VERSION"
echo "✅ Commit created!"

echo "🚀 Pushing branch to remote..."
git push origin "$BRANCH_NAME"
echo "✅ Branch pushed!"

# Create and push tag
echo "🏷️  Creating version tag: v$NEW_VERSION..."
git tag "v$NEW_VERSION"
echo "🚀 Pushing tag to remote..."
git push origin "v$NEW_VERSION"
echo "✅ Tag created and pushed!"

echo ""
echo "🎉 Version bump complete!"
echo "   New version: $NEW_VERSION"
echo "   Branch: $BRANCH_NAME"
echo "   Tag: v$NEW_VERSION"

echo "Version bumped to $NEW_VERSION, pushed to branch $BRANCH_NAME, and tagged as v$NEW_VERSION."
echo "Create a pull request from $BRANCH_NAME to main on GitHub."