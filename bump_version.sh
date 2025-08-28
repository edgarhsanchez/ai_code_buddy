#!/bin/bash
# filepath: /Volumes/U34 Bolt/Documents/github/ai_code_buddy/bump_version.sh

# Function to generate credits.rs with current contributors and dependencies
generate_credits() {
    echo "🔄 Generating credits.rs with current contributors and dependencies..."

    # Get project contributors from git
    CONTRIBUTORS=$(git log --format='%aN|%aE' | sort | uniq -c | sort -nr | head -10)

    # Parse Cargo.toml for dependencies (both regular and dev dependencies)
    DEPENDENCIES=$(sed -n "/^\[dependencies\]/,/^\[.*\]/p" Cargo.toml | grep "^[a-zA-Z0-9_-]* = " | sed "s/ = .*$//" | sort)
    DEV_DEPENDENCIES=$(sed -n "/^\[dev-dependencies\]/,/^\[.*\]/p" Cargo.toml | grep "^[a-zA-Z0-9_-]* = " | sed "s/ = .*$//" | sort)
    ALL_DEPENDENCIES=$(echo -e "$DEPENDENCIES\n$DEV_DEPENDENCIES" | sort | uniq)

    # Create credits.rs file
    cat > src/core/credits.rs << 'EOF'
use std::collections::HashMap;

/// Information about a library dependency
#[derive(Debug, Clone)]
pub struct LibraryInfo {
    pub name: &'static str,
    pub version: &'static str,
    pub license: &'static str,
    pub description: &'static str,
    pub repository: &'static str,
    pub contributors: Vec<&'static str>,
}

/// Project contributor information
#[derive(Debug, Clone)]
pub struct Contributor {
    pub name: &'static str,
    pub email: &'static str,
    pub contributions: u32,
}

/// Get all project contributors from git history
pub fn get_project_contributors() -> Vec<Contributor> {
    vec![
EOF

    # Add contributors from git
    FIRST=true
    echo "$CONTRIBUTORS" | while IFS= read -r line; do
        if [ "$FIRST" = true ]; then
            FIRST=false
        else
            echo "," >> src/core/credits.rs
        fi

        # Parse contributor info (count name|email)
        COUNT=$(echo "$line" | awk '{print $1}' | tr -d ' ')
        NAME_EMAIL=$(echo "$line" | sed 's/^[[:space:]]*[0-9][0-9]*[[:space:]]*//')  # Remove count and spaces
        NAME=$(echo "$NAME_EMAIL" | cut -d'|' -f1 | sed 's/^ *//;s/ *$//')  # Trim spaces
        EMAIL=$(echo "$NAME_EMAIL" | cut -d'|' -f2)

        cat >> src/core/credits.rs << EOF
        Contributor {
            name: "$NAME",
            email: "$EMAIL",
            contributions: $COUNT,
        }
EOF
    done

    cat >> src/core/credits.rs << 'EOF'
    ]
}

/// Get all library dependencies with their information
pub fn get_library_dependencies() -> Vec<LibraryInfo> {
    vec![
EOF

    # Add dependencies with license information
    FIRST=true
    echo "$ALL_DEPENDENCIES" | while IFS= read -r dep; do
        if [ -z "$dep" ]; then continue; fi

        if [ "$FIRST" = true ]; then
            FIRST=false
        else
            echo "," >> src/core/credits.rs
        fi

        # Get version from Cargo.toml
        DEP_LINE=$(grep "^$dep = " Cargo.toml)
        if [[ $DEP_LINE == *'"'* ]]; then
            VERSION=$(echo "$DEP_LINE" | sed 's/.*= "\([^"]*\)".*/\1/')
        elif [[ $DEP_LINE == *'{'* ]]; then
            VERSION=$(echo "$DEP_LINE" | sed 's/.*version = "\([^"]*\)".*/\1/')
        else
            VERSION="latest"
        fi
        if [ -z "$VERSION" ]; then
            VERSION="latest"
        fi

        # Get license info for common crates
        case $dep in
            "anyhow")
                LICENSE="MIT OR Apache-2.0"
                DESC="Flexible concrete Error type built on std::error::Error"
                REPO="https://github.com/dtolnay/anyhow"
                CONTRIBUTORS='vec!["David Tolnay", "And 50+ contributors"]'
                ;;
            "bevy")
                LICENSE="MIT OR Apache-2.0"
                DESC="A refreshingly simple data-driven game engine built in Rust"
                REPO="https://github.com/bevyengine/bevy"
                CONTRIBUTORS='vec!["Carter Anderson", "Alice Cecile", "And 300+ contributors"]'
                ;;
            "clap")
                LICENSE="MIT OR Apache-2.0"
                DESC="A full featured, fast Command Line Argument Parser for Rust"
                REPO="https://github.com/clap-rs/clap"
                CONTRIBUTORS='vec!["Kevin K. <kbknapp@gmail.com>", "And 200+ contributors"]'
                ;;
            "tokio")
                LICENSE="MIT"
                DESC="An event-driven, non-blocking I/O platform for writing async I/O"
                REPO="https://github.com/tokio-rs/tokio"
                CONTRIBUTORS='vec!["Carl Lerche", "Sean McArthur", "And 200+ contributors"]'
                ;;
            "serde")
                LICENSE="MIT OR Apache-2.0"
                DESC="A generic serialization/deserialization framework"
                REPO="https://github.com/serde-rs/serde"
                CONTRIBUTORS='vec!["David Tolnay", "And 100+ contributors"]'
                ;;
            "regex")
                LICENSE="MIT OR Apache-2.0"
                DESC="An implementation of regular expressions for Rust"
                REPO="https://github.com/rust-lang/regex"
                CONTRIBUTORS='vec!["Andrew Gallant", "And 50+ contributors"]'
                ;;
            "git2")
                LICENSE="MIT OR Apache-2.0"
                DESC="Rust bindings to libgit2 for interoperating with git repositories"
                REPO="https://github.com/rust-lang/git2-rs"
                CONTRIBUTORS='vec!["Alex Crichton", "And 50+ contributors"]'
                ;;
            "ratatui")
                LICENSE="MIT"
                DESC="A Rust library to build rich terminal user interfaces"
                REPO="https://github.com/ratatui/ratatui"
                CONTRIBUTORS='vec!["Florian Dehau", "Joshka", "And 100+ contributors"]'
                ;;
            "uuid")
                LICENSE="Apache-2.0 OR MIT"
                DESC="A library to generate and parse UUIDs"
                REPO="https://github.com/uuid-rs/uuid"
                CONTRIBUTORS='vec!["Ashley Mannix", "And 50+ contributors"]'
                ;;
            "futures")
                LICENSE="MIT OR Apache-2.0"
                DESC="An implementation of futures and streams featuring zero allocations"
                REPO="https://github.com/rust-lang/futures-rs"
                CONTRIBUTORS='vec!["Alex Crichton", "And 100+ contributors"]'
                ;;
            "crossterm")
                LICENSE="MIT"
                DESC="Cross-platform terminal manipulation library"
                REPO="https://github.com/crossterm-rs/crossterm"
                CONTRIBUTORS='vec!["T. Postma", "And 50+ contributors"]'
                ;;
            "kalosm")
                LICENSE="MIT OR Apache-2.0"
                DESC="A user-friendly interface for pre-trained large language models"
                REPO="https://github.com/floneum/kalosm"
                CONTRIBUTORS='vec!["Evan Almloff", "And 10+ contributors"]'
                ;;
            "bevy_ratatui")
                LICENSE="MIT OR Apache-2.0"
                DESC="A Bevy plugin for Ratatui (terminal UI library)"
                REPO="https://github.com/bevy-ratatui/bevy_ratatui"
                CONTRIBUTORS='vec!["Johan Klokkhammer Helsing", "And 10+ contributors"]'
                ;;
            "color-eyre")
                LICENSE="MIT OR Apache-2.0"
                DESC="An error report handler for panics and eyre::Report"
                REPO="https://github.com/eyre-rs/color-eyre"
                CONTRIBUTORS='vec!["Jane Lusby", "And 20+ contributors"]'
                ;;
            *)
                LICENSE="Unknown"
                DESC="Rust library dependency"
                REPO="https://crates.io/crates/$dep"
                CONTRIBUTORS='vec!["Various contributors"]'
                ;;
        esac

        cat >> src/core/credits.rs << EOF
        LibraryInfo {
            name: "$dep",
            version: "$VERSION",
            license: "$LICENSE",
            description: "$DESC",
            repository: "$REPO",
            contributors: $CONTRIBUTORS,
        }
