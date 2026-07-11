use std::path::Path;

use terminalos_shared::{Error, Result};

use crate::api::PluginManifest;

/// Loads plugin manifests from a directory.
#[derive(Debug, Default)]
pub struct PluginLoader {
    manifests: Vec<PluginManifest>,
}

impl PluginLoader {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn scan(&mut self, plugins_dir: impl AsRef<Path>) -> Result<usize> {
        let dir = plugins_dir.as_ref();
        if !dir.exists() {
            return Ok(0);
        }

        self.manifests.clear();
        let entries =
            std::fs::read_dir(dir).map_err(|e| Error::Plugin(format!("read plugins dir: {e}")))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "toml") {
                let content = std::fs::read_to_string(&path)
                    .map_err(|e| Error::Plugin(format!("read manifest: {e}")))?;
                let manifest: PluginManifest = toml::from_str(&content)
                    .map_err(|e| Error::Plugin(format!("parse manifest: {e}")))?;
                if manifest.enabled {
                    self.manifests.push(manifest);
                }
            }
        }

        Ok(self.manifests.len())
    }

    #[must_use]
    pub fn manifests(&self) -> &[PluginManifest] {
        &self.manifests
    }
}
