# EMD
[![coverage](https://codecov.io/gh/SteelCrab/emd/branch/main/graph/badge.svg)](https://codecov.io/gh/SteelCrab/emd)

[🇺🇸 English](README.md) | [🇰🇷 한국어](README.ko.md)

![alt text](images/emd-1.png)

**Global coverage gate:** `cargo llvm-cov` **>= 85%** (`tools/rust-coverage.sh`)

`emd` is a Terminal User Interface (TUI) application designed to explore your AWS resources and generate comprehensive Markdown documentation.

## Roadmap

- [ROADMAP.md](ROADMAP.md)
- [ROADMAP_KR.md](ROADMAP_KR.md)

## Features

- **Resource Exploration**: Easily browse EC2 instances, VPCs (Networks), Security Groups, and Load Balancers.
- **Blueprinter**: Select multiple resources across different regions and services to create a single, unified documentation blueprint.
- **Markdown Generation**: Automatically generate detailed Markdown documentation for selected resources, complete with network diagrams (Mermaid.js).
- **TUI Interface**: A user-friendly terminal interface built with `ratatui`.

## Installation

```bash
cargo build --release
cp ./target/release/emd /usr/local/bin/
```

## Usage

```bash
emd              # Run TUI mode
emd update       # Update to latest version
emd version      # Show version
emd help         # Show help
```

## Development

### Pre-commit

```bash
pipx install pre-commit  # or pip install pre-commit
pre-commit install
pre-commit run --all-files
```


## Configuration

Blueprints are saved locally in:
`~/.emd/blueprints.json`


## Coffe ☕️

https://ko-fi.com/pistacrab
