use crate::components::notification::{Notification, NotificationType};
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::card::{Card, CardBody, CardHeader};
use crate::components::ui::checkbox::Checkbox;
use crate::components::ui::confirm_modal::ConfirmModal;
use crate::components::ui::input::Input;
use crate::components::ui::modal::{
    Modal, ModalContent, ModalDescription, ModalHeader, ModalTitle,
};
use crate::components::ui::select::{Select, SelectOption};
use crate::components::ui::table::{Table, TableBody, TableCell, TableHead, TableHeader, TableRow};
use crate::hooks::use_trans::use_trans;
use crate::services::permission::{
    create_web_terminal_policy, delete_web_terminal_policy, fetch_web_terminal_policies,
    update_web_terminal_policy,
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
    let rules = p
        .get("terminal_command_rules")
        .or_else(|| p.get("command_rules"));
    let default_action = rules
        .and_then(|value| value.get("default_action"))
        .and_then(|value| value.as_str())
        .or_else(|| {
            let allowed = p.get("allowed_commands")?.as_array()?;
            if allowed.is_empty() {
                None
            } else {
                Some("Deny")
            }
        })
        .unwrap_or("Allow");
    let overrides_len = rules
        .and_then(|value| value.get("overrides"))
        .and_then(|value| value.as_array())
        .map(|value| value.len())
        .unwrap_or_else(|| {
            p.get("allowed_commands")
                .and_then(|v| v.as_array())
                .map(|v| v.len())
                .unwrap_or(0)
        });

    match (default_action, overrides_len) {
        ("Allow", 0) => "全部允许".to_string(),
        ("Deny", 0) => "全部禁止".to_string(),
        ("Deny", n) => format!("白名单（{} 条）", n),
        ("Allow", n) => format!("黑名单（{} 条）", n),
        _ => default_action.to_string(),
    }
}

fn rule_mode_help(mode: &str) -> (&'static str, &'static str) {
    match mode {
        "deny_all" => ("全部禁止", "只允许建立策略占位，不允许任何只读命令通过。"),
        "allow_list" => (
            "白名单",
            "ReadOnly 模式下仅允许列表中的命令执行，适合最小暴露。",
        ),
        "deny_list" => ("黑名单", "ReadOnly 模式下默认允许，但会拦截列表中命令。"),
        _ => (
            "全部允许",
            "ReadOnly 模式下不做命令级限制，仅依赖模式和审批。",
        ),
    }
}

fn form_rule_mode_from_policy(p: &serde_json::Value) -> String {
    let rules = p
        .get("terminal_command_rules")
        .or_else(|| p.get("command_rules"));
    let default_action = rules
        .and_then(|value| value.get("default_action"))
        .and_then(|value| value.as_str())
        .unwrap_or("Allow");
    let overrides = rules
        .and_then(|value| value.get("overrides"))
        .and_then(|value| value.as_array())
        .cloned()
        .unwrap_or_default();

    match (default_action, overrides.is_empty()) {
        ("Allow", true) => "allow_all".to_string(),
        ("Deny", true) => "deny_all".to_string(),
        ("Deny", false) => "allow_list".to_string(),
        ("Allow", false) => "deny_list".to_string(),
        _ => "allow_list".to_string(),
    }
}

fn form_commands_from_policy(p: &serde_json::Value) -> String {
    p.get("terminal_command_rules")
        .or_else(|| p.get("command_rules"))
        .and_then(|rules| rules.get("overrides"))
        .and_then(|value| value.as_array())
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|item| {
            item.get("pattern")
                .and_then(|value| value.as_str())
                .map(str::to_string)
        })
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

