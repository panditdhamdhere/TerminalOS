use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};
use terminalos_shared::{LogEntry, LogLevel, Theme};

use crate::event::FocusedPane;
use crate::theme::UiPalette;

/// Renders the bottom application log panel.
pub fn render_logs_pane(
    area: Rect,
    buf: &mut Buffer,
    logs: &[LogEntry],
    theme: &Theme,
    focused: FocusedPane,
    scroll: usize,
) {
    let palette = UiPalette::from(theme);
    let border_style = if focused == FocusedPane::Logs {
        Style::default()
            .fg(palette.accent)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(palette.border)
    };

    let block = Block::default()
        .title("  Logs  ")
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(
            Style::default()
                .bg(palette.background)
                .fg(palette.foreground),
        );

    let inner = block.inner(area);
    block.render(area, buf);

    let visible: Vec<Line> = logs
        .iter()
        .rev()
        .skip(scroll)
        .take(inner.height as usize)
        .map(|entry| {
            let level_color = match entry.level {
                LogLevel::Error => palette.error,
                LogLevel::Warn => palette.warning,
                LogLevel::Info => palette.accent,
                LogLevel::Debug | LogLevel::Trace => palette.muted,
            };
            Line::from(vec![
                Span::styled(
                    format!("{:>5} ", format!("{:?}", entry.level).to_lowercase()),
                    Style::default().fg(level_color),
                ),
                Span::raw(entry.message.as_str()),
            ])
        })
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();

    let paragraph = Paragraph::new(visible).style(Style::default().fg(palette.foreground));
    Widget::render(paragraph, inner, buf);
}
