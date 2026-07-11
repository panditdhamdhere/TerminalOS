# Quick Start

## Prerequisites

- Rust 1.85+ (stable)
- macOS, Linux, or Windows

## Build

```bash
git clone https://github.com/panditdhamdhere/WarpShell.git
cd WarpShell
cargo build --release
```

## Run

```bash
cargo run -p terminalos
```

Open a specific workspace:

```bash
cargo run -p terminalos -- --workspace /path/to/project
```

Launch with a configuration profile:

```bash
cargo run -p terminalos -- --profile minimal
```

## CLI Tools

The `tos` binary provides workspace, search, git, plugin, and configuration utilities:

```bash
# Git status
cargo run -p terminalos-cli -- status

# Index and search a project
cargo run -p terminalos-cli -- index --path .
cargo run -p terminalos-cli -- search "TerminalApp" --mode hybrid

# Configuration
cargo run -p terminalos-cli -- config show
cargo run -p terminalos-cli -- config profile list
cargo run -p terminalos-cli -- config profile use minimal
```

## Coding Agent

Focus the AI chat panel (`Ctrl+2`) and use slash commands such as `/search`, `/edit`, `/fix`, `/commit`, and `/health`. File writes and shell commands require pressing `y` to confirm.
