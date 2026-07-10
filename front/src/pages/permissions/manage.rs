use crate::components::notification::{Notification, NotificationType};
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::card::{Card, CardBody, CardHeader};
use crate::components::ui::checkbox::Checkbox;
use crate::components::ui::input::Input;
use crate::components::ui::modal::{Modal, ModalContent, ModalDescription, ModalHeader, ModalTitle};
use crate::components::ui::select::{Select, SelectOption};
use crate::components::ui::table::{Table, TableBody, TableCell, TableHead, TableHeader, TableRow};
use crate::hooks::use_trans::use_trans;
use crate::i18n::I18n;
use crate::services::permission::{
    create_permission_group, create_permission_rule, delete_permission_group,
    delete_permission_rule, fetch_permission_groups, fetch_permission_overview,
    fetch_permission_rules, update_permission_group, update_permission_rule,
};
use crate::types::PermissionOverviewResponse;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

fn split_csv_lines(value: &str) -> Vec<String> {
    value
        .split([',', '\n'])
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(str::to_string)
        .collect()
}

fn join_json_strings(value: &serde_json::Value) -> String {
    value
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|item| item.as_str().map(str::to_string))
        .collect::<Vec<_>>()
        .join("\n")
}

fn subject_type_label(subject_type: &str, t: &I18n) -> String {
    match subject_type {
        "Role" => t.t("permissions.manage.subject.role"),
        "User" => t.t("permissions.manage.subject.user"),
        "Group" => t.t("permissions.manage.subject.group"),
        other => other.to_string(),
    }
}

fn resource_type_label(resource_type: &str, t: &I18n) -> String {
    match resource_type {
        "Client" => t.t("permissions.manage.resource.client"),
        "Component" => t.t("permissions.manage.resource.component"),
        "Rack" => t.t("permissions.manage.resource.rack"),
        "Person" => t.t("permissions.manage.resource.person"),
        "Project" => t.t("permissions.manage.resource.project"),
        "Dictionary" => t.t("permissions.manage.resource.dictionary"),
        "Command" => t.t("permissions.manage.resource.command"),
        "User" => t.t("permissions.manage.resource.user"),
        other => other.to_string(),
    }
}

fn action_label(action: &str, t: &I18n) -> String {
    match action {
        "View" => t.t("permissions.manage.action.view"),
        "Create" => t.t("permissions.manage.action.create"),
        "Update" => t.t("permissions.manage.action.update"),
        "Delete" => t.t("permissions.manage.action.delete"),
        other => other.to_string(),
    }
}

fn subject_label(rule: &serde_json::Value, t: &I18n) -> String {
    let subject_type = rule["subject_type"].as_str().unwrap_or("-");
    let subject_id = rule["subject_id"].as_str().unwrap_or("-");
    format!("{}: {}", subject_type_label(subject_type, t), subject_id)
}

fn actions_label(rule: &serde_json::Value, t: &I18n) -> String {
    rule["actions"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|item| item.as_str().map(|value| action_label(value, t)))
        .collect::<Vec<_>>()
        .join(", ")
}

fn constraint_label(rule: &serde_json::Value) -> String {
    rule.get("constraint")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "null".into())
}

fn constraint_summary(rule: &serde_json::Value, t: &I18n) -> String {
    let Some(constraint) = rule.get("constraint") else {
        return t.t("permissions.manage.constraint.unset");
    };

    if let Some(obj) = constraint.as_object() {
        if obj.get("All").is_some() {
            return t.t("permissions.manage.constraint.all");
        }
        if obj.get("Owned").is_some() {
            return t.t("permissions.manage.constraint.owned");
        }
        if let Some(projects) = obj.get("Project").and_then(|value| value.as_array()) {
            let names = projects
                .iter()
                .filter_map(|value| value.as_str())
                .collect::<Vec<_>>();
            return if names.is_empty() {
                t.t("permissions.manage.constraint.project_scope")
            } else {
                format!("{}: {}", t.t("permissions.manage.constraint.project_list"), names.join(", "))
            };
        }
        if let Some(tags) = obj.get("Tag").and_then(|value| value.as_array()) {
            let names = tags
                .iter()
                .filter_map(|value| value.as_str())
                .collect::<Vec<_>>();
            return if names.is_empty() {
                t.t("permissions.manage.constraint.tag_scope")
            } else {
                format!("{}: {}", t.t("permissions.manage.constraint.tag_list"), names.join(", "))
            };
        }
        if obj.get("None").is_some() {
            return t.t("permissions.manage.constraint.none");
        }
    }

    constraint.to_string()
}

