use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, MouseEvent, MouseEventKind};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use crossterm::{ExecutableCommand, execute};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use terminalos_config::AppConfig;
use terminalos_core::{AppContext, InMemoryEventBus};
use terminalos_filesystem::{FileNode, FileTree};
use terminalos_shared::{LogEntry, LogLevel, Theme, ThemeMode};
use terminalos_terminal::ShellSession;
use terminalos_workspace::WorkspaceManager;
use tokio::runtime::Runtime;
use tracing::info;

use crate::components::chat_pane::ChatMessage;
use crate::components::{
    render_chat_pane, render_logs_pane, render_sidebar, render_status_bar, render_tab_bar,
    render_terminal_pane,
};
use crate::event::{AppAction, FocusedPane, map_key_event};
use crate::layout::{LayoutVisibility, compute_layout};

/// Options for launching the terminal application.
#[derive(Debug, Clone, Default)]
pub struct TerminalAppOptions {
    pub workspace_path: Option<PathBuf>,
    pub config: AppConfig,
}

/// Main TerminalOS TUI application.
pub struct TerminalApp {
    config: AppConfig,
    theme: Theme,
    session: ShellSession,
    workspace_manager: WorkspaceManager,
    file_tree: Option<FileNode>,
    workspace_name: String,
    branch: Option<String>,
    chat_messages: Vec<ChatMessage>,
    chat_input: String,
    logs: Vec<LogEntry>,
    focus: FocusedPane,
    sidebar_scroll: usize,
    chat_scroll: usize,
    logs_scroll: usize,
    show_sidebar: bool,
    show_chat: bool,
    show_logs: bool,
    should_quit: bool,
    tick: u8,
    last_frame: Instant,
}

impl TerminalApp {
    #[must_use]
    pub fn new(options: TerminalAppOptions) -> Self {
        let cwd = options
            .workspace_path
            .clone()
            .or_else(|| std::env::current_dir().ok())
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| ".".to_string());

        let theme = match options.config.ui.theme {
            ThemeMode::Dark => Theme::dark(),
            ThemeMode::Light => Theme::light(),
        };

        let mut workspace_manager = WorkspaceManager::new();
        let mut workspace_name = "local".to_string();
        let mut branch = None;
        let mut file_tree = None;

        if let Some(path) = options.workspace_path.clone() {
            if let Ok(id) = workspace_manager.open(&path) {
                if let Some(ws) = workspace_manager.get(id) {
                    workspace_name = ws.name.clone();
                    branch = ws.branch.clone();
                }
                if let Ok(tree) = FileTree::new(&path).build(3) {
                    file_tree = Some(tree);
                }
            }
        }

        let mut session = ShellSession::new(&cwd);
        session.active_tab_mut().buffer.push_str(WELCOME_MESSAGE);

        let mut logs = vec![
            LogEntry::info("TerminalOS started"),
            LogEntry::info(format!("Workspace: {workspace_name}")),
        ];

        if branch.is_some() {
            logs.push(LogEntry::info(format!(
                "Git branch: {}",
                branch.as_deref().unwrap_or("unknown")
            )));
        }

