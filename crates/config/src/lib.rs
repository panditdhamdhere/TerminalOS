//! TOML-based configuration for TerminalOS.

pub mod keybinding;
pub mod keybindings;
pub mod loader;
pub mod profiles;
pub mod settings;
pub mod themes;

pub use keybinding::{GlobalAction, KeybindingResolver, ParsedKey, parse_key_combo};
pub use keybindings::Keybindings;
pub use loader::ConfigLoader;
pub use profiles::{Profile, apply_profile, ensure_default_profiles, list_profiles, load_profile};
pub use settings::{
    AgentConfig, AppConfig, LayoutConfig, PluginConfig, ProviderConfig, ProviderType, SearchConfig,
    SearchMode, UiConfig, WorkspaceConfig,
};
pub use themes::{ThemePreset, builtin_preset, builtin_preset_names, resolve_theme};
