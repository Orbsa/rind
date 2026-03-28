use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::Path;

use syntect::easy::HighlightLines;
use syntect::highlighting::Style;
use syntect::parsing::SyntaxSet;

use ratatui::style::Color;
use ratatui::text::{Line, Span};

pub struct Previewer {
    syntax_set: SyntaxSet,
    theme: syntect::highlighting::Theme,
}

impl Previewer {
    pub fn new() -> Self {
        Self {
            syntax_set: two_face::syntax::extra_newlines(),
            theme: two_face::theme::extra()[two_face::theme::EmbeddedThemeName::Base16OceanDark].clone(),
        }
    }

    pub fn file_preview(&self, path: &Path, max_lines: usize) -> Vec<Line<'static>> {
        let file = match File::open(path) {
            Ok(f) => f,
            Err(e) => return vec![Line::raw(format!("[cannot read: {e}]"))],
        };

        // Read only the lines we need to display
        let reader = BufReader::new(file);
        let mut raw_lines: Vec<String> = Vec::with_capacity(max_lines);
        for line in reader.lines() {
            if raw_lines.len() >= max_lines {
                break;
            }
            match line {
                Ok(l) => raw_lines.push(l),
                Err(_) => {
                    // Binary or encoding error on this line
                    if raw_lines.is_empty() {
                        return vec![Line::raw("[binary file]")];
                    }
                    break;
                }
            }
        }

        if raw_lines.is_empty() {
            return vec![Line::raw("[empty file]")];
        }

        // Rejoin for syntect (it needs newlines)
        let content: String = raw_lines.iter().map(|l| format!("{l}\n")).collect();

        let syntax = self
            .syntax_set
            .find_syntax_for_file(path)
            .ok()
            .flatten()
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());

        let mut highlighter = HighlightLines::new(syntax, &self.theme);
        let mut lines: Vec<Line<'static>> = Vec::with_capacity(raw_lines.len());

        for line_str in content.lines() {
            let line_with_nl = format!("{line_str}\n");
            let ranges: Vec<(Style, &str)> = highlighter
                .highlight_line(&line_with_nl, &self.syntax_set)
                .unwrap_or_else(|_| vec![(Style::default(), line_str)]);

            let spans: Vec<Span<'static>> = ranges
                .into_iter()
                .map(|(style, text)| {
                    let fg = Color::Rgb(
                        style.foreground.r,
                        style.foreground.g,
                        style.foreground.b,
                    );
                    Span::styled(
                        text.to_string(),
                        ratatui::style::Style::default().fg(fg),
                    )
                })
                .collect();

            lines.push(Line::from(spans));
        }

        lines
    }

    pub fn dir_preview(&self, path: &Path, max_lines: usize) -> Vec<Line<'static>> {
        let entries = match fs::read_dir(path) {
            Ok(rd) => rd,
            Err(e) => return vec![Line::raw(format!("[cannot read dir: {e}]"))],
        };

        let mut items: Vec<String> = Vec::new();
        for entry in entries {
            if items.len() >= max_lines {
                break;
            }
            if let Ok(entry) = entry {
                let name = entry.file_name().to_string_lossy().to_string();
                let suffix = if entry.path().is_dir() { "/" } else { "" };
                items.push(format!("{name}{suffix}"));
            }
        }

        items.sort();

        if items.is_empty() {
            return vec![Line::raw("[empty directory]")];
        }

        items.into_iter().map(Line::raw).collect()
    }
}
