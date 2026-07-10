use crate::components::notification::{Notification, NotificationType};
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::card::{Card, CardBody, CardHeader};
use crate::components::ui::checkbox::Checkbox;
use crate::components::ui::confirm_modal::ConfirmModal;
use crate::components::ui::input::Input;
use crate::components::ui::modal::{Modal, ModalContent, ModalDescription, ModalHeader, ModalTitle};
use crate::components::ui::select::{Select, SelectOption};
use crate::components::ui::table::{Table, TableBody, TableCell, TableHead, TableHeader, TableRow};
use crate::services::permission::{
    create_exec_policy, delete_exec_policy, fetch_exec_policies, update_exec_policy,
};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

fn parse_command_lines(value: &str) -> Vec<String> {
    value
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect()
}

fn build_command_rules(mode: &str, commands: &str) -> serde_json::Value {
    let commands = parse_command_lines(commands);
    match mode {
        "allow_list" => serde_json::json!({
            "default_action": "Deny",
            "overrides": commands.into_iter().map(|pattern| serde_json::json!({
                "pattern": pattern,
                "action": "Allow"
            })).collect::<Vec<_>>()
        }),
        "deny_all" => serde_json::json!({
            "default_action": "Deny",
            "overrides": []
        }),
        "deny_list" => serde_json::json!({
            "default_action": "Allow",
            "overrides": commands.into_iter().map(|pattern| serde_json::json!({
                "pattern": pattern,
                "action": "Deny"
            })).collect::<Vec<_>>()
        }),
        _ => serde_json::json!({
            "default_action": "Allow",
            "overrides": []
        }),
    }
}

fn command_rules_summary(p: &serde_json::Value) -> String {
    let default_action = p
        .get("command_rules")
        .and_then(|rules| rules.get("default_action"))
        .and_then(|value| value.as_str())
        .unwrap_or("Allow");
    let overrides = p
        .get("command_rules")
        .and_then(|rules| rules.get("overrides"))
        .and_then(|value| value.as_array())
        .cloned()
        .unwrap_or_default();

    match (default_action, overrides.is_empty()) {
        ("Allow", true) => "全部允许".to_string(),
        ("Deny", true) => "全部禁止".to_string(),
        ("Deny", false) => format!("白名单（{} 条）", overrides.len()),
        ("Allow", false) => format!("黑名单（{} 条）", overrides.len()),
        _ => default_action.to_string(),
    }
}

fn rule_mode_help(mode: &str) -> (&'static str, &'static str) {
    match mode {
        "deny_all" => ("全部禁止", "默认阻止所有命令，适合逐步开放前的强限制场景。"),
        "allow_list" => ("白名单", "仅允许列表中的命令执行，未列出的命令全部拒绝。"),
        "deny_list" => ("黑名单", "默认允许命令执行，但会阻止列表中列出的高风险命令。"),
        _ => ("全部允许", "默认允许所有命令，仅依赖后续审批或其他外围控制。"),
    }
}

fn form_rule_mode_from_policy(p: &serde_json::Value) -> String {
    let default_action = p
        .get("command_rules")
        .and_then(|rules| rules.get("default_action"))
        .and_then(|value| value.as_str())
        .unwrap_or("Allow");
    let overrides = p
        .get("command_rules")
        .and_then(|rules| rules.get("overrides"))
        .and_then(|value| value.as_array())
        .cloned()
        .unwrap_or_default();

    match (default_action, overrides.is_empty()) {
        ("Allow", true) => "allow_all".to_string(),
        ("Deny", true) => "deny_all".to_string(),
        ("Deny", false) => "allow_list".to_string(),
        ("Allow", false) => "deny_list".to_string(),
        _ => "allow_all".to_string(),
    }
}

fn form_commands_from_policy(p: &serde_json::Value) -> String {
    p.get("command_rules")
        .and_then(|rules| rules.get("overrides"))
        .and_then(|value| value.as_array())
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|item| item.get("pattern").and_then(|value| value.as_str()).map(str::to_string))
        .collect::<Vec<_>>()
        .join("\n")
}

