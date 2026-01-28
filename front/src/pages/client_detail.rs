use gloo::timers::callback::Interval;
use gloo::utils::document;
use std::rc::Rc;
use yew::create_portal;
use yew::prelude::*;

use crate::components::error::ErrorDisplay;
use crate::components::hardware_history::HardwareHistory;
use crate::components::hardware_info::HardwareInfo;
use crate::components::loading::Loading;
use crate::hooks::use_trans::use_trans;
use crate::services::api::{self, ApiError};
use crate::types::{Client, ClientStatus, Environment, Hardware, Person, Project};
use crate::utils::format::{format_datetime, format_time_ago};
use wasm_bindgen_futures::spawn_local;

use crate::components::client_edit_modal::ClientEditModal;
use crate::components::permission_guard::PermissionGuard;
use common::entity::user::Role;

use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::card::{Card, CardContent, CardHeader, CardTitle};
use lucide_yew::{
    Activity, Box as BoxIcon, Calendar, CircleQuestionMark, Clock, Cpu, Folder, History, Info,
    MapPin, Pencil, Power, RefreshCw, Server, Tag, TriangleAlert, User,
};

#[derive(PartialEq, Clone, Copy)]
pub enum ClientDetailTab {
    Overview,
    Hardware,
    HardwareHistory,
}

#[allow(dead_code)]
pub enum ClientDetailAction {
    SetClient(Client),
    SetHardware(Hardware),
    SetPersons(Vec<Person>),
    SetProjects(Vec<Project>),
    SetLoading(bool),
    SetHardwareLoading(bool),
    SetError(Option<String>),
    SetHardwareError(Option<String>),
    ChangeTab(ClientDetailTab),
    RefreshClient,
    RefreshHardware,
    ToggleEditModal(bool),
}

pub struct ClientDetailState {
    client: Option<Client>,
    hardware: Option<Hardware>,
    persons: Vec<Person>,
    projects: Vec<Project>,
    loading: bool,
    hardware_loading: bool,
    error: Option<String>,
    hardware_error: Option<String>,
    active_tab: ClientDetailTab,
    show_edit_modal: bool,
}

impl Default for ClientDetailState {
    fn default() -> Self {
        Self {
            client: None,
            hardware: None,
            persons: Vec::new(),
            projects: Vec::new(),
            loading: true,
            hardware_loading: true,
            error: None,
            hardware_error: None,
            active_tab: ClientDetailTab::Overview,
            show_edit_modal: false,
        }
    }
}

