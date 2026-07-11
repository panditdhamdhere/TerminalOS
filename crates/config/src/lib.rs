//! TOML-based configuration for TerminalOS.

pub mod keybindings;
pub mod loader;
pub mod settings;

pub use keybindings::Keybindings;
pub use loader::ConfigLoader;
pub use settings::{AppConfig, LayoutConfig, ProviderConfig, ProviderType, UiConfig};
