# Conventional Commits Guide

This project uses [Conventional Commits](https://conventionalcommits.org/) to automatically determine version bumps and generate changelogs.

## Commit Message Format

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

## Types

- **feat**: A new feature
- **fix**: A bug fix
- **docs**: Documentation only changes
- **style**: Changes that do not affect the meaning of the code (formatting, etc.)
- **refactor**: A code change that neither fixes a bug nor adds a feature
- **perf**: A code change that improves performance
- **test**: Adding missing tests or correcting existing tests
- **build**: Changes that affect the build system or external dependencies
- **ci**: Changes to our CI configuration files and scripts
- **chore**: Other changes that don't modify src or test files

## Version Bump Rules

The version is automatically bumped based on commit types:

- **MAJOR** (X.y.z): Breaking changes (`feat!:` or `BREAKING CHANGE:` in footer)
- **MINOR** (x.Y.z): New features (`feat:`)
- **PATCH** (x.y.Z): Bug fixes (`fix:`), performance improvements (`perf:`)

## Examples

```bash
# New feature (minor bump)
feat: add user authentication system

# Bug fix (patch bump)
fix: resolve memory leak in analysis module

# Breaking change (major bump)
feat!: redesign API with breaking changes

# Documentation (no version bump)
docs: update installation instructions

# Refactoring (no version bump)
refactor: simplify error handling logic
```

## Quick Start

Use the provided commit helper script:

```bash
# Make your changes, then:
./commit.sh feat "add new analysis feature"
./commit.sh fix "resolve UI rendering bug"
```

Or commit manually following the format:

```bash
git commit -m "feat: add user authentication"
```

## Automated Process

When you merge a PR to `main`:

1. **Analysis**: Commits are analyzed to determine version bump type
2. **Version Update**: `Cargo.toml` version is automatically updated
3. **Tag Creation**: A new git tag is created (e.g., `v1.2.3`)
4. **Release**: GitHub release is created with changelog
5. **Publish**: Package is published to crates.io

## Benefits

- **Automated Versioning**: No manual version management
- **Consistent Releases**: Standardized commit format ensures predictable versioning
- **Clear Changelog**: Commit messages become release notes
- **Semantic Versioning**: Follows semver.org standards
- **CI/CD Integration**: Fully automated release process
