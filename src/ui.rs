use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::{self, App, Mode};
use crate::preview::Previewer;

use std::sync::LazyLock;

static PREVIEWER: LazyLock<Previewer> = LazyLock::new(Previewer::new);

// Truecolor palette
const ACCENT: Color = Color::Rgb(138, 180, 248);    // soft blue
const ACCENT_DIM: Color = Color::Rgb(80, 120, 180);  // dimmer blue
const HIGHLIGHT_FG: Color = Color::Rgb(30, 30, 46);   // dark fg for highlighted text
const HIGHLIGHT_BG: Color = Color::Rgb(166, 218, 149); // green highlight
const TEXT: Color = Color::Rgb(205, 214, 244);        // main text
const TEXT_DIM: Color = Color::Rgb(127, 132, 156);    // dimmed text
const SURFACE: Color = Color::Rgb(49, 50, 68);        // borders/separators
const YELLOW: Color = Color::Rgb(249, 226, 175);      // insert mode
const RED: Color = Color::Rgb(243, 139, 168);          // error/emphasis

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
        ])
        .split(f.area());

    draw_search_bar(f, app, chunks[0]);
    draw_results(f, app, chunks[1]);

    // "?" hint bottom-right
    if app.mode != Mode::Help {
        let hint = Span::styled(" ? ", Style::default().fg(TEXT_DIM));
        let area = f.area();
        let hint_area = Rect::new(area.width.saturating_sub(4), area.height.saturating_sub(1), 3, 1);
        f.render_widget(Paragraph::new(hint), hint_area);
    }

    if app.mode == Mode::Help {
        draw_help_modal(f);
    }
}

fn draw_search_bar(f: &mut Frame, app: &App, area: Rect) {
    let (indicator, indicator_style, border_color) = match app.mode {
        Mode::Insert => (
            " INSERT ",
            Style::default().fg(HIGHLIGHT_FG).bg(YELLOW).bold(),
            YELLOW,
        ),
        Mode::Normal | Mode::Help => (
            " NORMAL ",
            Style::default().fg(HIGHLIGHT_FG).bg(ACCENT).bold(),
            ACCENT,
        ),
    };

    let spans = vec![
        Span::styled(indicator, indicator_style),
        Span::styled(" > ", Style::default().fg(TEXT_DIM)),
        Span::styled(app.query.clone(), Style::default().fg(TEXT)),
    ];

    let search = Paragraph::new(Line::from(spans))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .title(Span::styled(" rind ", Style::default().fg(ACCENT).bold())),
        );

    f.render_widget(search, area);

    if app.mode == Mode::Insert {
        // cursor_x: border(1) + indicator len + " > " (3) + query len
        let cursor_x = area.x + 1 + (indicator.len() as u16) + 3 + (app.query.len() as u16);
        let cursor_y = area.y + 1;
        f.set_cursor_position((cursor_x, cursor_y));
    }
}

fn draw_results(f: &mut Frame, app: &App, area: Rect) {
    if app.results.is_empty() {
        let msg = if app.query.is_empty() {
            "Type to search with plocate"
        } else {
            "No results"
        };
        let para = Paragraph::new(msg)
            .style(Style::default().fg(TEXT_DIM))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(SURFACE))
                    .title(Span::styled(" Results ", Style::default().fg(TEXT_DIM))),
            );
        f.render_widget(para, area);
        return;
    }

    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(66),
            Constraint::Percentage(34),
        ])
        .split(area);

    draw_results_list(f, app, h_chunks[0]);
    draw_preview(f, app, h_chunks[1]);
}

fn draw_results_list(f: &mut Frame, app: &App, area: Rect) {
    let visible_height = area.height.saturating_sub(2) as usize;

    let scroll_offset = if app.selected >= visible_height {
        app.selected - visible_height + 1
    } else {
        0
    };

    let items: Vec<ListItem> = app
        .results
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(visible_height)
        .map(|(i, path)| {
            let spans = build_path_spans(path, i == app.selected, app);
            ListItem::new(Line::from(spans))
        })
        .collect();

    let title = format!(" {}/{} ", app.selected + 1, app.results.len());
    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(SURFACE))
            .title(Span::styled(title, Style::default().fg(ACCENT))),
    );

    f.render_widget(list, area);
}

