# TerminalOS

**The AI-native terminal for developers.**

TerminalOS is a production-grade, cross-platform terminal application built in Rust. It combines a modern terminal emulator, AI assistant, coding agent, Git assistant, workspace manager, and developer dashboard into a single fast, keyboard-driven TUI.

> Repository: [github.com/panditdhamdhere/WarpShell](https://github.com/panditdhamdhere/WarpShell)

## Features (Phase 1–10)

- **Real shell** — PTY-backed bash/zsh with streaming output
- **ANSI colors** — full vt100 terminal emulation in the UI
- **Multi-tab PTY** — each tab runs an independent shell session
- **Copy/paste** — Ctrl+Shift+C/V clipboard integration
- **Search** — Ctrl+Shift+F to highlight matches in terminal output
- **Scrollback** — 10,000 lines with mouse and keyboard scrolling
- **AI chat** — streaming responses with markdown and syntax highlighting
- **Multi-provider** — OpenAI, Anthropic, OpenRouter, Ollama, Gemini, DeepSeek
- **Chat history** — conversations persisted to SQLite across sessions
- **Coding agent** — slash commands for edit, fix, review, search, and more
- **Safe execution** — file writes and shell commands require explicit confirmation
- **Git assistant** — commit messages, PR summaries, diff explain, blame, staging, health checks
- **Workspace manager** — persist tabs, branches, UI layout, and env vars across sessions
- **Session restore** — reopen projects with terminal tabs and pane focus restored automatically
- **Semantic search** — tree-sitter chunking with Ollama embeddings and hybrid ranking
- **Code-aware results** — search hits include symbol names and line ranges
- **Plugin SDK** — dynamic Rust plugins with marketplace install and `/plugin` commands
- **Profiles** — `default`, `minimal`, and `coding` layouts with per-profile UI overrides
- **Theme presets** — dracula, nord, solarized-dark, and built-in dark/light themes
- **Keybindings** — configurable global shortcuts via `config.toml` or `keybindings.toml`
- **Developer tooling** — `cargo xtask` for CI, snapshots, benchmarks, and docs

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

Or with a configuration profile:

```bash
cargo run -p terminalos -- --profile minimal
```

### CLI Tools

```bash
# Git status
cargo run -p terminalos-cli -- status

# Index project for search
cargo run -p terminalos-cli -- index --path .

# Search indexed code
cargo run -p terminalos-cli -- search "TerminalApp"

# List recently opened workspaces
cargo run -p terminalos-cli -- workspaces

# Hybrid semantic + keyword search
cargo run -p terminalos-cli -- search "session restore" --mode hybrid

# Plugin marketplace and install
cargo run -p terminalos-cli -- plugins marketplace
cargo build -p terminalos-plugin-hello
cargo run -p terminalos-cli -- plugins install hello
cargo run -p terminalos-cli -- plugins run hello greet TerminalOS

# Configuration
cargo run -p terminalos-cli -- config show
cargo run -p terminalos-cli -- config themes
cargo run -p terminalos-cli -- config profile list
cargo run -p terminalos-cli -- config profile use minimal
```

### Coding Agent (Slash Commands)

Focus the AI chat panel (`Ctrl+2`) and use slash commands:

| Command | Description |
|---------|-------------|
| `/search <query>` | Search codebase (Tantivy index) |
| `/explain <path>` | Explain a file |
| `/edit <path> <instruction>` | Edit a file with AI |
| `/fix <path>` | Fix bugs in a file |
| `/refactor <path> <instruction>` | Refactor a file |
| `/create <path> <description>` | Create a new file |
| `/review <path>` | Code review with git context |
| `/test [args]` | Propose and run tests (requires confirmation) |
| `/docs <path>` | Generate documentation |
| `/analyze` | Repository architecture analysis |

File writes and shell commands require pressing `y` to confirm or `n` to cancel.

### Git Assistant (Slash Commands)

| Command | Description |
|---------|-------------|
| `/commit` | Generate commit message from staged changes |
| `/pr [base]` | PR summary vs base branch (default: `main`) |
| `/diff [path]` | Explain staged and unstaged changes |
| `/conflict [path]` | Analyze and resolve merge conflicts |
| `/stage [path]` | Stage files (lists changes if no path) |
| `/unstage [path]` | Unstage files |
| `/blame <path> [line]` | Explain git blame history |
| `/health` | Repository health check with recommendations |
| `/plugin <name> <cmd> [args]` | Run an installed plugin command |

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
| `Ctrl+Shift+C` | Copy terminal |
| `Ctrl+Shift+V` | Paste to terminal |
| `Ctrl+Shift+F` | Search in terminal |
| `Ctrl+Shift+↑/↓` | Scroll terminal |
| `Page Up/Down` | Scroll terminal |
| `Ctrl+Q` | Quit |

Keybindings are configurable in `~/.config/terminalos/config.toml` or `keybindings.toml`. See bundled profiles in `profiles/` for layout presets.

## Configuration

Default config directory: `~/.config/terminalos/`

| File | Purpose |
|------|---------|
| `config.toml` | Providers, workspace, active profile, keybindings |
| `keybindings.toml` | Optional global shortcut overrides |
| `profiles/*.toml` | Named profiles with partial UI/layout overrides |

Set `theme_preset = "dracula"` under `[ui]` for named color palettes.

### Developer Tools

```bash
cargo xtask ci          # local CI pipeline
cargo xtask snapshot    # verify UI/config snapshots
cargo xtask bench       # run benchmarks
cargo xtask docs        # build documentation site
```

Documentation site source lives in `docs/src/`. Build output is written to `docs/book/`.

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
