use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};
use terminalos_shared::{PaneId, Theme};
use terminalos_terminal::{Area, ShellManager};

use crate::event::FocusedPane;
use crate::theme::UiPalette;

/// Renders all panes in the active terminal tab.
pub fn render_terminal_pane(
    area: Rect,
    buf: &mut Buffer,
    shell: &ShellManager,
    theme: &Theme,
    focused: FocusedPane,
) {
    let pane_rects = shell.active_pane_rects(Area {
        x: area.x,
        y: area.y,
        width: area.width,
        height: area.height,
    });
    let active_pane = shell.active_pane_id();
    let tab = shell.session().active_tab();

    for (pane_id, pane_area) in pane_rects {
        let rect = Rect {
            x: pane_area.x,
            y: pane_area.y,
            width: pane_area.width,
            height: pane_area.height,
        };
        let is_active = pane_id == active_pane;
        let title = if shell.is_search_mode() && is_active {
            format!("  {} — search: {}_  ", tab.title, shell.search_input())
        } else if tab.pane_count() > 1 {
            format!("  {} [{}]  ", tab.title, pane_label(pane_id, active_pane))
        } else {
            format!("  {}  ", tab.title)
        };

        render_single_pane(
            rect,
            buf,
            shell,
            pane_id,
            &title,
            theme,
            focused == FocusedPane::Terminal && is_active,
        );
    }
}

fn pane_label(pane_id: PaneId, active_pane: PaneId) -> String {
    if pane_id == active_pane {
        "active".to_string()
    } else {
        "pane".to_string()
    }
}

fn render_single_pane(
    area: Rect,
    buf: &mut Buffer,
    shell: &ShellManager,
    pane_id: PaneId,
    title: &str,
    theme: &Theme,
    focused: bool,
) {
    let palette = UiPalette::from(theme);
    let border_style = if focused {
        Style::default()
            .fg(palette.accent)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(palette.border)
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
        .emulator(pane_id)
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