fn build_path_spans<'a>(
    path: &std::path::Path,
    is_selected_row: bool,
    app: &App,
) -> Vec<Span<'a>> {
    let parts = app::path_display_parts(path);
    let count = parts.len();
    let mut spans = Vec::new();

    // Leading /
    let slash_style = if is_selected_row {
        Style::default().fg(TEXT_DIM)
    } else {
        Style::default().fg(SURFACE)
    };
    spans.push(Span::styled("/", slash_style));

    for (i, part) in parts.iter().enumerate() {
        let is_leaf = i == count - 1;
        let is_highlighted = is_selected_row
            && match app.path_segment {
                None => is_leaf,
                Some(seg) => i == seg,
            };

        let style = if is_highlighted {
            Style::default().fg(HIGHLIGHT_FG).bg(HIGHLIGHT_BG).bold()
        } else if is_selected_row && is_leaf {
            Style::default().fg(TEXT).bold()
        } else if is_selected_row {
            Style::default().fg(ACCENT_DIM)
        } else if is_leaf {
            Style::default().fg(TEXT_DIM)
        } else {
            Style::default().fg(SURFACE)
        };

        spans.push(Span::styled(part.clone(), style));

        if !is_leaf {
            spans.push(Span::styled("/", slash_style));
        }
    }

    spans
}

fn draw_preview(f: &mut Frame, app: &App, area: Rect) {
    let max_lines = area.height.saturating_sub(2) as usize;

    let (title, title_color, lines) = match app.selected_path() {
        None => ("Preview".to_string(), TEXT_DIM, vec![Line::raw("")]),
        Some(path) => {
            if path.is_dir() {
                let title = format!(" {} ", path.display());
                let lines = PREVIEWER.dir_preview(&path, max_lines);
                (title, ACCENT, lines)
            } else if path.is_file() {
                let title = format!(" {} ", path.display());
                let lines = PREVIEWER.file_preview(&path, max_lines);
                (title, YELLOW, lines)
            } else {
                let title = format!(" {} ", path.display());
                (title, RED, vec![Line::raw("[not found on disk]")])
            }
        }
    };

    let max_title_len = area.width.saturating_sub(4) as usize;
    let display_title = if title.len() > max_title_len && max_title_len > 2 {
        format!("…{}", &title[title.len() - max_title_len + 1..])
    } else {
        title
    };

    let preview = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(SURFACE))
                .title(Span::styled(display_title, Style::default().fg(title_color))),
        );

    f.render_widget(preview, area);
}

fn draw_help_modal(f: &mut Frame) {
    let area = f.area();
    let width = 40u16.min(area.width.saturating_sub(4));
    let height = 16u16.min(area.height.saturating_sub(4));
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;
    let modal_area = Rect::new(x, y, width, height);

    f.render_widget(Clear, modal_area);

    let help_lines = vec![
        Line::from(vec![
            Span::styled("i", Style::default().fg(ACCENT).bold()),
            Span::styled("         insert (search) mode", Style::default().fg(TEXT)),
        ]),
        Line::from(vec![
            Span::styled("Esc", Style::default().fg(ACCENT).bold()),
            Span::styled("       normal mode / quit", Style::default().fg(TEXT)),
        ]),
        Line::from(vec![
            Span::styled("Enter", Style::default().fg(ACCENT).bold()),
            Span::styled("     cd dir / open file in vim", Style::default().fg(TEXT)),
        ]),
        Line::from(vec![
            Span::styled("j/k  ↑/↓", Style::default().fg(ACCENT).bold()),
            Span::styled(" navigate results", Style::default().fg(TEXT)),
        ]),
        Line::from(vec![
            Span::styled("h/l  ←/→", Style::default().fg(ACCENT).bold()),
            Span::styled(" navigate path segments", Style::default().fg(TEXT)),
        ]),
        Line::from(vec![
            Span::styled("g/G", Style::default().fg(ACCENT).bold()),
            Span::styled("       first / last result", Style::default().fg(TEXT)),
        ]),
        Line::from(vec![
            Span::styled("y", Style::default().fg(ACCENT).bold()),
            Span::styled("         open in yazi", Style::default().fg(TEXT)),
        ]),
        Line::from(vec![
            Span::styled("q", Style::default().fg(ACCENT).bold()),
            Span::styled("         quit", Style::default().fg(TEXT)),
        ]),
        Line::raw(""),
        Line::from(vec![
            Span::styled("Search uses POSIX regex", Style::default().fg(TEXT_DIM)),
        ]),
        Line::raw(""),
        Line::from(vec![
            Span::styled("Press ", Style::default().fg(TEXT_DIM)),
            Span::styled("?", Style::default().fg(ACCENT).bold()),
            Span::styled(" or ", Style::default().fg(TEXT_DIM)),
            Span::styled("Esc", Style::default().fg(ACCENT).bold()),
            Span::styled(" to close", Style::default().fg(TEXT_DIM)),
        ]),
    ];

    let help = Paragraph::new(help_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(ACCENT))
            .title(Span::styled(" Keybindings ", Style::default().fg(ACCENT).bold())),
    );

    f.render_widget(help, modal_area);
}
