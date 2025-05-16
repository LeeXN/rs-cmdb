use yew::prelude::*;
use std::collections::HashSet;
use common::models::{Component, ComponentStatus, PaginatedResult, ComponentType};
use crate::services::component::{get_components, update_component, create_component, batch_create_components, batch_update_components};
use crate::components::loading::Loading;
use crate::components::error::ErrorDisplay;
use crate::components::common::pagination::Pagination;
use crate::components::permission_guard::PermissionGuard;
use crate::components::ui::button::{Button, ButtonVariant, ButtonSize};
use crate::components::ui::table_action::TableActions;
use crate::components::ui::card::{Card, CardHeader, CardBody};
use crate::components::ui::table::{Table, TableHeader, TableBody, TableRow, TableHead, TableCell};
use crate::components::ui::checkbox::Checkbox;
use crate::components::ui::select::{Select, SelectOption};
use crate::components::ui::input::Input;
use common::entity::user::Role;
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlInputElement, Blob, BlobPropertyBag, Url, HtmlAnchorElement, FileReader};
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use rust_xlsxwriter::Workbook;
use calamine::{Reader, Xlsx, DataType};
use std::io::Cursor;
use lucide_yew::{Plus, FileCode, FileSpreadsheet};
use crate::hooks::use_trans::use_trans;
use std::collections::HashMap;

#[derive(Properties, PartialEq)]
pub struct ComponentFormProps {
    pub component: Component,
    pub on_save: Callback<Component>,
    pub on_cancel: Callback<()>,
    #[prop_or(false)]
    pub is_new: bool,
}

