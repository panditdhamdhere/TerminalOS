use terminalos_shared::TabId;

/// A single terminal tab backed by a PTY session.
#[derive(Debug, Clone)]
pub struct TerminalTab {
    pub id: TabId,
    pub title: String,
    pub cwd: String,
}

impl TerminalTab {
    #[must_use]
    pub fn new(title: impl Into<String>, cwd: impl Into<String>) -> Self {
        Self {
            id: TabId::new(),
            title: title.into(),
            cwd: cwd.into(),
        }
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
    pub fn active_tab(&self) -> &TerminalTab {
        &self.tabs[self.active_tab]
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
}
