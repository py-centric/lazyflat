use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::io;

mod app;
mod flatpak;
mod ui;

use app::App;

#[tokio::main]
async fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run
    let mut app = App::new();
    // Pre-load flatpak data async
    app.refresh_data().await?;

    let res = run_app(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    loop {
        let selected_id = app.get_selected_id();
        if selected_id != app.current_info_id {
            app.current_info_id = selected_id.clone();
            app.details_info = None;
            if let Some(id) = selected_id {
                let arc_mutex = app.fetched_details.clone();
                tokio::spawn(async move {
                    if let Ok(info) = crate::flatpak::get_app_info(&id).await {
                        if let Ok(mut g) = arc_mutex.lock() {
                            *g = Some((id, info));
                        }
                    }
                });
            }
        }
        
        if let Ok(mut g) = app.fetched_details.try_lock() {
            if let Some((id, info)) = g.take() {
                if Some(id) == app.current_info_id {
                    app.details_info = Some(info);
                }
            }
        }

        terminal.draw(|f| ui::draw(f, app))?;

        // Poll for event
        if event::poll(std::time::Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                match app.input_mode {
                    app::InputMode::Normal => {
                        if app.show_help {
                            match key.code {
                                KeyCode::Char('q') | KeyCode::Esc | KeyCode::Char('?') => {
                                    app.show_help = false;
                                }
                                _ => {}
                            }
                        } else {
                            match key.code {
                                KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                                KeyCode::Char('?') => app.show_help = true,
                                KeyCode::Char('/') => {
                                    app.input_mode = app::InputMode::Search;
                                    app.search_query.clear();
                                }
                        KeyCode::Char('r') => {
                            app.refresh_data().await?;
                        }
                        KeyCode::Right | KeyCode::Char('l') => app.next_tab(),
                        KeyCode::Left | KeyCode::Char('h') => app.previous_tab(),
                        KeyCode::Down | KeyCode::Char('j') => app.next_item(),
                        KeyCode::Up | KeyCode::Char('k') => app.previous_item(),
                        KeyCode::Char('x') => {
                            app.uninstall_selected().await?;
                        }
                        KeyCode::Char('u') => {
                            app.update_selected().await?;
                        }
                        KeyCode::Char('U') => {
                            app.update_all().await?;
                        }
                        _ => {}
                            }
                        }
                    }
                    app::InputMode::Search => match key.code {
                        KeyCode::Esc | KeyCode::Enter => {
                            app.input_mode = app::InputMode::Normal;
                        }
                        KeyCode::Char(c) => {
                            app.search_query.push(c);
                            app.table_state.select(Some(0));
                        }
                        KeyCode::Backspace => {
                            app.search_query.pop();
                            app.table_state.select(Some(0));
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}
