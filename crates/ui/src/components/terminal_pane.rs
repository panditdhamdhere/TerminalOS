use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};
use terminalos_shared::Theme;
use terminalos_terminal::ShellSession;

use crate::event::FocusedPane;
use crate::theme::UiPalette;

/// Renders the main terminal pane with output and input line.
pub fn render_terminal_pane(
    area: Rect,
    buf: &mut Buffer,
    session: &ShellSession,
    theme: &Theme,
    focused: FocusedPane,
) {
    let palette = UiPalette::from(theme);
    let tab = session.active_tab();

    let border_style = if focused == FocusedPane::Terminal {
        Style::default()
            .fg(palette.accent)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(palette.border)
    };

    let block = Block::default()
        .title(format!("  {}  ", tab.title))
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

    let output_height = inner.height.saturating_sub(1) as usize;
    let lines: Vec<Line> = tab
        .buffer
        .lines()
        .iter()
        .rev()
        .take(output_height)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .map(|l| Line::from(Span::raw(l.as_str())))
        .collect();

    let output_area = Rect {
        height: inner.height.saturating_sub(1),
        ..inner
    };
    let output = Paragraph::new(lines).style(Style::default().fg(palette.foreground));
    Widget::render(output, output_area, buf);

    let prompt = format!("{} $ {}", tab.cwd, tab.input);
    let input_area = Rect {
        y: inner.y + inner.height.saturating_sub(1),
        height: 1,
        ..inner
    };
    let input = Paragraph::new(Line::from(vec![
        Span::styled(
            prompt,
            Style::default()
                .fg(palette.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "▌",
            Style::default()
                .fg(palette.foreground)
                .add_modifier(Modifier::SLOW_BLINK),
        ),
    ]));
    Widget::render(input, input_area, buf);
}
