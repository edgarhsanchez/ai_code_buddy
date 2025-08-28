#!/bin/bash

# AI Code Buddy - Conventional Commit Helper
# This script helps create conventional commit messages

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🤖 AI Code Buddy - Conventional Commit Helper${NC}"
echo ""

# Function to show usage
show_usage() {
    echo -e "${YELLOW}Usage:${NC}"
    echo "  $0 [type] [description]"
    echo ""
    echo -e "${YELLOW}Types:${NC}"
    echo "  feat     - A new feature"
    echo "  fix      - A bug fix"
    echo "  docs     - Documentation only changes"
    echo "  style    - Changes that do not affect the meaning of the code"
    echo "  refactor - A code change that neither fixes a bug nor adds a feature"
    echo "  perf     - A code change that improves performance"
    echo "  test     - Adding missing tests or correcting existing tests"
    echo "  build    - Changes that affect the build system or external dependencies"
    echo "  ci       - Changes to our CI configuration files and scripts"
    echo "  chore    - Other changes that don't modify src or test files"
    echo ""
    echo -e "${YELLOW}Examples:${NC}"
    echo "  $0 feat 'add user authentication'"
    echo "  $0 fix 'resolve memory leak in analysis'"
    echo "  $0 docs 'update README with installation instructions'"
    echo ""
}

# Check if arguments are provided
if [ $# -eq 0 ]; then
    show_usage
    exit 1
fi

TYPE=$1
DESCRIPTION=$2

# Validate type
VALID_TYPES=("feat" "fix" "docs" "style" "refactor" "perf" "test" "build" "ci" "chore")
if [[ ! " ${VALID_TYPES[@]} " =~ " ${TYPE} " ]]; then
    echo -e "${RED}❌ Invalid commit type: ${TYPE}${NC}"
    echo ""
    show_usage
    exit 1
fi

# Check if description is provided
if [ -z "$DESCRIPTION" ]; then
    echo -e "${RED}❌ Description is required${NC}"
    echo ""
    show_usage
    exit 1
fi

# Create commit message
COMMIT_MSG="${TYPE}: ${DESCRIPTION}"

# Show preview
echo -e "${GREEN}✅ Commit message preview:${NC}"
echo "  $COMMIT_MSG"
echo ""

# Confirm
read -p "Do you want to create this commit? (y/N): " -n 1 -r
echo ""

if [[ $REPLY =~ ^[Yy]$ ]]; then
    # Add all changes
    git add .

    # Create commit
    git commit -m "$COMMIT_MSG"

    echo -e "${GREEN}✅ Commit created successfully!${NC}"
    echo ""
    echo -e "${BLUE}📝 Commit details:${NC}"
    echo "  Type: $TYPE"
    echo "  Description: $DESCRIPTION"
    echo "  Message: $COMMIT_MSG"
    echo ""
    echo -e "${YELLOW}🚀 Ready to push your changes!${NC}"
else
    echo -e "${YELLOW}Commit cancelled.${NC}"
fi
