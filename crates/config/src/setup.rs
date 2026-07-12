use std::io::{self, IsTerminal, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

use terminalos_shared::{Error, Result};

use crate::loader::ConfigLoader;
use crate::settings::{AppConfig, ProviderConfig, ProviderType};

/// First-run provider choices offered by the setup wizard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetupChoice {
    Ollama,
    Groq,
    OpenAi,
    Skip,
}

/// Loads project `.env` then `~/.config/terminalos/.env`.
pub fn load_env_files(loader: &ConfigLoader) {
    let _ = dotenvy::dotenv();
    let _ = dotenvy::from_path(loader.user_env_file_path());
}

/// Returns true when first-run setup has been completed.
pub fn is_setup_complete(loader: &ConfigLoader) -> bool {
    loader.setup_complete_path().exists()
}

/// Marks first-run setup as complete.
pub fn mark_setup_complete(loader: &ConfigLoader) -> Result<()> {
    std::fs::create_dir_all(loader.config_dir())
        .map_err(|e| Error::Config(format!("create config dir: {e}")))?;
    std::fs::write(loader.setup_complete_path(), "ok")
        .map_err(|e| Error::Config(format!("write setup marker: {e}")))?;
    Ok(())
}

/// Auto-enables a provider when its API key env var is set or Ollama is reachable.
#[must_use]
pub fn auto_configure_providers(config: &mut AppConfig) -> bool {
    let mut changed = false;

    if env_var_set("GROQ_API_KEY") {
        changed |= enable_provider(config, "groq", true);
    } else if env_var_set("OPENAI_API_KEY") {
        changed |= enable_provider(config, "openai", true);
    } else if ollama_reachable() {
        changed |= enable_provider(config, "ollama", true);
    }

    changed
}

/// Returns true when no enabled provider can serve requests.
#[must_use]
pub fn needs_interactive_setup(config: &AppConfig) -> bool {
    !config
        .providers
        .iter()
        .any(|provider| provider.enabled && provider_is_ready(provider))
}

/// Runs an interactive stdin setup wizard and persists config/env changes.
pub fn run_interactive_setup(loader: &ConfigLoader, mut config: AppConfig) -> Result<AppConfig> {
    println!();
    println!("Welcome to TerminalOS!");
    println!("No AI provider is configured yet. Choose one to power the assistant:");
    println!();
    println!("  1) Ollama (local, free — requires `ollama serve`)");
    println!("  2) Groq (cloud, fast — requires GROQ_API_KEY)");
    println!("  3) OpenAI (cloud — requires OPENAI_API_KEY)");
    println!("  4) Skip for now");
    println!();
    print!("Enter choice [1-4]: ");
    io::stdout()
        .flush()
        .map_err(|e| Error::Config(format!("stdout flush: {e}")))?;

    let mut line = String::new();
    io::stdin()
        .read_line(&mut line)
        .map_err(|e| Error::Config(format!("read choice: {e}")))?;

    let choice = match line.trim() {
        "1" => SetupChoice::Ollama,
        "2" => SetupChoice::Groq,
        "3" => SetupChoice::OpenAi,
        "4" => SetupChoice::Skip,
        _ => SetupChoice::Ollama,
    };

    apply_setup_choice(loader, &mut config, choice)?;
    mark_setup_complete(loader)?;
    loader.save(&config)?;
    Ok(config)
}

/// Applies a setup choice to config and optional user env file.
pub fn apply_setup_choice(
    loader: &ConfigLoader,
    config: &mut AppConfig,
    choice: SetupChoice,
) -> Result<()> {
    match choice {
        SetupChoice::Ollama => {
            enable_provider(config, "ollama", true);
            println!("Configured Ollama. Start it with: ollama serve");
        }
        SetupChoice::Groq => {
            let key = prompt_api_key("Groq API key (gsk_...)")?;
            write_user_env_var(loader, "GROQ_API_KEY", &key)?;
            unsafe { std::env::set_var("GROQ_API_KEY", &key) };
            enable_provider(config, "groq", true);
            println!("Configured Groq as the default AI provider.");
        }
        SetupChoice::OpenAi => {
            let key = prompt_api_key("OpenAI API key (sk-...)")?;
            write_user_env_var(loader, "OPENAI_API_KEY", &key)?;
            unsafe { std::env::set_var("OPENAI_API_KEY", &key) };
            enable_provider(config, "openai", true);
            println!("Configured OpenAI as the default AI provider.");
        }
        SetupChoice::Skip => {
            println!("Skipping setup. Enable a provider later in ~/.config/terminalos/config.toml");
        }
    }
    Ok(())
}

/// Enables a provider by name and optionally disables all others.
pub fn enable_provider(config: &mut AppConfig, name: &str, disable_others: bool) -> bool {
    let mut changed = false;
    let mut found = false;

    for provider in &mut config.providers {
        if provider.name == name {
            if !provider.enabled {
                provider.enabled = true;
                changed = true;
            }
            found = true;
        } else if disable_others && provider.enabled {
            provider.enabled = false;
            changed = true;
        }
    }

    if found && config.default_provider.as_deref() != Some(name) {
        config.default_provider = Some(name.to_string());
        changed = true;
    }

    changed
}

/// Sets the active default provider and persists config.
pub fn set_default_provider(loader: &ConfigLoader, name: &str) -> Result<AppConfig> {
    let mut config = loader.load()?;
    if !config.providers.iter().any(|p| p.name == name) {
        return Err(Error::Config(format!("unknown provider: {name}")));
    }
    enable_provider(&mut config, name, false);
    loader.save(&config)?;
    Ok(config)
}