#[function_component(ComponentForm)]
pub fn component_form(props: &ComponentFormProps) -> Html {
    let t = use_trans();
    let status = use_state(|| props.component.status.clone());
    let location = use_state(|| props.component.location.clone().unwrap_or_default());
    let purchase_date = use_state(|| props.component.purchase_date.clone().unwrap_or_default());
    let warranty_expiration = use_state(|| props.component.warranty_expiration.clone().unwrap_or_default());
    
    // New fields for creation
    let component_type = use_state(|| props.component.component_type.clone());
    let vendor = use_state(|| props.component.vendor.clone().unwrap_or_default());
    let model = use_state(|| props.component.model.clone());
    let serial_number = use_state(|| props.component.serial_number.clone());

    let onsubmit = {
        let status = status.clone();
        let location = location.clone();
        let purchase_date = purchase_date.clone();
        let warranty_expiration = warranty_expiration.clone();
        let component_type = component_type.clone();
        let vendor = vendor.clone();
        let model = model.clone();
        let serial_number = serial_number.clone();
        let props_component = props.component.clone();
        let on_save = props.on_save.clone();
        let is_new = props.is_new;

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let mut component = props_component.clone();
            component.status = (*status).clone();
            component.location = if (*location).is_empty() { None } else { Some((*location).clone()) };
            component.purchase_date = if (*purchase_date).is_empty() { None } else { Some((*purchase_date).clone()) };
            component.warranty_expiration = if (*warranty_expiration).is_empty() { None } else { Some((*warranty_expiration).clone()) };
            
            if is_new {
                component.component_type = (*component_type).clone();
                component.vendor = if (*vendor).is_empty() { None } else { Some((*vendor).clone()) };
                component.model = (*model).clone();
                component.serial_number = (*serial_number).clone();
            }
            
            on_save.emit(component);
        })
    };

    let type_options = vec![
        SelectOption { value: "Other".to_string(), label: t.t("components.type_other") },
        SelectOption { value: "GPU".to_string(), label: t.t("components.type_gpu") },
        SelectOption { value: "CPU".to_string(), label: t.t("components.type_cpu") },
        SelectOption { value: "Memory".to_string(), label: t.t("components.type_memory") },
        SelectOption { value: "Disk".to_string(), label: t.t("components.type_disk") },
        SelectOption { value: "NetworkCard".to_string(), label: t.t("components.type_network_card") },
        SelectOption { value: "Motherboard".to_string(), label: t.t("components.type_motherboard") },
        SelectOption { value: "PowerSupply".to_string(), label: t.t("components.type_power_supply") },
    ];

    let status_options = vec![
        SelectOption { value: "InStock".to_string(), label: t.t("components.status_in_stock") },
        SelectOption { value: "InUse".to_string(), label: t.t("components.status_in_use") },
        SelectOption { value: "LentOut".to_string(), label: t.t("components.status_lent_out") },
        SelectOption { value: "Faulty".to_string(), label: t.t("components.status_faulty") },
        SelectOption { value: "Decommissioned".to_string(), label: t.t("components.status_decommissioned") },
        SelectOption { value: "Unknown".to_string(), label: t.t("components.status_unknown") },
    ];

    html! {
        <Card class="shadow-xl">
            <CardHeader>
                <h6 class="text-white text-capitalize ps-3">
                    if props.is_new {
                        {t.t("components.new_component")}
                    } else {
                        {t.t_with_args("components.edit_component", &HashMap::from([
                            ("model".to_string(), props.component.model.clone()),
                            ("sn".to_string(), props.component.serial_number.clone())
                        ]))}
                    }
                </h6>
            </CardHeader>
            <CardBody>
                <form {onsubmit}>
                    if props.is_new {
                        <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mb-4">
                            <div>
                                <label class="block text-sm font-bold text-slate-400 mb-1">{t.t("components.serial_number")}</label>
                                <Input 
                                    value={(*serial_number).clone()}
                                    oninput={Callback::from(move |val: String| serial_number.set(val))}
                                    required=true
                                />
                            </div>
                            <div>
                                <label class="block text-sm font-bold text-slate-400 mb-1">{t.t("components.model")}</label>
                                <Input 
                                    value={(*model).clone()}
                                    oninput={Callback::from(move |val: String| model.set(val))}
                                    required=true
                                />
                            </div>
                        </div>
                    }

                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mb-4">
                        <div>
                            <label class="block text-sm font-bold text-slate-400 mb-1">{t.t("components.type")}</label>
                            if props.is_new {
                                <Select 
                                    options={type_options}
                                    value={format!("{:?}", *component_type)}
                                    onchange={
                                        let component_type = component_type.clone();
                                        Callback::from(move |val: String| {
                                            let new_type = match val.as_str() {
                                                "GPU" => ComponentType::GPU,
                                                "CPU" => ComponentType::CPU,
                                                "Memory" => ComponentType::Memory,
                                                "Disk" => ComponentType::Disk,
                                                "NetworkCard" => ComponentType::NetworkCard,
                                                "Motherboard" => ComponentType::Motherboard,
                                                "PowerSupply" => ComponentType::PowerSupply,
                                                _ => ComponentType::Other,
                                            };
                                            component_type.set(new_type);
                                        })
                                    }
                                />
                            } else {
                                <Input value={format!("{:?}", props.component.component_type)} disabled=true />
                            }
                        </div>
                        <div>
                            <label class="block text-sm font-bold text-slate-400 mb-1">{t.t("components.vendor")}</label>
                            if props.is_new {
                                <Input 
                                    value={(*vendor).clone()}
                                    oninput={Callback::from(move |val: String| vendor.set(val))}
                                />
                            } else {
                                <Input value={props.component.vendor.clone().unwrap_or_default()} disabled=true />
                            }
                        </div>
                    </div>

                    <div class="mb-4">
                        <label class="block text-sm font-bold text-slate-400 mb-1">{t.t("components.status")}</label>
                        <Select 
                            options={status_options}
                            value={format!("{:?}", *status)}
                            onchange={
                                let status = status.clone();
                                Callback::from(move |val: String| {
                                    let new_status = match val.as_str() {
                                        "InStock" => ComponentStatus::InStock,
                                        "InUse" => ComponentStatus::InUse,
                                        "LentOut" => ComponentStatus::LentOut,
                                        "Faulty" => ComponentStatus::Faulty,
                                        "Decommissioned" => ComponentStatus::Decommissioned,
                                        _ => ComponentStatus::Unknown,
                                    };
                                    status.set(new_status);
                                })
                            }
                        />
                    </div>

                    <div class="mb-4">
                        <label class="block text-sm font-bold text-slate-400 mb-1">{t.t("components.location")}</label>
                        <Input 
                            value={(*location).clone()}
                            oninput={Callback::from(move |val: String| location.set(val))}
                        />
                    </div>

                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mb-4">
                        <div>
                            <label class="block text-sm font-bold text-slate-400 mb-1">{t.t("components.purchase_date")}</label>
                            <Input 
                                type_="date"
                                value={(*purchase_date).clone()}
                                oninput={Callback::from(move |val: String| purchase_date.set(val))}
                            />
                        </div>
                        <div>
                            <label class="block text-sm font-bold text-slate-400 mb-1">{t.t("components.warranty_expiration")}</label>
                            <Input 
                                type_="date"
                                value={(*warranty_expiration).clone()}
                                oninput={Callback::from(move |val: String| warranty_expiration.set(val))}
                            />
                        </div>
                    </div>

                    <div class="flex justify-end gap-4 mt-6">
                        <Button type_="button" variant={ButtonVariant::Outline} onclick={props.on_cancel.reform(|_| ())}>{t.t("common.cancel")}</Button>
                        <Button type_="submit" variant={ButtonVariant::Default}>{t.t("common.save")}</Button>
                    </div>
                </form>
            </CardBody>
        </Card>
    }
}

#[derive(Properties, PartialEq)]
pub struct BatchCreateFormProps {
    pub on_save: Callback<Vec<Component>>,
    pub on_cancel: Callback<()>,
}

