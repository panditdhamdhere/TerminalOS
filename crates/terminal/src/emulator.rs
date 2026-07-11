/// A styled text span from the terminal emulator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyledSpan {
    pub text: String,
    pub fg: Option<(u8, u8, u8)>,
    pub bg: Option<(u8, u8, u8)>,
    pub fg_index: Option<u8>,
    pub bg_index: Option<u8>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

/// ANSI-aware terminal emulator backed by vt100.
pub struct TerminalEmulator {
    parser: vt100::Parser,
    rows: u16,
    cols: u16,
    search_query: Option<String>,
}

impl TerminalEmulator {
    #[must_use]
    pub fn new(rows: u16, cols: u16) -> Self {
        Self {
            parser: vt100::Parser::new(rows, cols, 10_000),
            rows,
            cols,
            search_query: None,
        }
    }

    pub fn resize(&mut self, rows: u16, cols: u16) {
        self.rows = rows;
        self.cols = cols;
        self.parser.set_size(rows, cols);
    }

    pub fn process(&mut self, bytes: &[u8]) {
        self.parser.process(bytes);
    }

    pub fn scroll_up(&mut self, amount: usize) {
        let current = self.parser.screen().scrollback();
        self.parser.set_scrollback(current + amount);
    }

    pub fn scroll_down(&mut self, amount: usize) {
        let current = self.parser.screen().scrollback();
        self.parser.set_scrollback(current.saturating_sub(amount));
    }

    pub fn set_search(&mut self, query: Option<String>) {
        self.search_query = query.filter(|q| !q.is_empty());
    }

    #[must_use]
    pub fn search_query(&self) -> Option<&str> {
        self.search_query.as_deref()
    }

    /// Renders visible rows as styled spans.
    #[must_use]
    pub fn render_rows(&self, height: usize, width: usize) -> Vec<Vec<StyledSpan>> {
        let screen = self.parser.screen();
        let (screen_rows, screen_cols) = screen.size();
        let render_rows = height.min(screen_rows as usize);
        let render_cols = width.min(screen_cols as usize);

        let mut lines = Vec::with_capacity(render_rows);
        for row in 0..render_rows {
            let mut spans = Vec::new();
            let mut current = StyledSpan::empty();

            for col in 0..render_cols {
                let Some(cell) = screen.cell(row as u16, col as u16) else {
                    continue;
                };
                if !cell.has_contents() {
                    continue;
                }

                let text = cell.contents();
                let span = StyledSpan::from_cell(&text, cell);

                if current.is_empty() {
                    current = span;
                } else if current.style_eq(&span) {
                    current.text.push_str(&text);
                } else {
                    spans.push(current);
                    current = span;
                }
            }

            if !current.is_empty() {
                spans.push(current);
            }

            if spans.is_empty() {
                spans.push(StyledSpan::blank());
            }

            if let Some(query) = &self.search_query {
                spans = highlight_search(spans, query);
            }

            lines.push(spans);
        }

        lines
    }

    /// Returns plain-text contents for clipboard copy.
    #[must_use]
    pub fn plain_text(&self) -> String {
        self.parser.screen().contents()
    }
}

impl StyledSpan {
    fn empty() -> Self {
        Self {
            text: String::new(),
            fg: None,
            bg: None,
            fg_index: None,
            bg_index: None,
            bold: false,
            italic: false,
            underline: false,
        }
    }

    fn blank() -> Self {
        Self {
            text: " ".to_string(),
            ..Self::empty()
        }
    }

    fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    fn from_cell(text: &str, cell: &vt100::Cell) -> Self {
        let (fg, fg_index) = color_to_parts(cell.fgcolor());
        let (bg, bg_index) = color_to_parts(cell.bgcolor());
        Self {
            text: text.to_string(),
            fg,
            bg,
            fg_index,
            bg_index,
            bold: cell.bold(),
            italic: cell.italic(),
            underline: cell.underline(),
        }
    }

    fn style_eq(&self, other: &Self) -> bool {
        self.fg == other.fg
            && self.bg == other.bg
            && self.fg_index == other.fg_index
            && self.bg_index == other.bg_index
            && self.bold == other.bold
            && self.italic == other.italic
            && self.underline == other.underline
    }
}

fn color_to_parts(color: vt100::Color) -> (Option<(u8, u8, u8)>, Option<u8>) {
    match color {
        vt100::Color::Default => (None, None),
        vt100::Color::Idx(i) => (None, Some(i)),
        vt100::Color::Rgb(r, g, b) => (Some((r, g, b)), None),
    }
}

fn highlight_search(spans: Vec<StyledSpan>, query: &str) -> Vec<StyledSpan> {
    let mut result = Vec::new();
    let query_lower = query.to_lowercase();

    for span in spans {
        let text_lower = span.text.to_lowercase();
        if let Some(pos) = text_lower.find(&query_lower) {
            let before = &span.text[..pos];
            let match_text = &span.text[pos..pos + query.len()];
            let after = &span.text[pos + query.len()..];

            if !before.is_empty() {
                result.push(StyledSpan {
                    text: before.to_string(),
                    ..span.clone()
                });
            }
            result.push(StyledSpan {
                text: match_text.to_string(),
                bg: Some((255, 255, 0)),
                fg: Some((0, 0, 0)),
                ..span.clone()
            });
            if !after.is_empty() {
                result.push(StyledSpan {
                    text: after.to_string(),
                    ..span
                });
            }
        } else {
            result.push(span);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn processes_ansi_output() {
        let mut emu = TerminalEmulator::new(24, 80);
        emu.process(b"Hello \x1b[31mWorld\x1b[0m\n");
        let rows = emu.render_rows(5, 80);
        assert!(!rows.is_empty());
    }
}
