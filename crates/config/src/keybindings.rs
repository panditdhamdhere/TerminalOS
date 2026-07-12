use serde::{Deserialize, Serialize};

/// Keyboard shortcut bindings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keybindings {
    pub new_tab: String,
    pub close_tab: String,
    pub next_tab: String,
    pub prev_tab: String,
    pub toggle_sidebar: String,
    pub toggle_chat: String,
    pub toggle_logs: String,
    pub focus_terminal: String,
    pub focus_chat: String,
    pub focus_sidebar: String,
    pub focus_logs: String,
    pub quit: String,
    pub resize_sidebar_increase: String,
    pub resize_sidebar_decrease: String,
    pub resize_chat_increase: String,
    pub resize_chat_decrease: String,
    pub resize_logs_increase: String,
    pub resize_logs_decrease: String,
    pub toggle_provider_picker: String,
}

impl Default for Keybindings {
    fn default() -> Self {
        Self {
            new_tab: "Ctrl+T".to_string(),
            close_tab: "Ctrl+W".to_string(),
            next_tab: "Ctrl+Tab".to_string(),
            prev_tab: "Ctrl+Shift+Tab".to_string(),
            toggle_sidebar: "Ctrl+B".to_string(),
            toggle_chat: "Ctrl+/".to_string(),
            toggle_logs: "Ctrl+`".to_string(),
            focus_terminal: "Ctrl+1".to_string(),
            focus_chat: "Ctrl+2".to_string(),
            focus_sidebar: "Ctrl+3".to_string(),
            focus_logs: "Ctrl+4".to_string(),
            quit: "Ctrl+Q".to_string(),
            resize_sidebar_increase: "Ctrl+Right".to_string(),
            resize_sidebar_decrease: "Ctrl+Left".to_string(),
            resize_chat_increase: "Ctrl+Up".to_string(),
            resize_chat_decrease: "Ctrl+Down".to_string(),
            resize_logs_increase: "Ctrl+Shift+Up".to_string(),
            resize_logs_decrease: "Ctrl+Shift+Down".to_string(),
            toggle_provider_picker: "Ctrl+P".to_string(),
        }
    }
}
