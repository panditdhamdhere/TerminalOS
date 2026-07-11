use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget, Wrap};
use terminalos_shared::Theme;

use crate::event::FocusedPane;
use crate::theme::UiPalette;

/// A message in the AI chat panel.
#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// Renders the AI chat panel with conversation history and input.
pub fn render_chat_pane(
    area: Rect,
    buf: &mut Buffer,
    messages: &[ChatMessage],
    input: &str,
    theme: &Theme,
    focused: FocusedPane,
    scroll: usize,
) {
    let palette = UiPalette::from(theme);
    let border_style = if focused == FocusedPane::Chat {
        Style::default()
            .fg(palette.accent)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(palette.border)
    };

    let block = Block::default()
        .title("  AI Assistant  ")
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(
            Style::default()
                .bg(palette.background)
                .fg(palette.foreground),
        );

    let inner = block.inner(area);
    block.render(area, buf);

    if inner.height < 2 {
        return;
    }

    let messages_area = Rect {
        height: inner.height.saturating_sub(1),
        ..inner
    };

    let mut lines: Vec<Line> = Vec::new();
    if messages.is_empty() {
        lines.push(Line::from(Span::styled(
            "Ask anything about your codebase...",
            Style::default().fg(palette.muted),
        )));
    } else {
        for msg in messages.iter().skip(scroll) {
            let role_style = Style::default()
                .fg(if msg.role == "user" {
                    palette.accent
                } else {
                    palette.success
                })
                .add_modifier(Modifier::BOLD);
            lines.push(Line::from(Span::styled(
                format!("{}:", msg.role),
                role_style,
            )));
            lines.push(Line::from(Span::raw(msg.content.as_str())));
            lines.push(Line::from(""));
        }
    }

    let paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(palette.foreground));
    Widget::render(paragraph, messages_area, buf);

    let input_area = Rect {
        y: inner.y + inner.height.saturating_sub(1),
        height: 1,
        ..inner
    };
    let input_line = Paragraph::new(Line::from(vec![
        Span::styled("› ", Style::default().fg(palette.accent)),
        Span::raw(input),
        Span::styled("▌", Style::default().add_modifier(Modifier::SLOW_BLINK)),
    ]));
    Widget::render(input_line, input_area, buf);
}
