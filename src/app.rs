use anyhow::Result;
use ratatui::widgets::TableState;

use crate::flatpak::{FlatpakApp, get_installed_apps, get_installed_runtimes, get_updates};

use std::sync::{Arc, Mutex};
use std::collections::HashSet;

#[derive(PartialEq)]
pub enum Tab {
    UpToDate,
    Updates,
    Runtimes,
    Discover,
}

pub enum InputMode {
    Normal,
    Search,
}

#[derive(PartialEq)]
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

    pub fn get_current_list(&self) -> Vec<&FlatpakApp> {
        let list = match self.current_tab {
            Tab::UpToDate => &self.up_to_date_apps,
            Tab::Updates => &self.updates,
            Tab::Runtimes => &self.runtimes,
            Tab::Discover => &self.discover_results,
        };
        
        if self.search_query.is_empty() || self.current_tab == Tab::Discover {
            list.iter().collect()
        } else {
            let q = self.search_query.to_lowercase();
            list.iter().filter(|app| {
                app.name.to_lowercase().contains(&q) || app.application_id.to_lowercase().contains(&q)
            }).collect()
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
