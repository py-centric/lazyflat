use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, Tabs},
    Frame,
};

use crate::app::{App, InputMode, Tab as AppTab};

pub fn draw(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),      // Header / Tabs
                Constraint::Min(10),        // Main area
                Constraint::Length(1),      // Footer
            ]
            .as_ref(),
        )
        .split(f.area());

    draw_header(f, app, chunks[0]);
    if app.view_mode == crate::app::ViewMode::Permissions {
        draw_permissions_view(f, app, chunks[1]);
    } else {
        draw_main(f, app, chunks[1]);
    }
    draw_footer(f, app, chunks[2]);

    if app.show_help {
        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(20), Constraint::Length(14), Constraint::Min(0)].as_ref())
            .split(f.area());
        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(20), Constraint::Length(40), Constraint::Min(0)].as_ref())
            .split(vertical[1]);
            
        let block = Block::default().title(" Help ").borders(Borders::ALL);
        let text = vec![
            Line::from("Keyboard Shortcuts:"),
            Line::from(""),
            Line::from(" ?    : Toggle Help"),
            Line::from(" q/Esc: Quit"),
            Line::from(" /    : Search"),
            Line::from(" j/k  : Up / Down"),
            Line::from(" h/l  : Left / Right Tabs"),
            Line::from(" x    : Uninstall selected"),
            Line::from(" u    : Update selected"),
            Line::from(" U    : Update All"),
            Line::from(" r    : Refresh list"),
        ];
        f.render_widget(Clear, horizontal[1]);
        f.render_widget(Paragraph::new(text).block(block), horizontal[1]);
    }
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let titles = vec![Line::from(" Up To Date "), Line::from(" Updates Available "), Line::from(" Runtimes "), Line::from(" Discover ")];
    
    let tab_index = match app.current_tab {
        AppTab::UpToDate => 0,
        AppTab::Updates => 1,
        AppTab::Runtimes => 2,
        AppTab::Discover => 3,
    };

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(" lazyflat "))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .select(tab_index);

    f.render_widget(tabs, area);
}

fn draw_main(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
        .split(area);

    draw_list(f, app, chunks[0]);
    draw_details(f, app, chunks[1]);
}

fn draw_list(f: &mut Frame, app: &mut App, area: Rect) {
    let list_items = app.get_current_list();

    let mut rows: Vec<Row> = list_items
        .iter()
        .map(|item| {
            let mut name_text = item.name.clone();
            let mut name_style = Style::default();
            
            if app.current_tab == AppTab::Discover && app.is_installed(&item.application_id) {
                name_text = format!("{} (installed)", item.name);
                name_style = name_style.fg(Color::Green);
            }
            
            let name_cell = Cell::from(name_text).style(name_style);
            let app_id_cell = Cell::from(item.application_id.as_str()).style(Style::default().fg(Color::DarkGray));
            let version_cell = Cell::from(item.version.as_str()).style(Style::default().fg(Color::Blue));
            
            Row::new(vec![name_cell, version_cell, app_id_cell])
        })
        .collect();

    if rows.is_empty() {
        let empty_msg = if app.loading || app.status_message.as_ref().map_or(false, |m| m.starts_with("Searching")) {
            "Searching..."
        } else if !app.search_query.is_empty() && app.current_tab == AppTab::Discover {
            "No search results found"
        } else if !app.search_query.is_empty() {
            "No search results found in this tab"
        } else if app.current_tab == AppTab::Discover {
            "Type '/' to search for packages"
        } else {
            "No items found"
        };

        rows.push(Row::new(vec![
            Cell::from(empty_msg).style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)),
            Cell::from(""),
            Cell::from(""),
        ]));
    }

    let title = if app.current_tab == AppTab::Discover { " Results " } else { " Installed " };

    let table = Table::new(rows, [Constraint::Percentage(40), Constraint::Length(10), Constraint::Percentage(50)])
        .block(Block::default().borders(Borders::ALL).title(title))
        .row_highlight_style(
            Style::default()
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(table, area, &mut app.table_state);
}

