use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use terminalos_shared::{Error, Result};

use crate::settings::{
    AgentConfig, AppConfig, LayoutConfig, PluginConfig, ProviderConfig, SearchConfig, UiConfig,
    WorkspaceConfig,
};

/// Named configuration profile with optional section overrides.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub ui: Option<UiConfig>,
    #[serde(default)]
    pub layout: Option<LayoutConfig>,
    #[serde(default)]
    pub default_provider: Option<String>,
    #[serde(default)]
    pub providers: Option<Vec<ProviderConfig>>,
    #[serde(default)]
    pub agent: Option<AgentConfig>,
    #[serde(default)]
    pub workspace: Option<WorkspaceConfig>,
    #[serde(default)]
    pub search: Option<SearchConfig>,
    #[serde(default)]
    pub plugins: Option<PluginConfig>,
}

/// Applies profile overrides onto a base configuration.
#[must_use]
pub fn apply_profile(mut base: AppConfig, profile: &Profile) -> AppConfig {
    if let Some(ui) = &profile.ui {
        base.ui = ui.clone();
    }
    if let Some(layout) = &profile.layout {
        base.layout = layout.clone();
    }
    if let Some(default_provider) = &profile.default_provider {
        base.default_provider = Some(default_provider.clone());
    }
    if let Some(providers) = &profile.providers {
        base.providers = providers.clone();
    }
    if let Some(agent) = &profile.agent {
        base.agent = agent.clone();
    }
    if let Some(workspace) = &profile.workspace {
        base.workspace = workspace.clone();
    }
    if let Some(search) = &profile.search {
        base.search = search.clone();
    }
    if let Some(plugins) = &profile.plugins {
        base.plugins = plugins.clone();
    }
    base.active_profile = Some(profile.name.clone());
    base
}

/// Loads a profile TOML file from the profiles directory.
pub fn load_profile(profiles_dir: &Path, name: &str) -> Result<Profile> {
    let path = profiles_dir.join(format!("{name}.toml"));
    let content = fs::read_to_string(&path)
        .map_err(|e| Error::Config(format!("failed to read profile {name}: {e}")))?;
    let profile: Profile = toml::from_str(&content)
        .map_err(|e| Error::Config(format!("failed to parse profile {name}: {e}")))?;
    Ok(profile)
}

/// Lists available profile names from the profiles directory.
pub fn list_profiles(profiles_dir: &Path) -> Result<Vec<String>> {
    if !profiles_dir.exists() {
        return Ok(Vec::new());
    }

    let mut names = Vec::new();
    for entry in fs::read_dir(profiles_dir)
        .map_err(|e| Error::Config(format!("read profiles dir: {e}")))?
        .flatten()
    {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "toml") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                names.push(stem.to_string());
            }
        }
    }
    names.sort();
    Ok(names)
}

/// Writes bundled default profiles if they do not already exist.
pub fn ensure_default_profiles(profiles_dir: &Path) -> Result<()> {
    fs::create_dir_all(profiles_dir)
        .map_err(|e| Error::Config(format!("create profiles dir: {e}")))?;

    for (name, contents) in default_profile_files() {
        let path = profiles_dir.join(format!("{name}.toml"));
        if !path.exists() {
            fs::write(&path, contents)
                .map_err(|e| Error::Config(format!("write profile {name}: {e}")))?;
        }
    }
    Ok(())
}

fn default_profile_files() -> [(&'static str, &'static str); 3] {
    [
        (
            "default",
            r#"name = "default"
description = "Balanced layout for everyday development"

[ui]
theme = "dark"
show_sidebar = true
show_chat = true
show_logs = true
animations = true
mouse_enabled = true
"#,
        ),
        (
            "minimal",
            r#"name = "minimal"
description = "Distraction-free terminal with hidden side panels"

[ui]
show_sidebar = false
show_chat = false
show_logs = false
"#,
        ),
        (
            "coding",
            r#"name = "coding"
description = "Coding-focused layout with wider terminal area"

[layout]
sidebar_width_percent = 15
chat_width_percent = 35
logs_height_percent = 12
"#,
        ),
    ]
}

#[must_use]
pub fn profiles_dir_for(config_dir: &Path) -> PathBuf {
    config_dir.join("profiles")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_overrides_ui() {
        let base = AppConfig::default();
        let profile = Profile {
            name: "minimal".to_string(),
            description: "test".to_string(),
            ui: Some(UiConfig {
                show_sidebar: false,
                show_chat: false,
                show_logs: false,
                ..UiConfig::default()
            }),
            layout: None,
            default_provider: None,
            providers: None,
            agent: None,
            workspace: None,
            search: None,
            plugins: None,
        };

        let merged = apply_profile(base, &profile);
        assert!(!merged.ui.show_sidebar);
        assert_eq!(merged.active_profile.as_deref(), Some("minimal"));
    }
}