fn is_default_item_id(id: &str) -> bool {
    id.starts_with("default-")
}

const RULE_ACTION_OPTIONS: &[&str] = &["View", "Create", "Update", "Delete"];

fn constraint_preset_from_json(value: &serde_json::Value) -> (String, String) {
    let Some(constraint) = value.get("constraint") else {
        return ("All".to_string(), String::new());
    };
    if let Some(obj) = constraint.as_object() {
        if obj.get("All").is_some() {
            return ("All".to_string(), String::new());
        }
        if obj.get("Owned").is_some() {
            return ("Owned".to_string(), String::new());
        }
        if let Some(projects) = obj.get("Project").and_then(|value| value.as_array()) {
            let names = projects
                .iter()
                .filter_map(|value| value.as_str().map(str::to_string))
                .collect::<Vec<_>>()
                .join(", ");
            return ("Project".to_string(), names);
        }
        if let Some(tags) = obj.get("Tag").and_then(|value| value.as_array()) {
            let names = tags
                .iter()
                .filter_map(|value| value.as_str().map(str::to_string))
                .collect::<Vec<_>>()
                .join(", ");
            return ("Tag".to_string(), names);
        }
        if obj.get("None").is_some() {
            return ("None".to_string(), String::new());
        }
    }
    ("Raw".to_string(), constraint.to_string())
}

fn build_constraint_json(preset: &str, values: &str) -> serde_json::Value {
    match preset {
        "All" => serde_json::json!({ "All": null }),
        "Owned" => serde_json::json!({ "Owned": null }),
        "None" => serde_json::json!({ "None": null }),
        "Project" | "Tag" => {
            let items = values
                .split([',', '\n'])
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(str::to_string)
                .collect::<Vec<_>>();
            if preset == "Project" {
                serde_json::json!({ "Project": items })
            } else {
                serde_json::json!({ "Tag": items })
            }
        }
        _ => serde_json::from_str::<serde_json::Value>(values).unwrap_or_else(|_| serde_json::json!({ "All": null })),
    }
}

