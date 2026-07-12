use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Widget};
use terminalos_config::ProviderStatus;
use terminalos_shared::Theme;

use crate::theme::UiPalette;

/// Renders a centered overlay for selecting the active AI provider.
pub fn render_provider_picker(
    area: Rect,
    buf: &mut Buffer,
    providers: &[ProviderStatus],
    selected: usize,
    theme: &Theme,
) {
    let palette = UiPalette::from(theme);
    let width = area.width.clamp(36, 52);
    let height = (providers.len() as u16 + 5)
        .min(area.height.saturating_sub(4))
        .max(8);
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    let popup = Rect {
        x,
        y,
        width,
        height,
    };

    Clear.render(popup, buf);

    let block = Block::default()
        .title("  Switch AI Provider  ")
        .borders(Borders::ALL)
        .border_style(
            Style::default()
                .fg(palette.accent)
                .add_modifier(Modifier::BOLD),
        )
        .style(
            Style::default()
                .bg(palette.background)
                .fg(palette.foreground),
        );

    let inner = block.inner(popup);
    block.render(popup, buf);

    let mut lines = vec![
        Line::from(Span::styled(
            "↑↓ navigate  Enter select  Esc cancel",
            Style::default().fg(palette.muted),
        )),
        Line::from(""),
    ];

    for (index, provider) in providers.iter().enumerate() {
        let marker = if index == selected { "›" } else { " " };
        let status = if provider.is_default {
            "active"
        } else if provider.ready {
            "ready"
        } else if provider.enabled {
            "no key"
        } else {
            "disabled"
        };
        let status_color = if provider.ready {
            palette.success
        } else if provider.enabled {
            palette.warning
        } else {
            palette.muted
        };
        let name_style = if index == selected {
            Style::default()
                .fg(palette.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(palette.foreground)
        };

        lines.push(Line::from(vec![
            Span::styled(format!("{marker} "), name_style),
            Span::styled(format!("{:<10}", provider.name), name_style),
            Span::styled(format!("{:<8}", status), Style::default().fg(status_color)),
            Span::styled(provider.model.clone(), Style::default().fg(palette.muted)),
        ]));
    }

    Widget::render(Paragraph::new(lines), inner, buf);
}
