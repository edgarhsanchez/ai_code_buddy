# AI Buddy

[![Crates.io](https://img.shields.io/crates/v/ai_buddy.svg)](https://crates.io/crates/ai_buddy)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

An AI-powered code review tool that analyzes Git repositories and provides intelligent feedback on code quality, security, performance, and maintainability.

## Features

- 🔍 **Intelligent Code Analysis**: Uses AI to analyze code patterns and provide meaningful suggestions
- 🌲 **Git Branch Comparison**: Compare commits between branches (e.g., feature branch vs. develop)
- 🎯 **Multi-Language Support**: Supports Rust and general programming patterns
- 📊 **Detailed Reporting**: Categorized issues by severity (Critical, High, Medium, Low, Info)
- 🖥️ **Dual Interface**: Both CLI and interactive TUI modes
- 📋 **Comprehensive Reports**: Generate detailed code review reports

## Installation

### From crates.io

```bash
cargo install ai_buddy
```

### From source

```bash
git clone https://github.com/edgarhsanchez/ai_buddy.git
cd ai_buddy
cargo build --release
```

## Usage

### CLI Mode

```bash
# Analyze current branch against develop
ai_buddy --cli

# Analyze specific branches
ai_buddy --cli --source feature-branch --target main
```

### Interactive TUI Mode

```bash
# Launch interactive interface
ai_buddy
```

The interactive mode provides:
- Overview of all findings
- Navigate through issues by category
- Detailed view of each issue
- Generate comprehensive reports

## Issue Categories

- **🔒 Security**: Potential security vulnerabilities
- **🐛 Potential Bugs**: Code patterns that might cause issues
- **⚡ Performance**: Performance optimization opportunities
- **📚 Documentation**: Missing or inadequate documentation
- **🎨 Style**: Code style and formatting issues
- **🔧 Maintainability**: Code maintainability improvements
- **📖 Readability**: Code readability enhancements
- **🧪 Testing**: Testing-related suggestions

## Configuration

AI Buddy automatically detects:
- Repository technology stack
- Programming languages used
- Code patterns and anti-patterns
- Git branch structure

## Examples

### CLI Output
```
🎯 Code Review Summary
Repository: /path/to/repo
Branches: feature-branch → main
Technology: Rust, JavaScript

📊 Issues Found: 15
🔴 Critical: 0
🟠 High: 2
🟡 Medium: 5
🔵 Low: 6
ℹ️  Info: 2
```

### Interactive Features
- **Arrow keys**: Navigate through issues
- **Enter**: View detailed issue information
- **R**: Generate comprehensive report
- **Q**: Quit application

## Requirements

- Rust 1.70+
- Git repository
- Network connection for AI processing

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Roadmap

- [ ] Support for more programming languages
- [ ] Custom rule configuration
- [ ] Integration with CI/CD pipelines
- [ ] Web interface
- [ ] Team collaboration features

## Acknowledgments

- Built with [Kalosm](https://crates.io/crates/kalosm) for AI language processing
- Uses [git2](https://crates.io/crates/git2) for Git repository analysis
