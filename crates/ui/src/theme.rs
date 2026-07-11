use ratatui::style::Color;
use terminalos_shared::Theme;

/// Converts a hex color string to a Ratatui Color.
#[must_use]
pub fn hex_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return Color::White;
    }

    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255);
    Color::Rgb(r, g, b)
}

/// Palette derived from the active theme.
#[derive(Debug, Clone)]
pub struct UiPalette {
    pub background: Color,
    pub foreground: Color,
    pub accent: Color,
    pub border: Color,
    pub sidebar: Color,
    pub status_bar: Color,
    pub error: Color,
    pub success: Color,
    pub warning: Color,
    pub muted: Color,
}

impl From<&Theme> for UiPalette {
    fn from(theme: &Theme) -> Self {
        Self {
            background: hex_color(&theme.background),
            foreground: hex_color(&theme.foreground),
            accent: hex_color(&theme.accent),
            border: hex_color(&theme.border),
            sidebar: hex_color(&theme.sidebar),
            status_bar: hex_color(&theme.status_bar),
            error: hex_color(&theme.error),
            success: hex_color(&theme.success),
            warning: hex_color(&theme.warning),
            muted: Color::DarkGray,
        }
    }
}
