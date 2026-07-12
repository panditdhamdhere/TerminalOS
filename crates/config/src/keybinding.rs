use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use terminalos_shared::{Error, Result};

use crate::keybindings::Keybindings;

/// Global shortcut actions resolved from configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlobalAction {
    Quit,
    NewTab,
    CloseTab,
    NextTab,
    PrevTab,
    ToggleSidebar,
    ToggleChat,
    ToggleLogs,
    FocusTerminal,
    FocusChat,
    FocusSidebar,
    FocusLogs,
    ResizeSidebarIncrease,
    ResizeSidebarDecrease,
    ResizeChatIncrease,
    ResizeChatDecrease,
    ResizeLogsIncrease,
    ResizeLogsDecrease,
    ToggleProviderPicker,
    SplitHorizontal,
    SplitVertical,
    ClosePane,
    FocusNextPane,
    FocusPrevPane,
}

/// Parsed key combination from a config string like `Ctrl+Shift+Tab`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedKey {
    pub modifiers: KeyModifiers,
    pub code: KeyCode,
}

/// Resolves configured keybindings against incoming key events.
#[derive(Debug, Clone)]
pub struct KeybindingResolver {
    entries: Vec<(ParsedKey, GlobalAction)>,
}

impl KeybindingResolver {
    #[must_use]
    pub fn new(bindings: &Keybindings) -> Self {
        let mut entries = Vec::new();
        add_binding(&mut entries, &bindings.quit, GlobalAction::Quit);
        add_binding(&mut entries, &bindings.new_tab, GlobalAction::NewTab);
        add_binding(&mut entries, &bindings.close_tab, GlobalAction::CloseTab);
        add_binding(&mut entries, &bindings.next_tab, GlobalAction::NextTab);
        add_binding(&mut entries, &bindings.prev_tab, GlobalAction::PrevTab);
        add_binding(
            &mut entries,
            &bindings.toggle_sidebar,
            GlobalAction::ToggleSidebar,
        );
        add_binding(
            &mut entries,
            &bindings.toggle_chat,
            GlobalAction::ToggleChat,
        );
        add_binding(
            &mut entries,
            &bindings.toggle_logs,
            GlobalAction::ToggleLogs,
        );
        add_binding(
            &mut entries,
            &bindings.focus_terminal,
            GlobalAction::FocusTerminal,
        );
        add_binding(&mut entries, &bindings.focus_chat, GlobalAction::FocusChat);
        add_binding(
            &mut entries,
            &bindings.focus_sidebar,
            GlobalAction::FocusSidebar,
        );
        add_binding(&mut entries, &bindings.focus_logs, GlobalAction::FocusLogs);
        add_binding(
            &mut entries,
            &bindings.resize_sidebar_increase,
            GlobalAction::ResizeSidebarIncrease,
        );
        add_binding(
            &mut entries,
            &bindings.resize_sidebar_decrease,
            GlobalAction::ResizeSidebarDecrease,
        );
        add_binding(
            &mut entries,
            &bindings.resize_chat_increase,
            GlobalAction::ResizeChatIncrease,
        );
        add_binding(
            &mut entries,
            &bindings.resize_chat_decrease,
            GlobalAction::ResizeChatDecrease,
        );
        add_binding(
            &mut entries,
            &bindings.resize_logs_increase,
            GlobalAction::ResizeLogsIncrease,
        );
        add_binding(
            &mut entries,
            &bindings.resize_logs_decrease,
            GlobalAction::ResizeLogsDecrease,
        );
        add_binding(
            &mut entries,
            &bindings.toggle_provider_picker,
            GlobalAction::ToggleProviderPicker,
        );
        add_binding(
            &mut entries,
            &bindings.split_horizontal,
            GlobalAction::SplitHorizontal,
        );
        add_binding(
            &mut entries,
            &bindings.split_vertical,
            GlobalAction::SplitVertical,
        );
        add_binding(&mut entries, &bindings.close_pane, GlobalAction::ClosePane);
        add_binding(
            &mut entries,
            &bindings.focus_next_pane,
            GlobalAction::FocusNextPane,
        );
        add_binding(
            &mut entries,
            &bindings.focus_prev_pane,
            GlobalAction::FocusPrevPane,
        );
        Self { entries }
    }

    pub fn resolve(&self, key: KeyEvent) -> Option<GlobalAction> {
        let normalized = normalize_key(key);
        self.entries
            .iter()
            .find(|(parsed, _)| keys_match(parsed, &normalized))
            .map(|(_, action)| *action)
    }
}

fn add_binding(entries: &mut Vec<(ParsedKey, GlobalAction)>, binding: &str, action: GlobalAction) {
    if let Ok(parsed) = parse_key_combo(binding) {
        entries.push((parsed, action));
    }
}

