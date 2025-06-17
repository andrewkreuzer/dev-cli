# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`dev-cli` is a Rust-based command-line tool for managing development environments. It provides a unified interface for running scripts in different languages (Python, JavaScript, Lua, Shell), managing Git repositories, and working with GitHub. The tool is designed to be extensible and configurable through TOML and YAML configuration files.

## Build Commands

```bash
# Build the project
cargo build

# Build with specific features
cargo build --features github

# Run the project
cargo run

# Run a specific command
cargo run -- run --file examples/main.py
cargo run -- run --name example-alias

# Check and format code
cargo check
cargo fmt
```

## Project Structure

- **Config Management**: The tool uses TOML configuration files to store settings, repository information, and run configurations.
- **Language Runners**: Modular system to run scripts in different languages (Python, JavaScript, Lua, Shell).
- **Git Integration**: Interface for working with Git repositories.
- **GitHub Integration**: Optional feature for GitHub API interactions, including PR management.

## Key Components

1. **Configuration System**:
   - Configuration loaded from global (`/etc/dev/dev.toml`) and local (`./dev.toml`) files
   - Manages repository configurations and run aliases

2. **Command Structure**:
   - Uses `clap` for command-line argument parsing
   - Main commands: init, git, github, scan, yaml, repo, repos, run, shell

3. **Runner System**:
   - Each supported language (Python, JavaScript, Lua, Shell) has its own runner implementation
   - The runner is selected based on file extension or explicit type specification

4. **Features**:
   - Language support is feature-gated (javascript, lua, python)
   - GitHub integration is an optional feature

## Development Conventions

1. **Error Handling**:
   - Use custom error types with thiserror
   - Propagate errors with anyhow where appropriate

2. **Testing**:
   - Run tests with `cargo test`

3. **Feature Flags**:
   - Language features: python, lua, javascript
   - GitHub integration: github

## Nix Integration

The project uses Nix for development environment setup:

```bash
# Enter development shell
nix develop

# Build with nix
nix build
```