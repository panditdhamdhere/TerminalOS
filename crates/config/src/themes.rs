use terminalos_shared::{Theme, ThemeMode};

/// Named theme preset definition.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct ThemePreset {
    pub name: String,
    pub mode: ThemeMode,
    pub background: String,
    pub foreground: String,
    pub accent: String,
    pub border: String,
    pub sidebar: String,
    pub status_bar: String,
    pub error: String,
    pub success: String,
    pub warning: String,
}

impl ThemePreset {
    #[must_use]
    pub fn to_theme(&self) -> Theme {
        Theme {
            mode: self.mode,
            name: self.name.clone(),
            background: self.background.clone(),
            foreground: self.foreground.clone(),
            accent: self.accent.clone(),
            border: self.border.clone(),
            sidebar: self.sidebar.clone(),
            status_bar: self.status_bar.clone(),
            error: self.error.clone(),
            success: self.success.clone(),
            warning: self.warning.clone(),
        }
    }
}

/// Returns a built-in theme preset by name.
#[must_use]
pub fn builtin_preset(name: &str) -> Option<ThemePreset> {
    match name {
        "dark" => Some(preset_from_theme(Theme::dark())),
        "light" => Some(preset_from_theme(Theme::light())),
        "dracula" => Some(ThemePreset {
            name: "Dracula".to_string(),
            mode: ThemeMode::Dark,
            background: "#282a36".to_string(),
            foreground: "#f8f8f2".to_string(),
            accent: "#bd93f9".to_string(),
            border: "#44475a".to_string(),
            sidebar: "#21222c".to_string(),
            status_bar: "#343746".to_string(),
            error: "#ff5555".to_string(),
            success: "#50fa7b".to_string(),
            warning: "#f1fa8c".to_string(),
        }),
        "nord" => Some(ThemePreset {
            name: "Nord".to_string(),
            mode: ThemeMode::Dark,
            background: "#2e3440".to_string(),
            foreground: "#eceff4".to_string(),
            accent: "#88c0d0".to_string(),
            border: "#4c566a".to_string(),
            sidebar: "#3b4252".to_string(),
            status_bar: "#434c5e".to_string(),
            error: "#bf616a".to_string(),
            success: "#a3be8c".to_string(),
            warning: "#ebcb8b".to_string(),
        }),
        "solarized-dark" => Some(ThemePreset {
            name: "Solarized Dark".to_string(),
            mode: ThemeMode::Dark,
            background: "#002b36".to_string(),
            foreground: "#839496".to_string(),
            accent: "#268bd2".to_string(),
            border: "#073642".to_string(),
            sidebar: "#073642".to_string(),
            status_bar: "#073642".to_string(),
            error: "#dc322f".to_string(),
            success: "#859900".to_string(),
            warning: "#b58900".to_string(),
        }),
        _ => None,
    }
}

/// Lists all built-in theme preset names.
#[must_use]
pub fn builtin_preset_names() -> &'static [&'static str] {
    &["dark", "light", "dracula", "nord", "solarized-dark"]
}

/// Resolves the active theme from mode and optional preset name.
#[must_use]
pub fn resolve_theme(mode: ThemeMode, preset: Option<&str>) -> Theme {
    if let Some(name) = preset {
        if let Some(found) = builtin_preset(name) {
            return found.to_theme();
        }
    }

    match mode {
        ThemeMode::Dark => Theme::dark(),
        ThemeMode::Light => Theme::light(),
    }
}

fn preset_from_theme(theme: Theme) -> ThemePreset {
    ThemePreset {
        name: theme.name,
        mode: theme.mode,
        background: theme.background,
        foreground: theme.foreground,
        accent: theme.accent,
        border: theme.border,
        sidebar: theme.sidebar,
        status_bar: theme.status_bar,
        error: theme.error,
        success: theme.success,
        warning: theme.warning,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_dracula_preset() {
        let theme = resolve_theme(ThemeMode::Dark, Some("dracula"));
        assert_eq!(theme.name, "Dracula");
    }
}
