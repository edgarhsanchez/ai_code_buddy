#!/bin/bash
# filepath: /Volumes/U34 Bolt/Documents/github/ai_code_buddy/bump_version.sh

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
if [ -f "Cargo.toml" ]; then
  sed -i.bak 's/^version = ".*"/version = "'"$NEW_VERSION"'"/' Cargo.toml
  rm Cargo.toml.bak  # Remove backup file
else
  echo "Error: Cargo.toml not found. Adjust the script for your version file."
  exit 1
fi

# Update Cargo.lock to reflect the new version
echo "Updating Cargo.lock..."
cargo update --workspace

# Create and switch to new branch
git checkout -b "$BRANCH_NAME"

# Add both Cargo.toml and Cargo.lock
git add Cargo.toml Cargo.lock
git commit -m "chore: bump version to $NEW_VERSION"
git push origin "$BRANCH_NAME"

# Create and push tag
git tag "v$NEW_VERSION"
git push origin "v$NEW_VERSION"

echo "Version bumped to $NEW_VERSION, pushed to branch $BRANCH_NAME, and tagged as v$NEW_VERSION."
echo "Create a pull request from $BRANCH_NAME to main on GitHub."