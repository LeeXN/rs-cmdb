use yew::prelude::*;
use yew_router::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::{Blob, BlobPropertyBag, Url, HtmlAnchorElement, HtmlInputElement, FileReader};
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use crate::services::api::{self, ApiError};
use crate::types::{Client, FilterOptions, ClientStatus, Environment};
use crate::pages::clients::state::{ClientsState, ClientsAction};
use crate::pages::clients::components::{
    statistics::Statistics,
    search_bar::SearchBar,
    table::ClientsTable,
};
use crate::components::permission_guard::PermissionGuard;
use common::entity::user::Role;
use rust_xlsxwriter::Workbook;
use calamine::{Reader, Xlsx, DataType};
use std::io::Cursor;
use crate::hooks::use_trans::use_trans;

use crate::components::ui::card::{Card, CardHeader, CardContent, CardTitle};
use crate::components::ui::button::{Button, ButtonVariant, ButtonSize};
use lucide_yew::{FileDown, X, TriangleAlert};

#[function_component(ClientsPage)]
pub fn clients_page() -> Html {
    let state = use_reducer(ClientsState::default);
    let location = use_location();
    let file_input_ref = use_node_ref();
    let import_progress = use_state(|| 0);
    let is_importing = use_state(|| false);
    let import_errors = use_state(|| Vec::<String>::new());
    let show_error_modal = use_state(|| false);
    let t = use_trans();
    
    // 初始化数据加载 - 静态数据
    {
        let state = state.clone();
        use_effect_with((), move |_| {
            // 加载筛选选项
            api::get_filter_options(Callback::from({
                let state = state.clone();
                move |result: Result<FilterOptions, ApiError>| {
                    match result {
                        Ok(options) => state.dispatch(ClientsAction::SetFilterOptions(options)),
                        Err(_) => {}
                    }
                }
            }));

            // 加载人员和项目数据
            let state_persons = state.clone();
            spawn_local(async move {
                if let Ok(persons) = api::fetch_persons(1, 1000, None, None).await {
                    state_persons.dispatch(ClientsAction::SetPersons(persons.items));
                }
            });

            let state_projects = state.clone();
            spawn_local(async move {
                if let Ok(projects) = api::fetch_projects(1, 1000, None, None).await {
                    state_projects.dispatch(ClientsAction::SetProjects(projects.items));
                }
            });

            let state_racks = state.clone();
            spawn_local(async move {
                if let Ok(racks) = api::fetch_racks(1, 1000, None, None).await {
                    state_racks.dispatch(ClientsAction::SetRacks(racks.items));
                }
            });

            // 加载总设备数
            let state_total = state.clone();
            spawn_local(async move {
                if let Ok(result) = api::fetch_clients(1, 1, None, None, None).await {
                    state_total.dispatch(ClientsAction::SetTotalDbItems(result.total));
                }
            });
            
            || ()
        });
    }

    // 数据加载 - 动态数据 (分页/搜索/筛选)
    {
        let state = state.clone();
        let current_page = state.current_page;
        let page_size = state.page_size;
        let search_term = state.search_term.clone();
        let filters = state.filters.clone();
        let reload_trigger = state.reload_trigger;
        
        use_effect_with((current_page, page_size, search_term, filters, reload_trigger), move |(page, size, search, filters, _)| {
            if filters.has_api_filters() {
                let params = crate::pages::clients::utils::build_api_filter_params(search, filters);
                let page_val = *page;
                let size_val = *size;
                
                api::get_clients_by_hardware_filter(
                    params.0, params.1, params.2, params.3, params.4, params.5,
                    params.6, params.7, params.8, params.9, None, params.10, params.11, params.12,
                    params.13, params.14, params.15, params.16, params.17, params.18,
                    Callback::from({
                        let state = state.clone();
                        move |result: Result<Vec<Client>, ApiError>| {
                            match result {
                                Ok(clients) => {
                                    // Calculate pagination locally
                                    let total = clients.len();
                                    let (total_pages, start, end) = crate::pages::clients::utils::calculate_pagination(total, page_val, size_val);
                                    let items = if start < total {
                                        clients[start..end].to_vec()
                                    } else {
                                        Vec::new()
                                    };

                                    let paginated = crate::types::PaginatedResult {
                                        items,
                                        total,
                                        page: page_val,
                                        page_size: size_val,
                                        total_pages,
                                    };
                                    state.dispatch(ClientsAction::SetClients(paginated));
                                },
                                Err(err) => state.dispatch(ClientsAction::SetError(Some(err.message))),
                            }
                        }
                    })
                );
            } else {
                api::get_clients(*page, *size, Some(search.clone()), Some(filters.os.clone()), Some(filters.status.clone()), Callback::from({
                    let state = state.clone();
                    move |result: Result<crate::types::PaginatedResult<Client>, ApiError>| {
                        match result {
                            Ok(result) => state.dispatch(ClientsAction::SetClients(result)),
                            Err(err) => state.dispatch(ClientsAction::SetError(Some(err.message))),
                        }
                    }
                }));
            }
            || ()
        });
    }
    
    // 监听 URL 位置变化
    {
        let state = state.clone();
        let location = location.clone();
        use_effect_with(location, move |location| {
            if let Some(loc) = location {
                if let Ok(params) = loc.query::<std::collections::HashMap<String, String>>() {
                    if let Some(q) = params.get("q") {
                        state.dispatch(ClientsAction::SetSearchTerm(q.clone()));
                    }
                }
            }
            || ()
        });
    }

    // 分页处理
    let on_page_change = {
        let state = state.clone();
        Callback::from(move |page: usize| {
            state.dispatch(ClientsAction::SetPage(page));
        })
    };

    // 页面大小变更处理
    let on_page_size_change = {
        let state = state.clone();
        Callback::from(move |size: usize| {
            state.dispatch(ClientsAction::SetPageSize(size));
        })
    };

    let on_toggle_selection = {
        let state = state.clone();
        Callback::from(move |id: String| {
            state.dispatch(ClientsAction::ToggleSelection(id));
        })
    };

    let on_select_all = {
        let state = state.clone();
        Callback::from(move |select: bool| {
            state.dispatch(ClientsAction::SelectAll(select));
        })
    };

    let on_clear_selection_click = {
        let state = state.clone();
        Callback::from(move |_| {
            state.dispatch(ClientsAction::ClearSelection);
        })
    };

    // 导出逻辑复用
    let perform_export = {
        let racks = state.racks.clone();
        let persons = state.persons.clone();
        let projects = state.projects.clone();
        
        std::rc::Rc::new(move |clients: &Vec<Client>| {
            let mut workbook = Workbook::new();
            let worksheet = workbook.add_worksheet();
            
            // Headers
            let headers = [
                "ID", "Hostname", "IP", "Rack", "Unit", "Height", "Power (W)", 
                "Owner", "Project", "Status", "Environment", 
                "Warranty Expiration", "Asset Tag", "Supplier", "Comment"
            ];
            
            for (col, header) in headers.iter().enumerate() {
                let _ = worksheet.write_string(0, col as u16, *header);
            }
            
            // Data Validation Lists
            let _rack_names: Vec<String> = racks.iter().map(|r| r.name.clone()).collect();
            let _owner_names: Vec<String> = persons.iter().map(|p| p.name.clone()).collect();
            let _project_names: Vec<String> = projects.iter().map(|p| p.name.clone()).collect();
            let _statuses = vec!["Active", "Maintenance", "InStock", "Decommissioned"];
            let _environments = vec!["Prod", "Dev", "Test", "Staging"];
            
            // Write Data
            for (row, client) in clients.iter().enumerate() {
                let r = (row + 1) as u32;
                
                let rack_name = client.rack.as_ref()
                    .and_then(|id| racks.iter().find(|r| &r.id == id))
                    .map(|r| r.name.clone())
                    .unwrap_or_default();
                
                let owner_name = client.owner_id.as_ref()
                    .and_then(|id| persons.iter().find(|p| &p.id == id))
                    .map(|p| p.name.clone())
                    .unwrap_or_default();
                    
                let project_name = client.project_id.as_ref()
                    .and_then(|id| projects.iter().find(|p| &p.id == id))
                    .map(|p| p.name.clone())
                    .unwrap_or_default();
                
                let status = client.status.as_ref().map(|s| format!("{:?}", s)).unwrap_or_default();
                let env = client.environment.as_ref().map(|e| format!("{:?}", e)).unwrap_or_default();
                
                let _ = worksheet.write_string(r, 0, &client.id);
                let _ = worksheet.write_string(r, 1, &client.hostname);
                let _ = worksheet.write_string(r, 2, &client.ip_address);
                let _ = worksheet.write_string(r, 3, &rack_name);
                let _ = worksheet.write_string(r, 4, &client.unit_position.clone().unwrap_or_default());
                let _ = worksheet.write_number(r, 5, client.u_height.unwrap_or(1));
                let _ = worksheet.write_number(r, 6, client.power_consumption.unwrap_or(0));
                let _ = worksheet.write_string(r, 7, &owner_name);
                let _ = worksheet.write_string(r, 8, &project_name);
                let _ = worksheet.write_string(r, 9, &status);
                let _ = worksheet.write_string(r, 10, &env);
                let _ = worksheet.write_string(r, 11, &client.warranty_expiration.clone().unwrap_or_default());
                let _ = worksheet.write_string(r, 12, &client.asset_tag.clone().unwrap_or_default());
                let _ = worksheet.write_string(r, 13, &client.supplier.clone().unwrap_or_default());
                let _ = worksheet.write_string(r, 14, &client.comment.clone().unwrap_or_default());
            }
            
            // Save to buffer
            let buf = workbook.save_to_buffer().unwrap();
            
            // Create Blob and Download
            let uint8_array = js_sys::Uint8Array::from(&buf[..]);
            let array = js_sys::Array::new();
            array.push(&uint8_array.buffer());
            
            let blob_options = BlobPropertyBag::new();
            blob_options.set_type("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet");
            let blob = Blob::new_with_u8_array_sequence_and_options(&array, &blob_options).unwrap();
            
            let url = Url::create_object_url_with_blob(&blob).unwrap();
            let document = web_sys::window().unwrap().document().unwrap();
            let a: HtmlAnchorElement = document.create_element("a").unwrap().unchecked_into();
            a.set_href(&url);
            a.set_download("clients_export.xlsx");
            document.body().unwrap().append_child(&a).unwrap();
            a.click();
            document.body().unwrap().remove_child(&a).unwrap();
            Url::revoke_object_url(&url).unwrap();
        })
    };

    // 导出处理 (Excel) - Global
    let _on_export_click = {
        let clients = state.clients.clone();
        let perform_export = perform_export.clone();
        Callback::from(move |_: MouseEvent| {
            perform_export(&clients);
        })
    };

    // 导出处理 (Excel) - Selected
    let on_export_selected_click = {
        let all_clients = state.clients.clone();
        let selected_ids = state.selected_clients.clone();
        let perform_export = perform_export.clone();
        Callback::from(move |_: MouseEvent| {
            let selected: Vec<Client> = all_clients.iter()
                .filter(|c| selected_ids.contains(&c.id))
                .cloned()
                .collect();
            perform_export(&selected);
        })
    };

    // 导入处理
    let on_import_click = {
        let file_input_ref = file_input_ref.clone();
        Callback::from(move |_: MouseEvent| {
            if let Some(input) = file_input_ref.cast::<HtmlInputElement>() {
                input.click();
            }
        })
    };

    let on_file_change = {
        let state = state.clone();
        let racks = state.racks.clone();
        let persons = state.persons.clone();
        let projects = state.projects.clone();
        let all_clients = state.clients.clone();
        let import_progress = import_progress.clone();
        let is_importing = is_importing.clone();
        let import_errors = import_errors.clone();
        let show_error_modal = show_error_modal.clone();
        let t = t.clone();

        Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            if let Some(files) = input.files() {
                if let Some(file) = files.get(0) {
                    let reader = FileReader::new().unwrap();
                    let reader_clone = reader.clone();
                    let state = state.clone();
                    let racks = racks.clone();
                    let persons = persons.clone();
                    let projects = projects.clone();
                    let all_clients = all_clients.clone();
                    let import_progress = import_progress.clone();
                    let is_importing = is_importing.clone();
                    let import_errors = import_errors.clone();
                    let show_error_modal = show_error_modal.clone();
                    let t = t.clone();
                    
                    let onload = Closure::wrap(Box::new(move |_e: Event| {
                        let result = reader_clone.result().unwrap();
                        let array_buffer = result.dyn_into::<js_sys::ArrayBuffer>().unwrap();
                        let uint8_array = js_sys::Uint8Array::new(&array_buffer);
                        let vec = uint8_array.to_vec();
                        
                        is_importing.set(true);
                        import_progress.set(0);
                        import_errors.set(Vec::new());
                        
                        let cursor = Cursor::new(vec);
                        let mut excel: Xlsx<_> = calamine::open_workbook_from_rs(cursor).unwrap();
                        
                        if let Ok(range) = excel.worksheet_range("Sheet1") {
                            let mut updates = Vec::new();
                            let mut errors = Vec::new();
                            let total_rows = range.height().saturating_sub(1); // Subtract header
                            
                            // Iterate rows (skip header)
                            for (i, row) in range.rows().skip(1).enumerate() {
                                // Update progress
                                let progress = ((i as f64 / total_rows as f64) * 100.0) as i32;
                                import_progress.set(progress);
                                
                                let id = row.get(0).and_then(|c| c.as_string()).unwrap_or_default();
                                if id.is_empty() { continue; }
                                
                                // Find client
                                if let Some(client) = all_clients.iter().find(|c| c.id == id) {
                                    let mut updated_client = client.clone();
                                    let row_num = i + 2;
                                    
                                    // Helper to get string from cell
                                    let get_str = |idx| row.get(idx).and_then(|c: &calamine::Data| c.as_string()).unwrap_or_default();
                                    let get_int = |idx| row.get(idx).and_then(|c: &calamine::Data| c.as_f64()).map(|f| f as u32).or_else(|| 
                                        row.get(idx).and_then(|c: &calamine::Data| c.as_string()).and_then(|s| s.parse::<u32>().ok())
                                    );

                                    let rack_name = get_str(3);
                                    let unit_pos = get_str(4);
                                    let height = get_int(5);
                                    let power = get_int(6);
                                    let owner_name = get_str(7);
                                    let project_name = get_str(8);
                                    let status_str = get_str(9);
                                    let env_str = get_str(10);
                                    
                                    let warranty = get_str(11);
                                    let asset_tag = get_str(12);
                                    let supplier = get_str(13);
                                    let comment = get_str(14);

                                    // Validate Rack
                                    if !rack_name.is_empty() {
                                        if let Some(rack) = racks.iter().find(|r| r.name == rack_name) {
                                            updated_client.rack = Some(rack.id.clone());
                                            updated_client.location = rack.location.clone();
                                            
                                            // Check Capacity
                                            if let Ok(pos) = unit_pos.parse::<u32>() {
                                                if pos > 0 && pos <= rack.height_u {
                                                    updated_client.unit_position = Some(unit_pos.clone());
                                                    
                                                    // Check overlap? (Simplified: just check bounds for now, overlap check is complex without full rack map)
                                                    // Ideally we should check if this position is taken by ANOTHER client
                                                    let is_occupied = all_clients.iter().any(|c| 
                                                        c.id != client.id && 
                                                        c.rack == Some(rack.id.clone()) && 
                                                        c.unit_position == Some(unit_pos.clone())
                                                    );
                                                    if is_occupied {
                                                        errors.push(format!("Row {}: Unit {} in Rack '{}' is already occupied", row_num, unit_pos, rack_name));
                                                    }
                                                } else {
                                                    errors.push(format!("Row {}: Invalid Unit Position {} (Rack Height: {})", row_num, unit_pos, rack.height_u));
                                                }
                                            }
                                            
                                            // Check Power Limit
                                            if let Some(p) = power {
                                                updated_client.power_consumption = Some(p);
                                                
                                                if let Some(limit) = rack.power_limit {
                                                    let current_usage: u32 = all_clients.iter()
                                                        .filter(|c| c.rack == Some(rack.id.clone()) && c.id != client.id)
                                                        .map(|c| c.power_consumption.unwrap_or(0))
                                                        .sum();
                                                    
                                                    if current_usage + p > limit {
                                                        errors.push(format!("Row {}: Power limit exceeded for Rack '{}' (Limit: {}, Current: {}, New: {})", row_num, rack_name, limit, current_usage, p));
                                                    }
                                                }
                                            }
                                        } else {
                                            errors.push(format!("Row {}: Rack '{}' not found", row_num, rack_name));
                                        }
                                    }
                                    
                                    // Validate Owner
                                    if !owner_name.is_empty() {
                                        if let Some(person) = persons.iter().find(|p| p.name == owner_name) {
                                            updated_client.owner_id = Some(person.id.clone());
                                        } else {
                                            errors.push(format!("Row {}: Owner '{}' not found", row_num, owner_name));
                                        }
                                    }
                                    
                                    // Validate Project
                                    if !project_name.is_empty() {
                                        if let Some(project) = projects.iter().find(|p| p.name == project_name) {
                                            updated_client.project_id = Some(project.id.clone());
                                        } else {
                                            errors.push(format!("Row {}: Project '{}' not found", row_num, project_name));
                                        }
                                    }
                                    
                                    if let Some(h) = height {
                                        updated_client.u_height = Some(h);
                                    }
                                    
                                    if !status_str.is_empty() {
                                        match status_str.as_str() {
                                            "Active" => updated_client.status = Some(ClientStatus::Active),
                                            "Maintenance" => updated_client.status = Some(ClientStatus::Maintenance),
                                            "InStock" => updated_client.status = Some(ClientStatus::InStock),
                                            "Decommissioned" => updated_client.status = Some(ClientStatus::Decommissioned),
                                            _ => errors.push(format!("Row {}: Invalid Status '{}'", row_num, status_str)),
                                        }
                                    }
                                    
                                    if !env_str.is_empty() {
                                        match env_str.as_str() {
                                            "Prod" => updated_client.environment = Some(Environment::Prod),
                                            "Dev" => updated_client.environment = Some(Environment::Dev),
                                            "Test" => updated_client.environment = Some(Environment::Test),
                                            "Staging" => updated_client.environment = Some(Environment::Staging),
                                            _ => errors.push(format!("Row {}: Invalid Environment '{}'", row_num, env_str)),
                                        }
                                    }
                                    
                                    if !warranty.is_empty() { updated_client.warranty_expiration = Some(warranty); }
                                    if !asset_tag.is_empty() { updated_client.asset_tag = Some(asset_tag); }
                                    if !supplier.is_empty() { updated_client.supplier = Some(supplier); }
                                    if !comment.is_empty() { updated_client.comment = Some(comment); }
                                    
                                    updates.push(updated_client);
                                }
                            }
                            
                            if !errors.is_empty() {
                                import_errors.set(errors);
                                show_error_modal.set(true);
                                is_importing.set(false);
                            } else {
                                // Apply updates
                                let state = state.clone();
                                let is_importing = is_importing.clone();
                                let import_progress = import_progress.clone();
                                let t = t.clone();
                                
                                spawn_local(async move {
                                    let total = updates.len();
                                    for (idx, client) in updates.iter().enumerate() {
                                        let _ = api::update_client(&client.id, client).await;
                                        let progress = ((idx as f64 / total as f64) * 100.0) as i32;
                                        import_progress.set(progress);
                                    }
                                    
                                    // Refresh
                                    if let Ok(result) = api::fetch_clients(state.current_page, state.page_size, Some(state.search_term.clone()), Some(state.filters.os.clone()), Some(state.filters.status.clone())).await {
                                        state.dispatch(ClientsAction::SetClients(result));
                                    }
                                    
                                    is_importing.set(false);
                                    import_progress.set(100);
                                    gloo::dialogs::alert(&t.t("clients.import.success"));
                                });
                            }
                        }
                        
                    }) as Box<dyn FnMut(_)>);
                    
                    reader.set_onload(Some(onload.as_ref().unchecked_ref()));
                    onload.forget();
                    reader.read_as_array_buffer(&file).unwrap();
                }
            }
            input.set_value("");
        })
    };

    let on_delete_client = {
        let state = state.clone();
        Callback::from(move |id: String| {
            let state = state.clone();
            spawn_local(async move {
                if let Ok(_) = api::delete_client(&id).await {
                    // Refresh
                    if let Ok(result) = api::fetch_clients(state.current_page, state.page_size, Some(state.search_term.clone()), Some(state.filters.os.clone()), Some(state.filters.status.clone())).await {
                        state.dispatch(ClientsAction::SetClients(result));
                    }
                }
            });
        })
    };

    // 错误显示
    let error_display = if let Some(error) = &state.error {
        html! {
            <div class="bg-destructive/15 text-destructive p-4 rounded-md mb-4 flex items-center gap-2">
                <TriangleAlert class="h-5 w-5" />
                <span>{t.t("common.error_prefix")}{error}</span>
            </div>
        }
    } else {
        html! {}
    };

    html! {
        <div class="p-4 space-y-6">
            <Card>
                <CardContent class="pt-6">
                    {error_display}
                    
                    <div class="space-y-6">
                        <Statistics 
                            total_db_items={state.total_db_items}
                            filtered_total={state.total_items}
                            filtered_clients={state.clients.clone()}
                        />
                        
                        <div class="flex flex-col gap-4">
                            <SearchBar 
                                state={(*state).clone()}
                                dispatcher={state.dispatcher()}
                                on_import_click={on_import_click}
                            />

                            if !state.selected_clients.is_empty() {
                                <div class="flex gap-2 items-center mb-2 p-2 bg-muted/50 rounded-lg">
                                    <div class="flex items-center gap-2 text-sm text-muted-foreground">
                                        <span>{format!("{} {}", t.t("clients.selection.selected"), state.selected_clients.len())}</span>
                                        <Button variant={ButtonVariant::Ghost} size={ButtonSize::Icon} onclick={on_clear_selection_click} class="h-6 w-6">
                                            <X class="h-4 w-4" />
                                        </Button>
                                    </div>
                                    <PermissionGuard min_role={Role::User}>
                                        <Button variant={ButtonVariant::Outline} size={ButtonSize::Sm} onclick={on_export_selected_click}>
                                            <FileDown class="h-4 w-4 mr-2" />
                                            {t.t("clients.selection.export_template")}
                                        </Button>
                                    </PermissionGuard>
                                </div>
                            }
                        </div>
                        <input type="file" ref={file_input_ref} style="display: none" accept=".xlsx" onchange={on_file_change} />
                        
                        <ClientsTable 
                            clients={state.clients.clone()}
                            persons={state.persons.clone()}
                            projects={state.projects.clone()}
                            current_page={state.current_page}
                            page_size={state.page_size}
                            total_items={state.total_items}
                            total_pages={state.total_pages}
                            on_page_change={on_page_change}
                            on_page_size_change={on_page_size_change}
                            selected_clients={state.selected_clients.clone()}
                            on_toggle_selection={on_toggle_selection}
                            on_select_all={on_select_all}
                            on_delete={on_delete_client}
                        />
                    </div>
                </CardContent>
            </Card>
            
            // Import Progress Modal
            if *is_importing {
                <div class="fixed inset-0 z-[9999] flex items-center justify-center bg-background/80 backdrop-blur-sm">
                    <Card class="w-96">
                        <CardHeader>
                            <CardTitle>{t.t("clients.import.progress")}</CardTitle>
                        </CardHeader>
                        <CardContent>
                            <div class="w-full bg-secondary rounded-full h-2.5 dark:bg-gray-700">
                                <div class="bg-primary h-2.5 rounded-full" style={format!("width: {}%", *import_progress)}></div>
                            </div>
                            <p class="text-center mt-2 text-sm text-muted-foreground">{format!("{}%", *import_progress)}</p>
                        </CardContent>
                    </Card>
                </div>
            }

            // Error Report Modal
            if *show_error_modal {
                <div class="fixed inset-0 z-[9999] flex items-center justify-center bg-background/80 backdrop-blur-sm">
                    <Card class="w-11/12 max-w-2xl max-h-[80vh] flex flex-col">
                        <CardHeader>
                            <CardTitle class="text-destructive">{t.t("clients.import.error_title")}</CardTitle>
                        </CardHeader>
                        <CardContent class="flex-1 overflow-hidden flex flex-col">
                            <div class="bg-destructive/15 text-destructive p-4 rounded-md mb-4 flex items-center gap-2">
                                <TriangleAlert class="h-5 w-5" />
                                <span>{t.t("clients.import.error_desc")}</span>
                            </div>
                            <div class="overflow-y-auto flex-1 bg-muted p-4 rounded text-sm">
                                <ul class="list-disc list-inside space-y-1">
                                    {
                                        for import_errors.iter().map(|err| html! {
                                            <li class="text-destructive">{err}</li>
                                        })
                                    }
                                </ul>
                            </div>
                            <div class="mt-4 flex justify-end">
                                <Button onclick={Callback::from(move |_| show_error_modal.set(false))}>{t.t("common.close")}</Button>
                            </div>
                        </CardContent>
                    </Card>
                </div>
            }
        </div>
    }
}