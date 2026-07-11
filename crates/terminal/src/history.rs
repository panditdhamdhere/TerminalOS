use std::collections::VecDeque;

/// Ring buffer of previously entered shell commands.
#[derive(Debug, Clone, Default)]
pub struct CommandHistory {
    entries: VecDeque<String>,
    cursor: Option<usize>,
    max_entries: usize,
}

impl CommandHistory {
    #[must_use]
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: VecDeque::new(),
            cursor: None,
            max_entries,
        }
    }

    pub fn push(&mut self, command: String) {
        let trimmed = command.trim();
        if trimmed.is_empty() {
            return;
        }
        if self.entries.back().is_some_and(|last| last == trimmed) {
            return;
        }
        self.entries.push_back(trimmed.to_string());
        if self.entries.len() > self.max_entries {
            self.entries.pop_front();
        }
        self.cursor = None;
    }

    #[must_use]
    pub fn entries(&self) -> &VecDeque<String> {
        &self.entries
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deduplicates_consecutive_commands() {
        let mut history = CommandHistory::new(100);
        history.push("ls".to_string());
        history.push("ls".to_string());
        assert_eq!(history.len(), 1);
    }
}
