use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseEventKind},
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

        let completion = if let Ok(mut g) = app.background_task_completed.lock() {
            g.take()
        } else {
            None
        };
        
        if let Ok(mut g) = app.pending_search_results.lock() {
            if let Some(res) = g.take() {
                app.discover_results = res;
            }
        }
        
        if let Ok(mut g) = app.pending_permissions.lock() {
            if let Some((id, perms)) = g.take() {
                if Some(id) == app.get_selected_id() {
                    app.permissions = perms;
                    app.permissions_state.select(Some(0));
                }
            }
        }
        
        if let Some(res) = completion {
            app.status_message = None;
            if let Err(e) = res {
                app.error = Some(e);
            }
            app.refresh_data().await?;
        }

        terminal.draw(|f| ui::draw(f, app))?;

        // Poll for event
        if event::poll(std::time::Duration::from_millis(16))? {
            let evt = event::read()?;
            match evt {
                Event::Key(key) => {
                    match app.input_mode {
                        app::InputMode::Normal => {
                            if app.show_help {
                                match key.code {
                                    KeyCode::Char('q') | KeyCode::Esc | KeyCode::Char('?') => {
                                        app.show_help = false;
                                    }
                                    _ => {}
                                }
                            } else if app.view_mode == app::ViewMode::Permissions {
                                match key.code {
                                    KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('p') => {
                                        app.toggle_permissions_view();
                                    }
                                    KeyCode::Down | KeyCode::Char('j') => {
                                        let len = app.permissions.len();
                                        if len > 0 {
                                            let i = match app.permissions_state.selected() {
                                                Some(i) => if i >= len - 1 { 0 } else { i + 1 },
                                                None => 0,
                                            };
                                            app.permissions_state.select(Some(i));
                                        }
                                    }
                                    KeyCode::Up | KeyCode::Char('k') => {
                                        let len = app.permissions.len();
                                        if len > 0 {
                                            let i = match app.permissions_state.selected() {
                                                Some(i) => if i == 0 { len - 1 } else { i - 1 },
                                                None => 0,
                                            };
                                            app.permissions_state.select(Some(i));
                                        }
                                    }
                                    KeyCode::Char(' ') => {
                                        app.toggle_selected_permission();
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
                                        app.uninstall_selected();
                                    }
                                    KeyCode::Char('u') => {
                                        app.update_selected();
                                    }
                                    KeyCode::Char('U') => {
                                        app.update_all();
                                    }
                                    KeyCode::Char('i') => {
                                        if app.current_tab == app::Tab::Discover {
                                            app.install_selected();
                                        }
                                    }
                                    KeyCode::Char('p') => {
                                        app.toggle_permissions_view();
                                    }
                                    _ => {}
                                }
                            }
                        }
                        app::InputMode::Search => match key.code {
                            KeyCode::Esc | KeyCode::Enter => {
                                if app.current_tab == app::Tab::Discover && key.code == KeyCode::Enter && !app.search_query.trim().is_empty() {
                                    let q = app.search_query.clone();
                                    app.search_remote(q);
                                }
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
                        },
                    }
                }
                Event::Mouse(mouse) => match mouse.kind {
                    MouseEventKind::ScrollDown => app.next_item(),
                    MouseEventKind::ScrollUp => app.previous_item(),
                    MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
                        if mouse.row <= 2 {
                            if mouse.column < 15 {
                                app.current_tab = app::Tab::UpToDate;
                            } else if mouse.column < 35 {
                                app.current_tab = app::Tab::Updates;
                            } else if mouse.column < 47 {
                                app.current_tab = app::Tab::Runtimes;
                            } else {
                                app.current_tab = app::Tab::Discover;
                            }
                            app.table_state.select(Some(0));
                        } else if mouse.row >= 4 {
                            let visible_row = (mouse.row - 4) as usize;
                            let offset = app.table_state.offset();
                            let target = offset + visible_row;
                            if target < app.get_current_list().len() {
                                app.table_state.select(Some(target));
                            }
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
}
