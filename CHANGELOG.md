# Changelog

All notable changes to TerminalOS are documented here.

## [0.14.0] - 2026-07-12

### Added

- GitHub Actions `release.yml` — builds and publishes tarballs on `v*` tags (Linux + macOS)
- GitHub Actions `docs-deploy.yml` — deploys mdBook site to GitHub Pages on push to `main`
- `cargo xtask dist` — local release binary packaging into `dist/`
- `site-url` in `book.toml` for GitHub Pages

## [0.13.0] - 2026-07-12

### Added

- Horizontal (`Alt+\`) and vertical (`Alt+-`) terminal pane splits per tab
- Per-pane PTY sessions with kernel resize (SIGWINCH)
- Pane focus cycling (`Ctrl+Shift+←/→`) and close pane (`Ctrl+Shift+W`)
- `PaneId` and split layout tree in the terminal crate

## [0.12.0] - 2026-07-12

### Added

- In-app AI provider picker (`Ctrl+P`) with hot-swap without restart
- Active provider and model shown in the status bar
- Inline chat error messages for missing keys, rate limits, network failures, and agent errors
- `ChatEngine::reload_from_config()` for runtime provider switching

## [0.11.0] - 2026-07-12

### Added

- First-class `ProviderType::Groq` with default model and API URL
- First-run setup wizard when no AI provider is ready
- Auto-detection of `GROQ_API_KEY`, `OPENAI_API_KEY`, and local Ollama on startup
- User-level secrets file at `~/.config/terminalos/.env`
- Bundled `groq` configuration profile
- CLI `config provider list` and `config provider use <name>` commands
- `--skip-setup` flag for non-interactive launches

## [0.10.0] - 2026-07-11

### Added

- `cargo xtask` automation for CI, formatting, tests, snapshots, benchmarks, and docs
- Snapshot tests for default config TOML, keybindings, themes, layout geometry, and status bar rendering
- Criterion benchmarks for keybinding parsing and Tantivy search
- mdBook documentation site under `docs/src/` with architecture, configuration, and developer guides
- Enhanced GitHub Actions workflow with xtask CI, benchmark, and docs build jobs

## [0.9.0] - 2026-07-11

### Added

- Configuration profiles (`default`, `minimal`, `coding`) with per-profile UI and layout overrides
- Theme presets: dark, light, dracula, nord, solarized-dark via `theme_preset` in config
- Configurable global keybindings with `KeybindingResolver` and optional `keybindings.toml` override
- `ConfigLoader::load_with_profile()` and `set_active_profile()` for profile switching
- Terminal `--profile` flag to launch with a specific profile
- CLI `config show`, `config themes`, `config profile list`, and `config profile use` commands
- `focus_logs` default keybinding (`Ctrl+4`)

## [0.8.0] - 2026-07-11

### Added

- Stable C ABI plugin API with `terminalos_plugin_entry` export symbol
- Dynamic plugin loading via `libloading` and `PluginManager`
- Plugin marketplace catalog with install and list commands
- Example `hello` dynamic plugin (`plugins/hello`)
- `/plugin <name> <command> [args]` slash command in the coding agent
- CLI `plugins list|marketplace|install|run` subcommands
- `PluginConfig` in `config.toml` (`enabled`, `auto_load`)

## [0.7.0] - 2026-07-11

### Added

- Tree-sitter code chunk extraction for Rust, Python, JavaScript/TypeScript, and Go
- Ollama embedding client for semantic vector indexing
- SQLite vector store (`semantic.db`) with cosine similarity search
- Hybrid search engine combining Tantivy keyword and semantic scores
- `SearchConfig` in `config.toml` (`mode`, weights, embedding model/base URL)
- CLI `--mode hybrid|keyword|semantic` flag for search queries
- `/search` slash command now returns symbol, line, and match type metadata

## [0.6.0] - 2026-07-11

### Added

- Workspace session persistence in SQLite (`workspace.db`)
- Stable workspace IDs derived from project paths (UUID v5)
- Terminal tab, branch, and UI state restoration on startup
- Workspace environment variable memory applied to new PTY sessions
- Periodic autosave and save-on-quit for workspace snapshots
- `WorkspaceConfig` in `config.toml` (`auto_restore`, `autosave_secs`)
- CLI `workspaces` command to list recently opened projects

## [0.5.0] - 2026-07-11

### Added

- Git Assistant slash commands: `/commit`, `/pr`, `/diff`, `/conflict`, `/stage`, `/unstage`, `/blame`, `/health`
- Extended `terminalos-git` crate with diff, blame, staging, conflict detection, and health assessment
- AI-powered commit message generation from staged diffs
- PR summary generation with commit log and branch diff
- Interactive staging with confirmation gates for `git add` and `git reset`
- Repository health checks (conflicts, remote sync, untracked files)

## [0.4.0] - 2026-07-11

### Added

- Coding agent crate with slash commands: `/edit`, `/create`, `/fix`, `/refactor`, `/explain`, `/test`, `/review`, `/search`, `/docs`, `/analyze`
- Multi-step agent tool loop with read, write, search, and run_command tools
- Workspace-sandboxed `FileOps` for read/write/create/rename/delete
- Confirmation UI for file writes, deletes, and shell commands (`y`/`n`)
- Agent config section in `config.toml` (`max_iterations`, confirm flags)
- Code search integration via Tantivy index in agent `/search` command

## [0.3.0] - 2026-07-11

### Added

- Streaming AI chat with OpenAI-compatible, Anthropic, and Gemini providers
- Provider registry with OpenAI, Anthropic, OpenRouter, Ollama, Gemini, and DeepSeek
- `ChatEngine` with async streaming and live response updates in the UI
- Markdown rendering and syntax highlighting in the AI chat panel (pulldown-cmark + syntect)
- SQLite conversation persistence with stable session history across restarts
- Ollama enabled by default in config for local AI chat

## [0.2.0] - 2026-07-11

### Added

- Real PTY shell execution per terminal tab (portable-pty)
- Streaming shell output with background reader threads
- vt100 ANSI terminal emulator with color rendering
- Per-tab independent shell sessions
- Keyboard forwarding to shell (arrows, Ctrl+C, etc.)
- Scrollback via Page Up/Down, Ctrl+Shift+Up/Down, and mouse wheel
- Copy terminal contents to clipboard (Ctrl+Shift+C)
- Paste from clipboard into shell (Ctrl+Shift+V)
- In-terminal text search with highlight (Ctrl+Shift+F)
- Command history ring buffer per tab

## [0.1.0] - 2026-07-11

### Added

- Initial Cargo workspace with 14 crates and 3 applications
- Phase 1 Ratatui UI with multi-pane layout
- Terminal tabs with create/close/switch
- Resizable sidebar, chat, and logs panes
- Workspace file tree sidebar
- AI chat panel with message history
- Application log panel
- Status bar with workspace and git branch info
- Keyboard shortcuts and focus cycling
- Mouse scroll support
- TOML configuration system
- AI provider trait and registry (OpenAI, Anthropic, OpenRouter, Ollama, Gemini, DeepSeek)
- Git repository status inspection
- Tantivy full-text search and project indexer
- SQLite session and conversation storage
- Plugin manifest loader
- IPC protocol definitions
- CLI tools: `status`, `index`, `search`, `open`, `config`
- Background daemon skeleton
- GitHub Actions CI pipeline
- MIT license and contributor documentation