EOF
    done

    cat >> src/core/credits.rs << 'EOF'
    ]
}

/// Display comprehensive credits information
pub fn display_comprehensive_credits() {
    println!("🎉 AI Code Buddy - Comprehensive Credits & Acknowledgments");
    println!("==========================================================");
    println!();

    // Project Information
    println!("📚 About AI Code Buddy:");
    println!("An intelligent code analysis tool with elegant Bevy-powered TUI");
    println!("that provides comprehensive code reviews with AI assistance.");
    println!("Repository: https://github.com/edgarhsanchez/ai_code_buddy");
    println!();

    // Project Contributors
    println!("👥 Project Contributors:");
    println!("------------------------");
    let contributors = get_project_contributors();
    for contributor in contributors {
        println!("  • {} <{}> ({} commits)", contributor.name, contributor.email, contributor.contributions);
    }
    println!();

    // Library Dependencies
    println!("📦 Library Dependencies & Licenses:");
    println!("-----------------------------------");
    let libraries = get_library_dependencies();

    for library in libraries {
        println!("🔧 {} v{}", library.name, library.version);
        println!("   📄 License: {}", library.license);
        println!("   📖 Description: {}", library.description);
        println!("   🔗 Repository: {}", library.repository);
        println!("   👥 Key Contributors:");

        for contributor in &library.contributors {
            println!("     • {}", contributor);
        }
        println!();
    }

    // Special Thanks
    println!("🙏 Special Thanks:");
    println!("------------------");
    println!("  • The Rust Programming Language team");
    println!("  • All open source contributors");
    println!("  • The Bevy game engine community");
    println!("  • The broader Rust ecosystem");
    println!();

    // Call to Action
    println!("💡 Want to contribute? Visit: https://github.com/edgarhsanchez/ai_code_buddy");
    println!("🐛 Found a bug? Report it: https://github.com/edgarhsanchez/ai_code_buddy/issues");
}
EOF

    echo "✅ Generated credits.rs with current contributors and dependencies"
}

