use std::collections::HashMap;

use terminalos_shared::{Error, PaneId, Result};

use crate::emulator::TerminalEmulator;
use crate::history::CommandHistory;
use crate::layout::{Area, SplitDirection, compute_pane_rects};
use crate::pty::{PtyOutput, PtySession, output_channel};
use crate::session::ShellSession;

/// Manages PTY-backed shell sessions for all terminal tabs and panes.
pub struct ShellManager {
    session: ShellSession,
    emulators: HashMap<PaneId, TerminalEmulator>,
    pty_sessions: HashMap<PaneId, PtySession>,
    pane_cwds: HashMap<PaneId, String>,
    history: HashMap<PaneId, CommandHistory>,
    output_rx: std::sync::mpsc::Receiver<PtyOutput>,
    output_tx: std::sync::mpsc::Sender<PtyOutput>,
    terminal_rows: u16,
    terminal_cols: u16,
    search_mode: bool,
    search_input: String,
    workspace_env: HashMap<String, String>,
}

impl ShellManager {
    pub fn new(cwd: impl Into<String>, rows: u16, cols: u16) -> Result<Self> {
        let (output_tx, output_rx) = output_channel();
        let cwd = cwd.into();

        let mut manager = Self {
            session: ShellSession::new(&cwd),
            emulators: HashMap::new(),
            pty_sessions: HashMap::new(),
            pane_cwds: HashMap::new(),
            history: HashMap::new(),
            output_rx,
            output_tx,
            terminal_rows: rows.max(4),
            terminal_cols: cols.max(20),
            search_mode: false,
            search_input: String::new(),
            workspace_env: HashMap::new(),
        };

        let tab = manager.session.active_tab();
        let pane_id = tab.active_pane;
        let env = manager.workspace_env.clone();
        manager.spawn_pane_shell(pane_id, &cwd, &env)?;
        Ok(manager)
    }

    pub fn from_restored(
        session: ShellSession,
        rows: u16,
        cols: u16,
        env: &HashMap<String, String>,
    ) -> Result<Self> {
        let (output_tx, output_rx) = output_channel();
        let mut manager = Self {
            session,
            emulators: HashMap::new(),
            pty_sessions: HashMap::new(),
            pane_cwds: HashMap::new(),
            history: HashMap::new(),
            output_rx,
            output_tx,
            terminal_rows: rows.max(4),
            terminal_cols: cols.max(20),
            search_mode: false,
            search_input: String::new(),
            workspace_env: env.clone(),
        };

        let panes: Vec<(PaneId, String)> = manager
            .session
            .tabs
            .iter()
            .flat_map(|tab| {
                tab.layout
                    .collect_panes()
                    .into_iter()
                    .map(|pane_id| (pane_id, tab.cwd.clone()))
                    .collect::<Vec<_>>()
            })
            .collect();

        for (pane_id, cwd) in panes {
            manager.spawn_pane_shell(pane_id, &cwd, env)?;
        }

        Ok(manager)
    }

    #[must_use]
    pub fn session(&self) -> &ShellSession {
        &self.session
    }

    #[must_use]
    pub fn session_mut(&mut self) -> &mut ShellSession {
        &mut self.session
    }

    #[must_use]
    pub fn active_pane_id(&self) -> PaneId {
        self.session.active_tab().active_pane
    }

    #[must_use]
    pub fn active_emulator(&self) -> Option<&TerminalEmulator> {
        self.emulators.get(&self.active_pane_id())
    }

    #[must_use]
    pub fn active_emulator_mut(&mut self) -> Option<&mut TerminalEmulator> {
        let pane_id = self.active_pane_id();
        self.emulators.get_mut(&pane_id)
    }

    #[must_use]
    pub fn emulator(&self, pane_id: PaneId) -> Option<&TerminalEmulator> {
        self.emulators.get(&pane_id)
    }

