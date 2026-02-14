# Changelog

## [0.2.0] - 2026-02-14

### ✨ Features

- Added AWS SDK-first execution path for CLI handlers, with ASG/ECR detail support and improved offline testability
- Added offline integration-style test coverage for core app flows and parser scenarios
- Added release quality tooling:
  - Large-scale scenario gate
  - 85% line-coverage gate using `cargo llvm-cov`
  - GitHub Codecov integration

### 🔧 Improvements

- Reworked AWS data handling for robustness:
  - Normalized EC2 output mapping and IAM policy document parsing
  - Added typed authentication error model for clearer i18n-based UI login messages
- Added tracing/logging foundation and startup diagnostics (`tracing` + file-backed logs)
- Strengthened code quality process with:
  - `pre-commit` hooks (`cargo fmt`, `clippy`, `test`)
  - quality script simplification (`check-scenarios` merged into `rust-coverage.sh`)
  - CI workflow updates for coverage artifact path and script entrypoints
- Expanded release/process documentation:
  - `ROADMAP.md`, `ROADMAP_KR.md`
  - `README` roadmap links
  - quality tooling docs updates (`tools/README.md`, `tools/README_KR.md`)

### 🐛 Fixes

- Normalized scenario reference validation and CLI argument parsing edge-cases
- Fixed stale/incorrect coverage and quality-gate behavior after refactors and merges
- Cleaned duplicate/unused code paths and restored resource detail loading flows
- Hardened IAM/auth-related and AWS output decoding paths to reduce runtime parse failures
- Removed outdated dead paths and improved error visibility without changing user-facing behavior

---

## [0.1.1] - 2026-02-03

### ✨ Features

- **Self-Update Command**: `emd update` - Update to the latest version directly from GitHub releases
- **i18n Markdown Output**: AWS resource details now render with language-dependent labels (Korean/English)

### 🔧 Improvements

- Modular AWS CLI structure with separate modules for each service
- Base64 decoding using Rust crate instead of shell command for better security

### 🐛 Fixes

- Use `rustls` instead of `native-tls` for musl cross-compilation compatibility

---
---
"This patch focused on refactoring the overall EMD. And I'm very happy that automatic updates are now possible through the new update feature, emd update. 🎉"

---

## [0.1.0] - 2025-01-31

# 🚀 Initial Release

**EMD** - A TUI application for exploring AWS resources and generating Markdown documentation.

---

## ☁️ AWS Services

| Service | Description |
|:---|:---|
| EC2 | Instance details, IPs, security groups, tags |
| VPC | Subnets, IGW, NAT, route tables, EIPs + Mermaid diagram |
| Security Group | Inbound/outbound rules |
| Load Balancer | ALB/NLB/CLB, listeners, target groups |
| ECR | Repositories, tag mutability, encryption |

## 📋 Blueprint

- Combine resources from multiple regions into one document
- Auto-generated table of contents
- Reorder with `Shift+↑↓`

## 🌐 Multi-language

- Korean (default) / English
- Switch via Settings tab (`→` key)

## 🗺️ Regions

Seoul, Tokyo, Osaka, Singapore, Sydney, Mumbai, Virginia, Ohio, California, Oregon, Ireland, Frankfurt

---

## ⌨️ Shortcuts

| Key | Action |
|:---|:---|
| `↑↓` / `jk` | Navigate |
| `Enter` | Select |
| `Esc` | Back |
| `→` / `←` | Switch tabs |
| `r` | Refresh |
| `s` | Save |
| `q` | Quit |
