use yew::prelude::*;
use wasm_bindgen::JsCast;
use log::info;

use crate::pages::clients::state::{ClientsState, ClientsAction};
use crate::pages::clients::filters::has_active_filters;
use crate::types::{ClientHardwareExport, Role};
use crate::services::api::{self, ApiError};
use crate::utils::export::{export_to_csv, export_to_json};
use crate::components::permission_guard::PermissionGuard;
use crate::hooks::use_trans::use_trans;

use crate::components::ui::card::{Card, CardContent, CardHeader, CardTitle};
use crate::components::ui::button::{Button, ButtonVariant, ButtonSize};
use crate::components::ui::input::Input;
use crate::components::ui::select::Select;
use lucide_yew::{Funnel, Search, X, FileDown, FileCode, FileInput, LoaderCircle};

#[derive(Properties, PartialEq)]
pub struct SearchBarProps {
    pub state: ClientsState,
    pub dispatcher: UseReducerDispatcher<ClientsState>,
    pub on_import_click: Callback<MouseEvent>,
}

#[function_component(SearchBar)]
pub fn search_bar(props: &SearchBarProps) -> Html {
    info!("SearchBar rendered");
    let state = &props.state;
    let dispatcher = &props.dispatcher;
    let on_import_click = &props.on_import_click;
    let t = use_trans();

    // 搜索处理（实时）
    let on_search = {
        let dispatcher = dispatcher.clone();
        Callback::from(move |val: String| {
            dispatcher.dispatch(ClientsAction::SetSearchTerm(val));
        })
    };

    // 筛选字段更新的通用函数
    let create_filter_callback = |field: &'static str| {
        let dispatcher = dispatcher.clone();
        let current_filters = state.filters.clone();
        Callback::from(move |value: String| {
            let mut new_filters = current_filters.clone();
            match field {
                "status" => new_filters.status = value,
                "client_status" => new_filters.client_status = value,
                "environment" => new_filters.environment = value,
                "rack_id" => new_filters.rack_id = value,
                "project_id" => new_filters.project_id = value,
                "owner_id" => new_filters.owner_id = value,
                "os" => new_filters.os = value,
                "os_kernel" => new_filters.os_kernel = value,
                "server_vendor" => new_filters.server_vendor = value,
                "cpu_vendor" => new_filters.cpu_vendor = value,
                "cpu_model" => new_filters.cpu_model = value,
                "gpu_vendor" => new_filters.gpu_vendor = value,
                "gpu_model" => new_filters.gpu_model = value,
                "memory_min" => new_filters.memory_min = value,
                "memory_max" => new_filters.memory_max = value,
                "network_type" => new_filters.network_type = value,
                "network_model" => new_filters.network_model = value,
                "storage_type" => new_filters.storage_type = value,
                _ => {}
            }
            dispatcher.dispatch(ClientsAction::SetFilters(new_filters));
        })
    };

    // 应用筛选
    let apply_filters = {
        let dispatcher = dispatcher.clone();
        Callback::from(move |_: MouseEvent| {
            dispatcher.dispatch(ClientsAction::ApplyFilters);
        })
    };

    // 清除筛选
    let clear_filters = {
        let dispatcher = dispatcher.clone();
        Callback::from(move |_: MouseEvent| {
            dispatcher.dispatch(ClientsAction::ClearAllFilters);
            
            // 清除DOM状态
            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    // 重置选择框
                    if let Ok(selects) = document.query_selector_all("select") {
                        for i in 0..selects.length() {
                            if let Some(select) = selects.item(i) {
                                if let Some(select_element) = select.dyn_ref::<web_sys::HtmlSelectElement>() {
                                    select_element.set_selected_index(0);
                                }
                            }
                        }
                    }
                    
                    // 重置输入框
                    if let Ok(inputs) = document.query_selector_all("input[type='number'], input[type='text']") {
                        for i in 0..inputs.length() {
                            if let Some(input) = inputs.item(i) {
                                if let Some(input_element) = input.dyn_ref::<web_sys::HtmlInputElement>() {
                                    input_element.set_value("");
                                }
                            }
                        }
                    }
                }
            }
        })
    };

    // 导出功能
    let export_csv = create_export_callback(dispatcher.clone(), &state.clients, "csv");
    let export_json = create_export_callback(dispatcher.clone(), &state.clients, "json");

    html! {
        <Card class="mb-6">
            <CardHeader>
                <CardTitle class="flex items-center gap-2">
                    <Funnel class="h-5 w-5" />
                    {t.t("clients.search.title")}
                </CardTitle>
            </CardHeader>
            <CardContent>
                // 基本搜索
                <div class="w-full mb-4 space-y-2">
                    <label class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">{t.t("clients.search.keyword_label")}</label>
                    <div class="relative">
                        <Search class="absolute left-2 top-2.5 h-4 w-4 text-muted-foreground" />
                        <Input 
                            class="pl-8"
                            value={state.search_term.clone()}
                            oninput={on_search}
                            placeholder={t.t("clients.search.placeholder")}
                        />
                    </div>
                    <p class="text-xs text-muted-foreground">{t.t("clients.search.hint")}</p>
                </div>
                
                // 筛选状态显示
                {render_filter_status(state, clear_filters.clone(), &t)}
                
                // 筛选选项
                {render_filter_options(state, create_filter_callback, &t)}
                
                // 操作按钮
                <div class="flex flex-wrap justify-between items-center gap-4 mt-6">
                    <div class="flex gap-2">
                        <Button 
                            variant={ButtonVariant::Default}
                            size={ButtonSize::Sm}
                            onclick={apply_filters}
                            disabled={state.loading}
                        >
                            {
                                if state.loading {
                                    html! {
                                        <>
                                            <LoaderCircle class="h-4 w-4 mr-2 animate-spin" />
                                            {t.t("loading")}
                                        </>
                                    }
                                } else {
                                    html! {
                                        <>
                                            <Funnel class="h-4 w-4 mr-2" />{t.t("clients.search.apply")}
                                        </>
                                    }
                                }
                            }
                        </Button>
                        <Button 
                            variant={ButtonVariant::Outline}
                            size={ButtonSize::Sm}
                            onclick={clear_filters}
                            disabled={!has_active_filters(&state.search_term, &state.filters) && state.current_page == 1}
                        >
                            <X class="h-4 w-4 mr-2" />{t.t("clients.search.clear")}
                        </Button>
                    </div>
                    <div class="flex gap-2">
                        <Button 
                            variant={ButtonVariant::Outline}
                            size={ButtonSize::Sm}
                            onclick={export_csv}
                            disabled={state.loading || state.exporting}
                        >
                            <FileDown class="h-4 w-4 mr-2" />{t.t("clients.search.export_csv")}
                        </Button>
                        <Button 
                            variant={ButtonVariant::Outline}
                            size={ButtonSize::Sm}
                            onclick={export_json}
                            disabled={state.loading || state.exporting}
                        >
                            <FileCode class="h-4 w-4 mr-2" />{t.t("clients.search.export_json")}
                        </Button>
                        <PermissionGuard min_role={Role::User}>
                            <Button 
                                variant={ButtonVariant::Outline}
                                size={ButtonSize::Sm}
                                onclick={on_import_click.clone()}
                            >
                                <FileInput class="h-4 w-4 mr-2" />{t.t("clients.search.import")}
                            </Button>
                        </PermissionGuard>
                    </div>
                </div>
            </CardContent>
        </Card>
    }
}

