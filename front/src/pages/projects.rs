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
use crate::services::api::{fetch_dictionaries, fetch_persons};
use crate::services::project::{create_project, delete_project, get_projects, update_project};
use crate::types::Role;
use common::entity::dictionary::Dictionary;
use common::models::{Person, Project};
use lucide_yew::Plus;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ProjectFormProps {
    pub project: Option<Project>,
    pub on_save: Callback<Project>,
    pub on_cancel: Callback<()>,
}

#[function_component(ProjectForm)]
pub fn project_form(props: &ProjectFormProps) -> Html {
    let t = use_trans();
    let name = use_state(|| {
        props
            .project
            .as_ref()
            .map(|p| p.name.clone())
            .unwrap_or_default()
    });
    let code = use_state(|| {
        props
            .project
            .as_ref()
            .and_then(|p| p.code.clone())
            .unwrap_or_default()
    });
    let department = use_state(|| {
        props
            .project
            .as_ref()
            .and_then(|p| p.department.clone())
            .unwrap_or_default()
    });
    let cost_center = use_state(|| {
        props
            .project
            .as_ref()
            .and_then(|p| p.cost_center.clone())
            .unwrap_or_default()
    });
    let manager_id = use_state(|| {
        props
            .project
            .as_ref()
            .and_then(|p| p.manager_id.clone())
            .unwrap_or_default()
    });

    let departments = use_state(Vec::<Dictionary>::new);
    let cost_centers = use_state(Vec::<Dictionary>::new);
    let persons = use_state(Vec::<Person>::new);

    {
        let departments = departments.clone();
        let cost_centers = cost_centers.clone();
        let persons = persons.clone();
        use_effect_with((), move |_| {
            let departments = departments.clone();
            let cost_centers = cost_centers.clone();
            let persons = persons.clone();
            spawn_local(async move {
                if let Ok(data) = fetch_dictionaries(Some("Department".to_string())).await {
                    departments.set(data);
                }
            });
            spawn_local(async move {
                if let Ok(data) = fetch_dictionaries(Some("CostCenter".to_string())).await {
                    cost_centers.set(data);
                }
            });
            spawn_local(async move {
                if let Ok(data) = fetch_persons(1, 1000, None, None).await {
                    persons.set(data.items);
                }
            });
            || ()
        });
    }

    let onsubmit = {
        let name = name.clone();
        let code = code.clone();
        let department = department.clone();
        let cost_center = cost_center.clone();
        let manager_id = manager_id.clone();
        let props_project = props.project.clone();
        let on_save = props.on_save.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let mut project = props_project.clone().unwrap_or_default();
            project.name = (*name).clone();
            project.code = if (*code).is_empty() {
                None
            } else {
                Some((*code).clone())
            };
            project.department = if (*department).is_empty() {
                None
            } else {
                Some((*department).clone())
            };
            project.cost_center = if (*cost_center).is_empty() {
                None
            } else {
                Some((*cost_center).clone())
            };
            project.manager_id = if (*manager_id).is_empty() {
                None
            } else {
                Some((*manager_id).clone())
            };

            on_save.emit(project);
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

    let cost_center_options: Vec<SelectOption> = std::iter::once(SelectOption {
        value: "".to_string(),
        label: t.t("projects.select_cost_center"),
    })
    .chain(cost_centers.iter().map(|c| SelectOption {
        value: c.value.clone(),
        label: c.value.clone(),
    }))
    .collect();

    let manager_options: Vec<SelectOption> = std::iter::once(SelectOption {
        value: "".to_string(),
        label: t.t("projects.select_manager"),
    })
    .chain(persons.iter().map(|p| SelectOption {
        value: p.id.clone(),
        label: p.name.clone(),
    }))
    .collect();

    html! {
        <Card class="shadow-xl">
            <CardHeader>
                <h6 class="text-white text-capitalize ps-3">
                    { if props.project.is_some() { t.t("projects.edit_project") } else { t.t("projects.add_project") } }
                </h6>
            </CardHeader>
            <CardBody>
                <form {onsubmit}>
                    <div class="mb-4">
                        <label class="block text-sm font-bold text-slate-400 mb-1">{t.t("projects.name")}</label>
                        <Input
                            value={(*name).clone()}
                            oninput={Callback::from(move |val: String| name.set(val))}
                            required=true
                        />
                    </div>
                    <div class="mb-4">
                        <label class="block text-sm font-bold text-slate-400 mb-1">{t.t("projects.code")}</label>
                        <Input
                            value={(*code).clone()}
                            oninput={Callback::from(move |val: String| code.set(val))}
                        />
                    </div>
                    <div class="mb-4">
                        <label class="block text-sm font-bold text-slate-400 mb-1">{t.t("projects.department")}</label>
                        <Select
                            options={department_options}
                            value={(*department).clone()}
                            onchange={Callback::from(move |val: String| department.set(val))}
                        />
                    </div>
                    <div class="mb-4">
                        <label class="block text-sm font-bold text-slate-400 mb-1">{t.t("projects.cost_center")}</label>
                        <Select
                            options={cost_center_options}
                            value={(*cost_center).clone()}
                            onchange={Callback::from(move |val: String| cost_center.set(val))}
                        />
                    </div>
                    <div class="mb-4">
                        <label class="block text-sm font-bold text-slate-400 mb-1">{t.t("projects.manager")}</label>
                        <Select
                            options={manager_options}
                            value={(*manager_id).clone()}
                            onchange={Callback::from(move |val: String| manager_id.set(val))}
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

#[function_component(Projects)]
pub fn projects() -> Html {
    let t = use_trans();
    let projects = use_state(Vec::<Project>::new);
    let loading = use_state(|| true);
    let error = use_state(|| None::<String>);
    let show_form = use_state(|| false);
    let editing_project = use_state(|| None::<Project>);
    let delete_modal_open = use_state(|| false);
    let project_to_delete = use_state(|| None::<String>);
    let delete_error = use_state(|| None::<String>);
    let notification = use_state(|| None::<(NotificationType, String)>);

    let fetch_data = {
        let projects = projects.clone();
        let loading = loading.clone();
        let error = error.clone();
        Callback::from(move |_| {
            let projects = projects.clone();
            let loading = loading.clone();
            let error = error.clone();
            loading.set(true);
            get_projects(Callback::from(
                move |result: Result<Vec<Project>, crate::services::project::ApiError>| {
                    loading.set(false);
                    match result {
                        Ok(data) => projects.set(data),
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
        let editing_project = editing_project.clone();
        Callback::from(move |_| {
            let mut new_project = Project::default();
            new_project.id = String::new(); // Ensure ID is empty for new project
            editing_project.set(Some(new_project));
            show_form.set(true);
        })
    };

    let on_edit_click = {
        let show_form = show_form.clone();
        let editing_project = editing_project.clone();
        Callback::from(move |project: Project| {
            editing_project.set(Some(project));
            show_form.set(true);
        })
    };

    let on_delete_click = {
        let delete_modal_open = delete_modal_open.clone();
        let project_to_delete = project_to_delete.clone();
        let delete_error = delete_error.clone();
        Callback::from(move |id: String| {
            project_to_delete.set(Some(id));
            delete_error.set(None);
            delete_modal_open.set(true);
        })
    };

    let on_confirm_delete = {
        let fetch_data = fetch_data.clone();
        let delete_modal_open = delete_modal_open.clone();
        let project_to_delete = project_to_delete.clone();
        let notification = notification.clone();
        let delete_error = delete_error.clone();
        let t = t.clone();
        Callback::from(move |_| {
            let fetch_data = fetch_data.clone();
            let delete_modal_open = delete_modal_open.clone();
            let project_to_delete = project_to_delete.clone();
            let notification = notification.clone();
            let delete_error = delete_error.clone();
            let t = t.clone();

            if let Some(id) = (*project_to_delete).clone() {
                spawn_local(async move {
                    match delete_project(&id).await {
                        Ok(_) => {
                            delete_modal_open.set(false);
                            project_to_delete.set(None);
                            delete_error.set(None);
                            notification.set(Some((
                                NotificationType::Success,
                                t.t("common.delete_success"),
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
        let project_to_delete = project_to_delete.clone();
        let delete_error = delete_error.clone();
        Callback::from(move |_| {
            delete_modal_open.set(false);
            project_to_delete.set(None);
            delete_error.set(None);
        })
    };

    let on_save = {
        let show_form = show_form.clone();
        let fetch_data = fetch_data.clone();
        let notification = notification.clone();
        let t = t.clone();
        Callback::from(move |project: Project| {
            let show_form = show_form.clone();
            let fetch_data = fetch_data.clone();
            let notification = notification.clone();
            let t = t.clone();
            spawn_local(async move {
                let result = if project.id.is_empty() {
                    create_project(&project).await
                } else {
                    update_project(&project.id, &project).await
                };

                match result {
                    Ok(_) => {
                        show_form.set(false);
                        notification.set(Some((
                            NotificationType::Success,
                            t.t("common.save_success"),
                        )));
                        fetch_data.emit(());
                    }
                    Err(e) => {
                        notification.set(Some((
                            NotificationType::Error,
                            t.t("common.save_failed").replace("{}", &e.message),
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
                                        <Plus class="h-4 w-4 mr-1" /> {t.t("projects.new_project")}
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
                                    <ProjectForm project={(*editing_project).clone()} {on_save} {on_cancel} />
                                </div>
                            } else {
                                <div class="table-responsive p-0">
                                    <Table>
                                        <TableHeader>
                                            <TableRow>
                                                <TableHead>{t.t("projects.name")}</TableHead>
                                                <TableHead>{t.t("projects.department")}</TableHead>
                                                <TableHead class="text-center">{t.t("projects.cost_center")}</TableHead>
                                                <TableHead class="text-center">{t.t("common.actions")}</TableHead>
                                            </TableRow>
                                        </TableHeader>
                                        <TableBody>
                                            {
                                                for projects.iter().map(|project| {
                                                    let p_edit = project.clone();
                                                    let p_delete = project.clone();
                                                    let on_edit = on_edit_click.clone();
                                                    let on_delete = on_delete_click.clone();
                                                    html! {
                                                        <TableRow class="hover:bg-slate-800/50 transition-colors">
                                                            <TableCell>
                                                                <div class="d-flex px-2 py-1">
                                                                    <div class="d-flex flex-column justify-content-center">
                                                                        <h6 class="mb-0 text-sm text-slate-200">{&project.name}</h6>
                                                                        <p class="text-xs text-slate-400 mb-0">{project.code.clone().unwrap_or_default()}</p>
                                                                    </div>
                                                                </div>
                                                            </TableCell>
                                                            <TableCell>
                                                                <p class="text-xs font-weight-bold mb-0 text-slate-300">{project.department.clone().unwrap_or_default()}</p>
                                                            </TableCell>
                                                            <TableCell class="align-middle text-center text-sm">
                                                                <span class="text-slate-400 text-xs font-weight-bold">{project.cost_center.clone().unwrap_or_default()}</span>
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
                title={t.t("common.confirm_delete")}
                message={t.t("projects.confirm_delete_msg")}
                on_confirm={on_confirm_delete}
                on_cancel={on_cancel_delete}
                error_message={(*delete_error).clone()}
            />
        </div>
    }
}