#[function_component(PermissionManagePage)]
pub fn permission_manage_page() -> Html {
    let t = use_trans();
    let overview = use_state(PermissionOverviewResponse::default);
    let rules = use_state(Vec::<serde_json::Value>::new);
    let groups = use_state(Vec::<serde_json::Value>::new);
    let loading = use_state(|| true);
    let notification = use_state(|| None::<(NotificationType, String)>);
    let active_tab = use_state(|| "rules".to_string());

    let show_rule_form = use_state(|| false);
    let editing_rule_id = use_state(|| None::<String>);
    let rule_id = use_state(String::new);
    let rule_subject_type = use_state(|| "Role".to_string());
    let rule_subject_id = use_state(|| "Admin".to_string());
    let rule_resource_type = use_state(|| "Command".to_string());
    let rule_actions = use_state(|| vec!["View".to_string(), "Create".to_string(), "Update".to_string(), "Delete".to_string()]);
    let rule_constraint_preset = use_state(|| "All".to_string());
    let rule_constraint_values = use_state(String::new);
    let rule_priority = use_state(|| "0".to_string());

    let show_group_form = use_state(|| false);
    let editing_group_id = use_state(|| None::<String>);
    let group_id = use_state(String::new);
    let group_name = use_state(String::new);
    let group_members = use_state(String::new);

    let reload_all = {
        let overview = overview.clone();
        let rules = rules.clone();
        let groups = groups.clone();
        let loading = loading.clone();
        let notification = notification.clone();
        Callback::from(move |_| {
            let overview = overview.clone();
            let rules = rules.clone();
            let groups = groups.clone();
            let loading = loading.clone();
            let notification = notification.clone();
            spawn_local(async move {
                loading.set(true);
                match fetch_permission_overview().await {
                    Ok(data) => overview.set(data),
                    Err(err) => notification.set(Some((NotificationType::Error, err.message))),
                }
                match fetch_permission_rules().await {
                    Ok(data) => rules.set(data),
                    Err(err) => notification.set(Some((NotificationType::Error, err.message))),
                }
                match fetch_permission_groups().await {
                    Ok(data) => groups.set(data),
                    Err(err) => notification.set(Some((NotificationType::Error, err.message))),
                }
                loading.set(false);
            });
        })
    };

    {
        let reload_all = reload_all.clone();
        use_effect_with((), move |_| {
            reload_all.emit(());
            || ()
        });
    }

    let close_notif = {
        let notification = notification.clone();
        Callback::from(move |_| notification.set(None))
    };

    let rule_subject_options = vec![
        SelectOption { value: "Role".into(), label: t.t("permissions.manage.subject.role") },
        SelectOption { value: "User".into(), label: t.t("permissions.manage.subject.user") },
        SelectOption { value: "Group".into(), label: t.t("permissions.manage.subject.group") },
    ];
    let resource_options = vec![
        "Client", "Component", "Rack", "Person", "Project", "Dictionary", "Command", "User",
    ]
    .into_iter()
    .map(|value| SelectOption { value: value.into(), label: resource_type_label(value, t.as_ref()) })
    .collect::<Vec<_>>();

    let close_rule_modal = {
        let show_rule_form = show_rule_form.clone();
        let editing_rule_id = editing_rule_id.clone();
        Callback::from(move |_| {
            show_rule_form.set(false);
            editing_rule_id.set(None);
        })
    };

    let close_group_modal = {
        let show_group_form = show_group_form.clone();
        let editing_group_id = editing_group_id.clone();
        Callback::from(move |_| {
            show_group_form.set(false);
            editing_group_id.set(None);
        })
    };

    let close_rule_modal_click = {
        let close_rule_modal = close_rule_modal.clone();
        Callback::from(move |_| close_rule_modal.emit(()))
    };

    let close_group_modal_click = {
        let close_group_modal = close_group_modal.clone();
        Callback::from(move |_| close_group_modal.emit(()))
    };

    let on_rule_submit = {
        let notification = notification.clone();
        let reload_all = reload_all.clone();
        let show_rule_form = show_rule_form.clone();
        let editing_rule_id = editing_rule_id.clone();
        let rule_id = rule_id.clone();
        let rule_subject_type = rule_subject_type.clone();
        let rule_subject_id = rule_subject_id.clone();
        let rule_resource_type = rule_resource_type.clone();
        let rule_actions = rule_actions.clone();
        let rule_constraint_preset = rule_constraint_preset.clone();
        let rule_constraint_values = rule_constraint_values.clone();
        let rule_priority = rule_priority.clone();
        let t = t.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let notification = notification.clone();
            let reload_all = reload_all.clone();
            let show_rule_form = show_rule_form.clone();
            let editing_rule_id = editing_rule_id.clone();
            let rule_id = (*rule_id).clone();
            let rule_subject_type = (*rule_subject_type).clone();
            let rule_subject_id = (*rule_subject_id).clone();
            let rule_resource_type = (*rule_resource_type).clone();
            let rule_actions = (*rule_actions).clone();
            let rule_constraint = build_constraint_json(&rule_constraint_preset, &rule_constraint_values);
            let rule_priority = rule_priority.parse::<i32>().unwrap_or_default();
            let t = t.clone();
            spawn_local(async move {
                let body = serde_json::json!({
                    "id": rule_id,
                    "subject_type": rule_subject_type,
                    "subject_id": rule_subject_id,
                    "resource_type": rule_resource_type,
                    "actions": rule_actions,
                    "constraint": rule_constraint,
                    "priority": rule_priority,
                });
                let result = if let Some(id) = (*editing_rule_id).clone() {
                    update_permission_rule(&id, &body).await
                } else {
                    create_permission_rule(&body).await
                };
                match result {
                    Ok(_) => {
                        notification.set(Some((NotificationType::Success, t.t("permissions.manage.messages.rule_saved"))));
                        show_rule_form.set(false);
                        editing_rule_id.set(None);
                        reload_all.emit(());
                    }
                    Err(err) => notification.set(Some((NotificationType::Error, err.message))),
                }
            });
        })
    };

    let on_group_submit = {
        let notification = notification.clone();
        let reload_all = reload_all.clone();
        let show_group_form = show_group_form.clone();
        let editing_group_id = editing_group_id.clone();
        let group_id = group_id.clone();
        let group_name = group_name.clone();
        let group_members = group_members.clone();
        let t = t.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let notification = notification.clone();
            let reload_all = reload_all.clone();
            let show_group_form = show_group_form.clone();
            let editing_group_id = editing_group_id.clone();
            let group_id = (*group_id).clone();
            let group_name = (*group_name).clone();
            let member_ids = split_csv_lines(&group_members);
            let t = t.clone();
            spawn_local(async move {
                let body = serde_json::json!({
                    "id": group_id,
                    "name": group_name,
                    "member_ids": member_ids,
                });
                let result = if let Some(id) = (*editing_group_id).clone() {
                    update_permission_group(&id, &body).await
                } else {
                    create_permission_group(&body).await
                };
                match result {
                    Ok(_) => {
                        notification.set(Some((NotificationType::Success, t.t("permissions.manage.messages.group_saved"))));
                        show_group_form.set(false);
                        editing_group_id.set(None);
                        reload_all.emit(());
                    }
                    Err(err) => notification.set(Some((NotificationType::Error, err.message))),
                }
            });
        })
    };

    html! {
        <div class="space-y-6">
            if let Some((ty, msg)) = (*notification).clone() {
                <Notification notification_type={ty} message={msg} show={true} on_close={close_notif} />
            }

            <div class="grid gap-4 md:grid-cols-4">
                <Card><CardBody><div class="text-xs text-muted-foreground">{t.t("permissions.manage.overview.rules")}</div><div class="text-2xl font-semibold">{overview.rules_count}</div></CardBody></Card>
                <Card><CardBody><div class="text-xs text-muted-foreground">{t.t("permissions.manage.overview.groups")}</div><div class="text-2xl font-semibold">{overview.groups_count}</div></CardBody></Card>
                <Card><CardBody><div class="text-xs text-muted-foreground">{t.t("permissions.manage.overview.exec_policies")}</div><div class="text-2xl font-semibold">{overview.exec_policies_count}</div></CardBody></Card>
                <Card><CardBody><div class="text-xs text-muted-foreground">{t.t("permissions.manage.overview.web_terminal_policies")}</div><div class="text-2xl font-semibold">{overview.web_terminal_policies_count}</div></CardBody></Card>
            </div>

            <div class="flex items-center justify-between">
                <h2 class="text-xl font-bold">{t.t("permissions.manage.title")}</h2>
                <div class="tabs tabs-boxed bg-muted p-1">
                    <button class={classes!("tab", (*active_tab == "rules").then_some("tab-active"))} onclick={{ let active_tab = active_tab.clone(); Callback::from(move |_| active_tab.set("rules".into())) }}>{t.t("permissions.manage.tab.rules")}</button>
                    <button class={classes!("tab", (*active_tab == "groups").then_some("tab-active"))} onclick={{ let active_tab = active_tab.clone(); Callback::from(move |_| active_tab.set("groups".into())) }}>{t.t("permissions.manage.tab.groups")}</button>
                </div>
            </div>

            if *active_tab == "rules" {
                <Card>
                    <CardHeader class="flex flex-row items-center justify-between">
                        <div>
                            <h3 class="text-lg font-semibold">{t.t("permissions.manage.rules.title")}</h3>
                            <p class="text-sm text-muted-foreground">{t.t("permissions.manage.rules.description")}</p>
                        </div>
                        <Button variant={ButtonVariant::Default} size={ButtonSize::Sm} onclick={{
                            let show_rule_form = show_rule_form.clone();
                            let editing_rule_id = editing_rule_id.clone();
                            let rule_id = rule_id.clone();
                            let rule_subject_type = rule_subject_type.clone();
                            let rule_subject_id = rule_subject_id.clone();
                            let rule_resource_type = rule_resource_type.clone();
                            let rule_actions = rule_actions.clone();
                            let rule_constraint_preset = rule_constraint_preset.clone();
                            let rule_constraint_values = rule_constraint_values.clone();
                            let rule_priority = rule_priority.clone();
                            Callback::from(move |_| {
                                editing_rule_id.set(None);
                                rule_id.set(String::new());
                                rule_subject_type.set("Role".to_string());
                                rule_subject_id.set("Admin".to_string());
                                rule_resource_type.set("Command".to_string());
                                rule_actions.set(vec!["View".to_string(), "Create".to_string(), "Update".to_string(), "Delete".to_string()]);
                                rule_constraint_preset.set("All".to_string());
                                rule_constraint_values.set(String::new());
                                rule_priority.set("0".to_string());
                                show_rule_form.set(true);
                            })
                        }}>
                            {t.t("permissions.manage.actions.new_rule")}
                        </Button>
                    </CardHeader>
                    <CardBody>
                        if *loading {
                            <div class="py-8 text-center text-muted-foreground">{t.t("permissions.manage.loading")}</div>
                        } else if rules.is_empty() {
                            <div class="py-8 text-center text-muted-foreground">{t.t("permissions.manage.empty_rules")}</div>
                        } else {
                            <Table>
                                <TableHeader>
                                    <TableRow>
                                        <TableHead>{t.t("permissions.manage.table.id")}</TableHead>
                                        <TableHead>{t.t("permissions.manage.table.subject")}</TableHead>
                                        <TableHead>{t.t("permissions.manage.table.resource")}</TableHead>
                                        <TableHead>{t.t("permissions.manage.table.actions")}</TableHead>
                                        <TableHead>{t.t("permissions.manage.table.constraint")}</TableHead>
                                        <TableHead>{t.t("permissions.manage.table.priority")}</TableHead>
                                        <TableHead class="text-right">{t.t("permissions.manage.table.operations")}</TableHead>
                                    </TableRow>
                                </TableHeader>
                                <TableBody>
                                        { for rules.iter().map(|rule| {
                                            let rule_value = rule.clone();
                                            let delete_id = rule["id"].as_str().unwrap_or_default().to_string();
                                            let is_default = is_default_item_id(&delete_id);
                                            html! {
                                                <TableRow>
                                                    <TableCell class="font-mono text-xs">
                                                        <div class="flex items-center gap-2">
                                                            <span>{rule["id"].as_str().unwrap_or("-")}</span>
                                                            if is_default {
                                                                <Badge variant={BadgeVariant::Secondary}>{t.t("permissions.manage.labels.default")}</Badge>
                                                            }
                                                        </div>
                                                    </TableCell>
                                                    <TableCell>{subject_label(rule, t.as_ref())}</TableCell>
                                                    <TableCell>{resource_type_label(rule["resource_type"].as_str().unwrap_or("-"), t.as_ref())}</TableCell>
                                                    <TableCell class="text-xs text-muted-foreground">{actions_label(rule, t.as_ref())}</TableCell>
                                                    <TableCell class="max-w-xs truncate text-xs text-muted-foreground">
                                                        <span title={constraint_label(rule)}>{constraint_summary(rule, t.as_ref())}</span>
                                                    </TableCell>
                                                    <TableCell>{rule["priority"].as_i64().unwrap_or_default()}</TableCell>
                                                    <TableCell class="text-right">
                                                         <div class="flex justify-end gap-2">
                                                             <Button variant={ButtonVariant::Outline} size={ButtonSize::Sm} onclick={{
                                                                 let show_rule_form = show_rule_form.clone();
                                                                let editing_rule_id = editing_rule_id.clone();
                                                                let rule_id = rule_id.clone();
                                                                let rule_subject_type = rule_subject_type.clone();
                                                                let rule_subject_id = rule_subject_id.clone();
                                                                 let rule_resource_type = rule_resource_type.clone();
                                                                 let rule_actions = rule_actions.clone();
                                                                 let rule_constraint_preset = rule_constraint_preset.clone();
                                                                 let rule_constraint_values = rule_constraint_values.clone();
                                                                 let rule_priority = rule_priority.clone();
                                                                 Callback::from(move |_| {
                                                                    editing_rule_id.set(rule_value.get("id").and_then(|v| v.as_str()).map(str::to_string));
                                                                    rule_id.set(rule_value["id"].as_str().unwrap_or_default().to_string());
                                                                    rule_subject_type.set(rule_value["subject_type"].as_str().unwrap_or("Role").to_string());
                                                                    rule_subject_id.set(rule_value["subject_id"].as_str().unwrap_or_default().to_string());
                                                                    rule_resource_type.set(rule_value["resource_type"].as_str().unwrap_or("Command").to_string());
                                                                     rule_actions.set(rule_value["actions"].as_array().cloned().unwrap_or_default().into_iter().filter_map(|item| item.as_str().map(str::to_string)).collect::<Vec<_>>());
                                                                     let (preset, values) = constraint_preset_from_json(&rule_value);
                                                                     rule_constraint_preset.set(preset);
                                                                     rule_constraint_values.set(values);
                                                                       rule_priority.set(rule_value["priority"].as_i64().unwrap_or_default().to_string());
                                                                      show_rule_form.set(true);
                                                                  })
                                                             }}>{t.t("permissions.manage.actions.edit")}</Button>
                                                             <span title={if is_default { t.t("permissions.manage.messages.default_protected") } else { String::new() }}>
                                                                 <Button variant={ButtonVariant::Destructive} size={ButtonSize::Sm} disabled={is_default} onclick={{
                                                                     let reload_all = reload_all.clone();
                                                                     let notification = notification.clone();
                                                                     let t = t.clone();
                                                                     Callback::from(move |_| {
                                                                         let reload_all = reload_all.clone();
                                                                         let notification = notification.clone();
                                                                         let delete_id = delete_id.clone();
                                                                         let t = t.clone();
                                                                         spawn_local(async move {
                                                                             match delete_permission_rule(&delete_id).await {
                                                                                 Ok(_) => {
                                                                                     notification.set(Some((NotificationType::Success, t.t("permissions.manage.messages.rule_deleted"))));
                                                                                     reload_all.emit(());
                                                                                 }
                                                                                 Err(err) => notification.set(Some((NotificationType::Error, err.message))),
                                                                             }
                                                                         });
                                                                     })
                                                                 }}>{t.t("permissions.manage.actions.delete")}</Button>
                                                             </span>
                                                         </div>
                                                     </TableCell>
                                                 </TableRow>
                                            }
                                        }) }
                                </TableBody>
                            </Table>
                        }
                    </CardBody>
                </Card>
            } else {
                <Card>
                    <CardHeader class="flex flex-row items-center justify-between">
                        <div>
                            <h3 class="text-lg font-semibold">{t.t("permissions.manage.groups.title")}</h3>
                            <p class="text-sm text-muted-foreground">{t.t("permissions.manage.groups.description")}</p>
                        </div>
                        <Button variant={ButtonVariant::Default} size={ButtonSize::Sm} onclick={{
                            let show_group_form = show_group_form.clone();
                            let editing_group_id = editing_group_id.clone();
                            let group_id = group_id.clone();
                            let group_name = group_name.clone();
                            let group_members = group_members.clone();
                            Callback::from(move |_| {
                                editing_group_id.set(None);
                                group_id.set(String::new());
                                group_name.set(String::new());
                                group_members.set(String::new());
                                show_group_form.set(true);
                            })
                        }}>
                            {t.t("permissions.manage.actions.new_group")}
                        </Button>
                    </CardHeader>
                    <CardBody>
                        if *loading {
                            <div class="py-8 text-center text-muted-foreground">{t.t("permissions.manage.loading")}</div>
                        } else if groups.is_empty() {
                            <div class="py-8 text-center text-muted-foreground">{t.t("permissions.manage.empty_groups")}</div>
                        } else {
                            <Table>
                                <TableHeader>
                                    <TableRow>
                                        <TableHead>{t.t("permissions.manage.table.id")}</TableHead>
                                        <TableHead>{t.t("permissions.manage.table.name")}</TableHead>
                                        <TableHead>{t.t("permissions.manage.table.member_count")}</TableHead>
                                        <TableHead>{t.t("permissions.manage.table.members")}</TableHead>
                                        <TableHead class="text-right">{t.t("permissions.manage.table.operations")}</TableHead>
                                    </TableRow>
                                </TableHeader>
                                <TableBody>
                                        { for groups.iter().map(|group| {
                                            let group_value = group.clone();
                                            let delete_id = group["id"].as_str().unwrap_or_default().to_string();
                                            let is_default = is_default_item_id(&delete_id);
                                            let members = join_json_strings(&group["member_ids"]);
                                            html! {
                                                <TableRow>
                                                    <TableCell class="font-mono text-xs">
                                                        <div class="flex items-center gap-2">
                                                            <span>{group["id"].as_str().unwrap_or("-")}</span>
                                                            if is_default {
                                                                <Badge variant={BadgeVariant::Secondary}>{t.t("permissions.manage.labels.default")}</Badge>
                                                            }
                                                        </div>
                                                    </TableCell>
                                                    <TableCell>{group["name"].as_str().unwrap_or("-")}</TableCell>
                                                    <TableCell>{group["member_ids"].as_array().map(|items| items.len()).unwrap_or_default()}</TableCell>
                                                    <TableCell class="max-w-md whitespace-pre-wrap break-all text-xs text-muted-foreground">{members.clone()}</TableCell>
                                                    <TableCell class="text-right">
                                                        <div class="flex justify-end gap-2">
                                                            <Button variant={ButtonVariant::Outline} size={ButtonSize::Sm} onclick={{
                                                                let show_group_form = show_group_form.clone();
                                                                let editing_group_id = editing_group_id.clone();
                                                                let group_id = group_id.clone();
                                                                let group_name = group_name.clone();
                                                                let group_members = group_members.clone();
                                                                Callback::from(move |_| {
                                                                    editing_group_id.set(group_value.get("id").and_then(|v| v.as_str()).map(str::to_string));
                                                                    group_id.set(group_value["id"].as_str().unwrap_or_default().to_string());
                                                                    group_name.set(group_value["name"].as_str().unwrap_or_default().to_string());
                                                                      group_members.set(join_json_strings(&group_value["member_ids"]));
                                                                      show_group_form.set(true);
                                                                  })
                                                             }}>{t.t("permissions.manage.actions.edit")}</Button>
                                                             <span title={if is_default { t.t("permissions.manage.messages.default_protected") } else { String::new() }}>
                                                                 <Button variant={ButtonVariant::Destructive} size={ButtonSize::Sm} disabled={is_default} onclick={{
                                                                     let reload_all = reload_all.clone();
                                                                     let notification = notification.clone();
                                                                     let t = t.clone();
                                                                     Callback::from(move |_| {
                                                                         let reload_all = reload_all.clone();
                                                                         let notification = notification.clone();
                                                                         let delete_id = delete_id.clone();
                                                                         let t = t.clone();
                                                                         spawn_local(async move {
                                                                             match delete_permission_group(&delete_id).await {
                                                                                 Ok(_) => {
                                                                                     notification.set(Some((NotificationType::Success, t.t("permissions.manage.messages.group_deleted"))));
                                                                                     reload_all.emit(());
                                                                                 }
                                                                                 Err(err) => notification.set(Some((NotificationType::Error, err.message))),
                                                                             }
                                                                         });
                                                                     })
                                                                 }}>{t.t("permissions.manage.actions.delete")}</Button>
                                                             </span>
                                                         </div>
                                                     </TableCell>
                                                 </TableRow>
                                            }
                                        }) }
                                </TableBody>
                            </Table>
                        }
                    </CardBody>
                </Card>
            }

            <Modal is_open={*show_rule_form} on_close={close_rule_modal.clone()} content_class={classes!("max-w-4xl")}>
                <ModalHeader>
                    <ModalTitle>
                        {if editing_rule_id.is_some() { t.t("permissions.manage.actions.update_rule") } else { t.t("permissions.manage.actions.new_rule") }}
                    </ModalTitle>
                    <ModalDescription>{t.t("permissions.manage.rules.description")}</ModalDescription>
                </ModalHeader>
                <ModalContent class="py-0">
                    <form onsubmit={on_rule_submit.clone()} class="grid gap-4 lg:grid-cols-2">
                        <div>
                            <label class="mb-1 block text-sm">{t.t("permissions.manage.rule.id")}</label>
                            <Input value={(*rule_id).clone()} oninput={let state = rule_id.clone(); Callback::from(move |v| state.set(v))} />
                        </div>
                        <div>
                            <label class="mb-1 block text-sm">{t.t("permissions.manage.rule.subject_type")}</label>
                            <Select options={rule_subject_options.clone()} value={(*rule_subject_type).clone()} onchange={let state = rule_subject_type.clone(); Callback::from(move |v| state.set(v))} />
                        </div>
                        <div>
                            <label class="mb-1 block text-sm">{t.t("permissions.manage.rule.subject_id")}</label>
                            <Input value={(*rule_subject_id).clone()} oninput={let state = rule_subject_id.clone(); Callback::from(move |v| state.set(v))} />
                        </div>
                        <div>
                            <label class="mb-1 block text-sm">{t.t("permissions.manage.rule.resource_type")}</label>
                            <Select options={resource_options.clone()} value={(*rule_resource_type).clone()} onchange={let state = rule_resource_type.clone(); Callback::from(move |v| state.set(v))} />
                        </div>
                        <div>
                            <label class="mb-1 block text-sm">{t.t("permissions.manage.rule.actions")}</label>
                            <div class="grid grid-cols-2 gap-2 rounded-lg border border-border/70 bg-background/60 p-3">
                                { for RULE_ACTION_OPTIONS.iter().map(|action| {
                                    let action_value = action.to_string();
                                    let checked = (*rule_actions).contains(&action_value);
                                    let rule_actions = rule_actions.clone();
                                    let onchange = Callback::from(move |selected: bool| {
                                        let mut next = (*rule_actions).clone();
                                        if selected {
                                            if !next.contains(&action_value) {
                                                next.push(action_value.clone());
                                            }
                                        } else {
                                            next.retain(|item| item != &action_value);
                                        }
                                        rule_actions.set(next);
                                    });
                                    html! {
                                        <label class="flex items-center gap-2 text-sm">
                                            <Checkbox checked={checked} onchange={onchange} />
                                            {action_label(action, t.as_ref())}
                                        </label>
                                    }
                                }) }
                            </div>
                            <p class="mt-1 text-xs text-muted-foreground">{t.t("permissions.manage.rule.actions_help")}</p>
                        </div>
                        <div>
                            <label class="mb-1 block text-sm">{t.t("permissions.manage.rule.constraint")}</label>
                            <Select
                                options={vec![
                                    SelectOption { value: "All".into(), label: t.t("permissions.manage.constraint.all") },
                                    SelectOption { value: "Owned".into(), label: t.t("permissions.manage.constraint.owned") },
                                    SelectOption { value: "Project".into(), label: t.t("permissions.manage.constraint.project_scope") },
                                    SelectOption { value: "Tag".into(), label: t.t("permissions.manage.constraint.tag_scope") },
                                    SelectOption { value: "None".into(), label: t.t("permissions.manage.constraint.none") },
                                ]}
                                value={(*rule_constraint_preset).clone()}
                                onchange={let state = rule_constraint_preset.clone(); Callback::from(move |v| state.set(v))}
                            />
                            if (*rule_constraint_preset == "Project" || *rule_constraint_preset == "Tag") {
                                <Input
                                    class="mt-2"
                                    placeholder={t.t("permissions.manage.rule.constraint_values_placeholder")}
                                    value={(*rule_constraint_values).clone()}
                                    oninput={let state = rule_constraint_values.clone(); Callback::from(move |v| state.set(v))}
                                />
                            }
                            <p class="mt-1 text-xs text-muted-foreground">{t.t("permissions.manage.rule.constraint_help")}</p>
                        </div>
                        <div>
                            <label class="mb-1 block text-sm">{t.t("permissions.manage.rule.priority")}</label>
                            <Input type_="number" value={(*rule_priority).clone()} oninput={let state = rule_priority.clone(); Callback::from(move |v| state.set(v))} />
                        </div>
                        <div class="flex items-end justify-end gap-2 lg:col-span-2">
                            <Button type_="button" variant={ButtonVariant::Outline} onclick={close_rule_modal_click.clone()}>{t.t("permissions.manage.actions.cancel")}</Button>
                            <Button type_="submit">{ if editing_rule_id.is_some() { t.t("permissions.manage.actions.update_rule") } else { t.t("permissions.manage.actions.save_rule") } }</Button>
                        </div>
                    </form>
                </ModalContent>
            </Modal>

            <Modal is_open={*show_group_form} on_close={close_group_modal.clone()} content_class={classes!("max-w-3xl")}>
                <ModalHeader>
                    <ModalTitle>
                        {if editing_group_id.is_some() { t.t("permissions.manage.actions.update_group") } else { t.t("permissions.manage.actions.new_group") }}
                    </ModalTitle>
                    <ModalDescription>{t.t("permissions.manage.groups.description")}</ModalDescription>
                </ModalHeader>
                <ModalContent class="py-0">
                    <form onsubmit={on_group_submit} class="grid gap-4 lg:grid-cols-2">
                        <div>
                            <label class="mb-1 block text-sm">{t.t("permissions.manage.group.id")}</label>
                            <Input value={(*group_id).clone()} oninput={let state = group_id.clone(); Callback::from(move |v| state.set(v))} />
                        </div>
                        <div>
                            <label class="mb-1 block text-sm">{t.t("permissions.manage.group.name")}</label>
                            <Input value={(*group_name).clone()} oninput={let state = group_name.clone(); Callback::from(move |v| state.set(v))} />
                        </div>
                        <div class="lg:col-span-2">
                            <label class="mb-1 block text-sm">{t.t("permissions.manage.group.members")}</label>
                            <textarea class="textarea textarea-bordered min-h-28 w-full bg-background/60" value={(*group_members).clone()} oninput={let state = group_members.clone(); Callback::from(move |e: InputEvent| state.set(e.target_unchecked_into::<web_sys::HtmlTextAreaElement>().value()))} />
                            <p class="mt-1 text-xs text-muted-foreground">{t.t("permissions.manage.group.members_help")}</p>
                        </div>
                        <div class="flex justify-end gap-2 lg:col-span-2">
                            <Button type_="button" variant={ButtonVariant::Outline} onclick={close_group_modal_click.clone()}>{t.t("permissions.manage.actions.cancel")}</Button>
                            <Button type_="submit">{ if editing_group_id.is_some() { t.t("permissions.manage.actions.update_group") } else { t.t("permissions.manage.actions.save_group") } }</Button>
                        </div>
                    </form>
                </ModalContent>
            </Modal>
        </div>
    }
}