#[function_component(WebTerminalPoliciesPage)]
pub fn web_terminal_policies_page() -> Html {
    let t = use_trans();
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
    let form_mode = use_state(|| "ReadOnly".to_string());
    let form_rule_mode = use_state(|| "allow_list".to_string());
    let form_commands = use_state(String::new);
    let form_timeout = use_state(|| "3600".to_string());
    let form_max_sessions = use_state(|| "5".to_string());
    let form_require_approval = use_state(|| false);

    {
        let pols = policies.clone();
        let ld = loading.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                if let Ok(p) = fetch_web_terminal_policies().await {
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
        SelectOption {
            value: "Role:Admin".into(),
            label: "所有管理员".into(),
        },
        SelectOption {
            value: "Role:User".into(),
            label: "所有用户".into(),
        },
        SelectOption {
            value: "Role:Viewer".into(),
            label: "所有只读用户".into(),
        },
        SelectOption {
            value: "__custom__".into(),
            label: "指定用户...".into(),
        },
    ];

    let mode_options = vec![
        SelectOption {
            value: "ReadOnly".into(),
            label: t.t("execution.terminal.mode_restricted"),
        },
        SelectOption {
            value: "ReadWrite".into(),
            label: t.t("execution.terminal.mode_standard"),
        },
    ];

    let rule_mode_options = vec![
        SelectOption {
            value: "allow_all".into(),
            label: "全部允许".into(),
        },
        SelectOption {
            value: "deny_all".into(),
            label: "全部禁止".into(),
        },
        SelectOption {
            value: "allow_list".into(),
            label: "白名单".into(),
        },
        SelectOption {
            value: "deny_list".into(),
            label: "黑名单".into(),
        },
    ];

    let open_create = {
        let show_form = show_form.clone();
        let editing_id = editing_id.clone();
        let form_id = form_id.clone();
        let form_name = form_name.clone();
        let form_subject = form_subject.clone();
        let form_subject_custom = form_subject_custom.clone();
        let form_mode = form_mode.clone();
        let form_rule_mode = form_rule_mode.clone();
        let form_commands = form_commands.clone();
        let form_timeout = form_timeout.clone();
        let form_max_sessions = form_max_sessions.clone();
        let form_require_approval = form_require_approval.clone();
        Callback::from(move |_| {
            editing_id.set(None);
            form_id.set(String::new());
            form_name.set(String::new());
            form_subject.set("Role:Admin".to_string());
            form_subject_custom.set(String::new());
            form_mode.set("ReadOnly".to_string());
            form_rule_mode.set("allow_list".to_string());
            form_commands.set(String::new());
            form_timeout.set(String::from("3600"));
            form_max_sessions.set("5".to_string());
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
        let form_mode = form_mode.clone();
        let form_rule_mode = form_rule_mode.clone();
        let form_commands = form_commands.clone();
        let form_timeout = form_timeout.clone();
        let form_max_sessions = form_max_sessions.clone();
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
            let form_mode = form_mode.clone();
            let form_rule_mode = form_rule_mode.clone();
            let form_commands = form_commands.clone();
            let form_timeout = form_timeout.clone();
            let form_max_sessions = form_max_sessions.clone();
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
                "mode": (*form_mode).clone(),
                "session_timeout_secs": (*form_timeout).parse::<u64>().unwrap_or(3600),
                "max_concurrent_sessions": (*form_max_sessions).parse::<u32>().unwrap_or(5),
                "require_approval": *form_require_approval,
                "priority": 0,
                "terminal_command_rules": build_command_rules(form_rule_mode.as_str(), &form_commands),
                "allowed_commands": [],
            });
            spawn_local(async move {
                let result = if let Some(id) = (*editing_id).clone() {
                    update_web_terminal_policy(&id, &body).await
                } else {
                    create_web_terminal_policy(&body).await
                };
                match result {
                    Ok(_) => {
                        notification.set(Some((
                            NotificationType::Success,
                            if editing_id.is_some() {
                                "更新成功".into()
                            } else {
                                "创建成功".into()
                            },
                        )));
                        editing_id.set(None);
                        form_id.set(String::new());
                        form_name.set(String::new());
                        form_subject_custom.set(String::new());
                        form_mode.set("ReadOnly".to_string());
                        form_rule_mode.set("allow_list".to_string());
                        form_commands.set(String::new());
                        form_timeout.set(String::from("3600"));
                        form_max_sessions.set("5".to_string());
                        form_require_approval.set(false);
                        show_form.set(false);
                        if let Ok(list) = fetch_web_terminal_policies().await {
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
                match delete_web_terminal_policy(&id).await {
                    Ok(_) => {
                        notification.set(Some((NotificationType::Success, "删除成功".into())));
                        pending_delete.set(None);
                        if let Ok(list) = fetch_web_terminal_policies().await {
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
        let form_mode = form_mode.clone();
        let form_rule_mode = form_rule_mode.clone();
        let form_commands = form_commands.clone();
        let form_timeout = form_timeout.clone();
        let form_max_sessions = form_max_sessions.clone();
        let form_require_approval = form_require_approval.clone();
        Callback::from(move |policy: serde_json::Value| {
            editing_id.set(
                policy
                    .get("id")
                    .and_then(|v| v.as_str())
                    .map(str::to_string),
            );
            form_id.set(
                policy
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string(),
            );
            form_name.set(
                policy
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string(),
            );
            let subject_type = policy
                .get("subject_type")
                .and_then(|v| v.as_str())
                .unwrap_or("Role");
            let subject_id = policy
                .get("subject_id")
                .and_then(|v| v.as_str())
                .unwrap_or("Admin");
            if subject_type == "User" {
                form_subject.set("__custom__".to_string());
                form_subject_custom.set(subject_id.to_string());
            } else {
                form_subject.set(format!("{}:{}", subject_type, subject_id));
                form_subject_custom.set(String::new());
            }
            form_mode.set(
                policy
                    .get("mode")
                    .and_then(|v| v.as_str())
                    .unwrap_or("ReadOnly")
                    .to_string(),
            );
            form_rule_mode.set(form_rule_mode_from_policy(&policy));
            form_commands.set(form_commands_from_policy(&policy));
            form_timeout.set(
                policy
                    .get("session_timeout_secs")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(3600)
                    .to_string(),
            );
            form_max_sessions.set(
                policy
                    .get("max_concurrent_sessions")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(5)
                    .to_string(),
            );
            form_require_approval.set(
                policy
                    .get("require_approval")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
            );
            show_form.set(true);
        })
    };

    let rule_mode_detail = rule_mode_help(form_rule_mode.as_str());

    html! {
        <div class="space-y-6">
            <Card class="border border-border/70 bg-card/95 shadow-xl">
                <CardHeader class="flex flex-row items-center justify-between gap-4">
                    <div class="space-y-1">
                        <h2 class="text-lg font-bold text-slate-200">{"终端策略"}</h2>
                        <p class="text-sm text-muted-foreground">{t.t("permissions.web_terminal.page_description")}</p>
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
                                        <TableHead>{"访问模式"}</TableHead>
                                        <TableHead>{"命令规则"}</TableHead>
                                        <TableHead>{"会话超时"}</TableHead>
                                        <TableHead>{"并发会话"}</TableHead>
                                        <TableHead>{"审批"}</TableHead>
                                        <TableHead class="text-right">{"操作"}</TableHead>
                                    </TableRow>
                                </TableHeader>
                                <TableBody>
                                    {for policies.iter().map(|policy| {
                                        let id = policy["id"].as_str().unwrap_or("").to_string();
                                        let name = policy["name"].as_str().unwrap_or("-").to_string();
                                        let sbj = subject_label(policy);
                                        let mode = policy["mode"].as_str().unwrap_or("-").to_string();
                                        let mode_label = match mode.as_str() {
                                            "ReadOnly" => t.t("execution.terminal.mode_restricted").to_string(),
                                            "ReadWrite" => t.t("execution.terminal.mode_standard").to_string(),
                                            _ => mode.as_str().to_string(),
                                        };
                                        let command_rules = command_rules_summary(policy);
                                        let timeout = format!("{}s", policy["session_timeout_secs"].as_u64().unwrap_or(0));
                                        let maxs = policy["max_concurrent_sessions"].as_u64().unwrap_or(0);
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
                                                <TableCell>{mode_label}</TableCell>
                                                <TableCell class="text-xs text-muted-foreground">{command_rules}</TableCell>
                                                <TableCell>{timeout}</TableCell>
                                                <TableCell>{maxs}</TableCell>
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

            <Modal is_open={*show_form} on_close={close_form.clone()} content_class={classes!("max-w-5xl")}>
                <ModalHeader>
                    <ModalTitle>
                        {if editing_id.is_some() { "编辑终端策略" } else { "新建终端策略" }}
                    </ModalTitle>
                    <ModalDescription>
                        {t.t("permissions.web_terminal.modal_description")}
                    </ModalDescription>
                </ModalHeader>
                <ModalContent class="py-0">
                    <form onsubmit={onsubmit} class="grid gap-6 xl:grid-cols-[1.05fr_0.95fr]">
                        <div class="space-y-4">
                            <div>
                                <label class="mb-1 block text-sm">{"策略名称"}</label>
                                <Input value={(*form_name).clone()} oninput={let state = form_name.clone(); Callback::from(move |v| state.set(v))} />
                            </div>
                            <div>
                                <label class="mb-1 block text-sm">{"ID"}</label>
                                <Input value={(*form_id).clone()} oninput={let state = form_id.clone(); Callback::from(move |v| state.set(v))} />
                                <p class="mt-1 text-xs text-muted-foreground">{"建议：用短横连接英文，如 admin-ssh"}</p>
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
                            <div class="grid gap-4 md:grid-cols-2">
                                <div>
                                    <label class="mb-1 block text-sm">{"会话超时（秒）"}</label>
                                    <Input type_="number" value={(*form_timeout).clone()} oninput={let state = form_timeout.clone(); Callback::from(move |v| state.set(v))} />
                                </div>
                                <div>
                                    <label class="mb-1 block text-sm">{"最大并发会话数"}</label>
                                    <Input type_="number" value={(*form_max_sessions).clone()} oninput={let state = form_max_sessions.clone(); Callback::from(move |v| state.set(v))} />
                                </div>
                            </div>
                        </div>

                        <div class="space-y-4 rounded-xl border border-border/70 bg-background/40 p-4">
                            <div>
                                <label class="mb-1 block text-sm">{"访问模式"}</label>
                                <Select
                                    options={mode_options.clone()}
                                    value={(*form_mode).clone()}
                                    onchange={let state = form_mode.clone(); Callback::from(move |v| state.set(v))}
                                />
                            </div>
                            <div>
                                <label class="mb-1 block text-sm">{"命令规则"}</label>
                                <Select
                                    options={rule_mode_options.clone()}
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
                title={"删除终端策略".to_string()}
                message={pending_delete.as_ref().map(|(_, name)| format!("确认删除策略 \"{}\" 吗？此操作不可撤销。", name)).unwrap_or_default()}
                on_confirm={on_confirm_delete}
                on_cancel={cancel_delete}
            />
        </div>
    }
}