fn subject_label(p: &serde_json::Value) -> String {
    let st = p["subject_type"].as_str().unwrap_or("");
    let si = p["subject_id"].as_str().unwrap_or("");
    match (st, si) {
        ("Role", "Admin") => "所有管理员".into(),
        ("Role", "User") => "所有用户".into(),
        ("Role", "Viewer") => "所有只读用户".into(),
        ("User", uid) => format!("用户 {}", uid),
        _ => format!("{}:{}", st, si),
    }
}

#[function_component(ExecPoliciesPage)]
pub fn exec_policies_page() -> Html {
    let policies = use_state(Vec::<serde_json::Value>::new);
    let loading = use_state(|| true);
    let notification = use_state(|| None::<(NotificationType, String)>);
    let show_form = use_state(|| false);
    let editing_id = use_state(|| None::<String>);
    let pending_delete = use_state(|| None::<(String, String)>);
    let form_id = use_state(String::new);
    let form_name = use_state(String::new);
    let form_subject = use_state(|| "Role:Admin".to_string());
    let form_subject_custom = use_state(String::new);
    let form_rule_mode = use_state(|| "allow_all".to_string());
    let form_commands = use_state(String::new);
    let form_require_approval = use_state(|| false);

    {
        let pols = policies.clone();
        let ld = loading.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                if let Ok(p) = fetch_exec_policies().await {
                    pols.set(p);
                }
                ld.set(false);
            });
            || ()
        });
    }

    let close_notif = {
        let notification = notification.clone();
        Callback::from(move |_| notification.set(None))
    };

    let subject_options = vec![
        SelectOption { value: "Role:Admin".into(), label: "所有管理员".into() },
        SelectOption { value: "Role:User".into(), label: "所有用户".into() },
        SelectOption { value: "Role:Viewer".into(), label: "所有只读用户".into() },
        SelectOption { value: "__custom__".into(), label: "指定用户...".into() },
    ];

    let action_options = vec![
        SelectOption { value: "allow_all".into(), label: "全部允许".into() },
        SelectOption { value: "deny_all".into(), label: "全部禁止".into() },
        SelectOption { value: "allow_list".into(), label: "白名单".into() },
        SelectOption { value: "deny_list".into(), label: "黑名单".into() },
    ];

    let open_create = {
        let show_form = show_form.clone();
        let editing_id = editing_id.clone();
        let form_id = form_id.clone();
        let form_name = form_name.clone();
        let form_subject = form_subject.clone();
        let form_subject_custom = form_subject_custom.clone();
        let form_rule_mode = form_rule_mode.clone();
        let form_commands = form_commands.clone();
        let form_require_approval = form_require_approval.clone();
        Callback::from(move |_| {
            editing_id.set(None);
            form_id.set(String::new());
            form_name.set(String::new());
            form_subject.set("Role:Admin".to_string());
            form_subject_custom.set(String::new());
            form_rule_mode.set("allow_all".to_string());
            form_commands.set(String::new());
            form_require_approval.set(false);
            show_form.set(true);
        })
    };

    let close_form = {
        let show_form = show_form.clone();
        let editing_id = editing_id.clone();
        Callback::from(move |_: ()| {
            show_form.set(false);
            editing_id.set(None);
        })
    };

    let close_form_click = {
        let close_form = close_form.clone();
        Callback::from(move |_| close_form.emit(()))
    };

    let onsubmit = {
        let notification = notification.clone();
        let show_form = show_form.clone();
        let policies = policies.clone();
        let editing_id = editing_id.clone();
        let form_id = form_id.clone();
        let form_name = form_name.clone();
        let form_subject = form_subject.clone();
        let form_subject_custom = form_subject_custom.clone();
        let form_rule_mode = form_rule_mode.clone();
        let form_commands = form_commands.clone();
        let form_require_approval = form_require_approval.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let notification = notification.clone();
            let show_form = show_form.clone();
            let policies = policies.clone();
            let editing_id = editing_id.clone();
            let form_id = form_id.clone();
            let form_name = form_name.clone();
            let form_subject = form_subject.clone();
            let form_subject_custom = form_subject_custom.clone();
            let form_rule_mode = form_rule_mode.clone();
            let form_commands = form_commands.clone();
            let form_require_approval = form_require_approval.clone();
            let (st, si) = if (*form_subject) == "__custom__" {
                ("User".to_string(), (*form_subject_custom).clone())
            } else {
                let parts: Vec<&str> = (*form_subject).splitn(2, ':').collect();
                if parts.len() == 2 {
                    (parts[0].to_string(), parts[1].to_string())
                } else {
                    ("Role".to_string(), "Admin".to_string())
                }
            };
            let body = serde_json::json!({
                "id": (*form_id).clone(),
                "name": (*form_name).clone(),
                "subject_type": st,
                "subject_id": si,
                "target_scope": "All",
                "command_rules": build_command_rules(form_rule_mode.as_str(), &form_commands),
                "priority": 0,
                "require_approval": *form_require_approval,
            });
            spawn_local(async move {
                let result = if let Some(id) = (*editing_id).clone() {
                    update_exec_policy(&id, &body).await
                } else {
                    create_exec_policy(&body).await
                };
                match result {
                    Ok(_) => {
                        notification.set(Some((
                            NotificationType::Success,
                            if editing_id.is_some() { "更新成功".into() } else { "创建成功".into() },
                        )));
                        editing_id.set(None);
                        form_id.set(String::new());
                        form_name.set(String::new());
                        form_subject_custom.set(String::new());
                        form_rule_mode.set("allow_all".to_string());
                        form_commands.set(String::new());
                        form_require_approval.set(false);
                        show_form.set(false);
                        if let Ok(list) = fetch_exec_policies().await {
                            policies.set(list);
                        }
                    }
                    Err(e) => notification.set(Some((NotificationType::Error, e.message))),
                }
            });
        })
    };

    let on_confirm_delete = {
        let notification = notification.clone();
        let policies = policies.clone();
        let pending_delete = pending_delete.clone();
        Callback::from(move |_: ()| {
            let Some((id, _)) = (*pending_delete).clone() else {
                return;
            };
            let notification = notification.clone();
            let policies = policies.clone();
            let pending_delete = pending_delete.clone();
            spawn_local(async move {
                match delete_exec_policy(&id).await {
                    Ok(_) => {
                        notification.set(Some((NotificationType::Success, "删除成功".into())));
                        pending_delete.set(None);
                        if let Ok(list) = fetch_exec_policies().await {
                            policies.set(list);
                        }
                    }
                    Err(e) => notification.set(Some((NotificationType::Error, e.message))),
                }
            });
        })
    };

    let cancel_delete = {
        let pending_delete = pending_delete.clone();
        Callback::from(move |_: ()| pending_delete.set(None))
    };

    let on_edit = {
        let show_form = show_form.clone();
        let editing_id = editing_id.clone();
        let form_id = form_id.clone();
        let form_name = form_name.clone();
        let form_subject = form_subject.clone();
        let form_subject_custom = form_subject_custom.clone();
        let form_rule_mode = form_rule_mode.clone();
        let form_commands = form_commands.clone();
        let form_require_approval = form_require_approval.clone();
        Callback::from(move |policy: serde_json::Value| {
            editing_id.set(policy.get("id").and_then(|v| v.as_str()).map(str::to_string));
            form_id.set(policy.get("id").and_then(|v| v.as_str()).unwrap_or_default().to_string());
            form_name.set(policy.get("name").and_then(|v| v.as_str()).unwrap_or_default().to_string());
            let subject_type = policy.get("subject_type").and_then(|v| v.as_str()).unwrap_or("Role");
            let subject_id = policy.get("subject_id").and_then(|v| v.as_str()).unwrap_or("Admin");
            if subject_type == "User" {
                form_subject.set("__custom__".to_string());
                form_subject_custom.set(subject_id.to_string());
            } else {
                form_subject.set(format!("{}:{}", subject_type, subject_id));
                form_subject_custom.set(String::new());
            }
            form_rule_mode.set(form_rule_mode_from_policy(&policy));
            form_commands.set(form_commands_from_policy(&policy));
            form_require_approval.set(policy.get("require_approval").and_then(|v| v.as_bool()).unwrap_or(false));
            show_form.set(true);
        })
    };

    let rule_mode_detail = rule_mode_help(form_rule_mode.as_str());

    html! {
        <div class="space-y-6">
            <Card class="border border-border/70 bg-card/95 shadow-xl">
                <CardHeader class="flex flex-row items-center justify-between gap-4">
                    <div class="space-y-1">
                        <h2 class="text-lg font-bold text-slate-200">{"执行策略"}</h2>
                        <p class="text-sm text-muted-foreground">{"控制批量执行、命令下发等远程执行能力的准入规则。"}</p>
                    </div>
                    <Button variant={ButtonVariant::Default} size={ButtonSize::Sm} onclick={open_create}>
                        {"创建策略"}
                    </Button>
                </CardHeader>
                <CardBody>
                    if let Some((ty, msg)) = (*notification).clone() {
                        <div class="mb-4">
                            <Notification notification_type={ty} message={msg} show={true} on_close={close_notif.clone()} />
                        </div>
                    }

                    if *loading {
                        <div class="py-8 text-center text-muted-foreground">{"加载中..."}</div>
                    } else if policies.is_empty() {
                        <div class="py-8 text-center text-muted-foreground">{"暂无策略"}</div>
                    } else {
                        <div class="rounded-xl border border-border/70 bg-background/30 p-2">
                            <Table>
                                <TableHeader>
                                    <TableRow>
                                        <TableHead>{"名称"}</TableHead>
                                        <TableHead>{"适用范围"}</TableHead>
                                        <TableHead>{"命令规则"}</TableHead>
                                        <TableHead>{"审批"}</TableHead>
                                        <TableHead class="text-right">{"操作"}</TableHead>
                                    </TableRow>
                                </TableHeader>
                                <TableBody>
                                    {for policies.iter().map(|policy| {
                                        let id = policy["id"].as_str().unwrap_or("").to_string();
                                        let name = policy["name"].as_str().unwrap_or("-").to_string();
                                        let sbj = subject_label(policy);
                                        let command_rules = command_rules_summary(policy);
                                        let need_approval = policy["require_approval"].as_bool().unwrap_or(false);
                                        let on_edit_click = on_edit.clone();
                                        let open_delete = {
                                            let pending_delete = pending_delete.clone();
                                            let id = id.clone();
                                            let name = name.clone();
                                            Callback::from(move |_| pending_delete.set(Some((id.clone(), name.clone()))))
                                        };
                                        let policy = policy.clone();
                                        html! {
                                            <TableRow>
                                                <TableCell>
                                                    <div class="space-y-1">
                                                        <div class="font-medium text-sm text-foreground">{name.clone()}</div>
                                                        <div class="font-mono text-[11px] text-muted-foreground">{id}</div>
                                                    </div>
                                                </TableCell>
                                                <TableCell class="text-xs text-muted-foreground">{sbj}</TableCell>
                                                <TableCell class="text-xs text-muted-foreground">{command_rules}</TableCell>
                                                <TableCell>
                                                    <Badge variant={if need_approval { BadgeVariant::Warning } else { BadgeVariant::Secondary }}>
                                                        {if need_approval { "需要审批" } else { "无需审批" }}
                                                    </Badge>
                                                </TableCell>
                                                <TableCell class="text-right">
                                                    <div class="flex justify-end gap-2">
                                                        <Button
                                                            variant={ButtonVariant::Outline}
                                                            size={ButtonSize::Sm}
                                                            onclick={Callback::from(move |_| on_edit_click.emit(policy.clone()))}
                                                        >
                                                            {"编辑"}
                                                        </Button>
                                                        <Button
                                                            variant={ButtonVariant::Destructive}
                                                            size={ButtonSize::Sm}
                                                            onclick={open_delete}
                                                        >
                                                            {"删除"}
                                                        </Button>
                                                    </div>
                                                </TableCell>
                                            </TableRow>
                                        }
                                    })}
                                </TableBody>
                            </Table>
                        </div>
                    }
                </CardBody>
            </Card>

            <Modal is_open={*show_form} on_close={close_form.clone()} content_class={classes!("max-w-4xl")}>
                <ModalHeader>
                    <ModalTitle>
                        {if editing_id.is_some() { "编辑执行策略" } else { "新建执行策略" }}
                    </ModalTitle>
                    <ModalDescription>
                        {"执行策略与 Web Terminal 策略独立生效；这里仅影响命令下发链路。"}
                    </ModalDescription>
                </ModalHeader>
                <ModalContent class="py-0">
                    <form onsubmit={onsubmit} class="grid gap-6 lg:grid-cols-[1.1fr_0.9fr]">
                        <div class="space-y-4">
                            <div>
                                <label class="mb-1 block text-sm">{"策略名称"}</label>
                                <Input value={(*form_name).clone()} oninput={let state = form_name.clone(); Callback::from(move |v| state.set(v))} />
                            </div>
                            <div>
                                <label class="mb-1 block text-sm">{"ID"}</label>
                                <Input value={(*form_id).clone()} oninput={let state = form_id.clone(); Callback::from(move |v| state.set(v))} />
                                <p class="mt-1 text-xs text-muted-foreground">{"建议：用短横连接英文，如 admin-all"}</p>
                            </div>
                            <div>
                                <label class="mb-1 block text-sm">{"适用范围"}</label>
                                <Select
                                    options={subject_options.clone()}
                                    value={(*form_subject).clone()}
                                    onchange={let state = form_subject.clone(); Callback::from(move |v| state.set(v))}
                                />
                            </div>
                            if *form_subject == "__custom__" {
                                <div>
                                    <label class="mb-1 block text-sm">{"用户名"}</label>
                                    <Input value={(*form_subject_custom).clone()} oninput={let state = form_subject_custom.clone(); Callback::from(move |v| state.set(v))} />
                                </div>
                            }
                        </div>

                        <div class="space-y-4 rounded-xl border border-border/70 bg-background/40 p-4">
                            <div>
                                <label class="mb-1 block text-sm">{"命令规则"}</label>
                                <Select
                                    options={action_options.clone()}
                                    value={(*form_rule_mode).clone()}
                                    onchange={let state = form_rule_mode.clone(); Callback::from(move |v| state.set(v))}
                                />
                            </div>
                            <div class="rounded-lg border border-border/60 bg-muted/20 p-3 text-xs text-muted-foreground">
                                <div class="mb-1 font-medium text-foreground">{rule_mode_detail.0}</div>
                                <div>{rule_mode_detail.1}</div>
                            </div>
                            if *form_rule_mode == "allow_list" || *form_rule_mode == "deny_list" {
                                <div>
                                    <label class="mb-1 block text-sm">{"命令列表（每行一个）"}</label>
                                    <textarea
                                        class="min-h-40 w-full rounded-md border border-input bg-slate-950/50 px-3 py-2 text-sm text-foreground transition-all duration-200 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:shadow-[0_0_10px_rgba(6,182,212,0.3)]"
                                        value={(*form_commands).clone()}
                                        oninput={let state = form_commands.clone(); Callback::from(move |e: InputEvent| {
                                            state.set(e.target_unchecked_into::<web_sys::HtmlTextAreaElement>().value());
                                        })}
                                    />
                                </div>
                            }
                            <label class="flex items-center gap-3 rounded-lg border border-border/60 bg-background/30 px-3 py-3 text-sm text-slate-300">
                                <Checkbox
                                    checked={*form_require_approval}
                                    onchange={let state = form_require_approval.clone(); Callback::from(move |checked| state.set(checked))}
                                />
                                {"命中后需要审批"}
                            </label>
                            <div class="flex justify-end gap-2 pt-2">
                                <Button type_="button" variant={ButtonVariant::Outline} onclick={close_form_click.clone()}>{"取消"}</Button>
                                <Button type_="submit" variant={ButtonVariant::Default}>{if editing_id.is_some() { "更新" } else { "保存" }}</Button>
                            </div>
                        </div>
                    </form>
                </ModalContent>
            </Modal>

            <ConfirmModal
                is_open={pending_delete.is_some()}
                title={"删除执行策略".to_string()}
                message={pending_delete.as_ref().map(|(_, name)| format!("确认删除策略 \"{}\" 吗？此操作不可撤销。", name)).unwrap_or_default()}
                on_confirm={on_confirm_delete}
                on_cancel={cancel_delete}
            />
        </div>
    }
}
