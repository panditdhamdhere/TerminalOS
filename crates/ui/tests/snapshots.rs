use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use terminalos_config::LayoutConfig;
use terminalos_shared::Theme;
use terminalos_terminal::ShellSession;
use terminalos_ui::components::render_status_bar;
use terminalos_ui::event::FocusedPane;
use terminalos_ui::layout::{LayoutVisibility, compute_layout};

fn format_rect(rect: Rect) -> String {
    format!("({},{} {}x{})", rect.x, rect.y, rect.width, rect.height)
}

fn format_layout(area: Rect, config: &LayoutConfig, visibility: &LayoutVisibility) -> String {
    let layout = compute_layout(area, config, visibility);
    let mut lines = vec![
        format!("root: {}", format_rect(layout.root)),
        format!("status_bar: {}", format_rect(layout.status_bar)),
        format!("main_row: {}", format_rect(layout.main_row)),
        format!("center_column: {}", format_rect(layout.center_column)),
        format!("tab_bar: {}", format_rect(layout.tab_bar)),
        format!("terminal: {}", format_rect(layout.terminal)),
    ];

    if let Some(sidebar) = layout.sidebar {
        lines.push(format!("sidebar: {}", format_rect(sidebar)));
    }
    if let Some(chat) = layout.chat {
        lines.push(format!("chat: {}", format_rect(chat)));
    }
    if let Some(logs) = layout.logs {
        lines.push(format!("logs: {}", format_rect(logs)));
    }

    lines.join("\n")
}

fn buffer_to_string(buf: &Buffer) -> String {
    let area = buf.area;
    let mut lines = Vec::new();
    for y in area.y..area.y.saturating_add(area.height) {
        let mut row = String::new();
        for x in area.x..area.x.saturating_add(area.width) {
            if let Some(cell) = buf.cell((x, y)) {
                row.push_str(cell.symbol());
            }
        }
        lines.push(row.trim_end().to_string());
    }
    lines.join("\n")
}

#[test]
fn snapshot_full_layout() {
    let area = Rect::new(0, 0, 120, 40);
    let config = LayoutConfig::default();
    let visibility = LayoutVisibility {
        show_sidebar: true,
        show_chat: true,
        show_logs: true,
    };
    insta::assert_snapshot!("full_layout", format_layout(area, &config, &visibility));
}

#[test]
fn snapshot_minimal_layout() {
    let area = Rect::new(0, 0, 120, 40);
    let config = LayoutConfig::default();
    let visibility = LayoutVisibility {
        show_sidebar: false,
        show_chat: false,
        show_logs: false,
    };
    insta::assert_snapshot!("minimal_layout", format_layout(area, &config, &visibility));
}

#[test]
fn snapshot_status_bar() {
    let area = Rect::new(0, 0, 100, 1);
    let mut buf = Buffer::empty(area);
    let session = ShellSession::new("/tmp/terminalos");
    let theme = Theme::dark();

    render_status_bar(
        area,
        &mut buf,
        &session,
        "ai_terminal",
        Some("main"),
        FocusedPane::Terminal,
        &theme,
    );

    insta::assert_snapshot!("status_bar", buffer_to_string(&buf));
}
