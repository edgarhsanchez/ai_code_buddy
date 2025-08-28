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
        Contributor {
            name: " 74 Edgar Sanchez",
            email: "esanchez@m2iab.com",
            contributions: 74,
        }
,
        Contributor {
            name: " 14 Edgar H Sanchez",
            email: "esanchez@m2iab.com",
            contributions: 14,
        }
    ]
}

/// Get all library dependencies with their information
pub fn get_library_dependencies() -> Vec<LibraryInfo> {
    vec![
        LibraryInfo {
            name: "anyhow",
            version: "anyhow = "1.0.95"",
            license: "MIT OR Apache-2.0",
            description: "Flexible concrete Error type built on std::error::Error",
            repository: "https://github.com/dtolnay/anyhow",
            contributors: vec!["David Tolnay", "And 50+ contributors"],
        }
,
        LibraryInfo {
            name: "bevy",
            version: "0.15",
            license: "MIT OR Apache-2.0",
            description: "A refreshingly simple data-driven game engine built in Rust",
            repository: "https://github.com/bevyengine/bevy",
            contributors: vec!["Carter Anderson", "Alice Cecile", "And 300+ contributors"],
        }
,
        LibraryInfo {
            name: "bevy_ratatui",
            version: "bevy_ratatui = "0.7.0"",
            license: "MIT OR Apache-2.0",
            description: "A Bevy plugin for Ratatui (terminal UI library)",
            repository: "https://github.com/bevy-ratatui/bevy_ratatui",
            contributors: vec!["Johan Klokkhammer Helsing", "And 10+ contributors"],
        }
,
        LibraryInfo {
            name: "bevy-tokio-tasks",
            version: "bevy-tokio-tasks = "0.15.0"",
            license: "Unknown",
            description: "Rust library dependency",
            repository: "https://crates.io/crates/bevy-tokio-tasks",
            contributors: vec!["Various contributors"],
        }
,
        LibraryInfo {
            name: "clap",
            version: "4.0",
            license: "MIT OR Apache-2.0",
            description: "A full featured, fast Command Line Argument Parser for Rust",
            repository: "https://github.com/clap-rs/clap",
            contributors: vec!["Kevin K. <kbknapp@gmail.com>", "And 200+ contributors"],
        }
,
        LibraryInfo {
            name: "color-eyre",
            version: "color-eyre = "0.6.3"",
            license: "MIT OR Apache-2.0",
            description: "An error report handler for panics and eyre::Report",
            repository: "https://github.com/eyre-rs/color-eyre",
            contributors: vec!["Jane Lusby", "And 20+ contributors"],
        }
,
        LibraryInfo {
            name: "crossterm",
            version: "crossterm = "0.28.1"",
            license: "MIT",
            description: "Cross-platform terminal manipulation library",
            repository: "https://github.com/crossterm-rs/crossterm",
            contributors: vec!["T. Postma", "And 50+ contributors"],
        }
,
        LibraryInfo {
            name: "futures",
            version: "futures = "0.3.31"",
            license: "MIT OR Apache-2.0",
            description: "An implementation of futures and streams featuring zero allocations",
            repository: "https://github.com/rust-lang/futures-rs",
            contributors: vec!["Alex Crichton", "And 100+ contributors"],
        }
,
        LibraryInfo {
            name: "git2",
            version: "git2 = "0.19"",
            license: "MIT OR Apache-2.0",
            description: "Rust bindings to libgit2 for interoperating with git repositories",
            repository: "https://github.com/rust-lang/git2-rs",
            contributors: vec!["Alex Crichton", "And 50+ contributors"],
        }
,
        LibraryInfo {
            name: "insta",
            version: "insta = "1.40.0"",
            license: "Unknown",
            description: "Rust library dependency",
            repository: "https://crates.io/crates/insta",
            contributors: vec!["Various contributors"],
        }
,
        LibraryInfo {
            name: "kalosm",
            version: "0.4.0",
            license: "MIT OR Apache-2.0",
            description: "A user-friendly interface for pre-trained large language models",
            repository: "https://github.com/floneum/kalosm",
            contributors: vec!["Evan Almloff", "And 10+ contributors"],
        }
,
        LibraryInfo {
            name: "mockall",
            version: "mockall = "0.13.0"",
            license: "Unknown",
            description: "Rust library dependency",
            repository: "https://crates.io/crates/mockall",
            contributors: vec!["Various contributors"],
        }
,
        LibraryInfo {
            name: "pretty_assertions",
            version: "pretty_assertions = "1.4.1"",
            license: "Unknown",
            description: "Rust library dependency",
            repository: "https://crates.io/crates/pretty_assertions",
            contributors: vec!["Various contributors"],
        }
,
        LibraryInfo {
            name: "proptest",
            version: "proptest = "1.5.0"",
            license: "Unknown",
            description: "Rust library dependency",
            repository: "https://crates.io/crates/proptest",
            contributors: vec!["Various contributors"],
        }
,
        LibraryInfo {
            name: "rat-cursor",
            version: "rat-cursor = "1.2.0"",
            license: "Unknown",
            description: "Rust library dependency",
            repository: "https://crates.io/crates/rat-cursor",
            contributors: vec!["Various contributors"],
        }
,
        LibraryInfo {
            name: "rat-event",
            version: "rat-event = "1.1.0"",
            license: "Unknown",
            description: "Rust library dependency",
            repository: "https://crates.io/crates/rat-event",
            contributors: vec!["Various contributors"],
        }
,
        LibraryInfo {
            name: "rat-focus",
            version: "rat-focus = "0.31.0"",
            license: "Unknown",
            description: "Rust library dependency",
            repository: "https://crates.io/crates/rat-focus",
            contributors: vec!["Various contributors"],
        }
,
        LibraryInfo {
            name: "ratatui",
            version: "0.29.0",
            license: "MIT",
            description: "A Rust library to build rich terminal user interfaces",
            repository: "https://github.com/ratatui/ratatui",
            contributors: vec!["Florian Dehau", "Joshka", "And 100+ contributors"],
        }
,
        LibraryInfo {
            name: "regex",
            version: "regex = "1.11.1"",
            license: "MIT OR Apache-2.0",
            description: "An implementation of regular expressions for Rust",
            repository: "https://github.com/rust-lang/regex",
            contributors: vec!["Andrew Gallant", "And 50+ contributors"],
        }
,
        LibraryInfo {
            name: "rstest",
            version: "rstest = "0.23.0"",
            license: "Unknown",
            description: "Rust library dependency",
            repository: "https://crates.io/crates/rstest",
            contributors: vec!["Various contributors"],
        }
,
        LibraryInfo {
            name: "serde",
            version: "1.0",
            license: "MIT OR Apache-2.0",
            description: "A generic serialization/deserialization framework",
            repository: "https://github.com/serde-rs/serde",
            contributors: vec!["David Tolnay", "And 100+ contributors"],
        }
,
        LibraryInfo {
            name: "serde_json",
            version: "serde_json = "1.0"",
            license: "Unknown",
            description: "Rust library dependency",
            repository: "https://crates.io/crates/serde_json",
            contributors: vec!["Various contributors"],
        }
,
        LibraryInfo {
            name: "serial_test",
            version: "serial_test = "3.1.1"",
            license: "Unknown",
            description: "Rust library dependency",
            repository: "https://crates.io/crates/serial_test",
            contributors: vec!["Various contributors"],
        }
,
        LibraryInfo {
            name: "tempfile",
            version: "tempfile = "3.13.0"",
            license: "Unknown",
            description: "Rust library dependency",
            repository: "https://crates.io/crates/tempfile",
            contributors: vec!["Various contributors"],
        }
,
        LibraryInfo {
            name: "tokio",
            version: "1.47.0",
            license: "MIT",
            description: "An event-driven, non-blocking I/O platform for writing async I/O",
            repository: "https://github.com/tokio-rs/tokio",
            contributors: vec!["Carl Lerche", "Sean McArthur", "And 200+ contributors"],
        }
,
        LibraryInfo {
            name: "tokio-test",
            version: "tokio-test = "0.4.4"",
            license: "Unknown",
            description: "Rust library dependency",
            repository: "https://crates.io/crates/tokio-test",
            contributors: vec!["Various contributors"],
        }
,
        LibraryInfo {
            name: "uuid",
            version: "1.11.0",
            license: "Apache-2.0 OR MIT",
            description: "A library to generate and parse UUIDs",
            repository: "https://github.com/uuid-rs/uuid",
            contributors: vec!["Ashley Mannix", "And 50+ contributors"],
        }
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
