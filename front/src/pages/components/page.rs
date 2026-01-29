use crate::components::common::pagination::Pagination;
use crate::components::error::ErrorDisplay;
use crate::components::loading::Loading;
use crate::components::permission_guard::PermissionGuard;
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::card::{Card, CardBody, CardHeader};
use crate::components::ui::checkbox::Checkbox;
use crate::components::ui::input::Input;
use crate::components::ui::select::{Select, SelectOption};
use crate::components::ui::table::{Table, TableBody, TableCell, TableHead, TableHeader, TableRow};
use crate::components::ui::table_action::TableActions;
use crate::hooks::use_trans::use_trans;
use crate::pages::components::batch_form::BatchCreateForm;
use crate::pages::components::form::ComponentForm;
use crate::pages::components::import_export::{export_components, handle_file_import};
use crate::services::component::{
    batch_create_components, batch_update_components, create_component, get_components,
    update_component,
};
use common::entity::user::Role;
use common::models::{Component, ComponentStatus, ComponentType, PaginatedResult};
use lucide_yew::{FileCode, FileSpreadsheet, Plus};
use std::collections::HashMap;
use std::collections::HashSet;
use wasm_bindgen_futures::spawn_local;
use web_sys::{Event, HtmlInputElement};
use yew::prelude::*;

