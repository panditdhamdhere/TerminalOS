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
    ScrollUp,
    ScrollDown,
    Noop,
}

/// Maps crossterm key events to application actions based on focused pane.
#[must_use]
pub fn map_key_event(key: KeyEvent, focus: FocusedPane, terminal_search: bool) -> AppAction {
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
        FocusedPane::Terminal => map_terminal_keys(key, terminal_search),
        FocusedPane::Chat => map_chat_keys(key),
        FocusedPane::Sidebar | FocusedPane::Logs => map_navigation_keys(key),
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
