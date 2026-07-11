# Changelog

All notable changes to TerminalOS are documented here.

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