use crate::i18n::I18n;

fn render_filter_status(state: &ClientsState, clear_callback: Callback<MouseEvent>, t: &I18n) -> Html {
    if has_active_filters(&state.search_term, &state.filters) {
        html! {
            <div class="flex items-center justify-between mb-4 p-2 bg-muted/50 rounded-lg">
                <span class="text-sm text-primary font-bold">
                    {format!("{} {} {}", t.t("clients.stats.filtered_results"), state.total_items, t.t("dashboard.unit_machines"))}
                </span>
                <Button 
                    variant={ButtonVariant::Ghost}
                    size={ButtonSize::Sm}
                    onclick={clear_callback}
                    class="h-8 px-2"
                >
                    <X class="h-4 w-4 mr-1" />{t.t("clients.search.clear")}
                </Button>
            </div>
        }
    } else {
        html! {}
    }
}

fn render_filter_options<F>(state: &ClientsState, create_callback: F, t: &I18n) -> Html 
where 
    F: Fn(&'static str) -> Callback<String>,
{
    html! {
        <>
            <div class="mb-2">
                <h6 class="text-sm font-bold mb-1">{t.t("clients.filter.active_filters")}</h6>
                <small class="text-muted-foreground">{t.t("clients.search.hint")}</small>
            </div>
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
                {render_status_filter(&state.filters.status, create_callback("status"), t)}
                {render_client_status_filter(&state.filters.client_status, create_callback("client_status"), t)}
                {render_environment_filter(&state.filters.environment, create_callback("environment"), t)}
                {render_rack_filter(&state.filters.rack_id, &state.racks, create_callback("rack_id"), t)}
                {render_project_filter(&state.filters.project_id, &state.projects, create_callback("project_id"), t)}
                {render_owner_filter(&state.filters.owner_id, &state.persons, create_callback("owner_id"), t)}
                {render_select_filter(&t.t("clients.filter.os"), &state.filters.os, &state.filter_options.os_names, create_callback("os"), t)}
                {render_select_filter(&t.t("clients.filter.kernel"), &state.filters.os_kernel, &state.filter_options.os_kernels, create_callback("os_kernel"), t)}
                {render_select_filter(&t.t("clients.filter.vendor"), &state.filters.server_vendor, &state.filter_options.server_vendors, create_callback("server_vendor"), t)}
                {render_select_filter(&t.t("clients.filter.cpu_vendor"), &state.filters.cpu_vendor, &state.filter_options.cpu_vendors, create_callback("cpu_vendor"), t)}
                {render_select_filter(&t.t("clients.filter.cpu_model"), &state.filters.cpu_model, &state.filter_options.cpu_models, create_callback("cpu_model"), t)}
                {render_input_filter(&t.t("clients.filter.memory_min"), "number", &state.filters.memory_min, "32", create_callback("memory_min"))}
                {render_input_filter(&t.t("clients.filter.memory_max"), "number", &state.filters.memory_max, "512", create_callback("memory_max"))}
                {render_select_filter(&t.t("clients.filter.gpu_vendor"), &state.filters.gpu_vendor, &state.filter_options.gpu_vendors, create_callback("gpu_vendor"), t)}
                {render_select_filter(&t.t("clients.filter.gpu_model"), &state.filters.gpu_model, &state.filter_options.gpu_models, create_callback("gpu_model"), t)}
                {render_select_filter(&t.t("clients.filter.network_type"), &state.filters.network_type, &state.filter_options.network_types, create_callback("network_type"), t)}
                {render_select_filter(&t.t("clients.filter.network_model"), &state.filters.network_model, &state.filter_options.network_models, create_callback("network_model"), t)}
                {render_select_filter(&t.t("clients.filter.storage_type"), &state.filters.storage_type, &state.filter_options.storage_types, create_callback("storage_type"), t)}
            </div>
        </>
    }
}

fn render_client_status_filter(current_value: &str, callback: Callback<String>, t: &I18n) -> Html {
    html! {
        <div class="space-y-2">
            <label class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">{t.t("clients.filter.status")}</label>
            <Select 
                value={current_value.to_string()}
                onchange={callback}
            >
                <option value="" selected={current_value.is_empty()}>{t.t("all")}</option>
                <option value="Active" selected={current_value == "Active"}>{t.t("clients.status.active")}</option>
                <option value="InStock" selected={current_value == "InStock"}>{t.t("clients.status.instock")}</option>
                <option value="Maintenance" selected={current_value == "Maintenance"}>{t.t("clients.status.maintenance")}</option>
                <option value="Decommissioned" selected={current_value == "Decommissioned"}>{t.t("clients.status.decommissioned")}</option>
            </Select>
        </div>
    }
}

fn render_environment_filter(current_value: &str, callback: Callback<String>, t: &I18n) -> Html {
    html! {
        <div class="space-y-2">
            <label class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">{t.t("clients.filter.environment")}</label>
            <Select 
                value={current_value.to_string()}
                onchange={callback}
            >
                <option value="" selected={current_value.is_empty()}>{t.t("all")}</option>
                <option value="Prod" selected={current_value == "Prod"}>{t.t("clients.env.prod")}</option>
                <option value="Staging" selected={current_value == "Staging"}>{t.t("clients.env.staging")}</option>
                <option value="Test" selected={current_value == "Test"}>{t.t("clients.env.test")}</option>
                <option value="Dev" selected={current_value == "Dev"}>{t.t("clients.env.dev")}</option>
            </Select>
        </div>
    }
}

fn render_rack_filter(current_value: &str, racks: &[crate::types::Rack], callback: Callback<String>, t: &I18n) -> Html {
    html! {
        <div class="space-y-2">
            <label class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">{t.t("clients.filter.rack")}</label>
            <Select 
                value={current_value.to_string()}
                onchange={callback}
            >
                <option value="" selected={current_value.is_empty()}>{t.t("all")}</option>
                {
                    racks.iter().map(|rack| {
                        html! {
                            <option key={rack.id.clone()} value={rack.id.clone()} selected={current_value == rack.id}>
                                {format!("{} ({})", rack.name, rack.location.clone().unwrap_or_default())}
                            </option>
                        }
                    }).collect::<Html>()
                }
            </Select>
        </div>
    }
}

fn render_project_filter(current_value: &str, projects: &[crate::types::Project], callback: Callback<String>, t: &I18n) -> Html {
    html! {
        <div class="space-y-2">
            <label class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">{t.t("clients.filter.project")}</label>
            <Select 
                value={current_value.to_string()}
                onchange={callback}
            >
                <option value="" selected={current_value.is_empty()}>{t.t("all")}</option>
                {
                    projects.iter().map(|project| {
                        html! {
                            <option key={project.id.clone()} value={project.id.clone()} selected={current_value == project.id}>
                                {project.name.clone()}
                            </option>
                        }
                    }).collect::<Html>()
                }
            </Select>
        </div>
    }
}

fn render_owner_filter(current_value: &str, persons: &[crate::types::Person], callback: Callback<String>, t: &I18n) -> Html {
    html! {
        <div class="space-y-2">
            <label class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">{t.t("clients.filter.owner")}</label>
            <Select 
                value={current_value.to_string()}
                onchange={callback}
            >
                <option value="" selected={current_value.is_empty()}>{t.t("all")}</option>
                {
                    persons.iter().map(|person| {
                        html! {
                            <option key={person.id.clone()} value={person.id.clone()} selected={current_value == person.id}>
                                {format!("{} ({})", person.name, person.email)}
                            </option>
                        }
                    }).collect::<Html>()
                }
            </Select>
        </div>
    }
}

fn render_status_filter(current_value: &str, callback: Callback<String>, t: &I18n) -> Html {
    html! {
        <div class="space-y-2">
            <label class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">{t.t("dashboard.status")}</label>
            <Select 
                value={current_value.to_string()}
                onchange={callback}
            >
                <option value="" selected={current_value.is_empty()}>{t.t("all")}</option>
                <option value="online" selected={current_value == "online"}>{t.t("dashboard.online")}</option>
                <option value="offline" selected={current_value == "offline"}>{t.t("dashboard.offline")}</option>
            </Select>
        </div>
    }
}

fn render_select_filter(label: &str, current_value: &str, options: &[String], callback: Callback<String>, t: &I18n) -> Html {
    let label_str = label.to_string();
    
    html! {
        <div class="space-y-2">
            <label class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">{label_str}</label>
            <Select 
                value={current_value.to_string()}
                onchange={callback}
            >
                <option value="" selected={current_value.is_empty()}>{t.t("all")}</option>
                {
                    options.iter().map(|option| {
                        html! {
                            <option key={option.clone()} value={option.clone()} selected={current_value == option}>
                                {option.clone()}
                            </option>
                        }
                    }).collect::<Html>()
                }
            </Select>
        </div>
    }
}

fn render_input_filter(label: &str, input_type: &str, current_value: &str, placeholder: &str, callback: Callback<String>) -> Html {
    let label_str = label.to_string();
    let input_type_str = input_type.to_string();
    let placeholder_str = placeholder.to_string();
    
    html! {
        <div class="space-y-2">
            <label class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">{label_str}</label>
            <Input 
                type_={input_type_str}
                value={current_value.to_string()}
                oninput={callback}
                placeholder={placeholder_str}
            />
        </div>
    }
}

fn create_export_callback(
    dispatcher: UseReducerDispatcher<ClientsState>, 
    filtered_clients: &[crate::types::Client], 
    format: &str
) -> Callback<MouseEvent> {
    let dispatcher = dispatcher.clone();
    let client_ids: Vec<String> = filtered_clients.iter().map(|c| c.id.clone()).collect();
    let format = format.to_string();
    
    Callback::from(move |_: MouseEvent| {
        dispatcher.dispatch(ClientsAction::SetExporting(true));
        
        if !client_ids.is_empty() {
            api::get_export_data(Callback::from({
                let dispatcher = dispatcher.clone();
                let client_ids = client_ids.clone();
                let format = format.clone();
                move |result: Result<Vec<ClientHardwareExport>, ApiError>| {
                    dispatcher.dispatch(ClientsAction::SetExporting(false));
                    match result {
                        Ok(all_data) => {
                            let filtered_data: Vec<ClientHardwareExport> = all_data.into_iter()
                                .filter(|item| client_ids.contains(&item.client_id))
                                .collect();
                            
                            let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
                            let filename = format!("cmdb_filtered_export_{}.{}", timestamp, format);
                            
                            let result = if format == "csv" {
                                export_to_csv(&filtered_data, &filename)
                            } else {
                                export_to_json(&filtered_data, &filename)
                            };
                            
                            if let Err(err) = result {
                                dispatcher.dispatch(ClientsAction::SetError(Some(format!("Export failed: {:?}", err))));
                            }
                        }
                        Err(err) => {
                            dispatcher.dispatch(ClientsAction::SetError(Some(err.message)));
                        }
                    }
                }
            }));
        } else {
            dispatcher.dispatch(ClientsAction::SetExporting(false));
        }
    })
}