# Changelog

All notable changes to TerminalOS are documented here.

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
