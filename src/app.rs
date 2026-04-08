use anyhow::Result;
use ratatui::widgets::TableState;
use ratatui::crossterm::event::KeyCode;

use crate::flatpak::{FlatpakApp, get_installed_apps, get_installed_runtimes, get_updates};

use std::sync::{Arc, Mutex};
use std::collections::HashSet;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Tab {
    UpToDate,
    Updates,
    Runtimes,
    Discover,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum InputMode {
    Normal,
    Search,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ViewMode {
    List,
    Permissions,
}

pub struct App {
    pub current_tab: Tab,
    pub up_to_date_apps: Vec<FlatpakApp>,
    pub updates: Vec<FlatpakApp>,
    pub runtimes: Vec<FlatpakApp>,
    pub table_state: TableState,
    pub error: Option<String>,
    pub loading: bool,
    pub input_mode: InputMode,
    pub search_query: String,
    pub fetched_details: Arc<Mutex<Option<(String, String)>>>,
    pub details_info: Option<String>,
    pub current_info_id: Option<String>,
    pub show_help: bool,
    pub status_message: Option<String>,
    pub background_task_completed: Arc<std::sync::Mutex<Option<Result<(), String>>>>,
    pub discover_results: Vec<FlatpakApp>,
    pub pending_search_results: Arc<std::sync::Mutex<Option<Vec<FlatpakApp>>>>,
    pub installed_ids: HashSet<String>,
    pub view_mode: ViewMode,
    pub permissions: Vec<(String, bool)>,
    pub permissions_state: TableState,
    pub pending_permissions: Arc<Mutex<Option<(String, Vec<(String, bool)>)>>>,
}

impl App {
    pub fn new() -> App {
        let mut table_state = TableState::default();
        table_state.select(Some(0));
        App {
            current_tab: Tab::UpToDate,
            up_to_date_apps: vec![],
            updates: vec![],
            runtimes: vec![],
            table_state,
            error: None,
            loading: false,
            input_mode: InputMode::Normal,
            search_query: String::new(),
            fetched_details: Arc::new(Mutex::new(None)),
            details_info: None,
            current_info_id: None,
            show_help: false,
            status_message: None,
            background_task_completed: Arc::new(std::sync::Mutex::new(None)),
            discover_results: vec![],
            pending_search_results: Arc::new(std::sync::Mutex::new(None)),
            installed_ids: HashSet::new(),
            view_mode: ViewMode::List,
            permissions: vec![],
            permissions_state: TableState::default(),
            pending_permissions: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn refresh_data(&mut self) -> Result<()> {
        self.loading = true;
        
        let installed = get_installed_apps().await.unwrap_or_default();
        match get_updates().await {
            Ok(updates) => {
                let update_ids: HashSet<_> = updates.iter().map(|u| u.application_id.clone()).collect();
                self.updates = updates;
                self.up_to_date_apps = installed.into_iter()
                    .filter(|a| !update_ids.contains(&a.application_id))
                    .collect();
            }
            Err(e) => {
                self.error = Some(format!("Failed to load updates: {}", e));
                self.up_to_date_apps = installed;
            }
        }
        
        match get_installed_runtimes().await {
            Ok(runtimes) => {
                self.runtimes = runtimes;
            }
            Err(e) => {
                self.error = Some(format!("Failed to parse runtimes: {}", e));
            }
        }

        // Update installed_ids set
        self.installed_ids.clear();
        for app in &self.up_to_date_apps {
            self.installed_ids.insert(app.application_id.clone());
        }
        for app in &self.updates {
            self.installed_ids.insert(app.application_id.clone());
        }
        for runtime in &self.runtimes {
            self.installed_ids.insert(runtime.application_id.clone());
        }

        self.loading = false;
        
        // Reset selection bounds
        let len = self.get_current_list_len();
        if len > 0 {
            if let Some(selected) = self.table_state.selected() {
                if selected >= len {
                    self.table_state.select(Some(len - 1));
                }
            } else {
                self.table_state.select(Some(0));
            }
        } else {
            self.table_state.select(None);
        }
        
        Ok(())
    }

    pub fn is_installed(&self, id: &str) -> bool {
        self.installed_ids.contains(id)
    }

    pub fn get_current_list_len(&self) -> usize {
        self.get_current_list().len()
    }

    pub fn get_selected_id(&self) -> Option<String> {
        let list = self.get_current_list();
        self.table_state.selected().and_then(|i| list.get(i).map(|a| a.application_id.clone()))
    }

    pub fn get_current_list(&self) -> Vec<FlatpakApp> {
        let list = match self.current_tab {
            Tab::UpToDate => &self.up_to_date_apps,
            Tab::Updates => &self.updates,
            Tab::Runtimes => &self.runtimes,
            Tab::Discover => &self.discover_results,
        };

        if self.search_query.is_empty() || self.current_tab == Tab::Discover {
            list.iter().cloned().collect()
        } else {
            let q = self.search_query.to_lowercase();
            list.iter().filter(|app| {
                app.name.to_lowercase().contains(&q) || app.application_id.to_lowercase().contains(&q)
            }).cloned().collect()
        }
    }

    pub fn next_tab(&mut self) {
        self.current_tab = match self.current_tab {
            Tab::UpToDate => Tab::Updates,
            Tab::Updates => Tab::Runtimes,
            Tab::Runtimes => Tab::Discover,
            Tab::Discover => Tab::UpToDate,
        };
        self.table_state.select(Some(0));
    }

    pub fn previous_tab(&mut self) {
        self.current_tab = match self.current_tab {
            Tab::UpToDate => Tab::Discover,
            Tab::Updates => Tab::UpToDate,
            Tab::Runtimes => Tab::Updates,
            Tab::Discover => Tab::Runtimes,
        };
        self.table_state.select(Some(0));
    }

    pub fn next_item(&mut self) {
        let len = self.get_current_list_len();
        if len == 0 {
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => {
                if i >= len - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    pub fn previous_item(&mut self) {
        let len = self.get_current_list_len();
        if len == 0 {
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    len - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }
    
    pub fn uninstall_selected(&mut self) {
        let list = self.get_current_list();
        if let Some(i) = self.table_state.selected() {
            if i < list.len() {
                let id = list[i].application_id.clone();
                if self.status_message.is_some() { return; }
                self.status_message = Some(format!("Uninstalling {}...", id));
                let flag = self.background_task_completed.clone();
                tokio::spawn(async move {
                    let res = crate::flatpak::uninstall_app(&id).await.map_err(|e| e.to_string());
                    if let Ok(mut g) = flag.lock() { *g = Some(res); }
                });
            }
        }
    }
    
    pub fn update_selected(&mut self) {
        let list = self.get_current_list();
        if let Some(i) = self.table_state.selected() {
            if i < list.len() {
                let id = list[i].application_id.clone();
                if self.status_message.is_some() { return; }
                self.status_message = Some(format!("Updating {}...", id));
                let flag = self.background_task_completed.clone();
                tokio::spawn(async move {
                    let res = crate::flatpak::update_app(&id).await.map_err(|e| e.to_string());
                    if let Ok(mut g) = flag.lock() { *g = Some(res); }
                });
            }
        }
    }

    pub fn update_all(&mut self) {
        if self.status_message.is_some() { return; }
        self.status_message = Some("Updating all packages...".to_string());
        let flag = self.background_task_completed.clone();
        tokio::spawn(async move {
            let res = crate::flatpak::update_all().await.map_err(|e| e.to_string());
            if let Ok(mut g) = flag.lock() { *g = Some(res); }
        });
    }

    pub fn install_selected(&mut self) {
        let list = self.get_current_list();
        if let Some(i) = self.table_state.selected() {
            if i < list.len() {
                let id = list[i].application_id.clone();
                if self.status_message.is_some() { return; }
                self.status_message = Some(format!("Installing {}...", id));
                let flag = self.background_task_completed.clone();
                tokio::spawn(async move {
                    let res = crate::flatpak::install_app(&id).await.map_err(|e| e.to_string());
                    if let Ok(mut g) = flag.lock() { *g = Some(res); }
                });
            }
        }
    }

    pub fn search_remote(&mut self, query: String) {
        if self.status_message.is_some() || query.trim().is_empty() { return; }
        self.status_message = Some(format!("Searching remote for '{}'...", query));
        self.discover_results.clear();
        
        let pending = self.pending_search_results.clone();
        let flag = self.background_task_completed.clone();
        tokio::spawn(async move {
            let res = crate::flatpak::search_remote_apps(&query).await.map_err(|e| e.to_string());
            match res {
                Ok(apps) => {
                    if let Ok(mut g) = pending.lock() { *g = Some(apps); }
                    if let Ok(mut g) = flag.lock() { *g = Some(Ok(())); }
                }
                Err(e) => {
                    if let Ok(mut g) = flag.lock() { *g = Some(Err(e)); }
                }
            }
        });
    }

    pub fn toggle_permissions_view(&mut self) {
        if self.view_mode == ViewMode::List {
            if let Some(id) = self.get_selected_id() {
                if self.is_installed(&id) {
                    self.view_mode = ViewMode::Permissions;
                    self.status_message = Some(format!("Loading permissions for {}...", id));
                    let flag = self.background_task_completed.clone();
                    let pending = self.pending_permissions.clone();
                    
                    let id_clone = id.clone();
                    tokio::spawn(async move {
                        let res = crate::flatpak::get_app_permissions(&id_clone).await;
                        match res {
                            Ok(perms) => {
                                if let Ok(mut g) = pending.lock() {
                                    *g = Some((id_clone, perms));
                                }
                                if let Ok(mut g) = flag.lock() {
                                    *g = Some(Ok(()));
                                }
                            }
                            Err(e) => {
                                if let Ok(mut g) = flag.lock() {
                                    *g = Some(Err(e.to_string()));
                                }
                            }
                        }
                    });
                }
            }
        } else {
            self.view_mode = ViewMode::List;
            self.permissions.clear();
            self.permissions_state.select(Some(0));
            self.error = None;
            self.status_message = None;
        }
    }

    pub fn handle_normal_key(&mut self, code: KeyCode) -> Option<bool> {
        if self.show_help {
            match code {
                KeyCode::Char('q') | KeyCode::Esc | KeyCode::Char('?') => {
                    self.show_help = false;
                }
                _ => {}
            }
            return None;
        }

        if self.view_mode == ViewMode::Permissions {
            match code {
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('p') => {
                    self.toggle_permissions_view();
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let len = self.permissions.len();
                    if len > 0 {
                        let i = match self.permissions_state.selected() {
                            Some(i) => if i >= len - 1 { 0 } else { i + 1 },
                            None => 0,
                        };
                        self.permissions_state.select(Some(i));
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    let len = self.permissions.len();
                    if len > 0 {
                        let i = match self.permissions_state.selected() {
                            Some(i) => if i == 0 { len - 1 } else { i - 1 },
                            None => 0,
                        };
                        self.permissions_state.select(Some(i));
                    }
                }
                KeyCode::Char(' ') => {
                    self.toggle_selected_permission();
                }
                _ => {}
            }
            return None;
        }

        match code {
            KeyCode::Char('q') | KeyCode::Esc => return Some(true), // Exit
            KeyCode::Char('?') => self.show_help = true,
            KeyCode::Char('/') => {
                self.input_mode = InputMode::Search;
                self.search_query.clear();
            }
            KeyCode::Char('r') => {
                // Return false to indicate async refresh needed or just return None
                return Some(false); 
            }
            KeyCode::Right | KeyCode::Char('l') => self.next_tab(),
            KeyCode::Left | KeyCode::Char('h') => self.previous_tab(),
            KeyCode::Down | KeyCode::Char('j') => self.next_item(),
            KeyCode::Up | KeyCode::Char('k') => self.previous_item(),
            KeyCode::Char('x') => self.uninstall_selected(),
            KeyCode::Char('u') => self.update_selected(),
            KeyCode::Char('U') => self.update_all(),
            KeyCode::Char('i') => {
                if self.current_tab == Tab::Discover {
                    self.install_selected();
                }
            }
            KeyCode::Char('p') => self.toggle_permissions_view(),
            _ => {}
        }
        None
    }

    pub fn handle_search_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc | KeyCode::Enter => {
                if self.current_tab == Tab::Discover && code == KeyCode::Enter && !self.search_query.trim().is_empty() {
                    let q = self.search_query.clone();
                    self.search_remote(q);
                }
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Char(c) => {
                self.search_query.push(c);
                self.table_state.select(Some(0));
            }
            KeyCode::Backspace => {
                self.search_query.pop();
                self.table_state.select(Some(0));
            }
            _ => {}
        }
    }

    pub fn toggle_selected_permission(&mut self) {
        if self.view_mode != ViewMode::Permissions { return; }
        if let Some(i) = self.permissions_state.selected() {
            if i < self.permissions.len() {
                let (perm, enabled) = self.permissions[i].clone();
                let app_id = match self.get_selected_id() {
                    Some(id) => id,
                    None => return,
                };
                
                let new_state = !enabled;
                self.status_message = Some(format!("{} permission {}...", if new_state { "Enabling" } else { "Disabling" }, perm));
                
                let flag = self.background_task_completed.clone();
                tokio::spawn(async move {
                    let res = crate::flatpak::set_app_permission(&app_id, &perm, new_state).await;
                    if let Ok(mut g) = flag.lock() {
                        *g = Some(res.map_err(|e| e.to_string()));
                    }
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flatpak::FlatpakApp;
    use pretty_assertions::assert_eq;

    fn create_test_app() -> App {
        let mut app = App::new();
        app.up_to_date_apps = vec![
            FlatpakApp { 
                name: "Alpha".to_string(), 
                application_id: "org.alpha.App".to_string(), 
                ..Default::default() 
            },
            FlatpakApp { 
                name: "Beta".to_string(), 
                application_id: "org.beta.App".to_string(), 
                ..Default::default() 
            },
        ];
        app.updates = app.up_to_date_apps.clone();
        app.installed_ids.insert("org.alpha.App".to_string());
        app.installed_ids.insert("org.beta.App".to_string());
        app.table_state.select(Some(0));
        app
    }

    #[test]
    fn test_tab_navigation() {
        let mut app = App::new();
        assert_eq!(app.current_tab, Tab::UpToDate);
        app.next_tab();
        assert_eq!(app.current_tab, Tab::Updates);
        app.next_tab();
        assert_eq!(app.current_tab, Tab::Runtimes);
        app.next_tab();
        assert_eq!(app.current_tab, Tab::Discover);
        app.next_tab();
        assert_eq!(app.current_tab, Tab::UpToDate);
        
        app.previous_tab();
        assert_eq!(app.current_tab, Tab::Discover);
    }

    #[test]
    fn test_item_navigation() {
        let mut app = create_test_app();
        assert_eq!(app.table_state.selected(), Some(0));
        app.next_item();
        assert_eq!(app.table_state.selected(), Some(1));
        app.next_item(); // Should wrap
        assert_eq!(app.table_state.selected(), Some(0));
        app.previous_item(); // Should wrap
        assert_eq!(app.table_state.selected(), Some(1));
    }

    #[test]
    fn test_search_filtering() {
        let mut app = create_test_app();
        app.search_query = "alp".to_string();
        let list = app.get_current_list();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "Alpha");

        app.search_query = "nonexistent".to_string();
        assert_eq!(app.get_current_list().len(), 0);
    }

    #[test]
    fn test_get_selected_id() {
        let mut app = create_test_app();
        app.table_state.select(Some(0));
        assert_eq!(app.get_selected_id(), Some("org.alpha.App".to_string()));
        app.table_state.select(Some(1));
        assert_eq!(app.get_selected_id(), Some("org.beta.App".to_string()));
        app.table_state.select(None);
        assert_eq!(app.get_selected_id(), None);
    }

    #[test]
    fn test_is_installed() {
        let app = create_test_app();
        assert!(app.is_installed("org.alpha.App"));
        assert!(!app.is_installed("org.gamma.App"));
    }

    #[tokio::test]
    async fn test_handle_normal_key() {
        let mut app = create_test_app();
        
        // Tab navigation
        app.handle_normal_key(KeyCode::Right);
        assert_eq!(app.current_tab, Tab::Updates);
        
        // Item navigation
        app.handle_normal_key(KeyCode::Down);
        assert_eq!(app.table_state.selected(), Some(1));
        
        // Help toggle
        app.handle_normal_key(KeyCode::Char('?'));
        assert!(app.show_help);
        app.handle_normal_key(KeyCode::Char('q'));
        assert!(!app.show_help);
        
        // Mode switch to Permissions
        app.handle_normal_key(KeyCode::Char('p'));
        assert_eq!(app.view_mode, ViewMode::Permissions);
        
        // Quit intent
        assert_eq!(app.handle_normal_key(KeyCode::Char('q')), None); // In permissions mode it toggles back
        assert_eq!(app.view_mode, ViewMode::List);
        assert_eq!(app.handle_normal_key(KeyCode::Char('q')), Some(true)); // In list mode it quits
    }

    #[test]
    fn test_handle_search_key() {
        let mut app = create_test_app();
        app.input_mode = InputMode::Search;
        
        app.handle_search_key(KeyCode::Char('f'));
        app.handle_search_key(KeyCode::Char('o'));
        app.handle_search_key(KeyCode::Char('o'));
        assert_eq!(app.search_query, "foo");
        
        app.handle_search_key(KeyCode::Backspace);
        assert_eq!(app.search_query, "fo");
        
        app.handle_search_key(KeyCode::Enter);
        assert_eq!(app.input_mode, InputMode::Normal);
    }
}
