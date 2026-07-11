use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use terminalos_shared::{Error, Result};

use crate::dynamic::library_filename;

/// A plugin listing in the marketplace catalog.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MarketplaceEntry {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub commands: Vec<String>,
    pub source: String,
}

/// Marketplace catalog containing installable plugins.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MarketplaceCatalog {
    pub plugins: Vec<MarketplaceEntry>,
}

/// Reads the bundled or configured plugin marketplace catalog.
#[derive(Debug, Clone)]
pub struct PluginMarketplace {
    catalog: MarketplaceCatalog,
}

impl PluginMarketplace {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let content = fs::read_to_string(path.as_ref())
            .map_err(|e| Error::Plugin(format!("read marketplace catalog: {e}")))?;
        let catalog: MarketplaceCatalog = serde_json::from_str(&content)
            .map_err(|e| Error::Plugin(format!("parse marketplace catalog: {e}")))?;
        Ok(Self { catalog })
    }

    #[must_use]
    pub fn bundled() -> Self {
        let catalog: MarketplaceCatalog =
            serde_json::from_str(include_str!("../assets/plugin-marketplace.json"))
                .unwrap_or_default();
        Self { catalog }
    }

    #[must_use]
    pub fn entries(&self) -> &[MarketplaceEntry] {
        &self.catalog.plugins
    }

    pub fn find(&self, name: &str) -> Option<&MarketplaceEntry> {
        self.catalog.plugins.iter().find(|entry| entry.name == name)
    }
}

/// Installs plugins from local marketplace sources into the plugins directory.
#[derive(Debug)]
pub struct PluginInstaller {
    plugins_dir: PathBuf,
}

impl PluginInstaller {
    #[must_use]
    pub fn new(plugins_dir: impl Into<PathBuf>) -> Self {
        Self {
            plugins_dir: plugins_dir.into(),
        }
    }

    pub fn install_from_source(
        &self,
        entry: &MarketplaceEntry,
        source_root: impl AsRef<Path>,
    ) -> Result<PathBuf> {
        let source_root = source_root.as_ref();
        let source_dir = source_root.join(&entry.source);
        if !source_dir.is_dir() {
            return Err(Error::Plugin(format!(
                "plugin source not found: {}",
                source_dir.display()
            )));
        }

        let manifest_path = source_dir.join("plugin.toml");
        if !manifest_path.is_file() {
            return Err(Error::Plugin(format!(
                "plugin manifest missing: {}",
                manifest_path.display()
            )));
        }

        let install_dir = self.plugins_dir.join(&entry.name);
        if install_dir.exists() {
            fs::remove_dir_all(&install_dir)
                .map_err(|e| Error::Plugin(format!("replace plugin dir: {e}")))?;
        }
        fs::create_dir_all(&install_dir)
            .map_err(|e| Error::Plugin(format!("create plugin dir: {e}")))?;

        fs::copy(&manifest_path, install_dir.join("plugin.toml"))
            .map_err(|e| Error::Plugin(format!("copy manifest: {e}")))?;

        self.copy_built_library(&source_dir, &install_dir, &entry.name)?;
        Ok(install_dir)
    }

    fn copy_built_library(
        &self,
        source_dir: &Path,
        install_dir: &Path,
        plugin_name: &str,
    ) -> Result<()> {
        let entry = format!("terminalos_plugin_{plugin_name}");
        let filename = library_filename(&entry);

        let candidates = [
            source_dir.join(&filename),
            workspace_target_path(&filename),
            workspace_target_path(&format!("deps/{filename}")),
        ];

        for candidate in candidates {
            if candidate.is_file() {
                fs::copy(&candidate, install_dir.join(&filename))
                    .map_err(|e| Error::Plugin(format!("copy plugin library: {e}")))?;
                return Ok(());
            }
        }

        Err(Error::Plugin(format!(
            "plugin library not built; run `cargo build -p terminalos-plugin-{plugin_name}` first"
        )))
    }
}

fn workspace_target_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/debug")
        .join(relative)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_catalog_contains_hello() {
        let market = PluginMarketplace::bundled();
        assert!(market.find("hello").is_some());
    }
}