/// Returns configured binding strings paired with action labels.
#[must_use]
pub fn binding_map(bindings: &Keybindings) -> Vec<(&'static str, String)> {
    vec![
        ("quit", bindings.quit.clone()),
        ("new_tab", bindings.new_tab.clone()),
        ("close_tab", bindings.close_tab.clone()),
        ("next_tab", bindings.next_tab.clone()),
        ("prev_tab", bindings.prev_tab.clone()),
        ("toggle_sidebar", bindings.toggle_sidebar.clone()),
        ("toggle_chat", bindings.toggle_chat.clone()),
        ("toggle_logs", bindings.toggle_logs.clone()),
        ("focus_terminal", bindings.focus_terminal.clone()),
        ("focus_chat", bindings.focus_chat.clone()),
        ("focus_sidebar", bindings.focus_sidebar.clone()),
        ("focus_logs", bindings.focus_logs.clone()),
        (
            "resize_sidebar_increase",
            bindings.resize_sidebar_increase.clone(),
        ),
        (
            "resize_sidebar_decrease",
            bindings.resize_sidebar_decrease.clone(),
        ),
        (
            "resize_chat_increase",
            bindings.resize_chat_increase.clone(),
        ),
        (
            "resize_chat_decrease",
            bindings.resize_chat_decrease.clone(),
        ),
        (
            "resize_logs_increase",
            bindings.resize_logs_increase.clone(),
        ),
        (
            "resize_logs_decrease",
            bindings.resize_logs_decrease.clone(),
        ),
        (
            "toggle_provider_picker",
            bindings.toggle_provider_picker.clone(),
        ),
        ("split_horizontal", bindings.split_horizontal.clone()),
        ("split_vertical", bindings.split_vertical.clone()),
        ("close_pane", bindings.close_pane.clone()),
        ("focus_next_pane", bindings.focus_next_pane.clone()),
        ("focus_prev_pane", bindings.focus_prev_pane.clone()),
    ]
}

/// Parses a human-readable key combination into crossterm key data.
pub fn parse_key_combo(input: &str) -> Result<ParsedKey> {
    let parts: Vec<&str> = input.split('+').map(str::trim).collect();
    if parts.is_empty() {
        return Err(Error::Config("empty keybinding".to_string()));
    }

    let mut modifiers = KeyModifiers::empty();
    let mut key_part = parts[parts.len() - 1];

    for part in &parts[..parts.len().saturating_sub(1)] {
        match part.to_ascii_lowercase().as_str() {
            "ctrl" | "control" => modifiers |= KeyModifiers::CONTROL,
            "shift" => modifiers |= KeyModifiers::SHIFT,
            "alt" | "option" => modifiers |= KeyModifiers::ALT,
            "meta" | "super" | "cmd" => modifiers |= KeyModifiers::META,
            other => {
                return Err(Error::Config(format!("unknown modifier: {other}")));
            }
        }
    }

    if parts.len() == 1 {
        key_part = parts[0];
    }

    let code = parse_key_code(key_part)?;
    Ok(ParsedKey { modifiers, code })
}

fn parse_key_code(input: &str) -> Result<KeyCode> {
    match input.to_ascii_lowercase().as_str() {
        "tab" => Ok(KeyCode::Tab),
        "backtab" => Ok(KeyCode::BackTab),
        "left" => Ok(KeyCode::Left),
        "right" => Ok(KeyCode::Right),
        "up" => Ok(KeyCode::Up),
        "down" => Ok(KeyCode::Down),
        "enter" | "return" => Ok(KeyCode::Enter),
        "esc" | "escape" => Ok(KeyCode::Esc),
        "backspace" => Ok(KeyCode::Backspace),
        "pageup" => Ok(KeyCode::PageUp),
        "pagedown" => Ok(KeyCode::PageDown),
        "home" => Ok(KeyCode::Home),
        "end" => Ok(KeyCode::End),
        "space" => Ok(KeyCode::Char(' ')),
        other if other.len() == 1 => {
            let ch = other.chars().next().expect("single char");
            Ok(KeyCode::Char(ch))
        }
        other => Err(Error::Config(format!("invalid key code: {other}"))),
    }
}

fn normalize_key(key: KeyEvent) -> ParsedKey {
    let code = match key.code {
        KeyCode::Char(c) => KeyCode::Char(c.to_ascii_lowercase()),
        other => other,
    };
    ParsedKey {
        modifiers: key.modifiers,
        code,
    }
}

fn keys_match(expected: &ParsedKey, actual: &ParsedKey) -> bool {
    expected.modifiers == actual.modifiers && expected.code == actual.code
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ctrl_shift_tab() {
        let parsed = parse_key_combo("Ctrl+Shift+Tab").expect("parse");
        assert_eq!(
            parsed.modifiers,
            KeyModifiers::CONTROL | KeyModifiers::SHIFT
        );
        assert_eq!(parsed.code, KeyCode::Tab);
    }

    #[test]
    fn resolves_quit_binding() {
        let bindings = Keybindings::default();
        let resolver = KeybindingResolver::new(&bindings);
        let action = resolver.resolve(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL));
        assert_eq!(action, Some(GlobalAction::Quit));
    }
}
