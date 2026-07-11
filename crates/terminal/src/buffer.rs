/// Scrollable terminal output buffer with line storage.
#[derive(Debug, Clone, Default)]
pub struct TerminalBuffer {
    lines: Vec<String>,
    cursor_line: usize,
    max_lines: usize,
}

impl TerminalBuffer {
    #[must_use]
    pub fn new(max_lines: usize) -> Self {
        Self {
            lines: Vec::new(),
            cursor_line: 0,
            max_lines,
        }
    }

    pub fn push_line(&mut self, line: impl Into<String>) {
        self.lines.push(line.into());
        if self.lines.len() > self.max_lines {
            let overflow = self.lines.len() - self.max_lines;
            self.lines.drain(0..overflow);
        }
        self.cursor_line = self.lines.len().saturating_sub(1);
    }

    pub fn push_str(&mut self, text: &str) {
        for line in text.lines() {
            self.push_line(line);
        }
    }

    #[must_use]
    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    #[must_use]
    pub fn cursor_line(&self) -> usize {
        self.cursor_line
    }

    pub fn scroll_up(&mut self, amount: usize) {
        self.cursor_line = self.cursor_line.saturating_sub(amount);
    }

    pub fn scroll_down(&mut self, amount: usize) {
        let max = self.lines.len().saturating_sub(1);
        self.cursor_line = (self.cursor_line + amount).min(max);
    }
}
