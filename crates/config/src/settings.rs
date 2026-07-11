use serde::{Deserialize, Serialize};
use terminalos_shared::ThemeMode;

/// Root application configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppConfig {
    pub ui: UiConfig,
    pub layout: LayoutConfig,
    pub providers: Vec<ProviderConfig>,
    pub default_provider: Option<String>,
}

/// UI-related configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub theme: ThemeMode,
    pub show_sidebar: bool,
    pub show_chat: bool,
    pub show_logs: bool,
    pub animations: bool,
    pub mouse_enabled: bool,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: ThemeMode::Dark,
            show_sidebar: true,
            show_chat: true,
            show_logs: true,
            animations: true,
            mouse_enabled: true,
        }
    }
}

/// Resizable pane layout configuration (percentages).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutConfig {
    pub sidebar_width_percent: u16,
    pub chat_width_percent: u16,
    pub logs_height_percent: u16,
    pub status_bar_height: u16,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            sidebar_width_percent: 20,
            chat_width_percent: 30,
            logs_height_percent: 15,
            status_bar_height: 1,
        }
    }
}

/// AI provider configuration entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub name: String,
    pub provider_type: ProviderType,
    pub api_key_env: String,
    pub base_url: Option<String>,
    pub model: String,
    pub enabled: bool,
}

/// Supported AI provider backends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderType {
    OpenAi,
    Anthropic,
    OpenRouter,
    Ollama,
    Gemini,
    DeepSeek,
}
