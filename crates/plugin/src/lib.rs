//! Plugin SDK and dynamic loading.

pub mod api;
pub mod loader;

pub use api::{Plugin, PluginInfo, PluginManifest};
pub use loader::PluginLoader;
