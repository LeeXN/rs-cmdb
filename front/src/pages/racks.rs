use crate::components::error::ErrorDisplay;
use crate::components::loading::Loading;
use crate::components::notification::{Notification, NotificationType};
use crate::components::permission_guard::PermissionGuard;
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::card::{Card, CardContent, CardHeader};
use crate::components::ui::confirm_modal::ConfirmModal;
use crate::components::ui::input::Input;
use crate::components::ui::table::{Table, TableBody, TableCell, TableHead, TableHeader, TableRow};
use crate::components::ui::table_action::TableActions;
use crate::hooks::use_trans::use_trans;
use crate::icons::{Grid3X3, LayoutGrid, List, Plus, Rows3};
use crate::routes::Route;
use crate::services::api::{create_rack, delete_rack, fetch_clients, fetch_racks, update_rack};
use crate::types::Client;
use common::entity::user::Role;
use common::models::Rack;
use std::collections::HashMap;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq)]
pub struct RackFormProps {
    pub rack: Option<Rack>,
    pub on_save: Callback<Rack>,
    pub on_cancel: Callback<()>,
}

#[function_component(RackForm)]
pub fn rack_form(props: &RackFormProps) -> Html {
    let t = use_trans();
    let name = use_state(|| {
        props
            .rack
            .as_ref()
            .map(|r| r.name.clone())
            .unwrap_or_default()
    });
    let location = use_state(|| {
        props
            .rack
            .as_ref()
            .and_then(|r| r.location.clone())
            .unwrap_or_default()
    });
    let height_u = use_state(|| props.rack.as_ref().map(|r| r.height_u).unwrap_or(42));
    let power_limit = use_state(|| props.rack.as_ref().and_then(|r| r.power_limit));
    let description = use_state(|| {
        props
            .rack
            .as_ref()
            .and_then(|r| r.description.clone())
            .unwrap_or_default()
    });

    let onsubmit = {
        let name = name.clone();
        let location = location.clone();
        let height_u = height_u.clone();
        let power_limit = power_limit.clone();
        let description = description.clone();
        let props_rack = props.rack.clone();
        let on_save = props.on_save.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let mut rack = props_rack.clone().unwrap_or_default();
            rack.name = (*name).clone();
            rack.location = if (*location).is_empty() {
                None
            } else {
                Some((*location).clone())
            };
            rack.height_u = *height_u;
            rack.power_limit = *power_limit;
            rack.description = if (*description).is_empty() {
                None
            } else {
                Some((*description).clone())
            };

            on_save.emit(rack);
        })
    };

    html! {
        <Card class="shadow-xl">
            <CardHeader>
                <h6 class="text-white text-capitalize ps-3">
                    { if props.rack.is_some() { t.t("racks.edit_rack") } else { t.t("racks.add_rack") } }
                </h6>
            </CardHeader>
            <CardContent class="pt-4 px-4 pb-4">
                <form {onsubmit}>
                    <div class="mb-4">
                        <label class="block text-sm font-bold text-slate-400 mb-1">{t.t("racks.rack_name")}</label>
                        <Input
                            value={(*name).clone()}
                            oninput={Callback::from(move |val: String| {
                                name.set(val);
                            })}
                            required=true
                        />
                    </div>
                    <div class="mb-4">
                        <label class="block text-sm font-bold text-slate-400 mb-1">{t.t("racks.location")}</label>
                        <Input
                            value={(*location).clone()}
                            oninput={Callback::from(move |val: String| {
                                location.set(val);
                            })}
                        />
                    </div>
                    <div class="mb-4">
                        <label class="block text-sm font-bold text-slate-400 mb-1">{t.t("racks.height_u")}</label>
                        <Input
                            type_="number"
                            value={(*height_u).to_string()}
                            oninput={Callback::from(move |val: String| {
                                if let Ok(v) = val.parse::<u32>() {
                                    height_u.set(v);
                                }
                            })}
                            required=true
                            min="1"
                        />
                    </div>
                    <div class="mb-4">
                        <label class="block text-sm font-bold text-slate-400 mb-1">{t.t("racks.power_limit_w")}</label>
                        <Input
                            type_="number"
                            value={(*power_limit).map(|v| v.to_string()).unwrap_or_default()}
                            oninput={Callback::from(move |val: String| {
                                if let Ok(v) = val.parse::<u32>() {
                                    power_limit.set(Some(v));
                                } else if val.is_empty() {
                                    power_limit.set(None);
                                }
                            })}
                            min="0"
                        />
                    </div>
                    <div class="mb-4">
                        <label class="block text-sm font-bold text-slate-400 mb-1">{t.t("racks.description")}</label>
                        <Input
                            value={(*description).clone()}
                            oninput={Callback::from(move |val: String| {
                                description.set(val);
                            })}
                        />
                    </div>
                    <div class="flex justify-end gap-4 mt-6">
                        <Button type_="button" variant={ButtonVariant::Outline} onclick={props.on_cancel.reform(|_| ())}>{t.t("racks.cancel")}</Button>
                        <Button type_="submit" variant={ButtonVariant::Default}>{t.t("racks.save")}</Button>
                    </div>
                </form>
            </CardContent>
        </Card>
    }
}

