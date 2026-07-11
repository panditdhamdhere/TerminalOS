# TerminalOS

**The AI-native terminal for developers.**

TerminalOS is a production-grade, cross-platform terminal application built in Rust. It combines a modern terminal emulator, AI assistant, coding agent, Git assistant, workspace manager, and developer dashboard into a single fast, keyboard-driven TUI.

> Repository: [github.com/panditdhamdhere/WarpShell](https://github.com/panditdhamdhere/WarpShell)

## Features (Phase 1 — Complete)

- **Multi-pane layout** — sidebar, terminal, AI chat, logs, status bar
- **Tabs** — create, close, and switch terminal tabs
- **Resizable panes** — keyboard-driven layout controls
- **Workspace sidebar** — live file tree with git-aware filtering
- **AI chat panel** — conversation UI ready for provider integration
- **Application logs** — real-time event stream in the bottom pane
- **Keyboard shortcuts** — full focus and navigation model
- **Mouse support** — scroll in any pane
- **Dark theme** — modern developer aesthetic

## Quick Start

### Prerequisites

- Rust 1.85+ (stable)
- macOS, Linux, or Windows

### Build

```bash
git clone https://github.com/panditdhamdhere/WarpShell.git
cd WarpShell
cargo build --release
```

### Run

```bash
cargo run -p terminalos
```

Or with a specific workspace:

```bash
cargo run -p terminalos -- --workspace /path/to/project
```

### CLI Tools

```bash
# Git status
cargo run -p terminalos-cli -- status

# Index project for search
cargo run -p terminalos-cli -- index --path .

# Search indexed code
cargo run -p terminalos-cli -- search "TerminalApp"
```

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+T` | New tab |
| `Ctrl+W` | Close tab |
| `Ctrl+Tab` | Next tab |
| `Ctrl+Shift+Tab` | Previous tab |
| `Ctrl+B` | Toggle sidebar |
| `Ctrl+/` | Toggle AI chat |
| `Ctrl+`` | Toggle logs |
| `Ctrl+1/2/3/4` | Focus terminal/chat/sidebar/logs |
| `Tab` | Cycle focus |
| `Ctrl+←/→` | Resize sidebar |
| `Ctrl+↑/↓` | Resize chat panel |
| `Ctrl+Shift+↑/↓` | Resize logs panel |
| `Ctrl+Q` | Quit |

## Architecture

TerminalOS uses a Cargo workspace with feature-based crates:

```
apps/terminal   — Main TUI application
apps/daemon     — Background workspace daemon
apps/cli        — Command-line utilities
crates/ui       — Ratatui interface components
crates/terminal — Terminal session management
crates/ai       — AI provider abstractions
crates/git      — Git operations
crates/search   — Tantivy full-text search
... and more
```

See [ARCHITECTURE.md](docs/ARCHITECTURE.md) for details.

## Roadmap

See [ROADMAP.md](docs/ROADMAP.md) for the full development plan across 10 phases.

## Contributing

Contributions are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

MIT — see [LICENSE](LICENSE).
