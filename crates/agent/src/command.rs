/// Parsed slash command from chat input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlashCommand {
    Edit { path: String, instruction: String },
    Create { path: String, description: String },
    Fix { path: String, instruction: String },
    Refactor { path: String, instruction: String },
    Explain { path: String, question: String },
    Test { args: String },
    Review { path: String },
    Search { query: String },
    Docs { path: String, focus: String },
    Analyze,
}

/// Parses `/command args` from chat input. Returns `None` for non-slash input.
#[must_use]
pub fn parse_slash_command(input: &str) -> Option<SlashCommand> {
    let trimmed = input.trim();
    if !trimmed.starts_with('/') {
        return None;
    }

    let body = trimmed.trim_start_matches('/').trim();
    if body.is_empty() {
        return None;
    }

    let mut parts = body.splitn(2, char::is_whitespace);
    let name = parts.next()?.to_ascii_lowercase();
    let rest = parts.next().unwrap_or("").trim();

    match name.as_str() {
        "edit" | "fix" | "refactor" | "explain" | "review" | "docs" => {
            let (path, text) = split_path_and_rest(rest)?;
            match name.as_str() {
                "edit" => Some(SlashCommand::Edit {
                    path,
                    instruction: text,
                }),
                "fix" => Some(SlashCommand::Fix {
                    path,
                    instruction: if text.is_empty() {
                        "Fix bugs and issues in this file.".to_string()
                    } else {
                        text
                    },
                }),
                "refactor" => Some(SlashCommand::Refactor {
                    path,
                    instruction: text,
                }),
                "explain" => Some(SlashCommand::Explain {
                    path,
                    question: if text.is_empty() {
                        "Explain this file.".to_string()
                    } else {
                        text
                    },
                }),
                "review" => Some(SlashCommand::Review { path }),
                "docs" => Some(SlashCommand::Docs {
                    path,
                    focus: if text.is_empty() {
                        "public API".to_string()
                    } else {
                        text
                    },
                }),
                _ => None,
            }
        }
        "create" => {
            let (path, text) = split_path_and_rest(rest)?;
            Some(SlashCommand::Create {
                path,
                description: text,
            })
        }
        "test" => Some(SlashCommand::Test {
            args: rest.to_string(),
        }),
        "search" => {
            if rest.is_empty() {
                None
            } else {
                Some(SlashCommand::Search {
                    query: rest.to_string(),
                })
            }
        }
        "analyze" => Some(SlashCommand::Analyze),
        _ => None,
    }
}

fn split_path_and_rest(input: &str) -> Option<(String, String)> {
    let input = input.trim();
    if input.is_empty() {
        return None;
    }

    if let Some(quoted) = input.strip_prefix('"') {
        let end = quoted.find('"')? + 1;
        let path = quoted[..quoted.find('"')?].to_string();
        let rest = input[end + 1..].trim().to_string();
        return Some((path, rest));
    }

    let mut parts = input.splitn(2, char::is_whitespace);
    let path = parts.next()?.to_string();
    let rest = parts.next().unwrap_or("").trim().to_string();
    Some((path, rest))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_edit_command() {
        let cmd = parse_slash_command("/edit src/main.rs add error handling").unwrap();
        assert_eq!(
            cmd,
            SlashCommand::Edit {
                path: "src/main.rs".to_string(),
                instruction: "add error handling".to_string(),
            }
        );
    }

    #[test]
    fn parses_search_command() {
        let cmd = parse_slash_command("/search ChatEngine submit").unwrap();
        assert_eq!(
            cmd,
            SlashCommand::Search {
                query: "ChatEngine submit".to_string(),
            }
        );
    }

    #[test]
    fn parses_test_without_args() {
        let cmd = parse_slash_command("/test").unwrap();
        assert_eq!(
            cmd,
            SlashCommand::Test {
                args: String::new()
            }
        );
    }

    #[test]
    fn rejects_non_slash() {
        assert!(parse_slash_command("hello").is_none());
    }
}
