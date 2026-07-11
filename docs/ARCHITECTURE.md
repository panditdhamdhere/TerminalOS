# TerminalOS Architecture

## Overview

TerminalOS is a modular Rust workspace following clean architecture principles. Each crate has a single responsibility and communicates through well-defined traits and shared types.

## Workspace Structure

```
terminalos/
├── apps/
│   ├── terminal/     # Main Ratatui application (binary: terminalos)
│   ├── daemon/       # Background workspace daemon
│   └── cli/          # CLI utilities (binary: tos)
├── crates/
│   ├── shared/       # Common types, errors, themes
│   ├── protocol/     # IPC message definitions
│   ├── config/       # TOML configuration
│   ├── core/         # AppContext, event bus, DI
│   ├── filesystem/   # File tree, file watcher
│   ├── memory/       # SQLite session storage
│   ├── ai/           # AI provider trait + registry
│   ├── terminal/     # Shell sessions, tabs, buffers
│   ├── git/          # Git repository operations
│   ├── workspace/    # Workspace state management
│   ├── search/       # Tantivy search engine
│   ├── indexer/      # Project file indexer
│   ├── plugin/       # Plugin SDK
│   └── ui/           # Ratatui components + event loop
├── docs/
├── tests/
└── .github/
```

## Layer Diagram

```
┌─────────────────────────────────────────────┐
│              apps/terminal (TUI)            │
├─────────────────────────────────────────────┤
│              crates/ui (Ratatui)            │
├──────────┬──────────┬──────────┬────────────┤
│ terminal │    ai    │   git    │ workspace  │
├──────────┴──────────┴──────────┴────────────┤
│         core │ config │ filesystem │ memory  │
├─────────────────────────────────────────────┤
│              crates/shared (types)          │
└─────────────────────────────────────────────┘
```

## Key Design Decisions

### Dependency Injection

`AppContext` holds shared state (config, theme, logs, event bus) and is injected into services. No global statics.

### AI Provider Abstraction

All AI backends implement the `AiProvider` trait with streaming `CompletionStream`. Providers are registered in `ProviderRegistry` from TOML config.

### UI Event Model

Keyboard and mouse events map to `AppAction` variants. The focused pane (`FocusedPane`) determines input routing. Layout is computed from percentage-based `LayoutConfig`.

### Search Pipeline

`ProjectIndexer` walks the filesystem (respecting `.gitignore`), indexes full files into Tantivy, and extracts tree-sitter code chunks into a SQLite vector store. `HybridSearchEngine` merges keyword and semantic scores. The CLI exposes `index` and `search` commands with `--mode hybrid|keyword|semantic`.

### Plugin System

Plugins export a stable C ABI (`terminalos_plugin_entry`) and ship with a `plugin.toml` manifest. `PluginManager` discovers installed plugins, loads dynamic libraries, and routes commands from the CLI and `/plugin` slash command. The bundled marketplace catalog supports local install of example plugins.

### Configuration

Configuration lives in `~/.config/terminalos/`:

- `config.toml` — active profile, providers, workspace, and inline keybindings
- `keybindings.toml` — optional override for global shortcuts
- `profiles/` — named profiles (`default`, `minimal`, `coding`) with partial section overrides

`ConfigLoader` merges the base config, keybindings override, and active profile at startup. Theme presets (`dracula`, `nord`, `solarized-dark`) resolve through `resolve_theme()`. The UI reads shortcuts from `KeybindingResolver` instead of hardcoded bindings.

### Security

AI-generated shell commands are never executed automatically. All destructive actions require explicit user confirmation (Phase 4+).

## Cross-Platform

- **Terminal I/O**: crossterm (raw mode, alternate screen, mouse)
- **Git**: git2 (libgit2 bindings)
- **Async runtime**: Tokio
- **Database**: SQLx + SQLite

## Performance Targets

- Startup: < 100ms (release build)
- Incremental indexing with background workers
- Lazy file tree loading (depth-limited)
- Bounded log and buffer sizes
