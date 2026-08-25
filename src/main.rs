mod ai;
mod app;
mod auth;
mod library;
mod local_ai;
mod ui;
mod wrap;

use anyhow::Result;
use app::App;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use library::Library;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::stdout;
use std::time::Duration;

fn main() -> Result<()> {
    let library = match Library::open_default() {
        Ok(lib) => lib,
        Err(e) => {
            eprintln!("Elfy (elfy)");
            eprintln!("{e}");
            std::process::exit(1);
        }
    };

    let mut app = App::new(library);
    run_tui(&mut app)?;
    Ok(())
}

fn run_tui(app: &mut App) -> Result<()> {
    enable_raw_mode()?;
    let mut out = stdout();
    execute!(out, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(out);
    let mut terminal = Terminal::new(backend)?;

    let result = loop {
        terminal.draw(|f| ui::draw(f, app))?;
        app.poll_oauth();

        if event::poll(Duration::from_millis(80))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    app.handle_key(key);
                    if app.should_quit {
                        break Ok(());
                    }
                }
                Event::Resize(_, _) => {}
                _ => {}
            }
        }
    };

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    result
}