impl Reducible for ClientDetailState {
    type Action = ClientDetailAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            ClientDetailAction::SetClient(client) => Rc::new(Self {
                client: Some(client),
                hardware: self.hardware.clone(),
                persons: self.persons.clone(),
                projects: self.projects.clone(),
                loading: false,
                hardware_loading: self.hardware_loading,
                error: None,
                hardware_error: self.hardware_error.clone(),
                active_tab: self.active_tab,
                show_edit_modal: self.show_edit_modal,
            }),
            ClientDetailAction::SetHardware(hardware) => Rc::new(Self {
                client: self.client.clone(),
                hardware: Some(hardware),
                persons: self.persons.clone(),
                projects: self.projects.clone(),
                loading: self.loading,
                hardware_loading: false,
                error: self.error.clone(),
                hardware_error: None,
                active_tab: self.active_tab,
                show_edit_modal: self.show_edit_modal,
            }),
            ClientDetailAction::SetPersons(persons) => Rc::new(Self {
                client: self.client.clone(),
                hardware: self.hardware.clone(),
                persons,
                projects: self.projects.clone(),
                loading: self.loading,
                hardware_loading: self.hardware_loading,
                error: self.error.clone(),
                hardware_error: self.hardware_error.clone(),
                active_tab: self.active_tab,
                show_edit_modal: self.show_edit_modal,
            }),
            ClientDetailAction::SetProjects(projects) => Rc::new(Self {
                client: self.client.clone(),
                hardware: self.hardware.clone(),
                persons: self.persons.clone(),
                projects,
                loading: self.loading,
                hardware_loading: self.hardware_loading,
                error: self.error.clone(),
                hardware_error: self.hardware_error.clone(),
                active_tab: self.active_tab,
                show_edit_modal: self.show_edit_modal,
            }),
            ClientDetailAction::SetLoading(loading) => Rc::new(Self {
                client: self.client.clone(),
                hardware: self.hardware.clone(),
                persons: self.persons.clone(),
                projects: self.projects.clone(),
                loading,
                hardware_loading: self.hardware_loading,
                error: if loading { None } else { self.error.clone() },
                hardware_error: self.hardware_error.clone(),
                active_tab: self.active_tab,
                show_edit_modal: self.show_edit_modal,
            }),
            ClientDetailAction::SetHardwareLoading(loading) => Rc::new(Self {
                client: self.client.clone(),
                hardware: self.hardware.clone(),
                persons: self.persons.clone(),
                projects: self.projects.clone(),
                loading: self.loading,
                hardware_loading: loading,
                error: self.error.clone(),
                hardware_error: if loading {
                    None
                } else {
                    self.hardware_error.clone()
                },
                active_tab: self.active_tab,
                show_edit_modal: self.show_edit_modal,
            }),
            ClientDetailAction::SetError(error) => Rc::new(Self {
                client: self.client.clone(),
                hardware: self.hardware.clone(),
                persons: self.persons.clone(),
                projects: self.projects.clone(),
                loading: false,
                hardware_loading: self.hardware_loading,
                error,
                hardware_error: self.hardware_error.clone(),
                active_tab: self.active_tab,
                show_edit_modal: self.show_edit_modal,
            }),
            ClientDetailAction::SetHardwareError(error) => Rc::new(Self {
                client: self.client.clone(),
                hardware: self.hardware.clone(),
                persons: self.persons.clone(),
                projects: self.projects.clone(),
                loading: self.loading,
                hardware_loading: false,
                error: self.error.clone(),
                hardware_error: error,
                active_tab: self.active_tab,
                show_edit_modal: self.show_edit_modal,
            }),
            ClientDetailAction::ChangeTab(tab) => Rc::new(Self {
                client: self.client.clone(),
                hardware: self.hardware.clone(),
                persons: self.persons.clone(),
                projects: self.projects.clone(),
                loading: self.loading,
                hardware_loading: self.hardware_loading,
                error: self.error.clone(),
                hardware_error: self.hardware_error.clone(),
                active_tab: tab,
                show_edit_modal: self.show_edit_modal,
            }),
            ClientDetailAction::RefreshClient => Rc::new(Self {
                client: self.client.clone(),
                hardware: self.hardware.clone(),
                persons: self.persons.clone(),
                projects: self.projects.clone(),
                loading: true,
                hardware_loading: self.hardware_loading,
                error: None,
                hardware_error: self.hardware_error.clone(),
                active_tab: self.active_tab,
                show_edit_modal: self.show_edit_modal,
            }),
            ClientDetailAction::RefreshHardware => Rc::new(Self {
                client: self.client.clone(),
                hardware: self.hardware.clone(),
                persons: self.persons.clone(),
                projects: self.projects.clone(),
                loading: self.loading,
                hardware_loading: true,
                error: self.error.clone(),
                hardware_error: None,
                active_tab: self.active_tab,
                show_edit_modal: self.show_edit_modal,
            }),
            ClientDetailAction::ToggleEditModal(show) => Rc::new(Self {
                client: self.client.clone(),
                hardware: self.hardware.clone(),
                persons: self.persons.clone(),
                projects: self.projects.clone(),
                loading: self.loading,
                hardware_loading: self.hardware_loading,
                error: self.error.clone(),
                hardware_error: self.hardware_error.clone(),
                active_tab: self.active_tab,
                show_edit_modal: show,
            }),
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct ClientDetailPageProps {
    pub client_id: String,
}

#[function_component(ClientDetailPage)]
pub fn client_detail_page(props: &ClientDetailPageProps) -> Html {
    let t = use_trans();
    let state = use_reducer(ClientDetailState::default);

    // Timer for auto-refresh
    let state_for_interval = state.clone();
    use_effect_with((), move |_| {
        let interval = Interval::new(60000, move || {
            state_for_interval.dispatch(ClientDetailAction::RefreshClient);
            if state_for_interval.active_tab == ClientDetailTab::Hardware {
                state_for_interval.dispatch(ClientDetailAction::RefreshHardware);
            }
        });
        || drop(interval)
    });

    // Fetch client data
    let state_load_client = state.clone();
    let client_id_load_client = props.client_id.clone();
    use_effect_with(state.loading, move |loading| {
        if *loading {
            api::get_client(
                client_id_load_client.clone(),
                Callback::from(move |result: Result<Client, ApiError>| match result {
                    Ok(client) => state_load_client.dispatch(ClientDetailAction::SetClient(client)),
                    Err(err) => {
                        state_load_client.dispatch(ClientDetailAction::SetError(Some(err.message)))
                    }
                }),
            );
        }
        || ()
    });

    // Fetch persons and projects
    let state_load_meta = state.clone();
    use_effect_with((), move |_| {
        let state = state_load_meta.clone();
        spawn_local(async move {
            if let Ok(result) = api::fetch_persons(1, 1000, None, None).await {
                state.dispatch(ClientDetailAction::SetPersons(result.items));
            }
        });
        let state = state_load_meta.clone();
        spawn_local(async move {
            if let Ok(result) = api::fetch_projects(1, 1000, None, None).await {
                state.dispatch(ClientDetailAction::SetProjects(result.items));
            }
        });
        || ()
    });

    // Fetch hardware data when tab is active and loading is true
    let state_load_hw = state.clone();
    let client_id_load_hw = props.client_id.clone();
    use_effect_with(
        (state.hardware_loading, state.active_tab),
        move |(loading, tab)| {
            if *loading && *tab == ClientDetailTab::Hardware {
                api::get_hardware_info(
                    client_id_load_hw.clone(),
                    Callback::from(move |result: Result<Hardware, ApiError>| match result {
                        Ok(hardware) => {
                            state_load_hw.dispatch(ClientDetailAction::SetHardware(hardware))
                        }
                        Err(err) => state_load_hw
                            .dispatch(ClientDetailAction::SetHardwareError(Some(err.message))),
                    }),
                );
            }
            || ()
        },
    );

    let on_refresh_client = {
        let state = state.clone();
        Callback::from(move |_: MouseEvent| {
            state.dispatch(ClientDetailAction::RefreshClient);
        })
    };

    let _on_refresh_hardware = {
        let state = state.clone();
        Callback::from(move |_: MouseEvent| {
            state.dispatch(ClientDetailAction::RefreshHardware);
        })
    };

    let on_retry_client = {
        let state = state.clone();
        Callback::from(move |_| {
            state.dispatch(ClientDetailAction::RefreshClient);
        })
    };

    let on_retry_hardware = {
        let state = state.clone();
        Callback::from(move |_| {
            state.dispatch(ClientDetailAction::RefreshHardware);
        })
    };

    let change_tab = {
        let state = state.clone();
        Callback::from(move |tab: ClientDetailTab| {
            state.dispatch(ClientDetailAction::ChangeTab(tab));
            if tab == ClientDetailTab::Hardware && state.hardware.is_none() {
                // Also trigger load if hardware not yet loaded
                state.dispatch(ClientDetailAction::SetHardwareLoading(true));
            }
        })
    };

    let on_edit_click = {
        let state = state.clone();
        Callback::from(move |_| {
            gloo::console::log!("Edit button clicked");
            state.dispatch(ClientDetailAction::ToggleEditModal(true));
        })
    };

    let on_close_edit_modal = {
        let state = state.clone();
        Callback::from(move |_| {
            gloo::console::log!("Closing edit modal");
            state.dispatch(ClientDetailAction::ToggleEditModal(false));
        })
    };

    let on_save_client = {
        let state = state.clone();
        let t = t.clone();
        Callback::from(move |updated_client: Client| {
            let state = state.clone();
            let t = t.clone();
            spawn_local(async move {
                match api::update_client(&updated_client.id, &updated_client).await {
                    Ok(client) => {
                        state.dispatch(ClientDetailAction::SetClient(client));
                        state.dispatch(ClientDetailAction::ToggleEditModal(false));
                        gloo::dialogs::alert(&t.t("client_detail.update_success"));
                    }
                    Err(e) => gloo::dialogs::alert(
                        &t.t("client_detail.update_failed")
                            .replace("{}", &e.message)
                            .to_string(),
                    ),
                }
            });
        })
    };

    let render_client_overview = {
        let t = t.clone();
        let state = state.clone();
        move |client: &Client| -> Html {
            let registered_dt = client
                .registered_at
                .as_ref()
                .map_or(t.t("unknown"), |time| format_datetime(time));
            let last_seen_dt = client
                .last_seen
                .as_ref()
                .map_or(t.t("unknown"), |time| format_datetime(time));
            let last_seen_ago = client
                .last_seen
                .as_ref()
                .map_or(t.t("unknown"), |time| format_time_ago(time));

            let is_online = client
                .last_seen
                .as_ref()
                .and_then(|time| chrono::DateTime::parse_from_rfc3339(time).ok())
                .map(|dt| {
                    chrono::Utc::now()
                        .signed_duration_since(dt.with_timezone(&chrono::Utc))
                        .num_minutes()
                        <= 5
                })
                .unwrap_or(false);

            let online_status_badge = if is_online {
                html! { <Badge variant={BadgeVariant::Outline} class="ml-2 bg-emerald-500/10 text-emerald-500 border-emerald-500/20 animate-pulse">{t.t("online")}</Badge> }
            } else {
                html! { <Badge variant={BadgeVariant::Secondary} class="ml-2">{t.t("offline")}</Badge> }
            };

            let owner_name = client
                .owner_id
                .as_ref()
                .and_then(|id| state.persons.iter().find(|p| &p.id == id))
                .map(|p| p.name.clone())
                .unwrap_or_else(|| client.owner_id.clone().unwrap_or_else(|| "-".to_string()));

            let project_name = client
                .project_id
                .as_ref()
                .and_then(|id| state.projects.iter().find(|p| &p.id == id))
                .map(|p| p.name.clone())
                .unwrap_or_else(|| client.project_id.clone().unwrap_or_else(|| "-".to_string()));

            html! {
                <Card>
                    <CardHeader>
                        <div class="flex justify-between items-center">
                            <div class="flex items-center">
                                <CardTitle>{t.t("client_detail.basic_info")}</CardTitle>
                                { online_status_badge }
                            </div>
                            <div class="flex gap-2">
                                <PermissionGuard min_role={Role::User}>
                                    <Button variant={ButtonVariant::Outline} size={ButtonSize::Sm} onclick={on_edit_click.clone()}>
                                        <Pencil class="h-4 w-4 mr-2" />
                                        <span>{t.t("client_detail.edit")}</span>
                                    </Button>
                                </PermissionGuard>
                                <Button variant={ButtonVariant::Outline} size={ButtonSize::Sm} onclick={on_refresh_client.clone()}>
                                    <RefreshCw class="h-4 w-4 mr-2" />
                                    <span>{t.t("client_detail.refresh")}</span>
                                </Button>
                            </div>
                        </div>
                    </CardHeader>
                    <CardContent>
                        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                            <div>
                                <div class="space-y-4">
                                    <div class="flex justify-between border-b border-border py-2">
                                        <span class="font-medium flex items-center"><Server class="h-4 w-4 mr-2 text-muted-foreground" />{t.t("client_detail.id") + ":"}</span>
                                        <span class="text-right font-mono text-sm">{ &client.id }</span>
                                    </div>
                                    <div class="flex justify-between border-b border-border py-2">
                                        <span class="font-medium flex items-center"><Activity class="h-4 w-4 mr-2 text-muted-foreground" />{t.t("client_detail.hostname") + ":"}</span>
                                        <span class="text-right">{ &client.hostname }</span>
                                    </div>
                                    <div class="flex justify-between border-b border-border py-2">
                                        <span class="font-medium flex items-center"><Server class="h-4 w-4 mr-2 text-muted-foreground" />{t.t("client_detail.ip") + ":"}</span>
                                        <span class="text-right">{ &client.ip_address }</span>
                                    </div>
                                    <div class="flex justify-between border-b border-border py-2">
                                        <span class="font-medium flex items-center"><BoxIcon class="h-4 w-4 mr-2 text-muted-foreground" />{t.t("client_detail.os") + ":"}</span>
                                        <span class="text-right">{ client.os.clone().unwrap_or_else(|| t.t("unknown")) }</span>
                                    </div>
                                    <div class="flex justify-between border-b border-border py-2">
                                        <span class="font-medium flex items-center"><BoxIcon class="h-4 w-4 mr-2 text-muted-foreground" />{t.t("client_detail.kernel") + ":"}</span>
                                        <span class="text-right">{ client.kernel_version.clone().unwrap_or_else(|| t.t("unknown")) }</span>
                                    </div>
                                    <div class="flex justify-between border-b border-border py-2">
                                        <span class="font-medium flex items-center"><MapPin class="h-4 w-4 mr-2 text-muted-foreground" />{t.t("client_detail.location") + ":"}</span>
                                        <span class="text-right">{ client.location.clone().unwrap_or_else(|| "-".to_string()) }</span>
                                    </div>
                                    <div class="flex justify-between border-b border-border py-2">
                                        <span class="font-medium flex items-center"><Server class="h-4 w-4 mr-2 text-muted-foreground" />{t.t("client_detail.rack") + ":"}</span>
                                        <span class="text-right">{ client.rack.clone().unwrap_or_else(|| "-".to_string()) }</span>
                                    </div>
                                    <div class="flex justify-between border-b border-border py-2">
                                        <span class="font-medium flex items-center"><Server class="h-4 w-4 mr-2 text-muted-foreground" />{t.t("client_detail.unit_position") + ":"}</span>
                                        <span class="text-right">{ client.unit_position.clone().unwrap_or_else(|| "-".to_string()) }</span>
                                    </div>
                                    <div class="flex justify-between border-b border-border py-2">
                                        <span class="font-medium flex items-center"><Server class="h-4 w-4 mr-2 text-muted-foreground" />{t.t("client_detail.u_height") + ":"}</span>
                                        <span class="text-right">{ client.u_height.map(|h| h.to_string()).unwrap_or_else(|| "1".to_string()) }</span>
                                    </div>
                                    <div class="flex justify-between border-b border-border py-2">
                                        <span class="font-medium flex items-center"><Power class="h-4 w-4 mr-2 text-muted-foreground" />{t.t("client_detail.power") + ":"}</span>
                                        <span class="text-right">{ client.power_consumption.map(|p| p.to_string()).unwrap_or_else(|| "-".to_string()) }</span>
                                    </div>
                                    <div class="flex justify-between border-b border-border py-2">
                                        <span class="font-medium flex items-center"><User class="h-4 w-4 mr-2 text-muted-foreground" />{t.t("client_detail.owner") + ":"}</span>
                                        <span class="text-right">{ owner_name }</span>
                                    </div>
                                    <div class="flex justify-between border-b border-border py-2">
                                        <span class="font-medium flex items-center"><Folder class="h-4 w-4 mr-2 text-muted-foreground" />{t.t("client_detail.project") + ":"}</span>
                                        <span class="text-right">{ project_name }</span>
                                    </div>
                                </div>
                            </div>
                            <div>
                                <div class="space-y-4">
                                    <div class="flex justify-between border-b border-border py-2">
                                        <span class="font-medium flex items-center"><Tag class="h-4 w-4 mr-2 text-muted-foreground" />{t.t("client_detail.serial") + ":"}</span>
                                        <span class="text-right">{ client.serial_number.clone().unwrap_or_else(|| t.t("unknown")) }</span>
                                    </div>
                                    <div class="flex justify-between border-b border-border py-2">
                                        <span class="font-medium flex items-center"><Tag class="h-4 w-4 mr-2 text-muted-foreground" />{t.t("client_detail.asset_tag") + ":"}</span>
                                        <span class="text-right">{ client.asset_tag.clone().unwrap_or_else(|| "-".to_string()) }</span>
                                    </div>
                                    <div class="flex justify-between border-b border-border py-2">
                                        <span class="font-medium flex items-center"><Calendar class="h-4 w-4 mr-2 text-muted-foreground" />{t.t("client_detail.warranty") + ":"}</span>
                                        <span class="text-right">{ client.warranty_expiration.clone().unwrap_or_else(|| "-".to_string()) }</span>
                                    </div>
                                    <div class="flex justify-between border-b border-border py-2">
                                        <span class="font-medium flex items-center"><BoxIcon class="h-4 w-4 mr-2 text-muted-foreground" />{t.t("client_detail.supplier") + ":"}</span>
                                        <span class="text-right">{ client.supplier.clone().unwrap_or_else(|| "-".to_string()) }</span>
                                    </div>
                                    <div class="flex justify-between border-b border-border py-2">
                                        <span class="font-medium flex items-center"><Clock class="h-4 w-4 mr-2 text-muted-foreground" />{t.t("client_detail.registered") + ":"}</span>
                                        <span class="text-right">{ registered_dt }</span>
                                    </div>
                                    <div class="flex justify-between border-b border-border py-2">
                                        <span class="font-medium flex items-center"><Activity class="h-4 w-4 mr-2 text-muted-foreground" />{t.t("client_detail.last_seen") + ":"}</span>
                                        <span class="text-right">{ format!("{} ({})", last_seen_dt, last_seen_ago) }</span>
                                    </div>
                                    <div class="flex justify-between border-b border-border py-2">
                                        <span class="font-medium flex items-center"><Info class="h-4 w-4 mr-2 text-muted-foreground" />{t.t("client_detail.status") + ":"}</span>
                                        <span class="text-right">
                                            {
                                                match &client.status {
                                                    Some(ClientStatus::Active) => html! { <Badge variant={BadgeVariant::Outline} class="bg-emerald-500/10 text-emerald-500 border-emerald-500/20">{t.t("client_status.active")}</Badge> },
                                                    Some(ClientStatus::Maintenance) => html! { <Badge variant={BadgeVariant::Outline} class="bg-amber-500/10 text-amber-500 border-amber-500/20">{t.t("client_status.maintenance")}</Badge> },
                                                    Some(ClientStatus::InStock) => html! { <Badge variant={BadgeVariant::Outline} class="bg-blue-500/10 text-blue-500 border-blue-500/20">{t.t("client_status.in_stock")}</Badge> },
                                                    Some(ClientStatus::Decommissioned) => html! { <Badge variant={BadgeVariant::Outline} class="bg-red-500/10 text-red-500 border-red-500/20">{t.t("client_status.decommissioned")}</Badge> },
                                                    None => html! { <span class="opacity-50">{"-"}</span> },
                                                }
                                            }
                                        </span>
                                    </div>
                                    <div class="flex justify-between border-b border-border py-2">
                                        <span class="font-medium flex items-center"><Info class="h-4 w-4 mr-2 text-muted-foreground" />{t.t("client_detail.environment") + ":"}</span>
                                        <span class="text-right">
                                            {
                                                match &client.environment {
                                                    Some(Environment::Prod) => html! { <Badge variant={BadgeVariant::Outline} class="bg-red-500/10 text-red-500 border-red-500/20">{t.t("environment.prod")}</Badge> },
                                                    Some(Environment::Staging) => html! { <Badge variant={BadgeVariant::Outline} class="bg-amber-500/10 text-amber-500 border-amber-500/20">{t.t("environment.staging")}</Badge> },
                                                    Some(Environment::Test) => html! { <Badge variant={BadgeVariant::Outline} class="bg-blue-500/10 text-blue-500 border-blue-500/20">{t.t("environment.test")}</Badge> },
                                                    Some(Environment::Dev) => html! { <Badge variant={BadgeVariant::Outline} class="bg-emerald-500/10 text-emerald-500 border-emerald-500/20">{t.t("environment.dev")}</Badge> },
                                                    None => html! { <span class="opacity-50">{"-"}</span> },
                                                }
                                            }
                                        </span>
                                    </div>
                                    <div class="flex justify-between border-b border-border py-2">
                                        <span class="font-medium flex items-center"><CircleQuestionMark class="h-4 w-4 mr-2 text-muted-foreground" />{t.t("client_detail.comment") + ":"}</span>
                                        <span class="text-right">{ client.comment.clone().unwrap_or_else(|| "-".to_string()) }</span>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </CardContent>
                </Card>
            }
        }
    };
    html! {
        <div class="p-4 space-y-6">

            // Main Content Area with Tabs
            <div class="flex flex-col gap-4">
                <div class="flex space-x-2 bg-muted/50 p-1 rounded-lg w-fit">
                    <Button
                        variant={if state.active_tab == ClientDetailTab::Overview { ButtonVariant::Default } else { ButtonVariant::Ghost }}
                        size={ButtonSize::Sm}
                        onclick={change_tab.reform(|_| ClientDetailTab::Overview)}
                    >
                        <Info class="h-4 w-4 mr-2" />{t.t("client_detail.tab_overview")}
                    </Button>
                    <Button
                        variant={if state.active_tab == ClientDetailTab::Hardware { ButtonVariant::Default } else { ButtonVariant::Ghost }}
                        size={ButtonSize::Sm}
                        onclick={change_tab.reform(|_| ClientDetailTab::Hardware)}
                    >
                        <Cpu class="h-4 w-4 mr-2" />{t.t("client_detail.tab_hardware")}
                    </Button>
                    <Button
                        variant={if state.active_tab == ClientDetailTab::HardwareHistory { ButtonVariant::Default } else { ButtonVariant::Ghost }}
                        size={ButtonSize::Sm}
                        onclick={change_tab.reform(|_| ClientDetailTab::HardwareHistory)}
                    >
                        <History class="h-4 w-4 mr-2" />{t.t("client_detail.tab_history")}
                    </Button>
                </div>

                <div class="mt-2">
                    <div class={classes!(if state.active_tab == ClientDetailTab::Overview { "block" } else { "hidden" })}>
                        {
                            if let Some(client_data) = &state.client {
                                html! {
                                    <>
                                        { render_client_overview(client_data) }
                                    </>
                                }
                            } else if state.loading { // Show loading only if client is None and loading
                                html! { <Loading message={t.t("client_detail.loading_overview")} /> }
                            } else if let Some(error_msg) = &state.error { // Show error if client is None and error exists
                                html! { <ErrorDisplay message={error_msg.clone()} on_retry={on_retry_client.clone()} /> }
                            } else {
                                html!{} // Should not happen if loading/error logic is correct
                            }
                        }
                    </div>
                    <div class={classes!(if state.active_tab == ClientDetailTab::Hardware { "block" } else { "hidden" })}>
                        {
                            if state.active_tab == ClientDetailTab::Hardware {
                                if state.hardware_loading {
                                    html! { <Loading message={t.t("client_detail.loading_hardware")} /> }
                                } else if let Some(error_msg) = &state.hardware_error {
                                    html! { <ErrorDisplay message={error_msg.clone()} on_retry={on_retry_hardware.clone()} /> }
                                } else if let Some(hw) = &state.hardware {
                                    html! { <HardwareInfo hardware={hw.clone()} /> }
                                } else {
                                    html! {
                                        <div class="flex flex-col items-center justify-center p-12 text-muted-foreground">
                                            <TriangleAlert class="h-12 w-12 mb-4 opacity-50" />
                                            <p>{t.t("client_detail.no_hardware")}</p>
                                        </div>
                                    }
                                }
                            } else {
                                html! {} // Tab not active, render nothing specific here
                            }
                        }
                    </div>
                    <div class={classes!(if state.active_tab == ClientDetailTab::HardwareHistory { "block" } else { "hidden" })}>
                        {
                            if state.active_tab == ClientDetailTab::HardwareHistory {
                                if let Some(client_data) = &state.client {
                                    html! { <HardwareHistory client_id={client_data.id.clone()} /> }
                                } else {
                                    html! { <Loading message={t.t("client_detail.loading_client")} /> }
                                }
                            } else {
                                html! {} // Tab not active, render nothing specific here
                            }
                        }
                    </div>
                </div>
            </div>

            // Modal rendered outside of tabs to avoid z-index/overflow issues
            {
                if state.show_edit_modal {
                    if let Some(client_data) = &state.client {
                        create_portal(
                            html! {
                                <ClientEditModal
                                    client={client_data.clone()}
                                    on_save={on_save_client.clone()}
                                    on_cancel={on_close_edit_modal.clone()}
                                />
                            },
                            document().body().unwrap().into()
                        )
                    } else {
                        html! {}
                    }
                } else {
                    html! {}
                }
            }
        </div>
    }
}
