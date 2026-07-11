use ratatui::layout::{Constraint, Direction, Layout, Rect};

use terminalos_config::LayoutConfig;

/// Computed layout rectangles for all UI panes.
#[derive(Debug, Clone, Copy)]
pub struct PaneLayout {
    pub root: Rect,
    pub status_bar: Rect,
    pub main_row: Rect,
    pub sidebar: Option<Rect>,
    pub center_column: Rect,
    pub tab_bar: Rect,
    pub terminal: Rect,
    pub chat: Option<Rect>,
    pub logs: Option<Rect>,
}

/// Minimum pane sizes to keep the UI usable.
const MIN_SIDEBAR_WIDTH: u16 = 16;
const MIN_CHAT_WIDTH: u16 = 24;
const MIN_LOGS_HEIGHT: u16 = 4;
const TAB_BAR_HEIGHT: u16 = 1;

/// Builds pane layout from terminal area and configuration.
#[must_use]
pub fn compute_layout(area: Rect, config: &LayoutConfig, ui: &LayoutVisibility) -> PaneLayout {
    let root = area;

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),
            Constraint::Length(config.status_bar_height.max(1)),
        ])
        .split(root);

    let main_row_area = vertical[0];
    let status_bar = vertical[1];

    let sidebar_width = if ui.show_sidebar {
        percent_width(main_row_area.width, config.sidebar_width_percent).max(MIN_SIDEBAR_WIDTH)
    } else {
        0
    };

    let chat_width = if ui.show_chat {
        percent_width(main_row_area.width, config.chat_width_percent).max(MIN_CHAT_WIDTH)
    } else {
        0
    };

    let logs_height = if ui.show_logs {
        percent_height(main_row_area.height, config.logs_height_percent).max(MIN_LOGS_HEIGHT)
    } else {
        0
    };

    let main_horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints({
            let mut constraints = Vec::new();
            if ui.show_sidebar {
                constraints.push(Constraint::Length(sidebar_width));
            }
            constraints.push(Constraint::Min(20));
            if ui.show_chat {
                constraints.push(Constraint::Length(chat_width));
            }
            constraints
        })
        .split(main_row_area);

    let mut idx = 0;
    let sidebar = if ui.show_sidebar {
        let rect = main_horizontal[idx];
        idx += 1;
        Some(rect)
    } else {
        None
    };

    let center_column = main_horizontal[idx];
    idx += 1;

    let chat = if ui.show_chat {
        let rect = main_horizontal[idx];
        Some(rect)
    } else {
        None
    };

    let center_vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints({
            let mut constraints = vec![Constraint::Min(6)];
            if ui.show_logs {
                constraints.push(Constraint::Length(logs_height));
            }
            constraints
        })
        .split(center_column);

    let terminal_column = center_vertical[0];
    let logs = if ui.show_logs {
        Some(center_vertical[1])
    } else {
        None
    };

    let terminal_split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(TAB_BAR_HEIGHT), Constraint::Min(4)])
        .split(terminal_column);

    PaneLayout {
        root,
        status_bar,
        main_row: main_row_area,
        sidebar,
        center_column,
        tab_bar: terminal_split[0],
        terminal: terminal_split[1],
        chat,
        logs,
    }
}

/// Visibility flags for optional panes.
#[derive(Debug, Clone, Copy)]
pub struct LayoutVisibility {
    pub show_sidebar: bool,
    pub show_chat: bool,
    pub show_logs: bool,
}

fn percent_width(total: u16, percent: u16) -> u16 {
    ((u32::from(total) * u32::from(percent.min(100))) / 100) as u16
}

fn percent_height(total: u16, percent: u16) -> u16 {
    ((u32::from(total) * u32::from(percent.min(100))) / 100) as u16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_respects_visibility() {
        let area = Rect::new(0, 0, 120, 40);
        let config = LayoutConfig::default();
        let layout = compute_layout(
            area,
            &config,
            &LayoutVisibility {
                show_sidebar: true,
                show_chat: true,
                show_logs: true,
            },
        );

        assert!(layout.sidebar.is_some());
        assert!(layout.chat.is_some());
        assert!(layout.logs.is_some());
        assert!(layout.terminal.height > 0);
    }
}
