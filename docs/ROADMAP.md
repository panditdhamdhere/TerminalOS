# TerminalOS Roadmap

## Phase 1 — Terminal UI ✅

- [x] Ratatui multi-pane layout
- [x] Sidebar, terminal, AI chat, logs, status bar
- [x] Tabs with create/close/switch
- [x] Resizable panes via keyboard
- [x] Keyboard shortcuts and focus model
- [x] Mouse scroll support
- [x] Dark theme

## Phase 2 — Terminal Emulator ✅

- [x] PTY-based shell execution (portable-pty)
- [x] Streaming stdout/stderr via background reader
- [x] Command history (shell-native via PTY)
- [x] Multiple tabs with independent PTY sessions
- [x] ANSI color support (vt100 parser)
- [x] Copy/paste (Ctrl+Shift+C/V)
- [x] In-terminal search (Ctrl+Shift+F)
- [x] Scrollback with mouse and keyboard
- [x] Split panes (horizontal and vertical)
- [ ] In-terminal selection search UI polish

## Phase 3 — AI Chat ✅

- [x] OpenAI, Anthropic, OpenRouter, Ollama, Gemini, DeepSeek
- [x] Interchangeable providers
- [x] Streaming responses
- [x] Conversation history (SQLite)
- [x] Markdown rendering
- [x] Syntax highlighting

## Phase 4 — Coding Agent ✅

- [x] `/edit`, `/create`, `/fix`, `/refactor`, `/explain`, `/test`, `/review`, `/search`, `/docs`, `/analyze`
- [x] Read/modify/rename/delete files
- [x] Run commands (with confirmation)
- [x] Repository analysis
- [x] Documentation generation

## Phase 5 — Git Assistant ✅

- [x] Commit message generation (`/commit`)
- [x] PR summaries (`/pr`)
- [x] Diff explanation (`/diff`)
- [x] Merge conflict resolution (`/conflict`)
- [x] Interactive staging (`/stage`, `/unstage`)
- [x] Git blame explanation (`/blame`)
- [x] Repository health checks (`/health`)

## Phase 6 — Workspace Manager

- [x] Persist projects, files, tabs, branches
- [x] Environment variable memory
- [x] Session restoration

## Phase 7 — Semantic Search

- [x] Tree-sitter parsing
- [x] Embeddings and vector search
- [x] Hybrid keyword + semantic search

## Phase 8 — Plugin SDK

- [x] Rust plugin API
- [x] Dynamic loading
- [x] Plugin marketplace

## Phase 9 — Configuration ✅

- [x] Profiles, themes, keybindings
- [x] Provider and workspace configs

## Phase 10 — Developer Experience ✅

- [x] Cargo xtask
- [x] CI/CD (GitHub Actions)
- [x] Snapshot tests
- [x] Benchmarks
- [x] Full documentation site

## Phase 11 — Groq & Onboarding ✅

- [x] First-class Groq provider type
- [x] First-run setup wizard
- [x] Auto-detect API keys from `.env`
- [x] Groq profile and `config provider` CLI

## Phase 12 — Provider Switcher & Chat Errors ✅

- [x] In-app provider picker (`Ctrl+P`) with hot-swap
- [x] Provider status in status bar
- [x] Inline chat error messages for API and agent failures

## Phase 13 — Split Terminal Panes ✅

- [x] Horizontal and vertical pane splits per tab
- [x] Per-pane PTY sessions with kernel resize
- [x] Pane focus cycling and close pane shortcuts

## Phase 14 — Release & Docs Deploy ✅

- [x] GitHub Actions release workflow on version tags
- [x] macOS and Linux release tarballs with checksums
- [x] GitHub Pages deployment for mdBook docs
- [x] `cargo xtask dist` local packaging helper

## Phase 15 — Split Persistence ✅

- [x] Serialize split layout tree per tab in workspace snapshot
- [x] Restore pane IDs and active pane on session reload

## Phase 16 — Per-Workspace Chat ✅

- [x] Derive chat session ID from workspace ID
- [x] Load workspace-specific conversation history on startup
