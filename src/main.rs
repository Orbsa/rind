mod app;
mod preview;
mod ui;

use std::io;
use std::time::Duration;

use app::{App, ExitAction};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    execute,
};
use ratatui::prelude::*;

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    app.mode = app::Mode::Insert;

    let exit_action = run_app(&mut terminal, &mut app)?;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    match exit_action {
        ExitAction::Cd(path) => {
            println!("__RFIND_CD__:{}", path.display());
        }
        ExitAction::EditFile(path) => {
            use std::os::unix::process::CommandExt;
            let err = std::process::Command::new("vim").arg(&path).exec();
            eprintln!("Failed to exec vim: {err}");
        }
        ExitAction::Yazi(path) => {
            use std::os::unix::process::CommandExt;
            let err = std::process::Command::new("yazi").arg(&path).exec();
            eprintln!("Failed to exec yazi: {err}");
        }
        ExitAction::Quit => {}
    }

    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> io::Result<ExitAction> {
    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        if !event::poll(Duration::from_millis(100))? {
            continue;
        }

        let ev = event::read()?;
        let Event::Key(key) = ev else { continue };
        if key.kind != KeyEventKind::Press {
            continue;
        }

        match app.mode {
            app::Mode::Normal => match key.code {
                KeyCode::Char('q') => return Ok(ExitAction::Quit),
                KeyCode::Char('i') => app.mode = app::Mode::Insert,
                KeyCode::Char('j') => app.move_down(),
                KeyCode::Char('k') => app.move_up(),
                KeyCode::Char('h') => app.move_path_left(),
                KeyCode::Char('l') => app.move_path_right(),
                KeyCode::Char('g') => app.select_first(),
                KeyCode::Char('G') => app.select_last(),
                KeyCode::Char('y') => {
                    if let Some(action) = app.yazi_selection() {
                        return Ok(action);
                    }
                }
                KeyCode::Enter => {
                    if let Some(action) = app.enter_selection() {
                        return Ok(action);
                    }
                }
                KeyCode::Esc => return Ok(ExitAction::Quit),
                _ => {}
            },
            app::Mode::Insert => match key.code {
                KeyCode::Esc => {
                    app.mode = app::Mode::Normal;
                    // Drain any queued escape-sequence events
                    while event::poll(Duration::from_millis(10))? {
                        let _ = event::read()?;
                    }
                }
                KeyCode::Char(c) => {
                    app.query.push(c);
                    app.run_search();
                }
                KeyCode::Backspace => {
                    app.query.pop();
                    app.run_search();
                }
                KeyCode::Enter => {
                    if !app.results.is_empty() {
                        app.mode = app::Mode::Normal;
                    }
                }
                _ => {}
            },
        }
    }
}