# Check for dry-run flag
DRY_RUN=false
if [ "$1" = "--dry-run" ]; then
    DRY_RUN=true
    shift
fi

# If no version provided, auto-increment patch version
if [ -z "$1" ] || [ "$DRY_RUN" = true ]; then
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
  
  if [ "$DRY_RUN" = true ]; then
    echo "Auto-incrementing version from $CURRENT_VERSION to $NEW_VERSION (DRY RUN)"
  else
    echo "Auto-incrementing version from $CURRENT_VERSION to $NEW_VERSION"
  fi
else
  NEW_VERSION=$1
fi

BRANCH_NAME="bump-version-$NEW_VERSION"

# Generate updated credits.rs before updating version
generate_credits

if [ "$DRY_RUN" = true ]; then
    echo "🔍 DRY RUN: Would update version to $NEW_VERSION"
    echo "🔍 DRY RUN: Would create branch $BRANCH_NAME"
    echo "🔍 DRY RUN: Would commit changes and create tag v$NEW_VERSION"
    echo "✅ Dry run completed successfully!"
    exit 0
fi

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

# Add both Cargo.toml, Cargo.lock, and the updated credits.rs
git add Cargo.toml Cargo.lock src/core/credits.rs
git commit -m "chore: bump version to $NEW_VERSION

- Updated project contributors from git history
- Refreshed library dependencies and licenses
- Auto-generated comprehensive credits information"
git push origin "$BRANCH_NAME"

# Create and push tag
git tag "v$NEW_VERSION"
git push origin "v$NEW_VERSION"

echo "Version bumped to $NEW_VERSION, pushed to branch $BRANCH_NAME, and tagged as v$NEW_VERSION."
echo "Create a pull request from $BRANCH_NAME to main on GitHub."