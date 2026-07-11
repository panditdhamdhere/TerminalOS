use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

use crate::theme::UiPalette;
use terminalos_shared::Theme;

/// Renders markdown content into ratatui lines with syntax highlighting.
pub fn render_markdown(content: &str, theme: &Theme, width: usize) -> Vec<Line<'static>> {
    let palette = UiPalette::from(theme);
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut current_spans: Vec<Span<'static>> = Vec::new();

    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    let parser = Parser::new_ext(content, options);

    let mut in_code_block = false;
    let mut code_buffer = String::new();
    let mut code_lang = String::new();

    let flush_line = |lines: &mut Vec<Line<'static>>, spans: &mut Vec<Span<'static>>| {
        if spans.is_empty() {
            lines.push(Line::from(""));
        } else {
            lines.push(Line::from(spans.clone()));
            spans.clear();
        }
    };

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                flush_line(&mut lines, &mut current_spans);
                current_spans.push(Span::styled(
                    format!("{} ", "#".repeat(level as usize)),
                    Style::default()
                        .fg(palette.accent)
                        .add_modifier(Modifier::BOLD),
                ));
            }
            Event::End(TagEnd::Heading(_)) => {
                flush_line(&mut lines, &mut current_spans);
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                flush_line(&mut lines, &mut current_spans);
                in_code_block = true;
                code_buffer.clear();
                code_lang = match kind {
                    CodeBlockKind::Fenced(lang) => lang.to_string(),
                    CodeBlockKind::Indented => String::new(),
                };
            }
            Event::End(TagEnd::CodeBlock) => {
                in_code_block = false;
                lines.extend(highlight_code(&code_buffer, &code_lang, &palette, width));
                code_buffer.clear();
            }
            Event::Start(Tag::Strong) => {
                current_spans.push(Span::styled(
                    "",
                    Style::default().add_modifier(Modifier::BOLD),
                ));
            }
            Event::End(TagEnd::Strong) => {}
            Event::Start(Tag::Emphasis) => {
                current_spans.push(Span::styled(
                    "",
                    Style::default().add_modifier(Modifier::ITALIC),
                ));
            }
            Event::End(TagEnd::Emphasis) => {}
            Event::Start(Tag::List(_)) => {
                flush_line(&mut lines, &mut current_spans);
            }
            Event::Start(Tag::Item) => {
                current_spans.push(Span::styled("• ", Style::default().fg(palette.accent)));
            }
            Event::End(TagEnd::Item) => {
                flush_line(&mut lines, &mut current_spans);
            }
            Event::Code(code) => {
                current_spans.push(Span::styled(
                    code.to_string(),
                    Style::default()
                        .fg(Color::Rgb(180, 220, 180))
                        .bg(Color::Rgb(30, 35, 40)),
                ));
            }
            Event::Text(text) => {
                if in_code_block {
                    code_buffer.push_str(&text);
                } else {
                    current_spans.push(Span::styled(
                        text.to_string(),
                        Style::default().fg(palette.foreground),
                    ));
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                flush_line(&mut lines, &mut current_spans);
            }
            _ => {}
        }
    }

    flush_line(&mut lines, &mut current_spans);
    lines
}

fn highlight_code(
    code: &str,
    lang: &str,
    palette: &UiPalette,
    _width: usize,
) -> Vec<Line<'static>> {
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    let syntax = ps
        .find_syntax_by_token(lang)
        .or_else(|| ps.find_syntax_by_extension(lang))
        .unwrap_or_else(|| ps.find_syntax_plain_text());

    let theme = &ts.themes["base16-ocean.dark"];
    let mut h = HighlightLines::new(syntax, theme);

    let mut lines = Vec::new();
    for line in LinesWithEndings::from(code) {
        let ranges = h.highlight_line(line, &ps).unwrap_or_default();
        let mut spans = Vec::new();
        for (style, text) in ranges {
            let fg = style.foreground;
            spans.push(Span::styled(
                text.to_string(),
                Style::default()
                    .fg(Color::Rgb(fg.r, fg.g, fg.b))
                    .bg(Color::Rgb(20, 25, 35)),
            ));
        }
        if spans.is_empty() {
            spans.push(Span::styled(
                line.to_string(),
                Style::default()
                    .fg(palette.muted)
                    .bg(Color::Rgb(20, 25, 35)),
            ));
        }
        lines.push(Line::from(spans));
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            " ",
            Style::default().bg(Color::Rgb(20, 25, 35)),
        )));
    }

    lines
}
