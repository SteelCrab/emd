# AWS CLI Installer TUI

A cross-platform terminal UI installer for AWS CLI v2, built with Rust.

## Features

- 🖥️ **Cross-platform**: macOS, Windows, Linux (x86_64, arm64)
- 📥 **Install**: Download and install AWS CLI v2
- 🗑️ **Uninstall**: Remove AWS CLI from your system
- 📊 **Progress bar**: Real-time download progress

## Installation

Download from [Releases](../../releases) or build from source:

```bash
cargo build --release
./target/release/t-aws
```

## Usage

1. Run the application
2. Press **Enter** at welcome screen
3. Select action:
   - **[1]** Install AWS CLI
   - **[2]** Uninstall AWS CLI
4. Follow the prompts

## Supported Platforms

| OS | Architecture |
|----|-------------|
| macOS | x86_64, arm64 |
| Windows | x86_64 |
| Linux | x86_64, arm64 |

## License

MIT
