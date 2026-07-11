use std::collections::HashMap;

use terminalos_shared::{Error, Result, TabId};

use crate::emulator::TerminalEmulator;
use crate::history::CommandHistory;
use crate::pty::{PtyOutput, PtySession, output_channel};
use crate::session::ShellSession;

/// Manages PTY-backed shell sessions for all terminal tabs.
pub struct ShellManager {
    session: ShellSession,
    emulators: HashMap<TabId, TerminalEmulator>,
    pty_sessions: HashMap<TabId, PtySession>,
    history: HashMap<TabId, CommandHistory>,
    output_rx: std::sync::mpsc::Receiver<PtyOutput>,
    output_tx: std::sync::mpsc::Sender<PtyOutput>,
    terminal_rows: u16,
    terminal_cols: u16,
    search_mode: bool,
    search_input: String,
}

impl ShellManager {
    pub fn new(cwd: impl Into<String>, rows: u16, cols: u16) -> Result<Self> {
        let (output_tx, output_rx) = output_channel();
        let cwd = cwd.into();

        let mut manager = Self {
            session: ShellSession::new(&cwd),
            emulators: HashMap::new(),
            pty_sessions: HashMap::new(),
            history: HashMap::new(),
            output_rx,
            output_tx,
            terminal_rows: rows.max(4),
            terminal_cols: cols.max(20),
            search_mode: false,
            search_input: String::new(),
        };

        manager.spawn_tab_shell(manager.session.active_tab().id, &cwd)?;
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
    pub fn active_emulator(&self) -> Option<&TerminalEmulator> {
        let id = self.session.active_tab().id;
        self.emulators.get(&id)
    }

    #[must_use]
    pub fn active_emulator_mut(&mut self) -> Option<&mut TerminalEmulator> {
        let id = self.session.active_tab().id;
        self.emulators.get_mut(&id)
    }

    pub fn resize(&mut self, rows: u16, cols: u16) {
        self.terminal_rows = rows.max(4);
        self.terminal_cols = cols.max(20);
        let rows = self.terminal_rows;
        let cols = self.terminal_cols;
        if let Some(emu) = self.active_emulator_mut() {
            emu.resize(rows, cols);
        }
    }

    pub fn poll_output(&mut self) {
        while let Ok(output) = self.output_rx.try_recv() {
            if let Some(emu) = self.emulators.get_mut(&output.tab_id) {
                emu.process(&output.data);
            }
        }
    }

    pub fn write_bytes(&mut self, data: &[u8]) -> Result<()> {
        let tab_id = self.session.active_tab().id;
        let session = self
            .pty_sessions
            .get_mut(&tab_id)
            .ok_or_else(|| Error::Terminal("no active pty session".to_string()))?;
        session.write(data)
    }

    pub fn write_to_active(&mut self, data: &[u8]) -> Result<()> {
        self.write_bytes(data)
    }

    pub fn new_tab(&mut self) -> Result<()> {
        let cwd = self.session.active_tab().cwd.clone();
        self.session.new_tab();
        let tab_id = self.session.active_tab().id;
        self.spawn_tab_shell(tab_id, &cwd)
    }

    pub fn close_active_tab(&mut self) -> bool {
        if self.session.tabs.len() <= 1 {
            return false;
        }
        let tab_id = self.session.active_tab().id;
        self.pty_sessions.remove(&tab_id);
        self.emulators.remove(&tab_id);
        self.history.remove(&tab_id);
        self.session.close_tab()
    }

    pub fn on_tab_switched(&mut self) {
        let rows = self.terminal_rows;
        let cols = self.terminal_cols;
        if let Some(emu) = self.active_emulator_mut() {
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

    fn spawn_tab_shell(&mut self, tab_id: TabId, cwd: &str) -> Result<()> {
        self.emulators.insert(
            tab_id,
            TerminalEmulator::new(self.terminal_rows, self.terminal_cols),
        );
        self.history.insert(tab_id, CommandHistory::new(1000));

        let pty = PtySession::spawn(
            tab_id,
            cwd,
            self.terminal_rows,
            self.terminal_cols,
            self.output_tx.clone(),
        )?;
        self.pty_sessions.insert(tab_id, pty);
        Ok(())
    }
}
