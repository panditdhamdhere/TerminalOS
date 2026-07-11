use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};
use terminalos_shared::Theme;
use terminalos_terminal::ShellSession;

use crate::theme::UiPalette;

/// Renders the terminal tab bar.
pub fn render_tab_bar(area: Rect, buf: &mut Buffer, session: &ShellSession, theme: &Theme) {
    let palette = UiPalette::from(theme);

    let spans: Vec<Span> = session
        .tabs
        .iter()
        .enumerate()
        .flat_map(|(i, tab)| {
            let active = i == session.active_tab;
            let style = if active {
                Style::default()
                    .fg(palette.background)
                    .bg(palette.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(palette.foreground).bg(palette.sidebar)
            };
            vec![
                Span::styled(format!(" {} ", tab.title), style),
                Span::raw(" "),
            ]
        })
        .collect();

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line).style(Style::default().bg(palette.sidebar));
    Widget::render(paragraph, area, buf);
}
