//! Plugin SDK and dynamic loading.
#![allow(unsafe_code)]

pub mod api;
pub mod dynamic;
pub mod loader;
pub mod manager;
pub mod marketplace;

pub use api::{
    PLUGIN_API_VERSION, PLUGIN_ENTRY_SYMBOL, PLUGIN_ERROR, PLUGIN_SUCCESS, Plugin, PluginCommand,
    PluginDescriptor, PluginEntryFn, PluginExecuteFn, PluginExports, PluginInfo, PluginInitFn,
    PluginManifest, PluginShutdownFn, parse_plugin_args, read_plugin_command, write_plugin_output,
};
pub use dynamic::DynamicPlugin;
pub use loader::PluginLoader;
pub use manager::{InstalledPlugin, PluginManager};
pub use marketplace::{MarketplaceCatalog, MarketplaceEntry, PluginInstaller, PluginMarketplace};
