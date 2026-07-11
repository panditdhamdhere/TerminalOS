use std::path::{Path, PathBuf};

use terminalos_config::ConfigLoader;
use terminalos_shared::{Error, Result};

use crate::api::{PluginCommand, PluginInfo, PluginManifest};
use crate::dynamic::DynamicPlugin;
use crate::loader::PluginLoader;

/// An installed plugin with optional dynamic library.
pub struct InstalledPlugin {
    pub manifest: PluginManifest,
    pub install_dir: PathBuf,
    pub dynamic: Option<DynamicPlugin>,
}

impl InstalledPlugin {
    #[must_use]
    pub fn info(&self) -> &PluginInfo {
        &self.manifest.info
    }

    #[must_use]
    pub fn commands(&self) -> &[PluginCommand] {
        &self.manifest.commands
    }

    pub fn execute(&self, command: &str, args: &[String]) -> Result<String> {
        let Some(plugin) = &self.dynamic else {
            return Err(Error::Plugin(format!(
                "plugin {} is installed but its library is not loaded",
                self.manifest.info.name
            )));
        };
        plugin.execute(command, args)
    }
}

/// Discovers, loads, and executes TerminalOS plugins.
#[derive(Default)]
pub struct PluginManager {
    plugins: Vec<InstalledPlugin>,
    plugins_dir: PathBuf,
}

impl PluginManager {
    #[must_use]
    pub fn new(plugins_dir: impl Into<PathBuf>) -> Self {
        Self {
            plugins: Vec::new(),
            plugins_dir: plugins_dir.into(),
        }
    }

    #[must_use]
    pub fn default_dir() -> PathBuf {
        ConfigLoader::default_paths()
            .config_file_path()
            .parent()
            .map_or_else(
                || PathBuf::from(".terminalos/plugins"),
                |parent| parent.join("plugins"),
            )
    }

    pub fn load_all(&mut self) -> Result<usize> {
        self.plugins.clear();
        let mut loader = PluginLoader::new();
        let count = loader.scan(&self.plugins_dir)?;

        for (install_dir, manifest) in loader.manifests() {
            let library_path = PluginLoader::resolve_library_path(install_dir, &manifest.entry);
            let dynamic = if library_path.exists() {
                match DynamicPlugin::load(&library_path) {
                    Ok(plugin) => Some(plugin),
                    Err(err) => {
                        tracing::warn!(
                            plugin = %manifest.info.name,
                            error = %err,
                            "failed to load plugin library"
                        );
                        None
                    }
                }
            } else {
                tracing::warn!(
                    plugin = %manifest.info.name,
                    path = %library_path.display(),
                    "plugin library not found"
                );
                None
            };

            self.plugins.push(InstalledPlugin {
                manifest: manifest.clone(),
                install_dir: install_dir.clone(),
                dynamic,
            });
        }

        Ok(count)
    }

    #[must_use]
    pub fn plugins(&self) -> &[InstalledPlugin] {
        &self.plugins
    }

    #[must_use]
    pub fn plugins_dir(&self) -> &Path {
        &self.plugins_dir
    }

    pub fn get(&self, name: &str) -> Option<&InstalledPlugin> {
        self.plugins
            .iter()
            .find(|plugin| plugin.manifest.info.name == name)
    }

    pub fn execute(&self, plugin_name: &str, command: &str, args: &[String]) -> Result<String> {
        let plugin = self
            .get(plugin_name)
            .ok_or_else(|| Error::Plugin(format!("plugin not found: {plugin_name}")))?;
        plugin.execute(command, args)
    }

    #[must_use]
    pub fn loaded_count(&self) -> usize {
        self.plugins.iter().filter(|p| p.dynamic.is_some()).count()
    }
}
