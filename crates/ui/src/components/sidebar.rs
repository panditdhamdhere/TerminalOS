use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, Widget};
use terminalos_filesystem::FileNode;
use terminalos_shared::Theme;

use crate::event::FocusedPane;
use crate::theme::UiPalette;

/// Renders the workspace file tree sidebar.
pub fn render_sidebar(
    area: Rect,
    buf: &mut Buffer,
    tree: Option<&FileNode>,
    theme: &Theme,
    focused: FocusedPane,
    scroll: usize,
) {
    let palette = UiPalette::from(theme);
    let border_style = if focused == FocusedPane::Sidebar {
        Style::default()
            .fg(palette.accent)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(palette.border)
    };

    let block = Block::default()
        .title("  Workspace  ")
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(Style::default().bg(palette.sidebar).fg(palette.foreground));

    let inner = block.inner(area);
    block.render(area, buf);

    let items: Vec<ListItem> = if let Some(tree) = tree {
        flatten_tree(tree, 0)
            .into_iter()
            .skip(scroll)
            .map(|line| ListItem::new(line).style(Style::default().fg(palette.foreground)))
            .collect()
    } else {
        vec![ListItem::new("No workspace open").style(Style::default().fg(palette.muted))]
    };

    let list = List::new(items).style(Style::default().bg(palette.sidebar));
    Widget::render(list, inner, buf);
}

fn flatten_tree(node: &FileNode, depth: usize) -> Vec<String> {
    let indent = "  ".repeat(depth);
    let icon = if node.is_dir { "📁" } else { "📄" };
    let mut lines = vec![format!("{indent}{icon} {}", node.name)];
    if node.is_dir {
        for child in &node.children {
            lines.extend(flatten_tree(child, depth + 1));
        }
    }
    lines
}
