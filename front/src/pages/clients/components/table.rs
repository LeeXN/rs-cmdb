use yew::prelude::*;
use yew_router::prelude::*;
use std::collections::HashSet;
use crate::routes::Route;
use crate::types::{Client, Person, Project, ClientStatus, Environment};
use crate::components::common::pagination::Pagination;
use crate::components::ui::table::{Table, TableHeader, TableBody, TableRow, TableHead, TableCell};
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonVariant, ButtonSize};
use crate::components::ui::checkbox::Checkbox;
use crate::hooks::use_trans::use_trans;
use lucide_yew::{Eye, Pencil, Trash2};
use crate::components::permission_guard::PermissionGuard;
use common::entity::user::Role;

#[derive(Properties, PartialEq)]
pub struct TableProps {
    pub clients: Vec<Client>,
    pub total_items: usize,
    pub total_pages: usize,
    pub persons: Vec<Person>,
    pub projects: Vec<Project>,
    pub current_page: usize,
    pub page_size: usize,
    pub on_page_change: Callback<usize>,
    pub on_page_size_change: Callback<usize>,
    pub selected_clients: HashSet<String>,
    pub on_toggle_selection: Callback<String>,
    pub on_select_all: Callback<bool>,
    pub on_delete: Callback<String>,
}

#[function_component(ClientsTable)]
pub fn clients_table(props: &TableProps) -> Html {
    let navigator = use_navigator();
    let t = use_trans();

    if props.clients.is_empty() {
        return html! {
            <div class="text-center py-12">
                <p class="text-muted-foreground">{t.t("clients.table.no_data")}</p>
            </div>
        };
    }
    
    let page_clients = &props.clients;
    
    // Check if all clients on the current page are selected
    let is_all_selected = !page_clients.is_empty() && page_clients.iter().all(|c| props.selected_clients.contains(&c.id));
    
    let on_select_all_click = {
        let on_select_all = props.on_select_all.clone();
        let is_all_selected = is_all_selected;
        Callback::from(move |_| {
            on_select_all.emit(!is_all_selected);
        })
    };

    html! {
        <div class="space-y-4">
            <div class="rounded-md border border-border">
                <Table>
                    <TableHeader>
                        <TableRow>
                            <TableHead class="w-12 text-center">
                                <Checkbox
                                    checked={is_all_selected}
                                    onchange={on_select_all_click}
                                />
                            </TableHead>
                            <TableHead>{t.t("clients.table.hostname")}</TableHead>
                            <TableHead>{t.t("clients.table.ip")}</TableHead>
                            <TableHead class="text-center">{t.t("clients.table.os")}</TableHead>
                            <TableHead class="text-center">{t.t("clients.table.owner")}</TableHead>
                            <TableHead class="text-center">{t.t("clients.table.project")}</TableHead>
                            <TableHead class="text-center">{t.t("clients.table.status")}</TableHead>
                            <TableHead class="text-center">{t.t("clients.table.environment")}</TableHead>
                            <TableHead class="text-right">{t.t("clients.table.actions")}</TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        {
                            page_clients.iter().map(|client| {
                                let owner_name = client.owner_id.as_ref()
                                    .and_then(|id| props.persons.iter().find(|p| &p.id == id))
                                    .map(|p| p.name.clone())
                                    .unwrap_or_else(|| "-".to_string());

                                let project_name = client.project_id.as_ref()
                                    .and_then(|id| props.projects.iter().find(|p| &p.id == id))
                                    .map(|p| p.name.clone())
                                    .unwrap_or_else(|| "-".to_string());

                                let status_display = match &client.status {
                                    Some(ClientStatus::Active) => html! { 
                                        <Badge variant={BadgeVariant::Outline} class="bg-emerald-500/10 text-emerald-500 border-emerald-500/20 hover:bg-emerald-500/20">{t.t("clients.status.active")}</Badge> 
                                    },
                                    Some(ClientStatus::Maintenance) => html! { 
                                        <Badge variant={BadgeVariant::Outline} class="bg-amber-500/10 text-amber-500 border-amber-500/20 hover:bg-amber-500/20">{t.t("clients.status.maintenance")}</Badge> 
                                    },
                                    Some(ClientStatus::InStock) => html! { 
                                        <Badge variant={BadgeVariant::Outline} class="bg-blue-500/10 text-blue-500 border-blue-500/20 hover:bg-blue-500/20">{t.t("clients.status.instock")}</Badge> 
                                    },
                                    Some(ClientStatus::Decommissioned) => html! { 
                                        <Badge variant={BadgeVariant::Outline} class="bg-red-500/10 text-red-500 border-red-500/20 hover:bg-red-500/20">{t.t("clients.status.decommissioned")}</Badge> 
                                    },
                                    None => html! { <span class="opacity-50">{"-"}</span> },
                                };

                                let env_display = match &client.environment {
                                    Some(Environment::Prod) => html! { 
                                        <Badge variant={BadgeVariant::Outline} class="bg-red-500/10 text-red-500 border-red-500/20 hover:bg-red-500/20">{t.t("clients.env.prod")}</Badge> 
                                    },
                                    Some(Environment::Staging) => html! { 
                                        <Badge variant={BadgeVariant::Outline} class="bg-amber-500/10 text-amber-500 border-amber-500/20 hover:bg-amber-500/20">{t.t("clients.env.staging")}</Badge> 
                                    },
                                    Some(Environment::Test) => html! { 
                                        <Badge variant={BadgeVariant::Outline} class="bg-blue-500/10 text-blue-500 border-blue-500/20 hover:bg-blue-500/20">{t.t("clients.env.test")}</Badge> 
                                    },
                                    Some(Environment::Dev) => html! { 
                                        <Badge variant={BadgeVariant::Outline} class="bg-emerald-500/10 text-emerald-500 border-emerald-500/20 hover:bg-emerald-500/20">{t.t("clients.env.dev")}</Badge> 
                                    },
                                    None => html! { <span class="opacity-50">{"-"}</span> },
                                };

                                let on_view = {
                                    let navigator = navigator.clone();
                                    let id = client.id.clone();
                                    Callback::from(move |_| {
                                        if let Some(navigator) = &navigator {
                                            navigator.push(&Route::ClientDetail { id: id.clone() });
                                        }
                                    })
                                };

                                let on_delete_click = {
                                    let on_delete = props.on_delete.clone();
                                    let id = client.id.clone();
                                    let confirm_msg = t.t("clients.actions.confirm_delete");
                                    Callback::from(move |_| {
                                        if gloo::dialogs::confirm(&confirm_msg) {
                                            on_delete.emit(id.clone());
                                        }
                                    })
                                };

                                let on_toggle = {
                                    let on_toggle_selection = props.on_toggle_selection.clone();
                                    let id = client.id.clone();
                                    Callback::from(move |_| {
                                        on_toggle_selection.emit(id.clone());
                                    })
                                };

                                let is_selected = props.selected_clients.contains(&client.id);

                                html! {
                                    <TableRow>
                                        <TableCell class="text-center">
                                            <Checkbox
                                                checked={is_selected}
                                                onchange={on_toggle}
                                            />
                                        </TableCell>
                                        <TableCell class="font-medium">
                                            <div class="flex flex-col">
                                                <span class="font-bold text-primary cursor-pointer hover:underline" onclick={on_view.clone()}>
                                                    { &client.hostname }
                                                </span>
                                                <span class="text-xs text-muted-foreground">{ &client.serial_number.clone().unwrap_or_default() }</span>
                                            </div>
                                        </TableCell>
                                        <TableCell>{ &*client.ip_address }</TableCell>
                                        <TableCell class="text-center">
                                            <div class="flex flex-col items-center">
                                                <span class="text-sm">{ client.os.clone().unwrap_or_else(|| "-".to_string()) }</span>
                                                <span class="text-xs text-muted-foreground">{ client.kernel_version.clone().unwrap_or_default() }</span>
                                            </div>
                                        </TableCell>
                                        <TableCell class="text-center">{ owner_name }</TableCell>
                                        <TableCell class="text-center">{ project_name }</TableCell>
                                        <TableCell class="text-center">{ status_display }</TableCell>
                                        <TableCell class="text-center">{ env_display }</TableCell>
                                        <TableCell class="text-right">
                                            <div class="flex justify-end gap-2">
                                                <Button variant={ButtonVariant::Ghost} size={ButtonSize::Icon} onclick={on_view} title={t.t("clients.actions.view")}>
                                                    <Eye class="h-4 w-4" />
                                                </Button>
                                                <PermissionGuard min_role={Role::User}>
                                                    <Button variant={ButtonVariant::Ghost} size={ButtonSize::Icon} title={t.t("clients.actions.edit")}>
                                                        <Pencil class="h-4 w-4" />
                                                    </Button>
                                                </PermissionGuard>
                                                <PermissionGuard min_role={Role::Admin}>
                                                    <Button variant={ButtonVariant::Ghost} size={ButtonSize::Icon} class="text-destructive hover:text-destructive" onclick={on_delete_click} title={t.t("clients.actions.delete")}>
                                                        <Trash2 class="h-4 w-4" />
                                                    </Button>
                                                </PermissionGuard>
                                            </div>
                                        </TableCell>
                                    </TableRow>
                                }
                            }).collect::<Html>()
                        }
                    </TableBody>
                </Table>
            </div>
            
            <Pagination
                total_pages={props.total_pages}
                total_items={props.total_items}
                page_size={props.page_size}
                current_page={props.current_page}
                on_page_change={props.on_page_change.clone()}
                on_page_size_change={props.on_page_size_change.clone()}
            />
        </div>
    }
} 