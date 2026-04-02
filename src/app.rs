use anyhow::Result;
use ratatui::widgets::TableState;

use crate::flatpak::{FlatpakApp, get_installed_apps, get_installed_runtimes, get_updates, update_all, uninstall_app, update_app};

use std::sync::{Arc, Mutex};
use std::collections::HashSet;

#[derive(PartialEq)]
pub enum Tab {
    UpToDate,
    Updates,
    Runtimes,
}

pub enum InputMode {
    Normal,
    Search,
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
        };
        
        if self.search_query.is_empty() {
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
            Tab::Runtimes => Tab::UpToDate,
        };
        self.table_state.select(Some(0));
    }

    pub fn previous_tab(&mut self) {
        self.current_tab = match self.current_tab {
            Tab::UpToDate => Tab::Runtimes,
            Tab::Updates => Tab::UpToDate,
            Tab::Runtimes => Tab::Updates,
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
    
    pub async fn uninstall_selected(&mut self) -> Result<()> {
        let list = self.get_current_list();
        if let Some(i) = self.table_state.selected() {
            if i < list.len() {
                let id = list[i].application_id.clone();
                match uninstall_app(&id).await {
                    Ok(_) => {
                        self.refresh_data().await?;
                    }
                    Err(e) => {
                        self.error = Some(format!("Failed to uninstall: {}", e));
                    }
                }
            }
        }
        Ok(())
    }
    
    pub async fn update_selected(&mut self) -> Result<()> {
        let list = self.get_current_list();
        if let Some(i) = self.table_state.selected() {
            if i < list.len() {
                let id = list[i].application_id.clone();
                match update_app(&id).await {
                    Ok(_) => {
                        self.refresh_data().await?;
                    }
                    Err(e) => {
                        self.error = Some(format!("Failed to update: {}", e));
                    }
                }
            }
        }
        Ok(())
    }
}
