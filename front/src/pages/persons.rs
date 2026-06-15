use crate::components::error::ErrorDisplay;
use crate::components::loading::Loading;
use crate::components::notification::{Notification, NotificationType};
use crate::components::permission_guard::PermissionGuard;
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::card::{Card, CardBody, CardHeader};
use crate::components::ui::confirm_modal::ConfirmModal;
use crate::components::ui::input::Input;
use crate::components::ui::select::{Select, SelectOption};
use crate::components::ui::table::{Table, TableBody, TableCell, TableHead, TableHeader, TableRow};
use crate::components::ui::table_action::TableActions;
use crate::hooks::use_trans::use_trans;
use crate::icons::Plus;
use crate::services::api::fetch_dictionaries;
use crate::services::person::{create_person, delete_person, get_persons, update_person};
use crate::types::Role;
use common::entity::dictionary::Dictionary;
use common::models::Person;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct PersonFormProps {
    pub person: Option<Person>,
    pub on_save: Callback<Person>,
    pub on_cancel: Callback<()>,
}

#[function_component(PersonForm)]
pub fn person_form(props: &PersonFormProps) -> Html {
    let t = use_trans();
    let name = use_state(|| {
        props
            .person
            .as_ref()
            .map(|p| p.name.clone())
            .unwrap_or_default()
    });
    let email = use_state(|| {
        props
            .person
            .as_ref()
            .map(|p| p.email.clone())
            .unwrap_or_default()
    });
    let department = use_state(|| {
        props
            .person
            .as_ref()
            .and_then(|p| p.department.clone())
            .unwrap_or_default()
    });
    let phone = use_state(|| {
        props
            .person
            .as_ref()
            .and_then(|p| p.phone.clone())
            .unwrap_or_default()
    });
    let title = use_state(|| {
        props
            .person
            .as_ref()
            .and_then(|p| p.title.clone())
            .unwrap_or_default()
    });

    let departments = use_state(Vec::<Dictionary>::new);
    let titles = use_state(Vec::<Dictionary>::new);

    {
        let departments = departments.clone();
        let titles = titles.clone();
        use_effect_with((), move |_| {
            let departments = departments.clone();
            let titles = titles.clone();
            spawn_local(async move {
                if let Ok(data) = fetch_dictionaries(Some("Department".to_string())).await {
                    departments.set(data);
                }
            });
            spawn_local(async move {
                if let Ok(data) = fetch_dictionaries(Some("Title".to_string())).await {
                    titles.set(data);
                }
            });
            || ()
        });
    }

    let onsubmit = {
        let name = name.clone();
        let email = email.clone();
        let department = department.clone();
        let phone = phone.clone();
        let title = title.clone();
        let props_person = props.person.clone();
        let on_save = props.on_save.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let mut person = props_person.clone().unwrap_or_default();
            person.name = (*name).clone();
            person.email = (*email).clone();
            person.department = if (*department).is_empty() {
                None
            } else {
                Some((*department).clone())
            };
            person.phone = if (*phone).is_empty() {
                None
            } else {
                Some((*phone).clone())
            };
            person.title = if (*title).is_empty() {
                None
            } else {
                Some((*title).clone())
            };

            on_save.emit(person);
        })
    };

    let department_options: Vec<SelectOption> = std::iter::once(SelectOption {
        value: "".to_string(),
        label: t.t("persons.select_department"),
    })
    .chain(departments.iter().map(|d| SelectOption {
        value: d.value.clone(),
        label: d.value.clone(),
    }))
    .collect();

    let title_options: Vec<SelectOption> = std::iter::once(SelectOption {
        value: "".to_string(),
        label: t.t("persons.select_title"),
    })
    .chain(titles.iter().map(|t| SelectOption {
        value: t.value.clone(),
        label: t.value.clone(),
    }))
    .collect();

    html! {
        <Card class="shadow-xl">
            <CardHeader>
                <h6 class="text-white text-capitalize ps-3">
                    { if props.person.is_some() { t.t("persons.edit_person") } else { t.t("persons.add_person") } }
                </h6>
            </CardHeader>
            <CardBody>
                <form {onsubmit}>
                    <div class="mb-4">
                        <label class="block text-sm font-bold text-slate-400 mb-1">{t.t("persons.name")}</label>
                        <Input
                            type_="text"
                            oninput={Callback::from(move |val: String| name.set(val))}
                            required=true
                        />
                    </div>
                    <div class="mb-4">
                        <label class="block text-sm font-bold text-slate-400 mb-1">{t.t("persons.email")}</label>
                        <Input
                            type_="email"
                            value={(*email).clone()}
                            oninput={Callback::from(move |val: String| email.set(val))}
                            required=true
                        />
                    </div>
                    <div class="mb-4">
                        <label class="block text-sm font-bold text-slate-400 mb-1">{t.t("persons.department")}</label>
                        <Select
                            options={department_options}
                            value={(*department).clone()}
                            onchange={Callback::from(move |val: String| department.set(val))}
                        />
                    </div>
                    <div class="mb-4">
                        <label class="block text-sm font-bold text-slate-400 mb-1">{t.t("persons.phone")}</label>
                        <Input
                            value={(*phone).clone()}
                            oninput={Callback::from(move |val: String| phone.set(val))}
                        />
                    </div>
                    <div class="mb-4">
                        <label class="block text-sm font-bold text-slate-400 mb-1">{t.t("persons.title")}</label>
                        <Select
                            options={title_options}
                            value={(*title).clone()}
                            onchange={Callback::from(move |val: String| title.set(val))}
                        />
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

#[function_component(Persons)]
pub fn persons() -> Html {
    let t = use_trans();
    let persons = use_state(Vec::<Person>::new);
    let loading = use_state(|| true);
    let error = use_state(|| None::<String>);
    let show_form = use_state(|| false);
    let editing_person = use_state(|| None::<Person>);
    let delete_modal_open = use_state(|| false);
    let person_to_delete = use_state(|| None::<String>);
    let delete_error = use_state(|| None::<String>);
    let notification = use_state(|| None::<(NotificationType, String)>);

    let fetch_data = {
        let persons = persons.clone();
        let loading = loading.clone();
        let error = error.clone();
        Callback::from(move |_| {
            let persons = persons.clone();
            let loading = loading.clone();
            let error = error.clone();
            loading.set(true);
            get_persons(Callback::from(
                move |result: Result<Vec<Person>, crate::services::person::ApiError>| {
                    loading.set(false);
                    match result {
                        Ok(data) => persons.set(data),
                        Err(err) => error.set(Some(err.message)),
                    }
                },
            ));
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
        let editing_person = editing_person.clone();
        Callback::from(move |_| {
            let mut new_person = Person { id: String::new(), ..Default::default() };
            editing_person.set(Some(new_person));
            show_form.set(true);
        })
    };

    let on_edit_click = {
        let show_form = show_form.clone();
        let editing_person = editing_person.clone();
        Callback::from(move |person: Person| {
            editing_person.set(Some(person));
            show_form.set(true);
        })
    };

    let on_delete_click = {
        let delete_modal_open = delete_modal_open.clone();
        let person_to_delete = person_to_delete.clone();
        let delete_error = delete_error.clone();
        Callback::from(move |id: String| {
            person_to_delete.set(Some(id));
            delete_error.set(None);
            delete_modal_open.set(true);
        })
    };

    let on_confirm_delete = {
        let fetch_data = fetch_data.clone();
        let delete_modal_open = delete_modal_open.clone();
        let person_to_delete = person_to_delete.clone();
        let notification = notification.clone();
        let delete_error = delete_error.clone();
        let t = t.clone();
        Callback::from(move |_| {
            let fetch_data = fetch_data.clone();
            let delete_modal_open = delete_modal_open.clone();
            let person_to_delete = person_to_delete.clone();
            let notification = notification.clone();
            let delete_error = delete_error.clone();
            let t = t.clone();

            if let Some(id) = (*person_to_delete).clone() {
                spawn_local(async move {
                    match delete_person(&id).await {
                        Ok(_) => {
                            delete_modal_open.set(false);
                            person_to_delete.set(None);
                            delete_error.set(None);
                            notification.set(Some((
                                NotificationType::Success,
                                t.t("persons.delete_success"),
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
        let person_to_delete = person_to_delete.clone();
        let delete_error = delete_error.clone();
        Callback::from(move |_| {
            delete_modal_open.set(false);
            person_to_delete.set(None);
            delete_error.set(None);
        })
    };

    let on_save = {
        let show_form = show_form.clone();
        let fetch_data = fetch_data.clone();
        let notification = notification.clone();
        let t = t.clone();
        Callback::from(move |person: Person| {
            let show_form = show_form.clone();
            let fetch_data = fetch_data.clone();
            let notification = notification.clone();
            let t = t.clone();
            spawn_local(async move {
                let result = if person.id.is_empty() {
                    create_person(&person).await
                } else {
                    update_person(&person.id, &person).await
                };

                match result {
                    Ok(_) => {
                        show_form.set(false);
                        notification.set(Some((
                            NotificationType::Success,
                            t.t("persons.save_success"),
                        )));
                        fetch_data.emit(());
                    }
                    Err(e) => {
                        notification.set(Some((
                            NotificationType::Error,
                            t.t("persons.save_failed").replace("{}", &e.message),
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
                            <div class="flex gap-2">
                                <PermissionGuard min_role={Role::User}>
                                    <Button variant={ButtonVariant::Default} size={ButtonSize::Sm} onclick={on_add_click}>
                                        <Plus class="h-4 w-4 mr-1" /> {t.t("persons.new_person")}
                                    </Button>
                                </PermissionGuard>
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
                                    <PersonForm person={(*editing_person).clone()} {on_save} {on_cancel} />
                                </div>
                            } else {
                                <div class="table-responsive p-0">
                                    <Table>
                                        <TableHeader>
                                            <TableRow>
                                                <TableHead>{t.t("persons.name")}</TableHead>
                                                <TableHead>{t.t("persons.department")}</TableHead>
                                                <TableHead class="text-center">{t.t("persons.title")}</TableHead>
                                                <TableHead class="text-center">{t.t("persons.actions")}</TableHead>
                                            </TableRow>
                                        </TableHeader>
                                        <TableBody>
                                            {
                                                for persons.iter().map(|person| {
                                                    let p_edit = person.clone();
                                                    let p_delete = person.clone();
                                                    let on_edit = on_edit_click.clone();
                                                    let on_delete = on_delete_click.clone();
                                                    html! {
                                                        <TableRow class="hover:bg-slate-800/50 transition-colors">
                                                            <TableCell>
                                                                <div class="d-flex px-2 py-1">
                                                                    <div class="d-flex flex-column justify-content-center">
                                                                        <h6 class="mb-0 text-sm text-slate-200">{&person.name}</h6>
                                                                        <p class="text-xs text-slate-400 mb-0">{&person.email}</p>
                                                                    </div>
                                                                </div>
                                                            </TableCell>
                                                            <TableCell>
                                                                <p class="text-xs font-weight-bold mb-0 text-slate-300">{person.department.clone().unwrap_or_default()}</p>
                                                                <p class="text-xs text-slate-500 mb-0">{person.phone.clone().unwrap_or_default()}</p>
                                                            </TableCell>
                                                            <TableCell class="align-middle text-center text-sm">
                                                                <span class="badge badge-sm bg-gradient-success">{person.title.clone().unwrap_or_default()}</span>
                                                            </TableCell>
                                                            <TableCell class="align-middle text-center">
                                                                <PermissionGuard min_role={Role::User}>
                                                                    <TableActions
                                                                        on_edit={
                                                                            let on_edit = on_edit.clone();
                                                                            let p_edit = p_edit.clone();
                                                                            Some(Callback::from(move |_| on_edit.emit(p_edit.clone())))
                                                                        }
                                                                        on_delete={
                                                                            let on_delete = on_delete.clone();
                                                                            let id = p_delete.id.clone();
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
                title={t.t("persons.confirm_delete")}
                message={t.t("persons.confirm_delete_msg")}
                on_confirm={on_confirm_delete}
                on_cancel={on_cancel_delete}
                error_message={(*delete_error).clone()}
            />
        </div>
    }
}
