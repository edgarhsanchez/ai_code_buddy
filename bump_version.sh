#!/bin/bash

# filepath: /Volumes/U34 Bolt/Documents/github/ai_code_buddy/bump_version.sh

# Check if a version argument is provided
if [ -z "$1" ]; then
  echo "Usage: $0 <new_version>"
  exit 1
fi

NEW_VERSION=$1
BRANCH_NAME="bump-version-$NEW_VERSION"

# Update version in Cargo.toml (adjust path if different)
if [ -f "Cargo.toml" ]; then
  sed -i.bak '0,/version = ".*"/s//version = "'"$NEW_VERSION"'"/' Cargo.toml
  rm Cargo.toml.bak  # Remove backup file
else
  echo "Error: Cargo.toml not found. Adjust the script for your version file."
  exit 1
fi

# ...existing code...

# Create and switch to new branch
git checkout -b "$BRANCH_NAME"

# Add, commit, and push
git add Cargo.toml
git commit -m "chore: bump version to $NEW_VERSION"
git push origin "$BRANCH_NAME"

# Create and push tag
git tag "v$NEW_VERSION"
git push origin "v$NEW_VERSION"

echo "Version bumped to $NEW_VERSION, pushed to branch $BRANCH_NAME, and tagged as v$NEW_VERSION."
echo "Create a pull request from $BRANCH_NAME to main on GitHub."