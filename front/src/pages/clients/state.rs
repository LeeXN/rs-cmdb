use std::rc::Rc;
use std::collections::HashSet;
use yew::prelude::*;
use crate::types::{Client, FilterOptions, Person, Project, Rack, PaginatedResult};

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum ClientsAction {
    SetClients(PaginatedResult<Client>),
    SetPersons(Vec<Person>),
    SetProjects(Vec<Project>),
    SetRacks(Vec<Rack>),
    SetLoading(bool),
    SetError(Option<String>),
    SetSearchTerm(String),
    SetFilters(FilterState),
    SetFilterOptions(FilterOptions),
    ClearAllFilters,
    ApplyFilters,
    SetPage(usize),
    SetPageSize(usize),
    SetExporting(bool),
    ToggleSelection(String),
    SelectAll(bool),
    ClearSelection,
    SetTotalDbItems(usize),
}

// 简化筛选状态结构
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FilterState {
    pub status: String, // 在线状态: online, offline
    pub client_status: String, // 设备状态: Active, Maintenance, etc.
    pub environment: String, // 环境: Prod, Dev, etc.
    pub rack_id: String,
    pub project_id: String,
    pub owner_id: String,
    pub os: String,
    pub os_kernel: String,
    pub server_vendor: String,
    pub cpu_vendor: String,
    pub cpu_model: String,
    pub gpu_vendor: String,
    pub gpu_model: String,
    pub memory_min: String,
    pub memory_max: String,
    pub network_type: String,
    pub network_model: String,
    pub storage_type: String,
}

impl FilterState {
    pub fn is_empty(&self) -> bool {
        self.status.is_empty() &&
        self.client_status.is_empty() &&
        self.environment.is_empty() &&
        self.rack_id.is_empty() &&
        self.project_id.is_empty() &&
        self.owner_id.is_empty() &&
        self.os.is_empty() &&
        self.os_kernel.is_empty() &&
        self.server_vendor.is_empty() &&
        self.cpu_vendor.is_empty() &&
        self.cpu_model.is_empty() &&
        self.gpu_vendor.is_empty() &&
        self.gpu_model.is_empty() &&
        self.memory_min.is_empty() &&
        self.memory_max.is_empty() &&
        self.network_type.is_empty() &&
        self.network_model.is_empty() &&
        self.storage_type.is_empty()
    }

    pub fn has_api_filters(&self) -> bool {
        !self.os.is_empty() ||
        !self.os_kernel.is_empty() ||
        !self.server_vendor.is_empty() ||
        !self.cpu_vendor.is_empty() ||
        !self.cpu_model.is_empty() ||
        !self.gpu_vendor.is_empty() ||
        !self.gpu_model.is_empty() ||
        !self.memory_min.is_empty() ||
        !self.memory_max.is_empty() ||
        !self.network_type.is_empty() ||
        !self.network_model.is_empty() ||
        !self.storage_type.is_empty() ||
        !self.status.is_empty() ||
        !self.client_status.is_empty() ||
        !self.environment.is_empty() ||
        !self.rack_id.is_empty() ||
        !self.project_id.is_empty() ||
        !self.owner_id.is_empty()
    }

    pub fn clear(&mut self) {
        *self = FilterState::default();
    }
}

// 简化的主状态结构
#[derive(Debug, Clone, PartialEq)]
pub struct ClientsState {
    pub clients: Vec<Client>, // Current page items
    pub total_items: usize,
    pub total_pages: usize,
    pub persons: Vec<Person>,
    pub projects: Vec<Project>,
    pub racks: Vec<Rack>,
    pub loading: bool,
    pub error: Option<String>,
    pub search_term: String,
    pub filters: FilterState,
    pub current_page: usize,
    pub page_size: usize,
    pub filter_options: FilterOptions,
    pub exporting: bool,
    pub selected_clients: HashSet<String>,
    pub total_db_items: usize,
    pub reload_trigger: usize,
}

impl Default for ClientsState {
    fn default() -> Self {
        Self {
            clients: Vec::new(),
            total_items: 0,
            total_pages: 0,
            persons: Vec::new(),
            projects: Vec::new(),
            racks: Vec::new(),
            loading: true,
            error: None,
            search_term: String::new(),
            filters: FilterState::default(),
            current_page: 1,
            page_size: 10,
            filter_options: FilterOptions::default(),
            exporting: false,
            selected_clients: HashSet::new(),
            total_db_items: 0,
            reload_trigger: 0,
        }
    }
}

impl Reducible for ClientsState {
    type Action = ClientsAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut new_state = (*self).clone();

        match action {
            ClientsAction::SetClients(result) => {
                new_state.clients = result.items;
                new_state.total_items = result.total;
                new_state.total_pages = result.total_pages;
                new_state.current_page = result.page;
                new_state.page_size = result.page_size;
                new_state.loading = false;
                new_state.error = None;
            }
            ClientsAction::SetPersons(persons) => {
                new_state.persons = persons;
            }
            ClientsAction::SetProjects(projects) => {
                new_state.projects = projects;
            }
            ClientsAction::SetRacks(racks) => {
                new_state.racks = racks;
            }
            ClientsAction::SetLoading(loading) => {
                new_state.loading = loading;
            }
            ClientsAction::SetError(error) => {
                new_state.error = error;
                new_state.loading = false;
            }
            ClientsAction::SetSearchTerm(term) => {
                new_state.search_term = term;
                new_state.current_page = 1;
                new_state.loading = true; // Trigger reload
            }
            ClientsAction::SetFilters(filters) => {
                new_state.filters = filters;
            }
            ClientsAction::SetFilterOptions(options) => {
                new_state.filter_options = options;
            }
            ClientsAction::ClearAllFilters => {
                if !new_state.filters.is_empty() || !new_state.search_term.is_empty() || new_state.current_page != 1 {
                    new_state.search_term.clear();
                    new_state.filters.clear();
                    new_state.current_page = 1;
                    new_state.loading = true; // Trigger reload
                    new_state.error = None;
                }
            }
            ClientsAction::ApplyFilters => {
                new_state.current_page = 1;
                new_state.loading = true; // Trigger reload
                new_state.reload_trigger += 1;
            }
            ClientsAction::SetPage(page) => {
                new_state.current_page = page;
                new_state.loading = true; // Trigger reload
            }
            ClientsAction::SetPageSize(size) => {
                new_state.page_size = size;
                new_state.current_page = 1;
                new_state.loading = true; // Trigger reload
            }
            ClientsAction::SetExporting(exporting) => {
                new_state.exporting = exporting;
            }
            ClientsAction::ToggleSelection(id) => {
                if new_state.selected_clients.contains(&id) {
                    new_state.selected_clients.remove(&id);
                } else {
                    new_state.selected_clients.insert(id);
                }
            }
            ClientsAction::SelectAll(select) => {
                if select {
                    // Select all visible clients
                    new_state.selected_clients = new_state.clients.iter().map(|c| c.id.clone()).collect();
                } else {
                    new_state.selected_clients.clear();
                }
            }
            ClientsAction::ClearSelection => {
                new_state.selected_clients.clear();
            }
            ClientsAction::SetTotalDbItems(total) => {
                new_state.total_db_items = total;
            }
        }

        Rc::new(new_state)
    }
}