use std::fs;
use std::path::{Path, PathBuf};

use terminalos_shared::{Error, Result};

use crate::settings::AppConfig;

/// Loads and persists application configuration from TOML files.
#[derive(Debug, Clone)]
pub struct ConfigLoader {
    config_dir: PathBuf,
}

impl ConfigLoader {
    #[must_use]
    pub fn new(config_dir: PathBuf) -> Self {
        Self { config_dir }
    }

    #[must_use]
    pub fn default_paths() -> Self {
        let config_dir = dirs_config_path();
        Self::new(config_dir)
    }

    /// Returns the path to the main config file.
    #[must_use]
    pub fn config_file_path(&self) -> PathBuf {
        self.config_dir.join("config.toml")
    }

    /// Loads configuration from disk, falling back to defaults if missing.
    pub fn load(&self) -> Result<AppConfig> {
        let path = self.config_file_path();
        if !path.exists() {
            return Ok(AppConfig::default());
        }

        let contents = fs::read_to_string(&path)
            .map_err(|e| Error::Config(format!("failed to read {}: {e}", path.display())))?;

        toml::from_str(&contents).map_err(|e| Error::Config(format!("failed to parse config: {e}")))
    }

    /// Saves configuration to disk, creating parent directories as needed.
    pub fn save(&self, config: &AppConfig) -> Result<()> {
        fs::create_dir_all(&self.config_dir)
            .map_err(|e| Error::Config(format!("failed to create config dir: {e}")))?;

        let contents = toml::to_string_pretty(config)
            .map_err(|e| Error::Config(format!("failed to serialize config: {e}")))?;

        fs::write(self.config_file_path(), contents)
            .map_err(|e| Error::Config(format!("failed to write config: {e}")))?;

        Ok(())
    }

    /// Ensures a default config file exists on disk.
    pub fn ensure_default(&self) -> Result<AppConfig> {
        let path = self.config_file_path();
        if path.exists() {
            self.load()
        } else {
            let config = AppConfig::default();
            self.save(&config)?;
            Ok(config)
        }
    }
}

fn dirs_config_path() -> PathBuf {
    if let Some(home) = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")) {
        Path::new(&home).join(".config").join("terminalos")
    } else {
        PathBuf::from(".terminalos")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_default_config() {
        let dir = tempfile::tempdir().expect("tempdir");
        let loader = ConfigLoader::new(dir.path().to_path_buf());
        let config = AppConfig::default();
        loader.save(&config).expect("save");
        let loaded = loader.load().expect("load");
        assert_eq!(loaded.ui.theme, config.ui.theme);
        assert_eq!(
            loaded.layout.sidebar_width_percent,
            config.layout.sidebar_width_percent
        );
    }
}
