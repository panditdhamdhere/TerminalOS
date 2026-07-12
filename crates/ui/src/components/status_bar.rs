use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Widget};
use terminalos_shared::Theme;
use terminalos_terminal::ShellSession;

use crate::event::FocusedPane;
use crate::theme::UiPalette;

/// Inputs for rendering the bottom status bar.
pub struct StatusBarProps<'a> {
    pub session: &'a ShellSession,
    pub workspace_name: &'a str,
    pub branch: Option<&'a str>,
    pub focus: FocusedPane,
    pub provider: &'a str,
    pub model: &'a str,
    pub provider_ready: bool,
}

/// Renders the bottom status bar with workspace and focus info.
pub fn render_status_bar(area: Rect, buf: &mut Buffer, props: &StatusBarProps<'_>, theme: &Theme) {
    let palette = UiPalette::from(theme);
    let block = Block::default().style(
        Style::default()
            .bg(palette.status_bar)
            .fg(palette.foreground),
    );
    let inner = block.inner(area);
    block.render(area, buf);

    let focus_label = match props.focus {
        FocusedPane::Terminal => "Terminal",
        FocusedPane::Chat => "AI Chat",
        FocusedPane::Sidebar => "Sidebar",
        FocusedPane::Logs => "Logs",
    };

    let branch_label = props.branch.unwrap_or("no branch");
    let tab = props.session.active_tab();
    let provider_color = if props.provider_ready {
        palette.success
    } else {
        palette.warning
    };

    let line = Line::from(vec![
        Span::styled(
            " TerminalOS ",
            Style::default()
                .fg(palette.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" │ "),
        Span::styled(
            props.workspace_name,
            Style::default().fg(palette.foreground),
        ),
        Span::raw(" │ "),
        Span::styled(branch_label, Style::default().fg(palette.success)),
        Span::raw(" │ "),
        Span::raw(format!("Tab: {} ", tab.title)),
        Span::raw("│ "),
        Span::styled(
            format!("AI: {} ", props.provider),
            Style::default().fg(provider_color),
        ),
        Span::styled(
            format!("({}) ", props.model),
            Style::default().fg(palette.muted),
        ),
        Span::raw("│ "),
        Span::styled(
            format!("Focus: {focus_label}"),
            Style::default().fg(palette.warning),
        ),
        Span::raw(" │ Ctrl+P provider"),
    ]);

    let paragraph = Paragraph::new(line);
    Widget::render(paragraph, inner, buf);
}