        Self {
            show_sidebar: options.config.ui.show_sidebar,
            show_chat: options.config.ui.show_chat,
            show_logs: options.config.ui.show_logs,
            config: options.config,
            theme,
            session,
            workspace_manager,
            file_tree,
            workspace_name,
            branch,
            chat_messages: Vec::new(),
            chat_input: String::new(),
            logs,
            focus: FocusedPane::Terminal,
            sidebar_scroll: 0,
            chat_scroll: 0,
            logs_scroll: 0,
            should_quit: false,
            tick: 0,
            last_frame: Instant::now(),
        }
    }

    /// Runs the terminal UI event loop until quit.
    pub fn run(&mut self) -> terminalos_shared::Result<()> {
        let rt = Runtime::new()
            .map_err(|e| terminalos_shared::Error::Ui(format!("tokio runtime: {e}")))?;

        let events = std::sync::Arc::new(InMemoryEventBus);
        let _ctx = AppContext::new(self.config.clone(), events);

        enable_raw_mode().map_err(|e| terminalos_shared::Error::Ui(format!("raw mode: {e}")))?;
        io::stdout()
            .execute(EnterAlternateScreen)
            .map_err(|e| terminalos_shared::Error::Ui(format!("alt screen: {e}")))?;
        if self.config.ui.mouse_enabled {
            execute!(io::stdout(), crossterm::event::EnableMouseCapture)
                .map_err(|e| terminalos_shared::Error::Ui(format!("mouse: {e}")))?;
        }

        let backend = CrosstermBackend::new(io::stdout());
        let mut terminal = Terminal::new(backend)
            .map_err(|e| terminalos_shared::Error::Ui(format!("terminal: {e}")))?;

        info!("TerminalOS UI started");

        while !self.should_quit {
            terminal
                .draw(|frame| self.render(frame))
                .map_err(|e| terminalos_shared::Error::Ui(format!("draw: {e}")))?;

            if event::poll(Duration::from_millis(50))
                .map_err(|e| terminalos_shared::Error::Ui(format!("poll: {e}")))?
            {
                match event::read()
                    .map_err(|e| terminalos_shared::Error::Ui(format!("read event: {e}")))?
                {
                    Event::Key(key) => {
                        let action = map_key_event(key, self.focus);
                        self.handle_action(action);
                    }
                    Event::Mouse(mouse) => self.handle_mouse(mouse),
                    Event::Resize(_, _) => {}
                    _ => {}
                }
            }

            if self.config.ui.animations && self.last_frame.elapsed() >= Duration::from_millis(500)
            {
                self.tick = self.tick.wrapping_add(1);
                self.last_frame = Instant::now();
            }

            let _ = rt.handle();
        }

        if self.config.ui.mouse_enabled {
            execute!(io::stdout(), crossterm::event::DisableMouseCapture)
                .map_err(|e| terminalos_shared::Error::Ui(format!("mouse off: {e}")))?;
        }
        disable_raw_mode()
            .map_err(|e| terminalos_shared::Error::Ui(format!("disable raw: {e}")))?;
        io::stdout()
            .execute(LeaveAlternateScreen)
            .map_err(|e| terminalos_shared::Error::Ui(format!("leave alt: {e}")))?;
        terminal
            .show_cursor()
            .map_err(|e| terminalos_shared::Error::Ui(format!("show cursor: {e}")))?;

        Ok(())
    }

    fn render(&self, frame: &mut ratatui::Frame<'_>) {
        let area = frame.area();
        let visibility = LayoutVisibility {
            show_sidebar: self.show_sidebar,
            show_chat: self.show_chat,
            show_logs: self.show_logs,
        };
        let layout = compute_layout(area, &self.config.layout, &visibility);
        let buf = frame.buffer_mut();

        if let Some(sidebar_area) = layout.sidebar {
            render_sidebar(
                sidebar_area,
                buf,
                self.file_tree.as_ref(),
                &self.theme,
                self.focus,
                self.sidebar_scroll,
            );
        }

        render_tab_bar(layout.tab_bar, buf, &self.session, &self.theme);
        render_terminal_pane(layout.terminal, buf, &self.session, &self.theme, self.focus);

        if let Some(chat_area) = layout.chat {
            render_chat_pane(
                chat_area,
                buf,
                &self.chat_messages,
                &self.chat_input,
                &self.theme,
                self.focus,
                self.chat_scroll,
            );
        }

        if let Some(logs_area) = layout.logs {
            render_logs_pane(
                logs_area,
                buf,
                &self.logs,
                &self.theme,
                self.focus,
                self.logs_scroll,
            );
        }

        render_status_bar(
            layout.status_bar,
            buf,
            &self.session,
            self.workspace_manager
                .active()
                .map(|w| w.name.as_str())
                .unwrap_or(&self.workspace_name),
            self.branch.as_deref(),
            self.focus,
            &self.theme,
        );
    }

    fn handle_action(&mut self, action: AppAction) {
        match action {
            AppAction::Quit => self.should_quit = true,
            AppAction::NewTab => {
                self.session.new_tab();
                self.push_log(LogLevel::Info, "New terminal tab created");
            }
            AppAction::CloseTab => {
                if self.session.close_tab() {
                    self.push_log(LogLevel::Info, "Terminal tab closed");
                }
            }
            AppAction::NextTab => self.session.next_tab(),
            AppAction::PrevTab => self.session.prev_tab(),
            AppAction::SelectTab(index) => self.session.select_tab(index),
            AppAction::ToggleSidebar => {
                self.show_sidebar = !self.show_sidebar;
                self.push_log(
                    LogLevel::Info,
                    if self.show_sidebar {
                        "Sidebar shown"
                    } else {
                        "Sidebar hidden"
                    },
                );
            }
            AppAction::ToggleChat => {
                self.show_chat = !self.show_chat;
                self.push_log(
                    LogLevel::Info,
                    if self.show_chat {
                        "AI chat shown"
                    } else {
                        "AI chat hidden"
                    },
                );
            }
            AppAction::ToggleLogs => {
                self.show_logs = !self.show_logs;
            }
            AppAction::FocusTerminal => self.focus = FocusedPane::Terminal,
            AppAction::FocusChat => self.focus = FocusedPane::Chat,
            AppAction::FocusSidebar => self.focus = FocusedPane::Sidebar,
            AppAction::FocusLogs => self.focus = FocusedPane::Logs,
            AppAction::CycleFocus => self.cycle_focus(),
            AppAction::ResizeSidebar(delta) => self.resize_sidebar(delta),
            AppAction::ResizeChat(delta) => self.resize_chat(delta),
            AppAction::ResizeLogs(delta) => self.resize_logs(delta),
            AppAction::TerminalInput(c) => {
                self.session.active_tab_mut().input.push(c);
            }
            AppAction::TerminalBackspace => {
                self.session.active_tab_mut().input.pop();
            }
            AppAction::TerminalSubmit => self.submit_terminal_command(),
            AppAction::ChatInput(c) => self.chat_input.push(c),
            AppAction::ChatBackspace => {
                self.chat_input.pop();
            }
            AppAction::ChatSubmit => self.submit_chat_message(),
            AppAction::ScrollUp => self.scroll_up(),
            AppAction::ScrollDown => self.scroll_down(),
            AppAction::Noop => {}
        }
    }

    fn handle_mouse(&mut self, event: MouseEvent) {
        if !self.config.ui.mouse_enabled {
            return;
        }

        match event.kind {
            MouseEventKind::ScrollUp => self.scroll_up(),
            MouseEventKind::ScrollDown => self.scroll_down(),
            _ => {}
        }
    }

    fn cycle_focus(&mut self) {
        self.focus = match self.focus {
            FocusedPane::Terminal => FocusedPane::Chat,
            FocusedPane::Chat => {
                if self.show_sidebar {
                    FocusedPane::Sidebar
                } else if self.show_logs {
                    FocusedPane::Logs
                } else {
                    FocusedPane::Terminal
                }
            }
            FocusedPane::Sidebar => {
                if self.show_logs {
                    FocusedPane::Logs
                } else {
                    FocusedPane::Terminal
                }
            }
            FocusedPane::Logs => FocusedPane::Terminal,
        };
    }

    fn resize_sidebar(&mut self, delta: i16) {
        let current = self.config.layout.sidebar_width_percent as i16;
        let next = (current + delta).clamp(10, 40);
        self.config.layout.sidebar_width_percent = next as u16;
    }

    fn resize_chat(&mut self, delta: i16) {
        let current = self.config.layout.chat_width_percent as i16;
        let next = (current + delta).clamp(15, 50);
        self.config.layout.chat_width_percent = next as u16;
    }

    fn resize_logs(&mut self, delta: i16) {
        let current = self.config.layout.logs_height_percent as i16;
        let next = (current + delta).clamp(8, 35);
        self.config.layout.logs_height_percent = next as u16;
    }

    fn scroll_up(&mut self) {
        match self.focus {
            FocusedPane::Terminal => self.session.active_tab_mut().buffer.scroll_up(1),
            FocusedPane::Chat => self.chat_scroll = self.chat_scroll.saturating_sub(1),
            FocusedPane::Sidebar => self.sidebar_scroll = self.sidebar_scroll.saturating_sub(1),
            FocusedPane::Logs => self.logs_scroll = self.logs_scroll.saturating_sub(1),
        }
    }

    fn scroll_down(&mut self) {
        match self.focus {
            FocusedPane::Terminal => self.session.active_tab_mut().buffer.scroll_down(1),
            FocusedPane::Chat => self.chat_scroll = self.chat_scroll.saturating_add(1),
            FocusedPane::Sidebar => self.sidebar_scroll = self.sidebar_scroll.saturating_add(1),
            FocusedPane::Logs => self.logs_scroll = self.logs_scroll.saturating_add(1),
        }
    }

    fn submit_terminal_command(&mut self) {
        let tab = self.session.active_tab_mut();
        let command = std::mem::take(&mut tab.input);
        if command.is_empty() {
            return;
        }

        tab.buffer.push_line(format!("{} $ {command}", tab.cwd));

        if command == "clear" {
            tab.buffer = terminalos_terminal::TerminalBuffer::new(10_000);
            self.push_log(LogLevel::Info, "Terminal cleared");
            return;
        }

        if command == "help" {
            tab.buffer.push_str(HELP_MESSAGE);
            return;
        }

        tab.buffer.push_line(format!(
            "Shell execution available in Phase 2. Received: {command}"
        ));
        self.push_log(LogLevel::Info, format!("Command entered: {command}"));
    }

    fn submit_chat_message(&mut self) {
        let content = std::mem::take(&mut self.chat_input);
        if content.is_empty() {
            return;
        }

        self.chat_messages.push(ChatMessage {
            role: "user".to_string(),
            content: content.clone(),
        });

        self.chat_messages.push(ChatMessage {
            role: "assistant".to_string(),
            content: "AI providers connect in Phase 3. Your message was received.".to_string(),
        });

        self.push_log(LogLevel::Info, "Chat message sent");
    }

    fn push_log(&mut self, level: LogLevel, message: impl Into<String>) {
        self.logs.push(LogEntry::new(level, message));
        if self.logs.len() > 200 {
            self.logs.drain(0..50);
        }
    }
}

