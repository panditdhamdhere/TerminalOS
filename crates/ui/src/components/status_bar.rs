use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Widget};
use terminalos_shared::Theme;
use terminalos_terminal::ShellSession;

use crate::event::FocusedPane;
use crate::theme::UiPalette;

/// Renders the bottom status bar with workspace and focus info.
pub fn render_status_bar(
    area: Rect,
    buf: &mut Buffer,
    session: &ShellSession,
    workspace_name: &str,
    branch: Option<&str>,
    focus: FocusedPane,
    theme: &Theme,
) {
    let palette = UiPalette::from(theme);
    let block = Block::default().style(
        Style::default()
            .bg(palette.status_bar)
            .fg(palette.foreground),
    );
    let inner = block.inner(area);
    block.render(area, buf);

    let focus_label = match focus {
        FocusedPane::Terminal => "Terminal",
        FocusedPane::Chat => "AI Chat",
        FocusedPane::Sidebar => "Sidebar",
        FocusedPane::Logs => "Logs",
    };

    let branch_label = branch.unwrap_or("no branch");
    let tab = session.active_tab();

    let line = Line::from(vec![
        Span::styled(
            " TerminalOS ",
            Style::default()
                .fg(palette.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" │ "),
        Span::styled(workspace_name, Style::default().fg(palette.foreground)),
        Span::raw(" │ "),
        Span::styled(branch_label, Style::default().fg(palette.success)),
        Span::raw(" │ "),
        Span::raw(format!("Tab: {} ", tab.title)),
        Span::raw("│ "),
        Span::styled(
            format!("Focus: {focus_label}"),
            Style::default().fg(palette.warning),
        ),
        Span::raw(" │ Ctrl+Q quit │ Ctrl+T tab │ Ctrl+B sidebar │ Ctrl+/ chat"),
    ]);

    let paragraph = Paragraph::new(line);
    Widget::render(paragraph, inner, buf);
}
