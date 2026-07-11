use serde::{Deserialize, Serialize};
use terminalos_shared::Result;

/// Metadata describing an installed plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
}

/// Plugin manifest parsed from plugin.toml.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub info: PluginInfo,
    pub entry: String,
    pub enabled: bool,
}

/// Trait implemented by TerminalOS plugins.
pub trait Plugin: Send + Sync {
    fn info(&self) -> &PluginInfo;

    fn on_load(&mut self) -> Result<()>;

    fn on_unload(&mut self) -> Result<()>;

    fn execute(&self, command: &str, args: &[String]) -> Result<String>;
}
