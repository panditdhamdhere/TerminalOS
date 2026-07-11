use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use terminalos_config::{GlobalAction, KeybindingResolver};

/// Which pane currently has keyboard focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FocusedPane {
    #[default]
    Terminal,
    Chat,
    Sidebar,
    Logs,
}

/// Application-level actions triggered by keyboard or mouse input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppAction {
    Quit,
    NewTab,
    CloseTab,
    NextTab,
    PrevTab,
    SelectTab(usize),
    ToggleSidebar,
    ToggleChat,
    ToggleLogs,
    FocusTerminal,
    FocusChat,
    FocusSidebar,
    FocusLogs,
    CycleFocus,
    ResizeSidebar(i16),
    ResizeChat(i16),
    ResizeLogs(i16),
    TerminalKey(KeyEvent),
    TerminalScrollUp,
    TerminalScrollDown,
    TerminalCopy,
    TerminalPaste,
    TerminalToggleSearch,
    SearchInput(char),
    SearchBackspace,
    SearchSubmit,
    ChatInput(char),
    ChatBackspace,
    ChatSubmit,
    ConfirmPending,
    RejectPending,
    ScrollUp,
    ScrollDown,
    Noop,
}

/// Maps crossterm key events to application actions based on focused pane.
#[must_use]
pub fn map_key_event(
    key: KeyEvent,
    focus: FocusedPane,
    terminal_search: bool,
    pending_action: bool,
    bindings: &KeybindingResolver,
) -> AppAction {
    if let Some(global) = bindings.resolve(key) {
        return global_to_action(global);
    }

    if key.code == KeyCode::Tab && !key.modifiers.contains(KeyModifiers::CONTROL) {
        return AppAction::CycleFocus;
    }

    match focus {
        FocusedPane::Terminal => map_terminal_keys(key, terminal_search),
        FocusedPane::Chat => map_chat_keys(key, pending_action),
        FocusedPane::Sidebar | FocusedPane::Logs => map_navigation_keys(key),
    }
}

fn global_to_action(action: GlobalAction) -> AppAction {
    match action {
        GlobalAction::Quit => AppAction::Quit,
        GlobalAction::NewTab => AppAction::NewTab,
        GlobalAction::CloseTab => AppAction::CloseTab,
        GlobalAction::NextTab => AppAction::NextTab,
        GlobalAction::PrevTab => AppAction::PrevTab,
        GlobalAction::ToggleSidebar => AppAction::ToggleSidebar,
        GlobalAction::ToggleChat => AppAction::ToggleChat,
        GlobalAction::ToggleLogs => AppAction::ToggleLogs,
        GlobalAction::FocusTerminal => AppAction::FocusTerminal,
        GlobalAction::FocusChat => AppAction::FocusChat,
        GlobalAction::FocusSidebar => AppAction::FocusSidebar,
        GlobalAction::FocusLogs => AppAction::FocusLogs,
        GlobalAction::ResizeSidebarIncrease => AppAction::ResizeSidebar(1),
        GlobalAction::ResizeSidebarDecrease => AppAction::ResizeSidebar(-1),
        GlobalAction::ResizeChatIncrease => AppAction::ResizeChat(1),
        GlobalAction::ResizeChatDecrease => AppAction::ResizeChat(-1),
        GlobalAction::ResizeLogsIncrease => AppAction::ResizeLogs(1),
        GlobalAction::ResizeLogsDecrease => AppAction::ResizeLogs(-1),
    }
}

fn map_terminal_keys(key: KeyEvent, search_mode: bool) -> AppAction {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    let shift = key.modifiers.contains(KeyModifiers::SHIFT);

    if search_mode {
        return match key.code {
            KeyCode::Esc => AppAction::TerminalToggleSearch,
            KeyCode::Char(c) => AppAction::SearchInput(c),
            KeyCode::Backspace => AppAction::SearchBackspace,
            KeyCode::Enter => AppAction::SearchSubmit,
            _ => AppAction::Noop,
        };
    }

    if ctrl && shift && key.code == KeyCode::Char('c') {
        return AppAction::TerminalCopy;
    }
    if ctrl && shift && key.code == KeyCode::Char('v') {
        return AppAction::TerminalPaste;
    }
    if ctrl && shift && key.code == KeyCode::Char('f') {
        return AppAction::TerminalToggleSearch;
    }

    if ctrl && shift && matches!(key.code, KeyCode::Up | KeyCode::Down) {
        return match key.code {
            KeyCode::Up => AppAction::TerminalScrollUp,
            KeyCode::Down => AppAction::TerminalScrollDown,
            _ => AppAction::Noop,
        };
    }

    if matches!(key.code, KeyCode::PageUp | KeyCode::PageDown) && !ctrl {
        return match key.code {
            KeyCode::PageUp => AppAction::TerminalScrollUp,
            KeyCode::PageDown => AppAction::TerminalScrollDown,
            _ => AppAction::Noop,
        };
    }

    AppAction::TerminalKey(key)
}

fn map_chat_keys(key: KeyEvent, pending_action: bool) -> AppAction {
    if pending_action {
        return match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => AppAction::ConfirmPending,
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => AppAction::RejectPending,
            _ => AppAction::Noop,
        };
    }

    match key.code {
        KeyCode::Char(c) => AppAction::ChatInput(c),
        KeyCode::Backspace => AppAction::ChatBackspace,
        KeyCode::Enter => AppAction::ChatSubmit,
        KeyCode::Up => AppAction::ScrollUp,
        KeyCode::Down => AppAction::ScrollDown,
        _ => AppAction::Noop,
    }
}

fn map_navigation_keys(key: KeyEvent) -> AppAction {
    match key.code {
        KeyCode::Up => AppAction::ScrollUp,
        KeyCode::Down => AppAction::ScrollDown,
        _ => AppAction::Noop,
    }
}
