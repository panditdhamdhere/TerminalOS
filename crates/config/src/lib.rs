//! TOML-based configuration for TerminalOS.

pub mod keybinding;
pub mod keybindings;
pub mod loader;
pub mod profiles;
pub mod settings;
pub mod setup;
pub mod themes;

pub use keybinding::{GlobalAction, KeybindingResolver, ParsedKey, binding_map, parse_key_combo};
pub use keybindings::Keybindings;
pub use loader::ConfigLoader;
pub use profiles::{Profile, apply_profile, ensure_default_profiles, list_profiles, load_profile};
pub use settings::{
    AgentConfig, AppConfig, LayoutConfig, PluginConfig, ProviderConfig, ProviderType, SearchConfig,
    SearchMode, UiConfig, WorkspaceConfig,
};
pub use setup::{
    ProviderStatus, SetupChoice, auto_configure_providers, enable_provider, is_setup_complete,
    load_env_files, mark_setup_complete, needs_interactive_setup, provider_is_ready,
    provider_statuses, run_interactive_setup, set_default_provider,
};
pub use themes::{ThemePreset, builtin_preset, builtin_preset_names, resolve_theme};