fn draw_details(f: &mut Frame, app: &App, area: Rect) {
    let list_items = app.get_current_list();
    
    let text = if let Some(i) = app.table_state.selected() {
        if i < list_items.len() {
            let item = &list_items[i];
            let mut info_lines = vec![
                Line::from(Span::styled(
                    format!("{} ({})", item.name, item.application_id),
                    Style::default().add_modifier(Modifier::BOLD),
                )),
                Line::from(format!("Version: {} | Branch: {}", item.version, item.branch)),
                Line::from(""),
                Line::from(item.description.clone()),
                Line::from(""),
            ];
            
            if let Some(ref text) = app.details_info {
                for line in text.lines() {
                    info_lines.push(Line::from(line.to_string()));
                }
            } else {
                info_lines.push(Line::from("Loading details (including size)..."));
            }
            
            info_lines.push(Line::from(""));
            info_lines.push(Line::from("(Press 'x' to uninstall, 'u' to update, '/' to search)"));
            info_lines
        } else {
            vec![Line::from("No selection")]
        }
    } else {
        vec![Line::from("No item selected")]
    };

    let block = Block::default()
        .title(" Details ")
        .borders(Borders::ALL);
        
    let paragraph = Paragraph::new(text).block(block);
    f.render_widget(paragraph, area);
}

fn draw_permissions_view(f: &mut Frame, app: &mut App, area: Rect) {
    let rows: Vec<Row> = app.permissions
        .iter()
        .map(|(perm, enabled)| {
            let status = if *enabled { "[X]" } else { "[ ]" };
            let style = if *enabled { Style::default().fg(Color::Green) } else { Style::default().fg(Color::DarkGray) };
            Row::new(vec![
                Cell::from(status).style(style),
                Cell::from(perm.as_str()),
            ])
        })
        .collect();

    let title = format!(" Permissions for {} ", app.get_selected_id().unwrap_or_default());
    
    let footer_text = vec![
        Line::from(""),
        Line::from(Span::styled("Note: Some changes may require root (system-wide apps).", Style::default().fg(Color::Yellow))),
        Line::from(Span::styled("If an error occurs, try launching lazyflat with 'sudo' or use:", Style::default().fg(Color::Gray))),
        Line::from(Span::styled(format!("  flatpak override --user --nosocket=wayland {}", app.get_selected_id().unwrap_or_default()), Style::default().fg(Color::Cyan))),
    ];
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(5)].as_ref())
        .split(area);

    let table = Table::new(rows, [Constraint::Length(5), Constraint::Min(20)])
        .block(Block::default().borders(Borders::ALL).title(title))
        .row_highlight_style(
            Style::default()
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(table, chunks[0], &mut app.permissions_state);
    f.render_widget(Paragraph::new(footer_text), chunks[1]);
}

fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    let status_text = match app.input_mode {
        InputMode::Search => format!(" Search: {}_ ", app.search_query),
        InputMode::Normal => {
            if let Some(ref msg) = app.status_message {
                format!(" [{}] ", msg)
            } else if app.loading {
                " [Loading...] ".to_string()
            } else if let Some(ref err) = app.error {
                err.clone()
            } else if app.view_mode == crate::app::ViewMode::Permissions {
                " Space: Toggle | Esc/p: Back | j/k: Nav | r: Refresh ".to_string()
            } else {
                if app.current_tab == AppTab::Discover {
                    " q: Quit | ?: Help | j/k: Nav | h/l: Tabs | i: Install | r: Refresh | /: Search ".to_string()
                } else {
                    " q: Quit | ?: Help | j/k: Nav | h/l: Tabs | p: Permissions | x: Uninstall | u: Update | U: Update All | r: Refresh | /: Search ".to_string()
                }
            }
        }
    };

    let style = match app.input_mode {
        InputMode::Search => Style::default().fg(Color::Black).bg(Color::Yellow),
        InputMode::Normal => {
            if app.status_message.is_some() {
                Style::default().fg(Color::Cyan).bg(Color::Black)
            } else if app.error.is_some() {
                Style::default().fg(Color::Red).bg(Color::Black)
            } else if app.loading {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Black).bg(Color::Green)
            }
        }
    };

    let paragraph = Paragraph::new(status_text).style(style);
    f.render_widget(paragraph, area);
}