    #[must_use]
    pub fn active_pane_rects(&self, area: Area) -> Vec<(PaneId, Area)> {
        compute_pane_rects(area, &self.session.active_tab().layout)
    }

    pub fn resize(&mut self, rows: u16, cols: u16) {
        self.terminal_rows = rows.max(4);
        self.terminal_cols = cols.max(20);
        let pane_id = self.active_pane_id();
        if let Some(emu) = self.emulators.get_mut(&pane_id) {
            emu.resize(self.terminal_rows, self.terminal_cols);
        }
    }

    pub fn resize_pane(&mut self, pane_id: PaneId, rows: u16, cols: u16) {
        let rows = rows.max(4);
        let cols = cols.max(20);
        if let Some(emu) = self.emulators.get_mut(&pane_id) {
            emu.resize(rows, cols);
        }
        if let Some(pty) = self.pty_sessions.get_mut(&pane_id) {
            let _ = pty.resize(rows, cols);
        }
    }

    pub fn poll_output(&mut self) {
        while let Ok(output) = self.output_rx.try_recv() {
            if let Some(emu) = self.emulators.get_mut(&output.pane_id) {
                emu.process(&output.data);
            }
        }
    }

    pub fn write_bytes(&mut self, data: &[u8]) -> Result<()> {
        let pane_id = self.active_pane_id();
        let session = self
            .pty_sessions
            .get_mut(&pane_id)
            .ok_or_else(|| Error::Terminal("no active pty session".to_string()))?;
        session.write(data)
    }

    pub fn write_to_active(&mut self, data: &[u8]) -> Result<()> {
        self.write_bytes(data)
    }

    pub fn set_workspace_env(&mut self, env: HashMap<String, String>) {
        self.workspace_env = env;
    }

    pub fn new_tab(&mut self) -> Result<()> {
        let cwd = self.session.active_tab().cwd.clone();
        self.session.new_tab();
        let pane_id = self.session.active_tab().active_pane;
        let env = self.workspace_env.clone();
        self.spawn_pane_shell(pane_id, &cwd, &env)
    }

    pub fn close_active_tab(&mut self) -> bool {
        if self.session.tabs.len() <= 1 {
            return false;
        }
        let panes = self.session.active_tab().layout.collect_panes();
        for pane_id in panes {
            self.remove_pane_resources(pane_id);
        }
        self.session.close_tab()
    }

    pub fn split_active_horizontal(&mut self) -> Result<()> {
        self.split_active(SplitDirection::Horizontal)
    }

    pub fn split_active_vertical(&mut self) -> Result<()> {
        self.split_active(SplitDirection::Vertical)
    }

    pub fn close_active_pane(&mut self) -> bool {
        let tab = self.session.active_tab();
        if tab.pane_count() <= 1 {
            return false;
        }

        let pane_id = tab.active_pane;
        let panes = tab.layout.collect_panes();
        let focus_index = panes.iter().position(|id| *id == pane_id).unwrap_or(0);
        let next_focus = if focus_index == 0 {
            panes[1]
        } else {
            panes[focus_index - 1]
        };

        self.remove_pane_resources(pane_id);
        let tab = self.session.active_tab_mut();
        tab.layout.remove_pane(pane_id);
        tab.active_pane = next_focus;
        true
    }

    pub fn focus_next_pane(&mut self) {
        self.session.focus_next_pane();
    }

    pub fn focus_prev_pane(&mut self) {
        self.session.focus_prev_pane();
    }

    pub fn on_tab_switched(&mut self) {
        let rows = self.terminal_rows;
        let cols = self.terminal_cols;
        let pane_id = self.active_pane_id();
        if let Some(emu) = self.emulators.get_mut(&pane_id) {
            emu.resize(rows, cols);
        }
    }

    pub fn scroll_active_up(&mut self, amount: usize) {
        if let Some(emu) = self.active_emulator_mut() {
            emu.scroll_up(amount);
        }
    }