#[derive(Properties, PartialEq)]
pub struct RackVisualProps {
    pub racks: Vec<Rack>,
    pub clients: Vec<Client>,
    #[prop_or(false)]
    pub is_single_column: bool,
}

#[function_component(RackVisual)]
pub fn rack_visual(props: &RackVisualProps) -> Html {
    let t = use_trans();
    let grid_class = if props.is_single_column {
        "flex flex-col items-center gap-12 p-6"
    } else {
        "grid grid-cols-1 md:grid-cols-[repeat(auto-fit,minmax(500px,1fr))] gap-12 p-6"
    };

    html! {
        <div class={grid_class}>
            {
                for props.racks.iter().map(|rack| {
                    let rack_clients: Vec<&Client> = props.clients.iter()
                        .filter(|c| c.rack.as_ref() == Some(&rack.id))
                        .collect();

                    let total_u = rack.height_u;
                    let used_u: u32 = rack_clients.iter().map(|c| c.u_height.unwrap_or(1)).sum();
                    let free_u = total_u.saturating_sub(used_u);
                    let usage_percent = if total_u > 0 { (used_u as f64 / total_u as f64) * 100.0 } else { 0.0 };

                    let used_power: u32 = rack_clients.iter().map(|c| c.power_consumption.unwrap_or(0)).sum();
                    let power_limit = rack.power_limit.unwrap_or(0);
                    let power_percent = if power_limit > 0 { (used_power as f64 / power_limit as f64) * 100.0 } else { 0.0 };

                    // Create a map of occupied units to clients
                    // Map: U -> Client
                    let mut occupied_units = std::collections::HashMap::new();
                    for client in &rack_clients {
                        if let Some(pos_str) = &client.unit_position {
                            if let Ok(start_pos) = pos_str.parse::<u32>() {
                                let height = client.u_height.unwrap_or(1);
                                for i in 0..height {
                                    let pos = start_pos + i;
                                    if pos <= total_u {
                                        occupied_units.insert(pos, client);
                                    }
                                }
                            }
                        }
                    }

                    let card_class = if props.is_single_column {
                        "card bg-[#0f172a] border border-[#1e293b] shadow-2xl overflow-hidden flex flex-col md:flex-row h-[800px] w-full max-w-[1200px]"
                    } else {
                        "card bg-[#0f172a] border border-[#1e293b] shadow-2xl overflow-hidden flex flex-col md:flex-row h-[800px]"
                    };

                    html! {
                        <div class={card_class}>
                            // Rack Visualization Column
                            <div class="flex-1 p-4 flex flex-col relative min-h-0 min-w-[250px]">
                                <div class="flex justify-between items-center mb-4 text-slate-300">
                                    <h3 class="font-bold text-lg truncate mr-2" title={rack.name.clone()}>{ &rack.name }</h3>
                                    <span class="text-xs bg-[#1e293b] px-2 py-1 rounded border border-[#334155] whitespace-nowrap">{ rack.location.clone().unwrap_or_default() }</span>
                                </div>

                                // The Rack Grid
                                <div class="flex-1 bg-[#0b1121] border border-[#334155] rounded relative overflow-y-auto custom-scrollbar flex flex-col">
                                    {
                                        for (1..=total_u).rev().map(|u| {
                                            let client_opt = occupied_units.get(&u);

                                            // Check if this U is the TOP of a client
                                            let is_top = if let Some(client) = client_opt {
                                                let start_pos = client.unit_position.as_ref().and_then(|p| p.parse::<u32>().ok()).unwrap_or(0);
                                                let height = client.u_height.unwrap_or(1);
                                                u == start_pos + height - 1
                                            } else {
                                                false
                                            };

                                            // Check if this U is occupied but NOT the top (so we skip rendering)
                                            let is_covered = client_opt.is_some() && !is_top;

                                            if is_covered {
                                                html! {} // Render nothing, it's covered by the block above
                                            } else if let Some(client) = client_opt {
                                                // Render Client Block
                                                let height = client.u_height.unwrap_or(1);
                                                let height_px = height * 24; // Base height per U

                                                let status_color = match client.status {
                                                    Some(crate::types::ClientStatus::Active) => "bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.6)]",
                                                    Some(crate::types::ClientStatus::Maintenance) => "bg-amber-500 shadow-[0_0_8px_rgba(245,158,11,0.6)]",
                                                    Some(crate::types::ClientStatus::InStock) => "bg-blue-500 shadow-[0_0_8px_rgba(59,130,246,0.6)]",
                                                    Some(crate::types::ClientStatus::Decommissioned) => "bg-red-500 shadow-[0_0_8px_rgba(239,68,68,0.6)]",
                                                    None => "bg-slate-500",
                                                };

                                                html! {
                                                    <div style={format!("height: {}px;", height_px)} class="w-full">
                                                        <Link<Route> to={Route::ClientDetail { id: client.id.clone() }}
                                                             classes="w-full h-full border-b border-[#1e293b] bg-[#1e293b] hover:bg-[#2d3b4e] transition-colors relative group flex items-center px-2 block text-decoration-none">
                                                            // U Number
                                                            <span class="absolute left-1 top-1 text-[9px] text-slate-600 font-mono select-none">{u}</span>

                                                            // Status Indicator
                                                            <div class={format!("h-3/4 w-1 rounded-full mr-3 {}", status_color)}></div>

                                                            // Content
                                                            <div class="flex flex-col overflow-hidden">
                                                                <span class="text-xs font-bold text-slate-200 truncate leading-tight">{ &client.hostname }</span>
                                                                <span class="text-[10px] text-slate-500 truncate">{ client.primary_ip.as_deref().unwrap_or(&client.ip_address) }</span>
                                                            </div>

                                                            // Tooltip or Hover Details
                                                            <div class="absolute right-2 opacity-0 group-hover:opacity-100 transition-opacity">
                                                                <i class="fas fa-info-circle text-slate-400"></i>
                                                            </div>
                                                        </Link<Route>>
                                                    </div>
                                                }
                                            } else {
                                                // Empty Slot
                                                html! {
                                                    <div style="height: 24px;" class="w-full border-b border-[#1e293b]/50 flex items-center px-2 hover:bg-[#1e293b]/30 transition-colors">
                                                        <span class="text-[9px] text-slate-700 font-mono w-6 text-center select-none">{u}</span>
                                                    </div>
                                                }
                                            }
                                        })
                                    }
                                </div>
                            </div>

                            // Rack Stats / Sidebar
                            <div class="w-full md:w-64 bg-[#0b1121]/50 border-t md:border-t-0 md:border-l border-[#1e293b] p-4 flex flex-col gap-6 shrink-0">
                                // Capacity Widget
                                <div class="bg-[#1e293b] rounded-lg p-4 border border-[#334155]">
                                    <h4 class="text-xs uppercase text-slate-500 font-bold mb-2">{t.t("racks.rack_capacity")}</h4>
                                    <div class="flex items-end gap-2 mb-1">
                                        <span class="text-3xl font-bold text-blue-400">{format!("{:.0}%", usage_percent)}</span>
                                        <span class="text-xs text-slate-400 mb-1">{t.t("racks.used")}</span>
                                    </div>
                                    <progress class="progress progress-info w-full h-2 bg-slate-700" value={usage_percent.to_string()} max="100"></progress>
                                    <div class="flex justify-between mt-2 text-[10px] text-slate-400">
                                        <span>{format!("{} {}", used_u, t.t("racks.used"))}</span>
                                        <span>{format!("{} {}", free_u, t.t("racks.free"))}</span>
                                    </div>
                                </div>

                                {
                                    if power_limit > 0 {
                                        html! {
                                            <div class="bg-[#1e293b] rounded-lg p-4 border border-[#334155]">
                                                <h4 class="text-xs uppercase text-slate-500 font-bold mb-2">{t.t("racks.power_usage")}</h4>
                                                <div class="flex items-end gap-2 mb-1">
                                                    <span class={classes!("text-3xl", "font-bold", if power_percent > 90.0 { "text-red-400" } else { "text-emerald-400" })}>
                                                        {format!("{:.0}%", power_percent)}
                                                    </span>
                                                    <span class="text-xs text-slate-400 mb-1">{t.t("racks.used")}</span>
                                                </div>
                                                <progress class={classes!("progress", "w-full", "h-2", "bg-slate-700", if power_percent > 90.0 { "progress-error" } else { "progress-success" })} value={power_percent.to_string()} max="100"></progress>
                                                <div class="flex justify-between mt-2 text-[10px] text-slate-400">
                                                    <span>{format!("{} W {}", used_power, t.t("racks.used"))}</span>
                                                    <span>{format!("{} W {}", power_limit.saturating_sub(used_power), t.t("racks.free"))}</span>
                                                </div>
                                            </div>
                                        }
                                    } else {
                                        html! {}
                                    }
                                }

                                // Stats
                                <div class="flex flex-col gap-2">
                                    <div class="flex justify-between items-center p-2 rounded bg-[#1e293b]/50 border border-[#334155]/50">
                                        <span class="text-xs text-slate-400">{t.t("racks.total_units")}</span>
                                        <span class="text-sm font-mono text-slate-200">{total_u}</span>
                                    </div>
                                    <div class="flex justify-between items-center p-2 rounded bg-[#1e293b]/50 border border-[#334155]/50">
                                        <span class="text-xs text-slate-400">{t.t("racks.power_limit")}</span>
                                        <span class="text-sm font-mono text-slate-200">
                                            {
                                                if power_limit > 0 {
                                                    format!("{} / {} W", used_power, power_limit)
                                                } else {
                                                    t.t_with_args("racks.used_no_limit", &HashMap::from([("val".to_string(), used_power.to_string())]))
                                                }
                                            }
                                        </span>
                                    </div>
                                    <div class="flex justify-between items-center p-2 rounded bg-[#1e293b]/50 border border-[#334155]/50">
                                        <span class="text-xs text-slate-400">{t.t("racks.devices")}</span>
                                        <span class="text-sm font-mono text-slate-200">{rack_clients.len()}</span>
                                    </div>
                                </div>

                                // Legend
                                <div class="mt-auto">
                                    <h4 class="text-[10px] uppercase text-slate-600 font-bold mb-2">{t.t("racks.status")}</h4>
                                    <div class="grid grid-cols-2 gap-2 text-[10px] text-slate-400">
                                        <div class="flex items-center gap-1"><div class="w-2 h-2 rounded-full bg-emerald-500 shadow-[0_0_4px_rgba(16,185,129,0.6)]"></div>{t.t("racks.status.active")}</div>
                                        <div class="flex items-center gap-1"><div class="w-2 h-2 rounded-full bg-amber-500 shadow-[0_0_4px_rgba(245,158,11,0.6)]"></div>{t.t("racks.status.maint")}</div>
                                        <div class="flex items-center gap-1"><div class="w-2 h-2 rounded-full bg-blue-500 shadow-[0_0_4px_rgba(59,130,246,0.6)]"></div>{t.t("racks.status.stock")}</div>
                                        <div class="flex items-center gap-1"><div class="w-2 h-2 rounded-full bg-red-500 shadow-[0_0_4px_rgba(239,68,68,0.6)]"></div>{t.t("racks.status.error")}</div>
                                    </div>
                                </div>
                            </div>
                        </div>
                    }
                })
            }
        </div>
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum ViewMode {
    List,
    Visual,
}
#[function_component(Racks)]
pub fn racks() -> Html {
    let t = use_trans();
    let racks = use_state(Vec::<Rack>::new);
    let clients = use_state(Vec::<Client>::new);
    let loading = use_state(|| true);
    let error = use_state(|| None::<String>);
    let show_form = use_state(|| false);
    let editing_rack = use_state(|| None::<Rack>);
    let view_mode = use_state(|| ViewMode::List);
    let is_single_column = use_state(|| false);
    let delete_modal_open = use_state(|| false);
    let rack_to_delete = use_state(|| None::<String>);
    let delete_error = use_state(|| None::<String>);
    let notification = use_state(|| None::<(NotificationType, String)>);

    let fetch_data = {
        let racks = racks.clone();
        let clients = clients.clone();
        let loading = loading.clone();
        let error = error.clone();
        Callback::from(move |_| {
            let racks = racks.clone();
            let clients = clients.clone();
            let loading = loading.clone();
            let error = error.clone();
            loading.set(true);
            spawn_local(async move {
                // Fetch racks
                match fetch_racks(1, 1000, None, None).await {
                    Ok(data) => racks.set(data.items),
                    Err(err) => {
                        error.set(Some(err.message));
                        loading.set(false);
                        return;
                    }
                }

                // Fetch clients for visualization
                match fetch_clients(1, 1000, None, None, None).await {
                    Ok(data) => clients.set(data.items),
                    Err(_) => {
                        // Ignore client fetch error for now, just show empty racks
                    }
                }

                loading.set(false);
            });
        })
    };

    use_effect_with((), {
        let fetch_data = fetch_data.clone();
        move |_| {
            fetch_data.emit(());
            || ()
        }
    });

    let on_add_click = {
        let show_form = show_form.clone();
        let editing_rack = editing_rack.clone();
        Callback::from(move |_| {
            let mut new_rack = Rack {
                id: String::new(),
                height_u: 42,
                ..Default::default()
            };
            editing_rack.set(Some(new_rack));
            show_form.set(true);
        })
    };

    let on_edit_click = {
        let show_form = show_form.clone();
        let editing_rack = editing_rack.clone();
        Callback::from(move |rack: Rack| {
            editing_rack.set(Some(rack));
            show_form.set(true);
        })
    };

    let on_delete_click = {
        let delete_modal_open = delete_modal_open.clone();
        let rack_to_delete = rack_to_delete.clone();
        let delete_error = delete_error.clone();
        Callback::from(move |id: String| {
            rack_to_delete.set(Some(id));
            delete_error.set(None);
            delete_modal_open.set(true);
        })
    };

    let on_confirm_delete = {
        let fetch_data = fetch_data.clone();
        let delete_modal_open = delete_modal_open.clone();
        let rack_to_delete = rack_to_delete.clone();
        let notification = notification.clone();
        let delete_error = delete_error.clone();
        let t = t.clone();
        Callback::from(move |_| {
            let fetch_data = fetch_data.clone();
            let delete_modal_open = delete_modal_open.clone();
            let rack_to_delete = rack_to_delete.clone();
            let notification = notification.clone();
            let delete_error = delete_error.clone();
            let t = t.clone();

            if let Some(id) = (*rack_to_delete).clone() {
                spawn_local(async move {
                    match delete_rack(&id).await {
                        Ok(_) => {
                            delete_modal_open.set(false);
                            rack_to_delete.set(None);
                            delete_error.set(None);
                            notification.set(Some((
                                NotificationType::Success,
                                t.t("racks.delete_success"),
                            )));
                            fetch_data.emit(());
                        }
                        Err(e) => {
                            delete_error.set(Some(e.message));
                        }
                    }
                });
            }
        })
    };

    let on_cancel_delete = {
        let delete_modal_open = delete_modal_open.clone();
        let rack_to_delete = rack_to_delete.clone();
        let delete_error = delete_error.clone();
        Callback::from(move |_| {
            delete_modal_open.set(false);
            rack_to_delete.set(None);
            delete_error.set(None);
        })
    };

    let on_save = {
        let show_form = show_form.clone();
        let fetch_data = fetch_data.clone();
        let notification = notification.clone();
        let t = t.clone();
        Callback::from(move |rack: Rack| {
            let show_form = show_form.clone();
            let fetch_data = fetch_data.clone();
            let notification = notification.clone();
            let t = t.clone();
            spawn_local(async move {
                let result = if rack.id.is_empty() {
                    create_rack(&rack).await
                } else {
                    update_rack(&rack.id, &rack).await
                };

                match result {
                    Ok(_) => {
                        show_form.set(false);
                        notification
                            .set(Some((NotificationType::Success, t.t("racks.save_success"))));
                        fetch_data.emit(());
                    }
                    Err(e) => {
                        notification.set(Some((
                            NotificationType::Error,
                            t.t_with_args(
                                "racks.save_failed",
                                &HashMap::from([("val".to_string(), e.message)]),
                            ),
                        )));
                    }
                }
            });
        })
    };

    let on_cancel = {
        let show_form = show_form.clone();
        Callback::from(move |_| {
            show_form.set(false);
        })
    };

    let close_notification = {
        let notification = notification.clone();
        Callback::from(move |_| notification.set(None))
    };

    html! {
        <div class="container-fluid py-4">
            <div class="row">
                <div class="col-12">
                    <Card class="my-4 shadow-xl">
                        <CardHeader class="flex flex-row justify-start items-center">
                            <div class="flex gap-2 mr-3">
                                <div class="btn-group flex gap-1">
                                    <Button
                                        variant={if *view_mode == ViewMode::List { ButtonVariant::Default } else { ButtonVariant::Outline }}
                                        size={ButtonSize::Sm}
                                        onclick={
                                            let view_mode = view_mode.clone();
                                            Callback::from(move |_| view_mode.set(ViewMode::List))
                                        }
                                    >
                                        <List class="h-4 w-4 mr-1" />
                                        {t.t("racks.list_view")}
                                    </Button>
                                    <Button
                                        variant={if *view_mode == ViewMode::Visual { ButtonVariant::Default } else { ButtonVariant::Outline }}
                                        size={ButtonSize::Sm}
                                        onclick={
                                            let view_mode = view_mode.clone();
                                            Callback::from(move |_| view_mode.set(ViewMode::Visual))
                                        }
                                    >
                                        <LayoutGrid class="h-4 w-4 mr-1" />
                                        {t.t("racks.rack_view")}
                                    </Button>
                                </div>

                                if *view_mode == ViewMode::Visual {
                                    <Button
                                        variant={ButtonVariant::Outline}
                                        size={ButtonSize::Sm}
                                        class="ms-2"
                                        onclick={
                                            let is_single_column = is_single_column.clone();
                                            let val = *is_single_column;
                                            Callback::from(move |_| is_single_column.set(!val))
                                        }
                                    >
                                        if *is_single_column {
                                            <Grid3X3 class="h-4 w-4 mr-1" />
                                        } else {
                                            <Rows3 class="h-4 w-4 mr-1" />
                                        }
                                        <span class="align-middle">{if *is_single_column { t.t("racks.grid_layout") } else { t.t("racks.single_column_layout") }}</span>
                                    </Button>
                                }

                                <PermissionGuard min_role={Role::User}>
                                    <Button variant={ButtonVariant::Default} size={ButtonSize::Sm} onclick={on_add_click}>
                                        <Plus class="h-4 w-4 mr-1" /> {t.t("racks.add_rack")}
                                    </Button>
                                </PermissionGuard>
                            </div>
                        </CardHeader>
                        <CardContent class="px-0 pb-2">
                            if let Some((type_, msg)) = (*notification).clone() {
                                <div class="px-4">
                                    <Notification
                                        notification_type={type_}
                                        message={msg}
                                        show={true}
                                        on_close={close_notification.clone()}
                                    />
                                </div>
                            }
                            if *loading {
                                <Loading />
                            } else if let Some(err) = &*error {
                                <ErrorDisplay message={err.clone()} />
                            } else if *show_form {
                                <div class="p-4">
                                    <RackForm rack={(*editing_rack).clone()} {on_save} {on_cancel} />
                                </div>
                            } else {
                                if *view_mode == ViewMode::List {
                                    <div class="table-responsive p-0">
                                        <Table>
                                            <TableHeader>
                                                <TableRow>
                                                    <TableHead>{t.t("racks.rack_name")}</TableHead>
                                                    <TableHead>{t.t("racks.location")}</TableHead>
                                                    <TableHead class="text-center">{t.t("racks.height_u")}</TableHead>
                                                    <TableHead class="text-center">{t.t("racks.capacity_status")}</TableHead>
                                                    <TableHead class="text-center">{t.t("racks.power_status")}</TableHead>
                                                    <TableHead class="text-center">{t.t("racks.description")}</TableHead>
                                                    <TableHead class="text-center">{t.t("racks.actions")}</TableHead>
                                                </TableRow>
                                            </TableHeader>
                                            <TableBody>
                                                {
                                                    for racks.iter().map(|rack| {
                                                        let r_edit = rack.clone();
                                                        let r_delete = rack.clone();
                                                        let on_edit = on_edit_click.clone();
                                                        let on_delete = on_delete_click.clone();

                                                        // Calculate Capacity
                                                        let rack_clients: Vec<&Client> = clients.iter()
                                                            .filter(|c| c.rack.as_ref() == Some(&rack.id))
                                                            .collect();
                                                        let total_u = rack.height_u;
                                                        let used_u: u32 = rack_clients.iter().map(|c| c.u_height.unwrap_or(1)).sum();
                                                        let free_u = total_u.saturating_sub(used_u);
                                                        let usage_percent = if total_u > 0 { (used_u as f64 / total_u as f64) * 100.0 } else { 0.0 };

                                                        html! {
                                                            <TableRow class="hover:bg-slate-800/50 transition-colors">
                                                                <TableCell>
                                                                    <div class="d-flex px-2 py-1">
                                                                        <div class="d-flex flex-column justify-content-center">
                                                                            <h6 class="mb-0 text-sm text-slate-200">{&rack.name}</h6>
                                                                        </div>
                                                                    </div>
                                                                </TableCell>
                                                                <TableCell>
                                                                    <p class="text-xs font-weight-bold mb-0 text-slate-300">{rack.location.clone().unwrap_or_default()}</p>
                                                                </TableCell>
                                                                <TableCell class="align-middle text-center text-sm">
                                                                    <span class="text-slate-400 text-xs font-weight-bold">{rack.height_u}</span>
                                                                </TableCell>
                                                                <TableCell class="align-middle text-center text-sm">
                                                                    <div class="w-full px-4" style="min-width: 150px;">
                                                                        <div class="flex justify-between text-[10px] mb-1">
                                                                            <span class="text-slate-300">{format!("{} / {} U", used_u, total_u)}</span>
                                                                            <span class="text-emerald-400">{format!("{} U {}", free_u, t.t("racks.free"))}</span>
                                                                        </div>
                                                                        <progress class="progress progress-primary w-full h-1.5 bg-slate-700" value={usage_percent.to_string()} max="100"></progress>
                                                                    </div>
                                                                </TableCell>
                                                                <TableCell class="align-middle text-center text-sm">
                                                                    {
                                                                        if let Some(limit) = rack.power_limit {
                                                                            let used_power: u32 = rack_clients.iter().map(|c| c.power_consumption.unwrap_or(0)).sum();
                                                                            let free_power = limit.saturating_sub(used_power);
                                                                            let power_percent = if limit > 0 { (used_power as f64 / limit as f64) * 100.0 } else { 0.0 };
                                                                            let color_class = if power_percent > 90.0 { "progress-error" } else { "progress-success" };

                                                                            html! {
                                                                                <div class="w-full px-4" style="min-width: 150px;">
                                                                                    <div class="flex justify-between text-[10px] mb-1">
                                                                                        <span class="text-slate-300">{format!("{} / {} W", used_power, limit)}</span>
                                                                                        <span class={if free_power == 0 { "text-red-400" } else { "text-emerald-400" }}>{format!("{} W {}", free_power, t.t("racks.remaining"))}</span>
                                                                                    </div>
                                                                                    <progress class={format!("progress {} w-full h-1.5 bg-slate-700", color_class)} value={power_percent.to_string()} max="100"></progress>
                                                                                </div>
                                                                            }
                                                                        } else {
                                                                            let used_power: u32 = rack_clients.iter().map(|c| c.power_consumption.unwrap_or(0)).sum();
                                                                            html! {
                                                                                <span class="text-slate-400 text-xs font-weight-bold">{t.t_with_args("racks.used_no_limit", &HashMap::from([("val".to_string(), used_power.to_string())]))}</span>
                                                                            }
                                                                        }
                                                                    }
                                                                </TableCell>
                                                                <TableCell class="align-middle text-center text-sm">
                                                                    <span class="text-slate-400 text-xs font-weight-bold">{rack.description.clone().unwrap_or_default()}</span>
                                                                </TableCell>
                                                                <TableCell class="align-middle text-center">
                                                                    <PermissionGuard min_role={Role::User}>
                                                                        <TableActions
                                                                            on_edit={
                                                                                let on_edit = on_edit.clone();
                                                                                let r_edit = r_edit.clone();
                                                                                Some(Callback::from(move |_| on_edit.emit(r_edit.clone())))
                                                                            }
                                                                            on_delete={
                                                                                let on_delete = on_delete.clone();
                                                                                let id = r_delete.id.clone();
                                                                                Some(Callback::from(move |_| on_delete.emit(id.clone())))
                                                                            }
                                                                        />
                                                                    </PermissionGuard>
                                                                </TableCell>
                                                            </TableRow>
                                                        }
                                                    })
                                                }
                                            </TableBody>
                                        </Table>
                                    </div>
                                } else {
                                    <RackVisual racks={(*racks).clone()} clients={(*clients).clone()} is_single_column={*is_single_column} />
                                }
                            }
                        </CardContent>
                    </Card>
                </div>
            </div>
            <ConfirmModal
                is_open={*delete_modal_open}
                title={t.t("racks.confirm_delete")}
                message={t.t("racks.confirm_delete_msg")}
                on_confirm={on_confirm_delete}
                on_cancel={on_cancel_delete}
                error_message={(*delete_error).clone()}
            />
        </div>
    }
}
