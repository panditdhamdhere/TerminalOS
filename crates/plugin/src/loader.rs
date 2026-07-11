use std::path::{Path, PathBuf};

use terminalos_shared::{Error, Result};

use crate::api::PluginManifest;
use crate::dynamic::library_filename;

/// Loads plugin manifests from a directory.
#[derive(Debug, Default)]
pub struct PluginLoader {
    manifests: Vec<(PathBuf, PluginManifest)>,
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
            if !path.is_dir() {
                continue;
            }

            let manifest_path = path.join("plugin.toml");
            if manifest_path.is_file() {
                self.load_manifest(&manifest_path, path)?;
            }
        }

        Ok(self.manifests.len())
    }

    fn load_manifest(&mut self, manifest_path: &Path, install_dir: PathBuf) -> Result<()> {
        let content = std::fs::read_to_string(manifest_path)
            .map_err(|e| Error::Plugin(format!("read manifest: {e}")))?;
        let manifest: PluginManifest =
            toml::from_str(&content).map_err(|e| Error::Plugin(format!("parse manifest: {e}")))?;
        if manifest.enabled {
            self.manifests.push((install_dir, manifest));
        }
        Ok(())
    }

    #[must_use]
    pub fn manifests(&self) -> &[(PathBuf, PluginManifest)] {
        &self.manifests
    }

    #[must_use]
    pub fn resolve_library_path(install_dir: &Path, entry: &str) -> PathBuf {
        install_dir.join(library_filename(entry))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::PluginInfo;

    #[test]
    fn scans_subdirectory_manifest() {
        let dir = tempfile::tempdir().expect("tempdir");
        let plugin_dir = dir.path().join("hello");
        std::fs::create_dir_all(&plugin_dir).expect("mkdir");
        let manifest = crate::api::PluginManifest {
            info: PluginInfo {
                name: "hello".to_string(),
                version: "0.1.0".to_string(),
                description: "test".to_string(),
                author: "test".to_string(),
            },
            entry: "terminalos_plugin_hello".to_string(),
            enabled: true,
            commands: Vec::new(),
        };
        let manifest_path = plugin_dir.join("plugin.toml");
        std::fs::write(
            &manifest_path,
            toml::to_string(&manifest).expect("serialize"),
        )
        .expect("write");

        let mut loader = PluginLoader::new();
        let count = loader.scan(dir.path()).expect("scan");
        assert_eq!(count, 1);
        assert_eq!(
            loader.manifests()[0].1.info,
            PluginInfo {
                name: "hello".to_string(),
                version: "0.1.0".to_string(),
                description: "test".to_string(),
                author: "test".to_string(),
            }
        );
    }
}
