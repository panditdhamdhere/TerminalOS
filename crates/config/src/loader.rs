use std::fs;
use std::path::{Path, PathBuf};

use terminalos_shared::{Error, Result};

use crate::keybindings::Keybindings;
use crate::profiles::{apply_profile, ensure_default_profiles, load_profile, profiles_dir_for};
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

    #[must_use]
    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }

    /// Returns the path to the main config file.
    #[must_use]
    pub fn config_file_path(&self) -> PathBuf {
        self.config_dir.join("config.toml")
    }

    /// Returns the path to the optional keybindings override file.
    #[must_use]
    pub fn keybindings_file_path(&self) -> PathBuf {
        self.config_dir.join("keybindings.toml")
    }

    /// Returns the profiles directory path.
    #[must_use]
    pub fn profiles_dir(&self) -> PathBuf {
        profiles_dir_for(&self.config_dir)
    }

    /// Loads configuration from disk, falling back to defaults if missing.
    pub fn load(&self) -> Result<AppConfig> {
        let path = self.config_file_path();
        let mut config = if !path.exists() {
            AppConfig::default()
        } else {
            let contents = fs::read_to_string(&path)
                .map_err(|e| Error::Config(format!("failed to read {}: {e}", path.display())))?;
            toml::from_str(&contents)
                .map_err(|e| Error::Config(format!("failed to parse config: {e}")))?
        };

        if let Ok(bindings) = self.load_keybindings_override() {
            config.keybindings = bindings;
        }

        self.resolve_active_profile(&mut config)?;
        Ok(config)
    }

    /// Loads configuration and applies an explicit profile override.
    pub fn load_with_profile(&self, profile_name: &str) -> Result<AppConfig> {
        let mut config = self.load()?;
        config.active_profile = Some(profile_name.to_string());
        self.resolve_active_profile(&mut config)?;
        Ok(config)
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

    /// Ensures default config, profiles, and keybindings exist on disk.
    pub fn ensure_default(&self) -> Result<AppConfig> {
        ensure_default_profiles(&self.profiles_dir())?;

        let path = self.config_file_path();
        if path.exists() {
            self.load()
        } else {
            let config = AppConfig::default();
            self.save(&config)?;
            self.write_default_keybindings()?;
            Ok(config)
        }
    }

    /// Lists available profile names.
    pub fn list_profiles(&self) -> Result<Vec<String>> {
        crate::profiles::list_profiles(&self.profiles_dir())
    }

    /// Returns a loaded profile by name.
    pub fn load_profile(&self, name: &str) -> Result<crate::profiles::Profile> {
        load_profile(&self.profiles_dir(), name)
    }

    /// Sets the active profile in config.toml and returns the resolved config.
    pub fn set_active_profile(&self, name: &str) -> Result<AppConfig> {
        let mut config = self.load()?;
        config.active_profile = Some(name.to_string());
        self.save(&config)?;
        self.resolve_active_profile(&mut config)?;
        Ok(config)
    }

    fn load_keybindings_override(&self) -> Result<Keybindings> {
        let path = self.keybindings_file_path();
        if !path.exists() {
            return Err(Error::Config("no keybindings override".to_string()));
        }

        let contents = fs::read_to_string(&path)
            .map_err(|e| Error::Config(format!("failed to read keybindings: {e}")))?;
        toml::from_str(&contents)
            .map_err(|e| Error::Config(format!("failed to parse keybindings: {e}")))
    }

    fn write_default_keybindings(&self) -> Result<()> {
        let path = self.keybindings_file_path();
        if path.exists() {
            return Ok(());
        }
        let bindings = Keybindings::default();
        let contents = toml::to_string_pretty(&bindings)
            .map_err(|e| Error::Config(format!("serialize keybindings: {e}")))?;
        fs::write(self.keybindings_file_path(), contents)
            .map_err(|e| Error::Config(format!("write keybindings: {e}")))?;
        Ok(())
    }

    fn resolve_active_profile(&self, config: &mut AppConfig) -> Result<()> {
        let Some(name) = config.active_profile.clone() else {
            return Ok(());
        };

        ensure_default_profiles(&self.profiles_dir())?;
        let profile = load_profile(&self.profiles_dir(), &name)?;
        *config = apply_profile(config.clone(), &profile);
        Ok(())
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

    #[test]
    fn applies_minimal_profile() {
        let dir = tempfile::tempdir().expect("tempdir");
        let loader = ConfigLoader::new(dir.path().to_path_buf());
        ensure_default_profiles(&loader.profiles_dir()).expect("profiles");
        let config = loader.load_with_profile("minimal").expect("load");
        assert!(!config.ui.show_sidebar);
        assert!(!config.ui.show_chat);
    }
}
