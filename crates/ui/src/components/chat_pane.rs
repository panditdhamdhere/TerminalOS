use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget, Wrap};
use terminalos_agent::PendingAction;
use terminalos_ai::{DisplayMessage, MessageRole};
use terminalos_shared::Theme;

use crate::event::FocusedPane;
use crate::markdown::render_markdown;
use crate::theme::UiPalette;

/// Inputs for rendering the AI chat panel.
pub struct ChatPaneProps<'a> {
    pub messages: &'a [DisplayMessage],
    pub input: &'a str,
    pub provider: &'a str,
    pub theme: &'a Theme,
    pub focused: FocusedPane,
    pub scroll: usize,
    pub streaming: bool,
    pub pending: Option<&'a PendingAction>,
}

/// Renders the AI chat panel with markdown and syntax-highlighted responses.
pub fn render_chat_pane(area: Rect, buf: &mut Buffer, props: &ChatPaneProps<'_>) {
    let palette = UiPalette::from(props.theme);
    let border_style = if props.focused == FocusedPane::Chat {
        Style::default()
            .fg(palette.accent)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(palette.border)
    };

    let status = if props.streaming {
        " ● streaming"
    } else {
        ""
    };
    let block = Block::default()
        .title(format!("  AI Assistant ({}){status}  ", props.provider))
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

    let pending_rows = if props.pending.is_some() { 3 } else { 0 };
    let messages_area = Rect {
        height: inner.height.saturating_sub(1 + pending_rows),
        ..inner
    };

    let width = inner.width as usize;
    let mut lines: Vec<Line> = Vec::new();

    let visible: Vec<&DisplayMessage> = props
        .messages
        .iter()
        .filter(|m| m.role != MessageRole::System)
        .skip(props.scroll)
        .collect();

    if visible.is_empty() {
        lines.push(Line::from(Span::styled(
            "Ask anything about your codebase...",
            Style::default().fg(palette.muted),
        )));
    } else {
        for msg in visible {
            let role_label = match msg.role {
                MessageRole::User => "you",
                MessageRole::Assistant => "assistant",
                MessageRole::System => "system",
            };
            let role_style = Style::default()
                .fg(if msg.role == MessageRole::User {
                    palette.accent
                } else {
                    palette.success
                })
                .add_modifier(Modifier::BOLD);
            lines.push(Line::from(Span::styled(
                format!("{role_label}:"),
                role_style,
            )));

            if msg.role == MessageRole::Assistant {
                if msg.is_error || msg.content.starts_with("⚠") {
                    lines.push(Line::from(Span::styled(
                        msg.content.clone(),
                        Style::default()
                            .fg(palette.error)
                            .add_modifier(Modifier::BOLD),
                    )));
                } else {
                    lines.extend(render_markdown(&msg.content, props.theme, width));
                }
            } else {
                lines.push(Line::from(Span::raw(msg.content.clone())));
            }

            if msg.streaming {
                lines.push(Line::from(Span::styled(
                    "▌",
                    Style::default().add_modifier(Modifier::SLOW_BLINK),
                )));
            }
            lines.push(Line::from(""));
        }
    }

    let paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(palette.foreground));
    Widget::render(paragraph, messages_area, buf);

    if let Some(pending) = props.pending {
        let confirm_area = Rect {
            y: inner.y + messages_area.height,
            height: 2,
            width: inner.width,
            x: inner.x,
        };
        let mut confirm_lines = vec![
            Line::from(Span::styled(
                format!("⚠ {}", pending.summary),
                Style::default()
                    .fg(palette.warning)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                "[y] confirm  [n] cancel",
                Style::default().fg(palette.muted),
            )),
        ];
        if !pending.preview.is_empty() {
            let preview: String = pending
                .preview
                .lines()
                .take(3)
                .collect::<Vec<_>>()
                .join("\n");
            confirm_lines.insert(
                1,
                Line::from(Span::styled(
                    preview,
                    Style::default().fg(palette.foreground),
                )),
            );
        }
        Widget::render(Paragraph::new(confirm_lines), confirm_area, buf);
    }

    let input_area = Rect {
        y: inner.y + inner.height.saturating_sub(1),
        height: 1,
        ..inner
    };
    let input_line = Paragraph::new(Line::from(vec![
        Span::styled("› ", Style::default().fg(palette.accent)),
        Span::raw(props.input),
        Span::styled("▌", Style::default().add_modifier(Modifier::SLOW_BLINK)),
    ]));
    Widget::render(input_line, input_area, buf);
}
