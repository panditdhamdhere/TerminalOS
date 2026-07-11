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
- [ ] Split panes
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

## Phase 10 — Developer Experience

- [ ] Cargo xtask
- [ ] CI/CD (GitHub Actions)
- [ ] Snapshot tests
- [ ] Benchmarks
- [ ] Full documentation site
