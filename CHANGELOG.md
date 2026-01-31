# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0-alpha.1] - 2026-01-30

### Added
- Initial alpha release
- Natural language to shell command conversion via OpenRouter
- Interactive terminal UI with Ink
- Safety checks for dangerous commands
- Context gathering (cwd, shell, git status, directory listing)
- TOML configuration file support
- Cross-platform support (Linux, macOS, Windows)
- npm distribution
- Standalone binaries for 5 platforms

### Security
- Dangerous command detection (rm -rf, dd, mkfs, etc.)
- Interactive confirmation prompts for high-risk operations
- Config file permission validation (Unix: 0600)

## [Unreleased]

### Planned for Beta
- Install script (curl | bash)
- Shell history integration
- Custom Homebrew tap
- Scoop bucket (Windows)

### Planned for Stable
- Auto-update mechanism
- Submit to Homebrew Core
- Chocolatey package
- AUR package
- Debian/RPM packages