/// Lists provider names with readiness hints.
#[must_use]
pub fn provider_statuses(config: &AppConfig) -> Vec<ProviderStatus> {
    config
        .providers
        .iter()
        .map(|provider| ProviderStatus {
            name: provider.name.clone(),
            enabled: provider.enabled,
            ready: provider_is_ready(provider),
            is_default: config.default_provider.as_deref() == Some(provider.name.as_str()),
            model: provider.model.clone(),
        })
        .collect()
}

/// Provider readiness summary for CLI and setup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderStatus {
    pub name: String,
    pub enabled: bool,
    pub ready: bool,
    pub is_default: bool,
    pub model: String,
}

#[must_use]
pub fn provider_is_ready(provider: &ProviderConfig) -> bool {
    if !provider.enabled {
        return false;
    }

    match provider.provider_type {
        ProviderType::Ollama => ollama_reachable(),
        ProviderType::Groq
        | ProviderType::OpenAi
        | ProviderType::Anthropic
        | ProviderType::OpenRouter
        | ProviderType::Gemini
        | ProviderType::DeepSeek => env_var_set(&provider.api_key_env),
    }
}

#[must_use]
pub fn env_var_set(name: &str) -> bool {
    std::env::var(name)
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
}

#[must_use]
pub fn ollama_reachable() -> bool {
    let Ok(addr) = "127.0.0.1:11434".parse::<SocketAddr>() else {
        return false;
    };
    TcpStream::connect_timeout(&addr, Duration::from_millis(400)).is_ok()
}

fn prompt_api_key(label: &str) -> Result<String> {
    print!("{label}: ");
    io::stdout()
        .flush()
        .map_err(|e| Error::Config(format!("stdout flush: {e}")))?;
    let mut key = String::new();
    io::stdin()
        .read_line(&mut key)
        .map_err(|e| Error::Config(format!("read api key: {e}")))?;
    let key = key.trim().to_string();
    if key.is_empty() {
        return Err(Error::Config("API key cannot be empty".to_string()));
    }
    Ok(key)
}

fn write_user_env_var(loader: &ConfigLoader, key: &str, value: &str) -> Result<()> {
    std::fs::create_dir_all(loader.config_dir())
        .map_err(|e| Error::Config(format!("create config dir: {e}")))?;

    let path = loader.user_env_file_path();
    let mut lines = if path.exists() {
        std::fs::read_to_string(&path)
            .map_err(|e| Error::Config(format!("read user env: {e}")))?
            .lines()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    } else {
        vec!["# TerminalOS user environment".to_string()]
    };

    if let Some(index) = lines
        .iter()
        .position(|line| line.starts_with(&format!("{key}=")))
    {
        lines[index] = format!("{key}={value}");
    } else {
        lines.push(format!("{key}={value}"));
    }

    let contents = format!("{}\n", lines.join("\n"));
    std::fs::write(&path, contents).map_err(|e| Error::Config(format!("write user env: {e}")))?;
    Ok(())
}

impl ConfigLoader {
    /// Path to the user-level `.env` file for API keys.
    #[must_use]
    pub fn user_env_file_path(&self) -> std::path::PathBuf {
        self.config_dir().join(".env")
    }

    /// Path to the first-run setup completion marker.
    #[must_use]
    pub fn setup_complete_path(&self) -> std::path::PathBuf {
        self.config_dir().join(".setup_complete")
    }

    /// Ensures config exists, auto-configures providers, and runs setup if needed.
    pub fn ensure_ready(&self, skip_setup: bool) -> Result<AppConfig> {
        let mut config = self.ensure_default()?;
        load_env_files(self);
        let changed = auto_configure_providers(&mut config);
        if changed {
            self.save(&config)?;
        }

        if skip_setup || is_setup_complete(self) || !needs_interactive_setup(&config) {
            return Ok(config);
        }

        if io::stdin().is_terminal() {
            config = run_interactive_setup(self, config)?;
        }

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::AppConfig;

    #[test]
    fn enables_groq_when_key_present() {
        unsafe { std::env::set_var("GROQ_API_KEY", "test-key") };
        let mut config = AppConfig::default();
        assert!(auto_configure_providers(&mut config));
        assert_eq!(config.default_provider.as_deref(), Some("groq"));
        let groq = config
            .providers
            .iter()
            .find(|p| p.name == "groq")
            .expect("groq");
        assert!(groq.enabled);
        unsafe { std::env::remove_var("GROQ_API_KEY") };
    }

    #[test]
    fn enable_provider_sets_default() {
        let mut config = AppConfig::default();
        assert!(enable_provider(&mut config, "groq", true));
        assert_eq!(config.default_provider.as_deref(), Some("groq"));
        assert!(
            config
                .providers
                .iter()
                .find(|p| p.name == "groq")
                .unwrap()
                .enabled
        );
        assert!(
            !config
                .providers
                .iter()
                .find(|p| p.name == "ollama")
                .unwrap()
                .enabled
        );
    }

    #[test]
    fn write_user_env_updates_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let loader = ConfigLoader::new(dir.path().to_path_buf());
        write_user_env_var(&loader, "GROQ_API_KEY", "gsk_test").expect("write");
        let content = std::fs::read_to_string(loader.user_env_file_path()).expect("read");
        assert!(content.contains("GROQ_API_KEY=gsk_test"));
    }
}