#[function_component(BatchCreateForm)]
pub fn batch_create_form(props: &BatchCreateFormProps) -> Html {
    let t = use_trans();
    let json_content = use_state(|| "".to_string());
    let error_msg = use_state(|| None::<String>);

    let onsubmit = {
        let json_content = json_content.clone();
        let error_msg = error_msg.clone();
        let on_save = props.on_save.clone();
        let t = t.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            match serde_json::from_str::<Vec<Component>>(&json_content) {
                Ok(components) => {
                    on_save.emit(components);
                },
                Err(e) => {
                    error_msg.set(Some(t.t("components.json_parse_error").replace("{error}", &e.to_string())));
                }
            }
        })
    };

    html! {
        <Card class="shadow-xl">
            <CardHeader>
                <h6 class="text-white text-capitalize ps-3">{t.t("components.batch_create_json")}</h6>
            </CardHeader>
            <CardBody>
                if let Some(err) = &*error_msg {
                    <div class="alert alert-danger text-white" role="alert">
                        {err}
                    </div>
                }
                <p class="text-sm text-slate-400 mb-2">
                    {t.t("components.json_input_hint")}
                    <pre class="bg-slate-800 p-2 border-radius-md text-slate-300 mt-2 overflow-auto">
{r#"[
  {
    "id": "uuid-1",
    "serial_number": "SN123456",
    "model": "RTX 4090",
    "component_type": "GPU",
    "vendor": "NVIDIA",
    "status": "InStock",
    "purchase_date": "2023-01-01",
    "warranty_expiration": "2026-01-01"
  }
]"#}
                    </pre>
                </p>
                <form {onsubmit}>
                    <div class="mb-4">
                        <textarea class="textarea textarea-bordered w-full bg-slate-800 text-slate-200 border-slate-700 focus:border-blue-500 font-mono" rows="10"
                            value={(*json_content).clone()}
                            oninput={Callback::from(move |e: InputEvent| {
                                let input: HtmlInputElement = e.target_unchecked_into();
                                json_content.set(input.value());
                            })}
                        ></textarea>
                    </div>
                    <div class="flex justify-end gap-4 mt-6">
                        <Button type_="button" variant={ButtonVariant::Outline} onclick={props.on_cancel.reform(|_| ())}>{t.t("common.cancel")}</Button>
                        <Button type_="submit" variant={ButtonVariant::Default}>{t.t("components.batch_create")}</Button>
                    </div>
                </form>
            </CardBody>
        </Card>
    }
}