#[function_component(Components)]
pub fn components() -> Html {
    let t = use_trans();
    let components = use_state(Vec::<Component>::new);
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
    let total_items = use_state(|| 0usize);

    // Filters
    let filter_type = use_state(|| "all".to_string());
    let filter_status = use_state(|| "all".to_string());
    let filter_search = use_state(|| "".to_string());

    // Batch Selection
    let selected_components = use_state(HashSet::<String>::new);

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
                Callback::from(
                    move |result: Result<
                        PaginatedResult<Component>,
                        crate::services::component::ApiError,
                    >| {
                        loading.set(false);
                        match result {
                            Ok(data) => {
                                components.set(data.items);
                                total_pages.set(data.total_pages);
                                total_items.set(data.total);
                            }
                            Err(err) => error.set(Some(err.message)),
                        }
                    },
                ),
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

            if !gloo::dialogs::confirm(&t.t_with_args(
                "components.confirm_batch_status_update",
                &HashMap::from([
                    ("count".to_string(), selected_ids.len().to_string()),
                    ("status".to_string(), val.clone()),
                ]),
            )) {
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
                    }
                    Err(e) => gloo::dialogs::alert(
                        &t.t("components.batch_status_update_failed")
                            .replace("{error}", &e.message),
                    ),
                }
            });
        })
    };

    let on_export_selected_click = {
        let components = components.clone();
        let selected_ids = selected_components.clone();
        Callback::from(move |_: MouseEvent| {
            let selected: Vec<Component> = components
                .iter()
                .filter(|c| selected_ids.contains(&c.id))
                .cloned()
                .collect();
            export_components(&selected);
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
                    handle_file_import(
                        file,
                        is_importing.clone(),
                        import_progress.clone(),
                        fetch_data.clone(),
                        t.clone(),
                    );
                }
            }
            input.set_value("");
        })
    };

    // Initial load and Pagination
    {
        let fetch_data = fetch_data.clone();
        use_effect_with((*current_page, *page_size), move |_| {
            fetch_data.emit(());
            || ()
        });
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
                    update_component(&component.id, &component)
                        .await
                        .map(|_| ())
                };

                match result {
                    Ok(_) => {
                        show_form.set(false);
                        fetch_data.emit(());
                    }
                    Err(e) => {
                        gloo::dialogs::alert(&t.t("persons.save_failed").replace("{}", &e.message))
                    }
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
                        gloo::dialogs::alert(&t.t_with_args(
                            "components.batch_create_success",
                            &HashMap::from([("count".to_string(), count.to_string())]),
                        ));
                    }
                    Err(e) => gloo::dialogs::alert(
                        &t.t("components.batch_create_failed")
                            .replace("{error}", &e.message),
                    ),
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
        SelectOption {
            value: "all".to_string(),
            label: t.t("all"),
        },
        SelectOption {
            value: "GPU".to_string(),
            label: t.t("components.type_gpu"),
        },
        SelectOption {
            value: "CPU".to_string(),
            label: t.t("components.type_cpu"),
        },
        SelectOption {
            value: "Memory".to_string(),
            label: t.t("components.type_memory"),
        },
        SelectOption {
            value: "Disk".to_string(),
            label: t.t("components.type_disk"),
        },
        SelectOption {
            value: "NetworkCard".to_string(),
            label: t.t("components.type_network_card"),
        },
    ];

    let filter_status_options = vec![
        SelectOption {
            value: "all".to_string(),
            label: t.t("all"),
        },
        SelectOption {
            value: "InStock".to_string(),
            label: t.t("components.status_in_stock"),
        },
        SelectOption {
            value: "InUse".to_string(),
            label: t.t("components.status_in_use"),
        },
        SelectOption {
            value: "LentOut".to_string(),
            label: t.t("components.status_lent_out"),
        },
        SelectOption {
            value: "Faulty".to_string(),
            label: t.t("components.status_faulty"),
        },
        SelectOption {
            value: "Decommissioned".to_string(),
            label: t.t("components.status_decommissioned"),
        },
    ];

    let batch_status_options = vec![
        SelectOption {
            value: "none".to_string(),
            label: t.t("components.quick_status_change"),
        },
        SelectOption {
            value: "InStock".to_string(),
            label: t.t("components.status_in_stock"),
        },
        SelectOption {
            value: "InUse".to_string(),
            label: t.t("components.status_in_use"),
        },
        SelectOption {
            value: "LentOut".to_string(),
            label: t.t("components.status_lent_out"),
        },
        SelectOption {
            value: "Faulty".to_string(),
            label: t.t("components.status_faulty"),
        },
        SelectOption {
            value: "Decommissioned".to_string(),
            label: t.t("components.status_decommissioned"),
        },
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
                                                                    let id = component.id.clone();

                                                                    html! {
                                                                        <TableRow>
                                                                            <TableCell>
                                                                                <Checkbox
                                                                                    checked={is_selected}
                                                                                    onchange={Callback::from(move |_| toggle_selection.emit(id.clone()))}
                                                                                />
                                                                            </TableCell>
                                                                            <TableCell>
                                                                                <div class="flex flex-col">
                                                                                    <span class="text-sm font-bold text-white">{&component.model}</span>
                                                                                    <span class="text-xs text-slate-400">{&component.serial_number}</span>
                                                                                </div>
                                                                            </TableCell>
                                                                            <TableCell>
                                                                                <div class="flex flex-col">
                                                                                    <span class="text-sm font-bold text-white">{format!("{:?}", component.component_type)}</span>
                                                                                    <span class="text-xs text-slate-400">{component.vendor.clone().unwrap_or_default()}</span>
                                                                                </div>
                                                                            </TableCell>
                                                                            <TableCell class="text-center">
                                                                                <span class={classes!(
                                                                                    "badge",
                                                                                    match component.status {
                                                                                        ComponentStatus::InStock => "bg-gradient-to-tl from-emerald-500 to-teal-400",
                                                                                        ComponentStatus::InUse => "bg-gradient-to-tl from-blue-500 to-violet-500",
                                                                                        ComponentStatus::Faulty => "bg-gradient-to-tl from-red-600 to-rose-400",
                                                                                        _ => "bg-gradient-to-tl from-slate-600 to-slate-300",
                                                                                    }
                                                                                )}>
                                                                                    {format!("{:?}", component.status)}
                                                                                </span>
                                                                            </TableCell>
                                                                            <TableCell class="text-center">
                                                                                <div class="flex flex-col">
                                                                                    <span class="text-sm text-slate-300">{component.location.clone().unwrap_or_default()}</span>
                                                                                    <span class="text-xs text-slate-500">{component.client_hostname.clone().unwrap_or_default()}</span>
                                                                                </div>
                                                                            </TableCell>
                                                                            <TableCell class="text-center">
                                                                                <PermissionGuard min_role={Role::User}>
                                                                                    <TableActions
                                                                                        on_edit={Callback::from(move |_| on_edit.emit(c_edit.clone()))}
                                                                                        on_delete={Callback::noop()} // TODO: Implement delete
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
                                            }
                                        </>
                                    }
                                }
                            }
                        </CardBody>
                        if !*show_form && !*show_batch_form {
                            <div class="p-4 border-t border-slate-700">
                                <Pagination
                                    current_page={*current_page}
                                    total_pages={*total_pages}
                                    on_page_change={on_page_change}
                                    page_size={*page_size}
                                    on_page_size_change={on_page_size_change}
                                    total_items={*total_items}
                                />
                            </div>
                        }
                    </Card>
                </div>
            </div>

            // Hidden file input for import
            <input
                type="file"
                ref={file_input_ref}
                class="hidden"
                accept=".xlsx,.xls"
                onchange={on_file_change}
            />

            // Import Progress Modal
            if *is_importing {
                <div class="fixed inset-0 bg-slate-900/50 z-50 flex justify-center items-center backdrop-blur-sm">
                    <Card class="w-96 shadow-2xl">
                        <CardHeader>
                            <h6 class="text-white mb-0">{t.t("components.importing")}</h6>
                        </CardHeader>
                        <CardBody class="text-center py-8">
                            <div class="w-full bg-slate-700 rounded-full h-4 mb-4 overflow-hidden">
                                <div
                                    class="bg-blue-500 h-4 rounded-full transition-all duration-300 ease-out"
                                    style={format!("width: {}%", *import_progress)}
                                ></div>
                            </div>
                            <p class="text-white">{format!("{}%", *import_progress)}</p>
                        </CardBody>
                    </Card>
                </div>
            }
        </div>
    }
}
