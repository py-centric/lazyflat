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
    draw_main(f, app, chunks[1]);
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
    let titles = vec![Line::from(" Up To Date "), Line::from(" Updates Available "), Line::from(" Runtimes ")];
    
    let tab_index = match app.current_tab {
        AppTab::UpToDate => 0,
        AppTab::Updates => 1,
        AppTab::Runtimes => 2,
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
    let list_items = match app.current_tab {
        AppTab::UpToDate => &app.up_to_date_apps,
        AppTab::Updates => &app.updates,
        AppTab::Runtimes => &app.runtimes,
    };

    let rows: Vec<Row> = list_items
        .iter()
        .map(|item| {
            let name_cell = Cell::from(item.name.as_str());
            let app_id_cell = Cell::from(item.application_id.as_str()).style(Style::default().fg(Color::DarkGray));
            let version_cell = Cell::from(item.version.as_str()).style(Style::default().fg(Color::Blue));
            
            Row::new(vec![name_cell, version_cell, app_id_cell])
        })
        .collect();

    let table = Table::new(rows, [Constraint::Percentage(40), Constraint::Length(10), Constraint::Percentage(50)])
        .block(Block::default().borders(Borders::ALL).title(" Installed "))
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
                Line::from(format!("Version: {}", item.version)),
                Line::from(format!("Branch: {}", item.branch)),
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

fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    let status_text = match app.input_mode {
        InputMode::Search => format!(" Search: {}_ ", app.search_query),
        InputMode::Normal => {
            if app.loading {
                " [Loading...] ".to_string()
            } else if let Some(ref err) = app.error {
                err.clone()
            } else {
                " q: Quit | ?: Help | j/k: Nav | h/l: Tabs | x: Uninstall | u: Update | U: Update All | r: Refresh | /: Search ".to_string()
            }
        }
    };

    let style = match app.input_mode {
        InputMode::Search => Style::default().fg(Color::Black).bg(Color::Yellow),
        InputMode::Normal => {
            if app.error.is_some() {
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