const WELCOME_MESSAGE: &str = r"
  ╔══════════════════════════════════════════════════════════╗
  ║                                                          ║
  ║   ████████╗███████╗██████╗ ███╗   ███╗██╗███╗   ██╗ █████╗ ║
  ║   ╚══██╔══╝██╔════╝██╔══██╗████╗ ████║██║████╗  ██║██╔══██╗║
  ║      ██║   █████╗  ██████╔╝██╔████╔██║██║██╔██╗ ██║███████║║
  ║      ██║   ██╔══╝  ██╔══██╗██║╚██╔╝██║██║██║╚██╗██║██╔══██║║
  ║      ██║   ███████╗██║  ██║██║ ╚═╝ ██║██║██║ ╚████║██║  ██║║
  ║      ╚═╝   ╚══════╝╚═╝  ╚═╝╚═╝     ╚═╝╚═╝╚═╝  ╚═══╝╚═╝  ╚═╝║
  ║                        OS v0.1.0                         ║
  ║          The AI-native terminal for developers.          ║
  ║                                                          ║
  ╚══════════════════════════════════════════════════════════╝

  Type 'help' for keyboard shortcuts.
";

const HELP_MESSAGE: &str = r"
  Keyboard Shortcuts:
  ─────────────────────────────────────────
  Ctrl+T        New tab
  Ctrl+W        Close tab
  Ctrl+Tab      Next tab
  Ctrl+Shift+Tab Previous tab
  Ctrl+B        Toggle sidebar
  Ctrl+/        Toggle AI chat
  Ctrl+`        Toggle logs
  Ctrl+1/2/3/4  Focus terminal/chat/sidebar/logs
  Tab           Cycle focus
  Ctrl+←/→      Resize sidebar
  Ctrl+↑/↓      Resize chat panel
  Ctrl+Shift+↑/↓ Resize logs panel
  Ctrl+Q        Quit
";