    pub fn scroll_active_down(&mut self, amount: usize) {
        if let Some(emu) = self.active_emulator_mut() {
            emu.scroll_down(amount);
        }
    }

    pub fn copy_active_to_clipboard(&self) -> Result<()> {
        let text = self
            .active_emulator()
            .map(TerminalEmulator::plain_text)
            .unwrap_or_default();
        let mut clipboard =
            arboard::Clipboard::new().map_err(|e| Error::Terminal(format!("clipboard: {e}")))?;
        clipboard
            .set_text(text)
            .map_err(|e| Error::Terminal(format!("clipboard set: {e}")))?;
        Ok(())
    }

    pub fn paste_to_active(&mut self) -> Result<()> {
        let text = {
            let mut clipboard = arboard::Clipboard::new()
                .map_err(|e| Error::Terminal(format!("clipboard: {e}")))?;
            clipboard
                .get_text()
                .map_err(|e| Error::Terminal(format!("clipboard get: {e}")))?
        };
        self.write_bytes(text.as_bytes())
    }

    pub fn toggle_search(&mut self) {
        self.search_mode = !self.search_mode;
        if !self.search_mode {
            self.search_input.clear();
            if let Some(emu) = self.active_emulator_mut() {
                emu.set_search(None);
            }
        }
    }

    #[must_use]
    pub fn is_search_mode(&self) -> bool {
        self.search_mode
    }

    #[must_use]
    pub fn search_input(&self) -> &str {
        &self.search_input
    }

    pub fn search_input_push(&mut self, c: char) {
        self.search_input.push(c);
        self.apply_search();
    }

    pub fn search_input_pop(&mut self) {
        self.search_input.pop();
        self.apply_search();
    }

    pub fn search_submit(&mut self) {
        self.apply_search();
        self.search_mode = false;
    }

    fn apply_search(&mut self) {
        let query = if self.search_input.is_empty() {
            None
        } else {
            Some(self.search_input.clone())
        };
        if let Some(emu) = self.active_emulator_mut() {
            emu.set_search(query);
        }
    }

    #[must_use]
    pub fn render_height(&self) -> usize {
        self.terminal_rows as usize
    }

    #[must_use]
    pub fn render_width(&self) -> usize {
        self.terminal_cols as usize
    }

    fn split_active(&mut self, direction: SplitDirection) -> Result<()> {
        let active_pane = self.active_pane_id();
        let cwd = self
            .pane_cwds
            .get(&active_pane)
            .cloned()
            .unwrap_or_else(|| self.session.active_tab().cwd.clone());
        let new_pane = PaneId::new();
        let env = self.workspace_env.clone();
        self.spawn_pane_shell(new_pane, &cwd, &env)?;

        let tab = self.session.active_tab_mut();
        tab.layout.split_pane(active_pane, direction, new_pane);
        tab.active_pane = new_pane;
        Ok(())
    }

    fn remove_pane_resources(&mut self, pane_id: PaneId) {
        self.pty_sessions.remove(&pane_id);
        self.emulators.remove(&pane_id);
        self.history.remove(&pane_id);
        self.pane_cwds.remove(&pane_id);
    }

    fn spawn_pane_shell(
        &mut self,
        pane_id: PaneId,
        cwd: &str,
        env: &HashMap<String, String>,
    ) -> Result<()> {
        self.emulators.insert(
            pane_id,
            TerminalEmulator::new(self.terminal_rows, self.terminal_cols),
        );
        self.history.insert(pane_id, CommandHistory::new(1000));
        self.pane_cwds.insert(pane_id, cwd.to_string());

        let pty = PtySession::spawn(
            pane_id,
            cwd,
            self.terminal_rows,
            self.terminal_cols,
            self.output_tx.clone(),
            env,
        )?;
        self.pty_sessions.insert(pane_id, pty);
        Ok(())
    }
}
