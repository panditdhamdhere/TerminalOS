use terminalos_shared::{PaneId, TabId};

use crate::layout::SplitNode;

/// A single terminal tab backed by one or more PTY panes.
#[derive(Debug, Clone)]
pub struct TerminalTab {
    pub id: TabId,
    pub title: String,
    pub cwd: String,
    pub layout: SplitNode,
    pub active_pane: PaneId,
}

impl TerminalTab {
    #[must_use]
    pub fn new(title: impl Into<String>, cwd: impl Into<String>) -> Self {
        let pane_id = PaneId::new();
        Self {
            id: TabId::new(),
            title: title.into(),
            cwd: cwd.into(),
            layout: SplitNode::single(pane_id),
            active_pane: pane_id,
        }
    }

    #[must_use]
    pub fn with_id(id: TabId, title: impl Into<String>, cwd: impl Into<String>) -> Self {
        let pane_id = PaneId::new();
        Self {
            id,
            title: title.into(),
            cwd: cwd.into(),
            layout: SplitNode::single(pane_id),
            active_pane: pane_id,
        }
    }

    #[must_use]
    pub fn restore(
        id: TabId,
        title: impl Into<String>,
        cwd: impl Into<String>,
        layout: SplitNode,
        active_pane: PaneId,
    ) -> Self {
        let active_pane = if layout.contains_pane(active_pane) {
            active_pane
        } else {
            layout.collect_panes()[0]
        };
        Self {
            id,
            title: title.into(),
            cwd: cwd.into(),
            layout,
            active_pane,
        }
    }

    #[must_use]
    pub fn pane_count(&self) -> usize {
        self.layout.pane_count()
    }
}

/// Tab collection with active tab tracking.
#[derive(Debug, Clone)]
pub struct ShellSession {
    pub tabs: Vec<TerminalTab>,
    pub active_tab: usize,
}

impl ShellSession {
    #[must_use]
    pub fn new(cwd: impl Into<String>) -> Self {
        let cwd = cwd.into();
        Self {
            tabs: vec![TerminalTab::new("Terminal 1", &cwd)],
            active_tab: 0,
        }
    }

    #[must_use]
    pub fn from_tabs(tabs: Vec<TerminalTab>, active_tab: usize) -> Self {
        let active_tab = if tabs.is_empty() {
            0
        } else {
            active_tab.min(tabs.len() - 1)
        };
        Self { tabs, active_tab }
    }

    #[must_use]
    pub fn active_tab(&self) -> &TerminalTab {
        &self.tabs[self.active_tab]
    }

    #[must_use]
    pub fn active_tab_mut(&mut self) -> &mut TerminalTab {
        &mut self.tabs[self.active_tab]
    }

    pub fn new_tab(&mut self) {
        let index = self.tabs.len() + 1;
        let cwd = self.tabs[self.active_tab].cwd.clone();
        self.tabs
            .push(TerminalTab::new(format!("Terminal {index}"), cwd));
        self.active_tab = self.tabs.len() - 1;
    }

    pub fn close_tab(&mut self) -> bool {
        if self.tabs.len() <= 1 {
            return false;
        }
        self.tabs.remove(self.active_tab);
        if self.active_tab >= self.tabs.len() {
            self.active_tab = self.tabs.len() - 1;
        }
        true
    }

    pub fn next_tab(&mut self) {
        if !self.tabs.is_empty() {
            self.active_tab = (self.active_tab + 1) % self.tabs.len();
        }
    }

    pub fn prev_tab(&mut self) {
        if !self.tabs.is_empty() {
            self.active_tab = if self.active_tab == 0 {
                self.tabs.len() - 1
            } else {
                self.active_tab - 1
            };
        }
    }

    pub fn select_tab(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.active_tab = index;
        }
    }

    pub fn focus_next_pane(&mut self) {
        let panes = self.active_tab().layout.collect_panes();
        if panes.len() <= 1 {
            return;
        }
        let tab = self.active_tab_mut();
        let current = panes
            .iter()
            .position(|pane| *pane == tab.active_pane)
            .unwrap_or(0);
        tab.active_pane = panes[(current + 1) % panes.len()];
    }

    pub fn focus_prev_pane(&mut self) {
        let panes = self.active_tab().layout.collect_panes();
        if panes.len() <= 1 {
            return;
        }
        let tab = self.active_tab_mut();
        let current = panes
            .iter()
            .position(|pane| *pane == tab.active_pane)
            .unwrap_or(0);
        let previous = if current == 0 {
            panes.len() - 1
        } else {
            current - 1
        };
        tab.active_pane = panes[previous];
    }
}
