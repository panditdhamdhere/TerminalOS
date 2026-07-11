use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Converts a crossterm key event to bytes for PTY input.
#[must_use]
pub fn key_event_to_bytes(key: KeyEvent) -> Option<Vec<u8>> {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    let alt = key.modifiers.contains(KeyModifiers::ALT);

    if ctrl {
        return match key.code {
            KeyCode::Char('c') => Some(vec![0x03]),
            KeyCode::Char('d') => Some(vec![0x04]),
            KeyCode::Char('z') => Some(vec![0x1a]),
            KeyCode::Char('l') => Some(b"\x0c".to_vec()),
            _ => None,
        };
    }

    if alt {
        if let KeyCode::Char(c) = key.code {
            return Some(format!("\x1b{c}").into_bytes());
        }
    }

    match key.code {
        KeyCode::Char(c) => Some(c.to_string().into_bytes()),
        KeyCode::Enter => Some(b"\r".to_vec()),
        KeyCode::Backspace => Some(b"\x7f".to_vec()),
        KeyCode::Tab => Some(b"\t".to_vec()),
        KeyCode::Esc => Some(b"\x1b".to_vec()),
        KeyCode::Up => Some(b"\x1b[A".to_vec()),
        KeyCode::Down => Some(b"\x1b[B".to_vec()),
        KeyCode::Right => Some(b"\x1b[C".to_vec()),
        KeyCode::Left => Some(b"\x1b[D".to_vec()),
        KeyCode::Home => Some(b"\x1b[H".to_vec()),
        KeyCode::End => Some(b"\x1b[F".to_vec()),
        KeyCode::PageUp => Some(b"\x1b[5~".to_vec()),
        KeyCode::PageDown => Some(b"\x1b[6~".to_vec()),
        KeyCode::Delete => Some(b"\x1b[3~".to_vec()),
        KeyCode::Insert => Some(b"\x1b[2~".to_vec()),
        KeyCode::F(n) => Some(format!("\x1b[{n}~").into_bytes()),
        _ => None,
    }
}

/// Returns true if the key should scroll the terminal view rather than send to shell.
#[must_use]
pub fn is_scroll_key(key: KeyEvent) -> bool {
    let shift = key.modifiers.contains(KeyModifiers::SHIFT);
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

    if ctrl && shift {
        return matches!(key.code, KeyCode::Up | KeyCode::Down);
    }

    matches!(key.code, KeyCode::PageUp | KeyCode::PageDown) && !ctrl
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enter_sends_carriage_return() {
        let bytes = key_event_to_bytes(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(bytes, Some(b"\r".to_vec()));
    }

    #[test]
    fn ctrl_c_sends_interrupt() {
        let bytes = key_event_to_bytes(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
        assert_eq!(bytes, Some(vec![0x03]));
    }
}
