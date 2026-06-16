use crate::components::error::ErrorDisplay;
use crate::components::loading::Loading;
use crate::components::notification::{Notification, NotificationType};
use crate::components::permission_guard::PermissionGuard;
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::card::{Card, CardBody, CardHeader};
use crate::components::ui::confirm_modal::ConfirmModal;
use crate::components::ui::input::Input;
use crate::components::ui::table::{Table, TableBody, TableCell, TableHead, TableHeader, TableRow};
use crate::components::ui::table_action::TableActions;
use crate::hooks::use_trans::use_trans;
use crate::icons::{Briefcase, Building2, Plus, Wallet};
use crate::services::api::{
    create_dictionary, delete_dictionary, fetch_dictionaries, update_dictionary,
};
use common::entity::dictionary::Dictionary;
use common::entity::user::Role;
use std::collections::HashMap;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct DictionaryFormProps {
    pub item: Dictionary,
    pub category: String,
    pub on_save: Callback<Dictionary>,
    pub on_cancel: Callback<()>,
}

#[function_component(DictionaryForm)]
pub fn dictionary_form(props: &DictionaryFormProps) -> Html {
    let t = use_trans();
    let key = use_state(|| props.item.key.clone());
    let value = use_state(|| props.item.value.clone());
    let description = use_state(|| props.item.description.clone().unwrap_or_default());

    let onsubmit = {
        let key = key.clone();
        let value = value.clone();
        let description = description.clone();
        let props_item = props.item.clone();
        let category = props.category.clone();
        let on_save = props.on_save.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let mut item = props_item.clone();
            item.category = category.clone();
            item.key = (*key).clone();
            item.value = (*value).clone();
            item.description = if (*description).is_empty() {
                None
            } else {
                Some((*description).clone())
            };

            on_save.emit(item);
        })
    };

    let category_name = match props.category.as_str() {
        "Department" => t.t("dictionaries.department"),
        "Title" => t.t("dictionaries.title"),
        "CostCenter" => t.t("dictionaries.cost_center"),
        _ => t.t("dictionaries.dictionary_item"),
    };

    html! {
        <Card class="shadow-xl">
            <CardHeader>
                <h6 class="text-white text-capitalize ps-3">
                    {if props.item.id.is_empty() { format!("{}{}", t.t("dictionaries.create_prefix"), category_name) } else { format!("{}{}", t.t("dictionaries.edit_prefix"), category_name) }}
                </h6>
            </CardHeader>
            <CardBody>
                <div class="alert alert-info text-white mb-4 text-sm">
                    <div class="flex flex-col gap-1">
                        <p><strong>{t.t("dictionaries.key_label")}</strong> {t.t("dictionaries.key_desc")}</p>
                        <p><strong>{t.t("dictionaries.value_label")}</strong> {t.t("dictionaries.value_desc")}</p>
                    </div>
                </div>
                <form {onsubmit}>
                    <div class="mb-4">
                        <label class="block text-sm font-bold text-slate-400 mb-1">{t.t("dictionaries.key")}</label>
                        <Input
                            value={(*key).clone()}
                            oninput={Callback::from(move |val: String| key.set(val))}
                            required=true
                            placeholder={t.t("dictionaries.key_placeholder")}
                        />
                    </div>

                    <div class="mb-4">
                        <label class="block text-sm font-bold text-slate-400 mb-1">{t.t("dictionaries.value")}</label>
                        <Input
                            value={(*value).clone()}
                            oninput={Callback::from(move |val: String| value.set(val))}
                            required=true
                            placeholder={t.t("dictionaries.value_placeholder")}
                        />
                    </div>

                    <div class="mb-4">
                        <label class="block text-sm font-bold text-slate-400 mb-1">{t.t("dictionaries.description")}</label>
                        <Input
                            value={(*description).clone()}
                            oninput={Callback::from(move |val: String| description.set(val))}
                            placeholder={t.t("dictionaries.description_placeholder")}
                        />
                    </div>

                    <div class="flex justify-end gap-4 mt-6">
                        <Button type_="button" variant={ButtonVariant::Outline} onclick={props.on_cancel.reform(|_| ())}>{t.t("dictionaries.cancel")}</Button>
                        <Button type_="submit" variant={ButtonVariant::Default}>{t.t("dictionaries.save")}</Button>
                    </div>
                </form>
            </CardBody>
        </Card>
    }
}

