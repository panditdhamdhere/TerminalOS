use serde::{Deserialize, Serialize};
use terminalos_shared::ThemeMode;

/// Root application configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub ui: UiConfig,
    pub layout: LayoutConfig,
    pub providers: Vec<ProviderConfig>,
    pub default_provider: Option<String>,
    #[serde(default)]
    pub agent: AgentConfig,
    #[serde(default)]
    pub workspace: WorkspaceConfig,
    #[serde(default)]
    pub search: SearchConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            ui: UiConfig::default(),
            layout: LayoutConfig::default(),
            providers: default_providers(),
            default_provider: Some("ollama".to_string()),
            agent: AgentConfig::default(),
            workspace: WorkspaceConfig::default(),
            search: SearchConfig::default(),
        }
    }
}

fn default_providers() -> Vec<ProviderConfig> {
    vec![
        ProviderConfig {
            name: "ollama".to_string(),
            provider_type: ProviderType::Ollama,
            api_key_env: "OLLAMA_API_KEY".to_string(),
            base_url: Some("http://localhost:11434/v1".to_string()),
            model: "llama3.2".to_string(),
            enabled: true,
        },
        ProviderConfig {
            name: "openai".to_string(),
            provider_type: ProviderType::OpenAi,
            api_key_env: "OPENAI_API_KEY".to_string(),
            base_url: None,
            model: "gpt-4o".to_string(),
            enabled: false,
        },
        ProviderConfig {
            name: "anthropic".to_string(),
            provider_type: ProviderType::Anthropic,
            api_key_env: "ANTHROPIC_API_KEY".to_string(),
            base_url: None,
            model: "claude-sonnet-4-20250514".to_string(),
            enabled: false,
        },
        ProviderConfig {
            name: "openrouter".to_string(),
            provider_type: ProviderType::OpenRouter,
            api_key_env: "OPENROUTER_API_KEY".to_string(),
            base_url: None,
            model: "anthropic/claude-sonnet-4".to_string(),
            enabled: false,
        },
        ProviderConfig {
            name: "gemini".to_string(),
            provider_type: ProviderType::Gemini,
            api_key_env: "GEMINI_API_KEY".to_string(),
            base_url: None,
            model: "gemini-2.0-flash".to_string(),
            enabled: false,
        },
        ProviderConfig {
            name: "deepseek".to_string(),
            provider_type: ProviderType::DeepSeek,
            api_key_env: "DEEPSEEK_API_KEY".to_string(),
            base_url: None,
            model: "deepseek-chat".to_string(),
            enabled: false,
        },
    ]
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

/// Coding agent configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub max_iterations: u8,
    pub require_confirm_write: bool,
    pub require_confirm_shell: bool,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_iterations: 8,
            require_confirm_write: true,
            require_confirm_shell: true,
        }
    }
}

/// Workspace persistence and session restoration settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    pub auto_restore: bool,
    pub autosave_secs: u64,
}

impl Default for WorkspaceConfig {
    fn default() -> Self {
        Self {
            auto_restore: true,
            autosave_secs: 30,
        }
    }
}

/// Semantic and hybrid search configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    pub mode: SearchMode,
    pub keyword_weight: f32,
    pub semantic_weight: f32,
    pub embedding_base_url: String,
    pub embedding_model: String,
    pub embedding_api_key_env: String,
}

/// Search mode for keyword, semantic, or hybrid queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SearchMode {
    #[default]
    Hybrid,
    Keyword,
    Semantic,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            mode: SearchMode::Hybrid,
            keyword_weight: 0.4,
            semantic_weight: 0.6,
            embedding_base_url: "http://localhost:11434".to_string(),
            embedding_model: "nomic-embed-text".to_string(),
            embedding_api_key_env: "OLLAMA_API_KEY".to_string(),
        }
    }
}
