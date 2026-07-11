use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

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
    TerminalInput(char),
    TerminalBackspace,
    TerminalSubmit,
    ChatInput(char),
    ChatBackspace,
    ChatSubmit,
    ScrollUp,
    ScrollDown,
    Noop,
}

/// Maps crossterm key events to application actions based on focused pane.
#[must_use]
pub fn map_key_event(key: KeyEvent, focus: FocusedPane) -> AppAction {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    let shift = key.modifiers.contains(KeyModifiers::SHIFT);

    if ctrl && key.code == KeyCode::Char('q') {
        return AppAction::Quit;
    }
    if ctrl && key.code == KeyCode::Char('t') {
        return AppAction::NewTab;
    }
    if ctrl && key.code == KeyCode::Char('w') {
        return AppAction::CloseTab;
    }
    if ctrl && shift && key.code == KeyCode::BackTab {
        return AppAction::PrevTab;
    }
    if ctrl && key.code == KeyCode::Tab {
        return AppAction::NextTab;
    }
    if ctrl && key.code == KeyCode::Char('b') {
        return AppAction::ToggleSidebar;
    }
    if ctrl && key.code == KeyCode::Char('/') {
        return AppAction::ToggleChat;
    }
    if ctrl && key.code == KeyCode::Char('`') {
        return AppAction::ToggleLogs;
    }
    if ctrl && key.code == KeyCode::Char('1') {
        return AppAction::FocusTerminal;
    }
    if ctrl && key.code == KeyCode::Char('2') {
        return AppAction::FocusChat;
    }
    if ctrl && key.code == KeyCode::Char('3') {
        return AppAction::FocusSidebar;
    }
    if ctrl && key.code == KeyCode::Char('4') {
        return AppAction::FocusLogs;
    }
    if ctrl && shift && key.code == KeyCode::Up {
        return AppAction::ResizeLogs(1);
    }
    if ctrl && shift && key.code == KeyCode::Down {
        return AppAction::ResizeLogs(-1);
    }
    if ctrl && key.code == KeyCode::Right {
        return AppAction::ResizeSidebar(1);
    }
    if ctrl && key.code == KeyCode::Left {
        return AppAction::ResizeSidebar(-1);
    }
    if ctrl && key.code == KeyCode::Up {
        return AppAction::ResizeChat(1);
    }
    if ctrl && key.code == KeyCode::Down {
        return AppAction::ResizeChat(-1);
    }
    if key.code == KeyCode::Tab && !ctrl {
        return AppAction::CycleFocus;
    }

    match focus {
        FocusedPane::Terminal => map_terminal_keys(key),
        FocusedPane::Chat => map_chat_keys(key),
        FocusedPane::Sidebar | FocusedPane::Logs => map_navigation_keys(key),
    }
}

fn map_terminal_keys(key: KeyEvent) -> AppAction {
    match key.code {
        KeyCode::Char(c) => AppAction::TerminalInput(c),
        KeyCode::Backspace => AppAction::TerminalBackspace,
        KeyCode::Enter => AppAction::TerminalSubmit,
        KeyCode::Up => AppAction::ScrollUp,
        KeyCode::Down => AppAction::ScrollDown,
        _ => AppAction::Noop,
    }
}

fn map_chat_keys(key: KeyEvent) -> AppAction {
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

/// Maps digit keys with Ctrl modifier to tab selection.
#[must_use]
pub fn map_tab_shortcut(key: KeyEvent) -> Option<usize> {
    if !key.modifiers.contains(KeyModifiers::CONTROL) {
        return None;
    }

    match key.code {
        KeyCode::Char('5') => Some(4),
        KeyCode::Char('6') => Some(5),
        KeyCode::Char('7') => Some(6),
        KeyCode::Char('8') => Some(7),
        KeyCode::Char('9') => Some(8),
        _ => None,
    }
}