#[function_component(Components)]
pub fn components() -> Html {
    let t = use_trans();
    let components = use_state(|| Vec::<Component>::new());
    let loading = use_state(|| true);
    let error = use_state(|| None::<String>);
    let show_form = use_state(|| false);
    let show_batch_form = use_state(|| false);
    let editing_component = use_state(|| None::<Component>);
    let is_creating = use_state(|| false);
    let file_input_ref = use_node_ref();
    let is_importing = use_state(|| false);
    let import_progress = use_state(|| 0);
    
    // Pagination
    let total_pages = use_state(|| 1);
    let current_page = use_state(|| 1);
    let page_size = use_state(|| 10);
    let total_items = use_state(|| 0); // Added total_items tracking if available from API, otherwise we might need to adjust
    
    // Filters
    let filter_type = use_state(|| "all".to_string());
    let filter_status = use_state(|| "all".to_string());
    let filter_search = use_state(|| "".to_string());

    // Batch Selection
    let selected_components = use_state(|| HashSet::<String>::new());

    let toggle_selection = {
        let selected_components = selected_components.clone();
        Callback::from(move |id: String| {
            let mut set = (*selected_components).clone();
            if set.contains(&id) {
                set.remove(&id);
            } else {
                set.insert(id);
            }
            selected_components.set(set);
        })
    };

    let toggle_all_selection = {
        let selected_components = selected_components.clone();
        let components = components.clone();
        Callback::from(move |checked: bool| {
            if checked {
                let set: HashSet<String> = components.iter().map(|c| c.id.clone()).collect();
                selected_components.set(set);
            } else {
                selected_components.set(HashSet::new());
            }
        })
    };

    let fetch_data = {

        let components = components.clone();
        let loading = loading.clone();
        let error = error.clone();
        let filter_type = filter_type.clone();
        let filter_status = filter_status.clone();
        let filter_search = filter_search.clone();
        let current_page = current_page.clone();
        let page_size = page_size.clone();
        let total_pages = total_pages.clone();
        let total_items = total_items.clone();
        
        Callback::from(move |_| {
            let components = components.clone();
            let loading = loading.clone();
            let error = error.clone();
            let total_pages = total_pages.clone();
            let total_items = total_items.clone();
            
            loading.set(true);
            
            get_components(
                Some(*current_page),
                Some(*page_size),
                None, // client_id
                Some((*filter_type).clone()),
                Some((*filter_status).clone()),
                Some((*filter_search).clone()),
                Callback::from(move |result: Result<PaginatedResult<Component>, crate::services::component::ApiError>| {
                    loading.set(false);
                    match result {
                        Ok(data) => {
                            components.set(data.items);
                            total_pages.set(data.total_pages);
                            total_items.set(data.total);
                        },
                        Err(err) => error.set(Some(err.message)),
                    }
                })
            );
        })
    };

    let on_batch_update_status = {
        let selected_components = selected_components.clone();
        let fetch_data = fetch_data.clone();
        let t = t.clone();
        Callback::from(move |val: String| {
            if val == "none" {
                return;
            }
            
            let selected_ids: Vec<String> = (*selected_components).iter().cloned().collect();
            if selected_ids.is_empty() {
                gloo::dialogs::alert(&t.t("components.select_components_first"));
                return;
            }

            let new_status = match val.as_str() {
                "InStock" => ComponentStatus::InStock,
                "InUse" => ComponentStatus::InUse,
                "LentOut" => ComponentStatus::LentOut,
                "Faulty" => ComponentStatus::Faulty,
                "Decommissioned" => ComponentStatus::Decommissioned,
                _ => return,
            };

            if !gloo::dialogs::confirm(&t.t_with_args("components.confirm_batch_status_update", &HashMap::from([
                ("count".to_string(), selected_ids.len().to_string()),
                ("status".to_string(), val.clone())
            ]))) {
                return;
            }

            let fetch_data = fetch_data.clone();
            let selected_components = selected_components.clone();
            let t = t.clone();
            
            spawn_local(async move {
                match batch_update_components(selected_ids, Some(new_status)).await {
                    Ok(_) => {
                        selected_components.set(HashSet::new());
                        fetch_data.emit(());
                        gloo::dialogs::alert(&t.t("components.batch_status_update_success"));
                    },
                    Err(e) => gloo::dialogs::alert(&t.t("components.batch_status_update_failed").replace("{error}", &e.message)),
                }
            });
        })
    };

    // Export Logic
    let perform_export = {
        std::rc::Rc::new(move |components: &Vec<Component>| {
            let mut workbook = Workbook::new();
            let worksheet = workbook.add_worksheet();
            
            // Headers
            let headers = [
                "ID", "Serial Number", "Model", "Type", "Vendor", "Status", 
                "Location", "Purchase Date", "Warranty Expiration"
            ];
            
            for (col, header) in headers.iter().enumerate() {
                let _ = worksheet.write_string(0, col as u16, *header);
            }
            
            // Write Data
            for (row, component) in components.iter().enumerate() {
                let r = (row + 1) as u32;
                
                let status = format!("{:?}", component.status);
                let c_type = format!("{:?}", component.component_type);
                
                let _ = worksheet.write_string(r, 0, &component.id);
                let _ = worksheet.write_string(r, 1, &component.serial_number);
                let _ = worksheet.write_string(r, 2, &component.model);
                let _ = worksheet.write_string(r, 3, &c_type);
                let _ = worksheet.write_string(r, 4, &component.vendor.clone().unwrap_or_default());
                let _ = worksheet.write_string(r, 5, &status);
                let _ = worksheet.write_string(r, 6, &component.location.clone().unwrap_or_default());
                let _ = worksheet.write_string(r, 7, &component.purchase_date.clone().unwrap_or_default());
                let _ = worksheet.write_string(r, 8, &component.warranty_expiration.clone().unwrap_or_default());
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
            a.set_download("components_export.xlsx");
            document.body().unwrap().append_child(&a).unwrap();
            a.click();
            document.body().unwrap().remove_child(&a).unwrap();
            Url::revoke_object_url(&url).unwrap();
        })
    };

    let on_export_selected_click = {
        let components = components.clone();
        let selected_ids = selected_components.clone();
        let perform_export = perform_export.clone();
        Callback::from(move |_: MouseEvent| {
            let selected: Vec<Component> = components.iter()
                .filter(|c| selected_ids.contains(&c.id))
                .cloned()
                .collect();
            perform_export(&selected);
        })
    };

    // Import Logic
    let on_import_click = {
        let file_input_ref = file_input_ref.clone();
        Callback::from(move |_: MouseEvent| {
            if let Some(input) = file_input_ref.cast::<HtmlInputElement>() {
                input.click();
            }
        })
    };

    let on_file_change = {
        let fetch_data = fetch_data.clone();
        let is_importing = is_importing.clone();
        let import_progress = import_progress.clone();
        let t = t.clone();

        Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            if let Some(files) = input.files() {
                if let Some(file) = files.get(0) {
                    let reader = FileReader::new().unwrap();
                    let reader_clone = reader.clone();
                    let fetch_data = fetch_data.clone();
                    let is_importing = is_importing.clone();
                    let import_progress = import_progress.clone();
                    let t = t.clone();
                    
                    let onload = Closure::wrap(Box::new(move |_e: Event| {
                        let result = reader_clone.result().unwrap();
                        let array_buffer = result.dyn_into::<js_sys::ArrayBuffer>().unwrap();
                        let uint8_array = js_sys::Uint8Array::new(&array_buffer);
                        let vec = uint8_array.to_vec();
                        
                        is_importing.set(true);
                        import_progress.set(0);
                        
                        let cursor = Cursor::new(vec);
                        let mut excel: Xlsx<_> = calamine::open_workbook_from_rs(cursor).unwrap();
                        
                        if let Ok(range) = excel.worksheet_range("Sheet1") {
                            let mut updates = Vec::new();
                            let _total_rows = range.height().saturating_sub(1); // Subtract header
                            
                            // Iterate rows (skip header)
                            for (_i, row) in range.rows().skip(1).enumerate() {
                                let id = row.get(0).and_then(|c| c.as_string()).unwrap_or_default();
                                if id.is_empty() { continue; }
                                
                                // Helper to get string from cell
                                let get_str = |idx| row.get(idx).and_then(|c: &calamine::Data| c.as_string()).unwrap_or_default();
                                
                                let serial_number = get_str(1);
                                let model = get_str(2);
                                let type_str = get_str(3);
                                let vendor = get_str(4);
                                let status_str = get_str(5);
                                let location = get_str(6);
                                let purchase_date = get_str(7);
                                let warranty_expiration = get_str(8);
                                
                                let component_type = match type_str.as_str() {
                                    "GPU" => ComponentType::GPU,
                                    "CPU" => ComponentType::CPU,
                                    "Memory" => ComponentType::Memory,
                                    "Disk" => ComponentType::Disk,
                                    "NetworkCard" => ComponentType::NetworkCard,
                                    "Motherboard" => ComponentType::Motherboard,
                                    "PowerSupply" => ComponentType::PowerSupply,
                                    _ => ComponentType::Other,
                                };
                                
                                let status = match status_str.as_str() {
                                    "InStock" => ComponentStatus::InStock,
                                    "InUse" => ComponentStatus::InUse,
                                    "LentOut" => ComponentStatus::LentOut,
                                    "Faulty" => ComponentStatus::Faulty,
                                    "Decommissioned" => ComponentStatus::Decommissioned,
                                    _ => ComponentStatus::Unknown,
                                };
                                
                                let component = Component {
                                    id: id.clone(),
                                    serial_number,
                                    model,
                                    component_type,
                                    vendor: if vendor.is_empty() { None } else { Some(vendor) },
                                    status,
                                    location: if location.is_empty() { None } else { Some(location) },
                                    purchase_date: if purchase_date.is_empty() { None } else { Some(purchase_date) },
                                    warranty_expiration: if warranty_expiration.is_empty() { None } else { Some(warranty_expiration) },
                                    client_id: None, // Not updating associations via Excel for now
                                    client_hostname: None,
                                    missing_since: None,
                                    created_at: "".to_string(), // Will be ignored on update
                                    updated_at: "".to_string(),
                                };
                                
                                updates.push(component);
                            }
                            
                            // Apply updates
                            let fetch_data = fetch_data.clone();
                            let is_importing = is_importing.clone();
                            let import_progress = import_progress.clone();
                            let t = t.clone();
                            
                            spawn_local(async move {
                                let total = updates.len();
                                let mut success_count = 0;
                                for (idx, component) in updates.iter().enumerate() {
                                    if let Ok(_) = update_component(&component.id, component).await {
                                        success_count += 1;
                                    }
                                    let progress = ((idx as f64 / total as f64) * 100.0) as i32;
                                    import_progress.set(progress);
                                }
                                
                                fetch_data.emit(());
                                is_importing.set(false);
                                import_progress.set(100);
                                gloo::dialogs::alert(&t.t_with_args("components.batch_update_complete", &HashMap::from([
                                    ("success".to_string(), success_count.to_string()),
                                    ("total".to_string(), total.to_string())
                                ])));
                            });
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

    // Initial load and Pagination
    {
        let fetch_data = fetch_data.clone();
        use_effect_with(
            (*current_page, *page_size),
            move |_| {
                fetch_data.emit(());
                || ()
            }
        );
    }

    let on_create_click = {
        let show_form = show_form.clone();
        let editing_component = editing_component.clone();
        let is_creating = is_creating.clone();
        Callback::from(move |_| {
            editing_component.set(Some(Component {
                id: "".to_string(),
                serial_number: "".to_string(),
                model: "".to_string(),
                component_type: ComponentType::Other,
                vendor: None,
                status: ComponentStatus::InStock,
                purchase_date: None,
                warranty_expiration: None,
                location: None,
                client_id: None,
                client_hostname: None,
                missing_since: None,
                created_at: "".to_string(),
                updated_at: "".to_string(),
            }));
            is_creating.set(true);
            show_form.set(true);
        })
    };

    let on_batch_create_click = {
        let show_batch_form = show_batch_form.clone();
        Callback::from(move |_| {
            show_batch_form.set(true);
        })
    };

    let on_edit_click = {
        let show_form = show_form.clone();
        let editing_component = editing_component.clone();
        let is_creating = is_creating.clone();
        Callback::from(move |component: Component| {
            editing_component.set(Some(component));
            is_creating.set(false);
            show_form.set(true);
        })
    };

    let on_save = {
        let show_form = show_form.clone();
        let fetch_data = fetch_data.clone();
        let is_creating = is_creating.clone();
        let t = t.clone();
        Callback::from(move |component: Component| {
            let show_form = show_form.clone();
            let fetch_data = fetch_data.clone();
            let is_creating = *is_creating;
            let t = t.clone();
            spawn_local(async move {
                let result = if is_creating {
                    create_component(&component).await.map(|_| ())
                } else {
                    update_component(&component.id, &component).await.map(|_| ())
                };

                match result {
                    Ok(_) => {
                        show_form.set(false);
                        fetch_data.emit(());
                    },
                    Err(e) => gloo::dialogs::alert(&t.t("persons.save_failed").replace("{}", &e.message)),
                }
            });
        })
    };

    let on_batch_save = {
        let show_batch_form = show_batch_form.clone();
        let fetch_data = fetch_data.clone();
        let t = t.clone();
        Callback::from(move |components: Vec<Component>| {
            let show_batch_form = show_batch_form.clone();
            let fetch_data = fetch_data.clone();
            let t = t.clone();
            spawn_local(async move {
                match batch_create_components(components).await {
                    Ok(count) => {
                        show_batch_form.set(false);
                        fetch_data.emit(());
                        gloo::dialogs::alert(&t.t_with_args("components.batch_create_success", &HashMap::from([("count".to_string(), count.to_string())])));
                    },
                    Err(e) => gloo::dialogs::alert(&t.t("components.batch_create_failed").replace("{error}", &e.message)),
                }
            });
        })
    };

    let on_cancel = {
        let show_form = show_form.clone();
        let show_batch_form = show_batch_form.clone();
        Callback::from(move |_| {
            show_form.set(false);
            show_batch_form.set(false);
        })
    };
    
    let on_search = {
        let fetch_data = fetch_data.clone();
        let current_page = current_page.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            current_page.set(1); // Reset to first page on search
            fetch_data.emit(());
        })
    };

    let on_page_change = {
        let current_page = current_page.clone();
        Callback::from(move |page: usize| {
            current_page.set(page);
        })
    };

    let on_page_size_change = {
        let page_size = page_size.clone();
        let current_page = current_page.clone();
        Callback::from(move |size: usize| {
            page_size.set(size);
            current_page.set(1);
        })
    };

    let filter_type_options = vec![
        SelectOption { value: "all".to_string(), label: t.t("all") },
        SelectOption { value: "GPU".to_string(), label: t.t("components.type_gpu") },
        SelectOption { value: "CPU".to_string(), label: t.t("components.type_cpu") },
        SelectOption { value: "Memory".to_string(), label: t.t("components.type_memory") },
        SelectOption { value: "Disk".to_string(), label: t.t("components.type_disk") },
        SelectOption { value: "NetworkCard".to_string(), label: t.t("components.type_network_card") },
    ];

    let filter_status_options = vec![
        SelectOption { value: "all".to_string(), label: t.t("all") },
        SelectOption { value: "InStock".to_string(), label: t.t("components.status_in_stock") },
        SelectOption { value: "InUse".to_string(), label: t.t("components.status_in_use") },
        SelectOption { value: "LentOut".to_string(), label: t.t("components.status_lent_out") },
        SelectOption { value: "Faulty".to_string(), label: t.t("components.status_faulty") },
        SelectOption { value: "Decommissioned".to_string(), label: t.t("components.status_decommissioned") },
    ];

    let batch_status_options = vec![
        SelectOption { value: "none".to_string(), label: t.t("components.quick_status_change") },
        SelectOption { value: "InStock".to_string(), label: t.t("components.status_in_stock") },
        SelectOption { value: "InUse".to_string(), label: t.t("components.status_in_use") },
        SelectOption { value: "LentOut".to_string(), label: t.t("components.status_lent_out") },
        SelectOption { value: "Faulty".to_string(), label: t.t("components.status_faulty") },
        SelectOption { value: "Decommissioned".to_string(), label: t.t("components.status_decommissioned") },
    ];

    html! {
        <div class="container-fluid py-4">
            <div class="row">
                <div class="col-12">
                    <Card class="my-4 shadow-xl">
                        <CardHeader class="flex flex-row justify-start items-center">
                            <div class="flex gap-2">
                                <PermissionGuard min_role={Role::User}>
                                    <Button variant={ButtonVariant::Default} size={ButtonSize::Sm} onclick={on_create_click}>
                                        <Plus class="h-4 w-4 mr-1" /> {t.t("components.new_component")}
                                    </Button>
                                    <Button variant={ButtonVariant::Outline} size={ButtonSize::Sm} onclick={on_batch_create_click}>
                                        <FileCode class="h-4 w-4 mr-1" /> {t.t("components.json_import")}
                                    </Button>
                                    <Button variant={ButtonVariant::Outline} size={ButtonSize::Sm} onclick={on_import_click}>
                                        <FileSpreadsheet class="h-4 w-4 mr-1" /> {t.t("components.excel_import")}
                                    </Button>
                                </PermissionGuard>
                            </div>
                        </CardHeader>
                        
                        <CardBody class="px-4 pb-2">
                            <div class="flex flex-wrap items-center gap-2 mb-4">
                                if !selected_components.is_empty() {
                                    <div class="w-px h-6 bg-slate-700 mx-2"></div>
                                    
                                    <PermissionGuard min_role={Role::User}>
                                        <Button variant={ButtonVariant::Outline} size={ButtonSize::Sm} onclick={on_export_selected_click}>
                                            <i class="fas fa-file-export me-1"></i>{t.t("components.batch_edit_export")}
                                        </Button>
                                        
                                        <div class="w-40">
                                            <Select 
                                                options={batch_status_options}
                                                value="none"
                                                onchange={on_batch_update_status}
                                            />
                                        </div>
                                    </PermissionGuard>
                                }
                            </div>

                            <form onsubmit={on_search}>
                                <div class="flex flex-wrap items-end gap-4">
                                    <div class="w-full sm:w-auto">
                                        <label class="form-label text-slate-400 text-xs font-bold mb-2 block">{t.t("components.type")}</label>
                                        <div class="w-32">
                                            <Select 
                                                options={filter_type_options}
                                                value={(*filter_type).clone()}
                                                onchange={Callback::from(move |val: String| filter_type.set(val))}
                                            />
                                        </div>
                                    </div>
                                    <div class="w-full sm:w-auto">
                                        <label class="form-label text-slate-400 text-xs font-bold mb-2 block">{t.t("components.status")}</label>
                                        <div class="w-32">
                                            <Select 
                                                options={filter_status_options}
                                                value={(*filter_status).clone()}
                                                onchange={Callback::from(move |val: String| filter_status.set(val))}
                                            />
                                        </div>
                                    </div>
                                    <div class="w-full sm:flex-1 sm:max-w-md">
                                        <label class="form-label text-slate-400 text-xs font-bold mb-2 block">{t.t("components.search_label")}</label>
                                        <div class="relative">
                                            <Input 
                                                placeholder={t.t("components.search_placeholder")}
                                                value={(*filter_search).clone()}
                                                oninput={Callback::from(move |val: String| filter_search.set(val))}
                                            />
                                        </div>
                                    </div>
                                    <div class="w-full sm:w-auto">
                                        <Button type_="submit" variant={ButtonVariant::Default} size={ButtonSize::Sm} class="w-full sm:w-auto shadow-lg shadow-blue-500/30">
                                            {t.t("components.search")}
                                        </Button>
                                    </div>
                                </div>
                            </form>
                        </CardBody>
                        <CardBody class="px-0 pb-2 relative min-h-[400px]">
                            {
                                if *loading && components.is_empty() {
                                    html! { <Loading /> }
                                } else {
                                    html! {
                                        <>
                                            if *loading {
                                                <div class="absolute inset-0 bg-slate-900/50 z-10 flex justify-center items-center backdrop-blur-sm">
                                                    <Loading />
                                                </div>
                                            }
                                            
                                            if let Some(err) = &*error {
                                                <ErrorDisplay message={err.clone()} />
                                            } else if *show_form {
                                                <div class="p-4">
                                                    if let Some(component) = &*editing_component {
                                                        <ComponentForm component={component.clone()} {on_save} {on_cancel} is_new={*is_creating} />
                                                    }
                                                </div>
                                            } else if *show_batch_form {
                                                <div class="p-4">
                                                    <BatchCreateForm on_save={on_batch_save} {on_cancel} />
                                                </div>
                                            } else {
                                                <div class="table-responsive p-0">
                                                    <Table>
                                                        <TableHeader>
                                                            <TableRow>
                                                                <TableHead>
                                                                    <Checkbox
                                                                        checked={!components.is_empty() && selected_components.len() == components.len()}
                                                                        onchange={Callback::from(move |checked: bool| {
                                                                            toggle_all_selection.emit(checked);
                                                                        })}
                                                                    />
                                                                </TableHead>
                                                                <TableHead>{t.t("components.component_info")}</TableHead>
                                                                <TableHead>{t.t("components.type_vendor")}</TableHead>
                                                                <TableHead class="text-center">{t.t("components.status")}</TableHead>
                                                                <TableHead class="text-center">{t.t("components.location_owner")}</TableHead>
                                                                <TableHead class="text-center">{t.t("components.actions")}</TableHead>
                                                            </TableRow>
                                                        </TableHeader>
                                                        <TableBody>
                                                            {
                                                                for components.iter().map(|component| {
                                                                    let c_edit = component.clone();
                                                                    let on_edit = on_edit_click.clone();
                                                                    let toggle_selection = toggle_selection.clone();
                                                                    let is_selected = selected_components.contains(&component.id);
                                                                    
                                                                    let (status_class, status_text) = match component.status {
                                                                        ComponentStatus::InUse => ("bg-emerald-500/20 text-emerald-400 border-emerald-500/50 shadow-[0_0_10px_rgba(16,185,129,0.3)]", t.t("components.status_in_use")),
                                                                        ComponentStatus::InStock => ("bg-blue-500/20 text-blue-400 border-blue-500/50 shadow-[0_0_10px_rgba(59,130,246,0.3)]", t.t("components.status_in_stock")),
                                                                        ComponentStatus::LentOut => ("bg-amber-500/20 text-amber-400 border-amber-500/50 shadow-[0_0_10px_rgba(245,158,11,0.3)]", t.t("components.status_lent_out")),
                                                                        ComponentStatus::Faulty => ("bg-red-500/20 text-red-400 border-red-500/50 shadow-[0_0_10px_rgba(239,68,68,0.3)]", t.t("components.status_faulty")),
                                                                        ComponentStatus::Decommissioned => ("bg-slate-500/20 text-slate-400 border-slate-500/50", t.t("components.status_decommissioned")),
                                                                        ComponentStatus::Unknown => ("bg-slate-700/50 text-slate-300 border-slate-600", t.t("components.status_unknown")),
                                                                    };

                                                                    html! {
                                                                        <TableRow class="hover:bg-slate-800/50 transition-colors">
                                                                            <TableCell>
                                                                                <Checkbox
                                                                                    checked={is_selected}
                                                                                    onchange={
                                                                                        let id = component.id.clone();
                                                                                        Callback::from(move |_| toggle_selection.emit(id.clone()))
                                                                                    }
                                                                                />
                                                                            </TableCell>
                                                                            <TableCell>
                                                                                <div class="d-flex px-2 py-1">
                                                                                    <div class="d-flex flex-column justify-content-center">
                                                                                        <h6 class="mb-0 text-sm text-slate-200">{&component.model}</h6>
                                                                                        <p class="text-xs text-slate-400 mb-0">{"SN: "}{&component.serial_number}</p>
                                                                                    </div>
                                                                                </div>
                                                                            </TableCell>
                                                                            <TableCell>
                                                                                <p class="text-xs font-weight-bold mb-0 text-slate-300">
                                                                                    {
                                                                                        match component.component_type {
                                                                                            ComponentType::GPU => t.t("components.type_gpu"),
                                                                                            ComponentType::CPU => t.t("components.type_cpu"),
                                                                                            ComponentType::Memory => t.t("components.type_memory"),
                                                                                            ComponentType::Disk => t.t("components.type_disk"),
                                                                                            ComponentType::NetworkCard => t.t("components.type_network_card"),
                                                                                            ComponentType::Motherboard => t.t("components.type_motherboard"),
                                                                                            ComponentType::PowerSupply => t.t("components.type_power_supply"),
                                                                                            ComponentType::Other => t.t("components.type_other"),
                                                                                        }
                                                                                    }
                                                                                </p>
                                                                                <p class="text-xs text-slate-500 mb-0">{component.vendor.clone().unwrap_or_default()}</p>
                                                                            </TableCell>
                                                                            <TableCell class="align-middle text-center text-sm">
                                                                                <span class={format!("badge badge-sm border {}", status_class)}>{status_text}</span>
                                                                            </TableCell>
                                                                            <TableCell class="align-middle text-center">
                                                                                if let Some(client_id) = &component.client_id {
                                                                                    <a href={format!("/clients/{}", client_id)} class="text-xs font-weight-bold text-blue-400 hover:text-blue-300">
                                                                                        if let Some(hostname) = &component.client_hostname {
                                                                                            {hostname}
                                                                                        } else {
                                                                                            {t.t("components.server_prefix")}{client_id.chars().take(8).collect::<String>()}{"..."}
                                                                                        }
                                                                                    </a>
                                                                                } else if let Some(loc) = &component.location {
                                                                                    <span class="text-xs font-weight-bold text-slate-300">{loc}</span>
                                                                                } else {
                                                                                    <span class="text-xs text-slate-500">{"-"}</span>
                                                                                }
                                                                            </TableCell>
                                                                            <TableCell class="align-middle text-center">
                                                                                <PermissionGuard min_role={Role::User}>
                                                                                    <TableActions 
                                                                                        on_edit={
                                                                                            let on_edit = on_edit.clone();
                                                                                            let c_edit = c_edit.clone();
                                                                                            Some(Callback::from(move |_| on_edit.emit(c_edit.clone())))
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
                                                    
                                                    <Pagination 
                                                        total_pages={*total_pages}
                                                        current_page={*current_page}
                                                        page_size={*page_size}
                                                        total_items={*total_items}
                                                        on_page_change={on_page_change}
                                                        on_page_size_change={on_page_size_change}
                                                    />
                                                </div>
                                            }
                                        </>
                                    }
                                }
                            }
                        </CardBody>
                        <input type="file" ref={file_input_ref} style="display: none" accept=".xlsx" onchange={on_file_change} />
                        
                        // Import Progress Modal
                        if *is_importing {
                            <div class="fixed inset-0 z-[9999] flex items-center justify-center bg-black/50 backdrop-blur-sm">
                                <div class="bg-base-100 w-96 p-6 rounded-xl shadow-2xl">
                                    <h3 class="font-bold text-lg mb-4">{t.t("components.importing")}</h3>
                                    <progress class="progress progress-primary w-full" value={import_progress.to_string()} max="100"></progress>
                                    <p class="text-center mt-2">{format!("{}%", *import_progress)}</p>
                                </div>
                            </div>
                        }
                    </Card>
                </div>
            </div>
        </div>
    }
}
