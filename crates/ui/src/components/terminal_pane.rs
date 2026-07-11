use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};
use terminalos_shared::Theme;
use terminalos_terminal::ShellManager;

use crate::event::FocusedPane;
use crate::theme::UiPalette;

/// Renders the PTY-backed terminal pane with ANSI colors.
pub fn render_terminal_pane(
    area: Rect,
    buf: &mut Buffer,
    shell: &ShellManager,
    theme: &Theme,
    focused: FocusedPane,
) {
    let palette = UiPalette::from(theme);
    let tab = shell.session().active_tab();

    let border_style = if focused == FocusedPane::Terminal {
        Style::default()
            .fg(palette.accent)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(palette.border)
    };

    let title = if shell.is_search_mode() {
        format!("  {} — search: {}_  ", tab.title, shell.search_input())
    } else {
        format!("  {}  ", tab.title)
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(
            Style::default()
                .bg(palette.background)
                .fg(palette.foreground),
        );

    let inner = block.inner(area);
    block.render(area, buf);

    if inner.height == 0 {
        return;
    }

    let width = inner.width as usize;
    let height = inner.height as usize;

    let lines: Vec<Line> = shell
        .active_emulator()
        .map(|emu| emu.render_rows(height, width))
        .unwrap_or_default()
        .into_iter()
        .map(|spans| {
            Line::from(
                spans
                    .iter()
                    .map(|s| {
                        let mut style = Style::default();
                        if let Some((r, g, b)) = s.fg {
                            style = style.fg(Color::Rgb(r, g, b));
                        } else if let Some(idx) = s.fg_index {
                            style = style.fg(Color::Indexed(idx));
                        }
                        if let Some((r, g, b)) = s.bg {
                            style = style.bg(Color::Rgb(r, g, b));
                        } else if let Some(idx) = s.bg_index {
                            style = style.bg(Color::Indexed(idx));
                        }
                        if s.bold {
                            style = style.add_modifier(Modifier::BOLD);
                        }
                        if s.italic {
                            style = style.add_modifier(Modifier::ITALIC);
                        }
                        if s.underline {
                            style = style.add_modifier(Modifier::UNDERLINED);
                        }
                        Span::styled(s.text.clone(), style)
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .collect();

    let paragraph = Paragraph::new(lines).style(Style::default().bg(palette.background));
    Widget::render(paragraph, inner, buf);
}
