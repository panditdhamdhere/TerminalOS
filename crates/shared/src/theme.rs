use serde::{Deserialize, Serialize};

/// Light or dark theme mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ThemeMode {
    #[default]
    Dark,
    Light,
}

/// Color palette for the TerminalOS UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub mode: ThemeMode,
    pub name: String,
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

impl Theme {
    #[must_use]
    pub fn dark() -> Self {
        Self {
            mode: ThemeMode::Dark,
            name: "TerminalOS Dark".to_string(),
            background: "#0d1117".to_string(),
            foreground: "#e6edf3".to_string(),
            accent: "#58a6ff".to_string(),
            border: "#30363d".to_string(),
            sidebar: "#161b22".to_string(),
            status_bar: "#21262d".to_string(),
            error: "#f85149".to_string(),
            success: "#3fb950".to_string(),
            warning: "#d29922".to_string(),
        }
    }

    #[must_use]
    pub fn light() -> Self {
        Self {
            mode: ThemeMode::Light,
            name: "TerminalOS Light".to_string(),
            background: "#ffffff".to_string(),
            foreground: "#1f2328".to_string(),
            accent: "#0969da".to_string(),
            border: "#d0d7de".to_string(),
            sidebar: "#f6f8fa".to_string(),
            status_bar: "#eaeef2".to_string(),
            error: "#cf222e".to_string(),
            success: "#1a7f37".to_string(),
            warning: "#9a6700".to_string(),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}