#[function_component(Dictionaries)]
pub fn dictionaries() -> Html {
    let t = use_trans();
    let items = use_state(Vec::<Dictionary>::new);
    let loading = use_state(|| true);
    let error = use_state(|| None::<String>);
    let show_form = use_state(|| false);
    let editing_item = use_state(Dictionary::default);
    let delete_modal_open = use_state(|| false);
    let item_to_delete = use_state(|| None::<String>);
    let delete_error = use_state(|| None::<String>);
    let notification = use_state(|| None::<(NotificationType, String)>);

    // Category selection
    let active_category = use_state(|| "Department".to_string());

    let fetch_data = {
        let items = items.clone();
        let loading = loading.clone();
        let error = error.clone();
        let active_category = active_category.clone();

        Callback::from(move |_| {
            let items = items.clone();
            let loading = loading.clone();
            let error = error.clone();
            let category = (*active_category).clone();

            loading.set(true);

            spawn_local(async move {
                match fetch_dictionaries(Some(category)).await {
                    Ok(data) => {
                        items.set(data);
                        loading.set(false);
                    }
                    Err(err) => {
                        error.set(Some(err.message));
                        loading.set(false);
                    }
                }
            });
        })
    };

    // Initial load & Category change
    use_effect_with(active_category.clone(), {
        let fetch_data = fetch_data.clone();
        move |_| {
            fetch_data.emit(());
            || ()
        }
    });

    let on_category_change = {
        let active_category = active_category.clone();
        Callback::from(move |category: String| {
            active_category.set(category);
        })
    };

    let on_add_click = {
        let show_form = show_form.clone();
        let editing_item = editing_item.clone();
        Callback::from(move |_| {
            let mut item = Dictionary {
                id: String::new(),
                ..Default::default()
            };
            editing_item.set(item);
            show_form.set(true);
        })
    };

    let on_edit_click = {
        let show_form = show_form.clone();
        let editing_item = editing_item.clone();
        Callback::from(move |item: Dictionary| {
            editing_item.set(item);
            show_form.set(true);
        })
    };

    let on_delete_click = {
        let delete_modal_open = delete_modal_open.clone();
        let item_to_delete = item_to_delete.clone();
        let delete_error = delete_error.clone();
        Callback::from(move |id: String| {
            item_to_delete.set(Some(id));
            delete_error.set(None);
            delete_modal_open.set(true);
        })
    };

    let on_confirm_delete = {
        let fetch_data = fetch_data.clone();
        let delete_modal_open = delete_modal_open.clone();
        let item_to_delete = item_to_delete.clone();
        let notification = notification.clone();
        let delete_error = delete_error.clone();
        let t = t.clone();
        Callback::from(move |_| {
            let fetch_data = fetch_data.clone();
            let delete_modal_open = delete_modal_open.clone();
            let item_to_delete = item_to_delete.clone();
            let notification = notification.clone();
            let delete_error = delete_error.clone();
            let t = t.clone();

            if let Some(id) = (*item_to_delete).clone() {
                spawn_local(async move {
                    match delete_dictionary(&id).await {
                        Ok(_) => {
                            delete_modal_open.set(false);
                            item_to_delete.set(None);
                            delete_error.set(None);
                            notification.set(Some((
                                NotificationType::Success,
                                t.t("dictionaries.delete_success"),
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
        let item_to_delete = item_to_delete.clone();
        let delete_error = delete_error.clone();
        Callback::from(move |_| {
            delete_modal_open.set(false);
            item_to_delete.set(None);
            delete_error.set(None);
        })
    };

    let on_save = {
        let show_form = show_form.clone();
        let fetch_data = fetch_data.clone();
        let notification = notification.clone();
        let t = t.clone();
        Callback::from(move |item: Dictionary| {
            let show_form = show_form.clone();
            let fetch_data = fetch_data.clone();
            let notification = notification.clone();
            let t = t.clone();
            spawn_local(async move {
                let result = if item.id.is_empty() {
                    create_dictionary(&item).await
                } else {
                    update_dictionary(&item.id, &item).await
                };

                match result {
                    Ok(_) => {
                        show_form.set(false);
                        notification.set(Some((
                            NotificationType::Success,
                            t.t("dictionaries.save_success"),
                        )));
                        fetch_data.emit(());
                    }
                    Err(e) => {
                        notification.set(Some((
                            NotificationType::Error,
                            t.t_with_args(
                                "dictionaries.save_failed",
                                &HashMap::from([("error".to_string(), e.message)]),
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
                        <CardHeader class="flex flex-row justify-start items-center gap-4">
                            <PermissionGuard min_role={Role::User}>
                                <Button variant={ButtonVariant::Default} size={ButtonSize::Sm} onclick={on_add_click}>
                                    <Plus class="h-4 w-4 mr-1" />
                                    {
                                        match active_category.as_str() {
                                            "Department" => t.t("dictionaries.create_department"),
                                            "Title" => t.t("dictionaries.create_title"),
                                            "CostCenter" => t.t("dictionaries.create_cost_center"),
                                            _ => t.t("dictionaries.create_item"),
                                        }
                                    }
                                </Button>
                            </PermissionGuard>

                            <div class="flex space-x-2 bg-muted/50 p-1 rounded-lg">
                                <Button
                                    variant={if *active_category == "Department" { ButtonVariant::Default } else { ButtonVariant::Ghost }}
                                    size={ButtonSize::Sm}
                                    onclick={on_category_change.reform(|_| "Department".to_string())}
                                >
                                    <Building2 class="h-4 w-4 mr-2" />{t.t("dictionaries.department")}
                                </Button>
                                <Button
                                    variant={if *active_category == "Title" { ButtonVariant::Default } else { ButtonVariant::Ghost }}
                                    size={ButtonSize::Sm}
                                    onclick={on_category_change.reform(|_| "Title".to_string())}
                                >
                                    <Briefcase class="h-4 w-4 mr-2" />{t.t("dictionaries.title")}
                                </Button>
                                <Button
                                    variant={if *active_category == "CostCenter" { ButtonVariant::Default } else { ButtonVariant::Ghost }}
                                    size={ButtonSize::Sm}
                                    onclick={on_category_change.reform(|_| "CostCenter".to_string())}
                                >
                                    <Wallet class="h-4 w-4 mr-2" />{t.t("dictionaries.cost_center")}
                                </Button>
                            </div>
                        </CardHeader>

                        <CardBody class="px-0 pb-2">
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
                                    <DictionaryForm
                                        item={(*editing_item).clone()}
                                        category={(*active_category).clone()}
                                        {on_save}
                                        {on_cancel}
                                    />
                                </div>
                            } else {
                                <div class="table-responsive p-0">
                                    <Table>
                                        <TableHeader>
                                            <TableRow>
                                                <TableHead>{t.t("dictionaries.key_label")}</TableHead>
                                                <TableHead>{t.t("dictionaries.value_label")}</TableHead>
                                                <TableHead>{t.t("dictionaries.description")}</TableHead>
                                                <TableHead class="text-center">{t.t("dictionaries.actions")}</TableHead>
                                            </TableRow>
                                        </TableHeader>
                                        <TableBody>
                                            {
                                                for items.iter().map(|item| {
                                                    let i_edit = item.clone();
                                                    let i_del = item.clone();
                                                    let on_edit = on_edit_click.clone();
                                                    let on_delete = on_delete_click.clone();

                                                    html! {
                                                        <TableRow class="hover:bg-slate-800/50 transition-colors">
                                                            <TableCell>
                                                                <span class="font-mono text-xs bg-slate-800 px-2 py-1 rounded text-slate-300">{&item.key}</span>
                                                            </TableCell>
                                                            <TableCell>
                                                                <span class="font-bold text-sm text-slate-200">{&item.value}</span>
                                                            </TableCell>
                                                            <TableCell>
                                                                <span class="text-sm text-slate-400">{item.description.clone().unwrap_or_default()}</span>
                                                            </TableCell>
                                                            <TableCell class="align-middle text-center">
                                                                <PermissionGuard min_role={Role::User}>
                                                                    <TableActions
                                                                        on_edit={
                                                                            let on_edit = on_edit.clone();
                                                                            let i_edit = i_edit.clone();
                                                                            Some(Callback::from(move |_| on_edit.emit(i_edit.clone())))
                                                                        }
                                                                        on_delete={
                                                                            let on_delete = on_delete.clone();
                                                                            let id = i_del.id.clone();
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
                            }
                        </CardBody>
                    </Card>
                </div>
            </div>
            <ConfirmModal
                is_open={*delete_modal_open}
                title={t.t("dictionaries.confirm_delete_title")}
                message={t.t("dictionaries.confirm_delete_message")}
                on_confirm={on_confirm_delete}
                on_cancel={on_cancel_delete}
                error_message={(*delete_error).clone()}
            />
        </div>
    }
}
