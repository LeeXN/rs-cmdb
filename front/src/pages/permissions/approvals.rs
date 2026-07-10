use yew::prelude::*;
use wasm_bindgen_futures::spawn_local;
use yew_router::prelude::*;

use crate::components::notification::{Notification, NotificationType};
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::card::{Card, CardBody};
use crate::components::ui::table::{Table, TableBody, TableCell, TableHead, TableHeader, TableRow};
use crate::hooks::use_trans::use_trans;
use crate::i18n::I18n;
use crate::routes::Route;
use crate::services::permission::{
    approve_request, fetch_approval_summary, list_my_approvals, list_pending_approvals,
    reject_request,
};
use crate::types::ApprovalSummaryResponse;
use crate::utils::format::format_datetime_with_ago;

fn approval_status_badge(status: &str, t: &I18n) -> Html {
    let (variant, label) = match status {
        "Pending" => (BadgeVariant::Warning, t.t("permissions.approvals.status.pending")),
        "Approved" => (BadgeVariant::Success, t.t("permissions.approvals.status.approved")),
        "Rejected" => (BadgeVariant::Destructive, t.t("permissions.approvals.status.rejected")),
        "Expired" => (BadgeVariant::Secondary, t.t("permissions.approvals.status.expired")),
        other => (BadgeVariant::Outline, other.to_string()),
    };

    html! { <Badge variant={variant}>{label}</Badge> }
}

fn approval_policy_type_label(policy_type: &str, t: &I18n) -> String {
    match policy_type {
        "command" | "Command" => t.t("permissions.approvals.policy_type.command"),
        other => other.to_string(),
    }
}

#[function_component(ApprovalsPage)]
pub fn approvals_page() -> Html {
    let t = use_trans();
    let navigator = use_navigator();
    let pending = use_state(Vec::<serde_json::Value>::new);
    let mine = use_state(Vec::<serde_json::Value>::new);
    let loading = use_state(|| true);
    let tab = use_state(|| 0usize);
    let notification = use_state(|| None::<(NotificationType, String)>);
    let approved_task_id = use_state(|| None::<String>);
    let summary = use_state(ApprovalSummaryResponse::default);

    // initial load: runs once on mount
    {
        let pd = pending.clone();
        let mn = mine.clone();
        let ld = loading.clone();
        let summary = summary.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                if let Ok(s) = fetch_approval_summary().await {
                    summary.set(s);
                }
                if let Ok(p) = list_pending_approvals().await {
                    pd.set(p);
                }
                if let Ok(m) = list_my_approvals().await {
                    mn.set(m);
                }
                ld.set(false);
            });
            || ()
        });
    }

    let close_notif = {
        let n = notification.clone();
        let approved_task_id = approved_task_id.clone();
        Callback::from(move |_| {
            n.set(None);
            approved_task_id.set(None);
        })
    };

    let data = if *tab == 0 { &pending } else { &mine };

    let refresh = {
        let pd = pending.clone();
        let mn = mine.clone();
        let summary = summary.clone();
        Callback::from(move |_| {
            let pd = pd.clone();
            let mn = mn.clone();
            let summary = summary.clone();
            spawn_local(async move {
                if let Ok(s) = fetch_approval_summary().await {
                    summary.set(s);
                }
                if let Ok(p) = list_pending_approvals().await {
                    pd.set(p);
                }
                if let Ok(m) = list_my_approvals().await {
                    mn.set(m);
                }
            });
        })
    };

    let on_approve = {
        let n = notification.clone();
        let r = refresh.clone();
        let approved_task_id = approved_task_id.clone();
        let t = t.clone();
        Callback::from(move |id: String| {
            let n = n.clone();
            let r = r.clone();
            let approved_task_id = approved_task_id.clone();
            let t = t.clone();
            spawn_local(async move {
                match approve_request(&id).await {
                    Ok(resp) => {
                        let task_id = resp["task_id"].as_str().unwrap_or_default().to_string();
                        let approval_id = resp["approval_id"].as_str().unwrap_or_default().to_string();
                        let message = if !task_id.is_empty() {
                            approved_task_id.set(Some(task_id.clone()));
                            format!("{} {}", t.t("permissions.approvals.messages.approved_created_task"), task_id)
                        } else if !approval_id.is_empty() {
                            approved_task_id.set(None);
                            format!("{} {}", t.t("permissions.approvals.messages.approved_updated"), approval_id)
                        } else {
                            approved_task_id.set(None);
                            t.t("permissions.approvals.messages.approved")
                        };
                        n.set(Some((NotificationType::Success, message)));
                        r.emit(());
                    }
                    Err(e) => n.set(Some((NotificationType::Error, e.message))),
                }
            });
        })
    };

    let on_reject = {
        let n = notification.clone();
        let r = refresh.clone();
        let approved_task_id = approved_task_id.clone();
        let t = t.clone();
        Callback::from(move |id: String| {
            let n = n.clone();
            let r = r.clone();
            let approved_task_id = approved_task_id.clone();
            let t = t.clone();
            spawn_local(async move {
                match reject_request(&id, None).await {
                    Ok(_) => {
                        approved_task_id.set(None);
                        n.set(Some((NotificationType::Success, t.t("permissions.approvals.messages.rejected"))));
                        r.emit(());
                    }
                    Err(e) => n.set(Some((NotificationType::Error, e.message))),
                }
            });
        })
    };

    let goto_history = {
        let navigator = navigator.clone();
        let approved_task_id = approved_task_id.clone();
        Callback::from(move |_| {
            let Some(task_id) = (*approved_task_id).clone() else {
                return;
            };
            if let Some(nav) = navigator.clone() {
                let mut query = std::collections::HashMap::new();
                query.insert("search".to_string(), task_id);
                let _ = nav.push_with_query(&Route::ExecutionHistory, &query);
            }
        })
    };

    let open_task_link = approved_task_id.as_ref().cloned();

    html! {
        <div class="space-y-4">
            if let Some((ty, msg)) = (*notification).clone() {
                <Notification notification_type={ty} message={msg} show={true} on_close={close_notif.clone()} />
                if let Some(task_id) = open_task_link.clone() {
                    <div class="flex items-center gap-3 rounded-lg border border-primary/30 bg-primary/5 px-4 py-3 text-sm">
                        <span class="text-muted-foreground">{format!("{} {}", t.t("permissions.approvals.messages.task_created"), task_id)}</span>
                        <Button variant={ButtonVariant::Outline} size={ButtonSize::Sm} onclick={goto_history.clone()}>
                            {t.t("permissions.approvals.actions.go_history")}
                        </Button>
                    </div>
                }
            }
            <div class="flex items-center justify-between">
                <h2 class="text-xl font-bold">{t.t("permissions.approvals.title")}</h2>
                <div class="tabs tabs-boxed bg-muted p-1">
                    <button class={classes!("tab", (*tab == 0).then_some("tab-active"))}
                        onclick={{ let tab = tab.clone(); Callback::from(move |_| tab.set(0)) }}>
                        {t.t("permissions.approvals.tab.pending")}
                    </button>
                    <button class={classes!("tab", (*tab == 1).then_some("tab-active"))}
                        onclick={{ let tab = tab.clone(); Callback::from(move |_| tab.set(1)) }}>
                        {t.t("permissions.approvals.tab.mine")}
                    </button>
                </div>
            </div>
            <div class="grid gap-4 md:grid-cols-3 xl:grid-cols-6">
                <Card><CardBody><div class="text-xs text-muted-foreground">{t.t("permissions.approvals.summary.total")}</div><div class="text-2xl font-semibold">{summary.total}</div></CardBody></Card>
                <Card><CardBody><div class="text-xs text-muted-foreground">{t.t("permissions.approvals.summary.pending")}</div><div class="text-2xl font-semibold text-warning">{summary.pending}</div></CardBody></Card>
                <Card><CardBody><div class="text-xs text-muted-foreground">{t.t("permissions.approvals.summary.approved")}</div><div class="text-2xl font-semibold text-success">{summary.approved}</div></CardBody></Card>
                <Card><CardBody><div class="text-xs text-muted-foreground">{t.t("permissions.approvals.summary.rejected")}</div><div class="text-2xl font-semibold">{summary.rejected}</div></CardBody></Card>
                <Card><CardBody><div class="text-xs text-muted-foreground">{t.t("permissions.approvals.summary.expired")}</div><div class="text-2xl font-semibold">{summary.expired}</div></CardBody></Card>
                <Card><CardBody><div class="text-xs text-muted-foreground">{t.t("permissions.approvals.summary.executed")}</div><div class="text-2xl font-semibold text-primary">{summary.executed}</div></CardBody></Card>
            </div>
            <Card>
                <CardBody>
                    if *loading {
                        <div class="text-center py-8 text-muted-foreground">{t.t("permissions.approvals.loading")}</div>
                    } else if data.is_empty() {
                        <div class="text-center py-8 text-muted-foreground">{t.t("permissions.approvals.empty")}</div>
                    } else {
                        <Table>
                            <TableHeader>
                            <TableRow>
                                    <TableHead>{t.t("permissions.approvals.table.id")}</TableHead>
                                    <TableHead>{t.t("permissions.approvals.table.type")}</TableHead>
                                    <TableHead>{t.t("permissions.approvals.table.requester")}</TableHead>
                                    <TableHead>{t.t("permissions.approvals.table.client")}</TableHead>
                                    <TableHead>{t.t("permissions.approvals.table.command")}</TableHead>
                                    <TableHead>{t.t("permissions.approvals.table.task")}</TableHead>
                                    <TableHead>{t.t("permissions.approvals.table.status")}</TableHead>
                                    <TableHead>{t.t("permissions.approvals.table.created_at")}</TableHead>
                                    if *tab == 0 {
                                        <TableHead class="text-right">{t.t("permissions.approvals.table.operations")}</TableHead>
                                    }
                                </TableRow>
                            </TableHeader>
                            <TableBody>
                                { for data.iter().map(|p| {
                                        let id = p["id"].as_str().unwrap_or("-").to_string();
                                        let atype = p["policy_type"].as_str().unwrap_or("-").to_string();
                                        let requester = p["username"].as_str().unwrap_or("-").to_string();
                                        let client_id = p["target_client_id"].as_str().unwrap_or("-").to_string();
                                        let command = p["command"].as_str().unwrap_or("-").to_string();
                                        let task_id = p["executed_task_id"].as_str().unwrap_or("").to_string();
                                        let status = p["status"].as_str().unwrap_or("-").to_string();
                                        let created = p["created_at"].as_str().unwrap_or("-").to_string();
                                        let short_id = if id.len() > 8 { format!("{}...", &id[..8]) } else { id.clone() };
                                        let created_label = format_datetime_with_ago(&created);
                                        let task_nav = navigator.clone();
                                        let task_id_for_nav = task_id.clone();
                                        let on_app = on_approve.clone();
                                        let on_rej = on_reject.clone();
                                        let id_app = id.clone();
                                        let id_rej = id.clone();
                                        html! {
                                            <TableRow>
                                                <TableCell><span class="font-mono text-xs">{short_id}</span></TableCell>
                                                <TableCell>{approval_policy_type_label(&atype, t.as_ref())}</TableCell>
                                                <TableCell>{requester}</TableCell>
                                                <TableCell><span class="font-mono text-xs">{client_id}</span></TableCell>
                                                <TableCell><div class="max-w-xs truncate font-mono text-xs" title={command.clone()}>{command}</div></TableCell>
                                                <TableCell>
                                                    if task_id.is_empty() {
                                                        <span class="text-xs text-muted-foreground">{t.t("permissions.approvals.none")}</span>
                                                    } else {
                                                        <button
                                                            type="button"
                                                            class="font-mono text-xs text-primary underline-offset-2 hover:underline"
                                                            onclick={Callback::from(move |_| {
                                                                if let Some(nav) = task_nav.clone() {
                                                                    let mut query = std::collections::HashMap::new();
                                                                    query.insert("search".to_string(), task_id_for_nav.clone());
                                                                    let _ = nav.push_with_query(&Route::ExecutionHistory, &query);
                                                                }
                                                            })}
                                                        >
                                                            {task_id}
                                                        </button>
                                                    }
                                                </TableCell>
                                                <TableCell>{approval_status_badge(&status, t.as_ref())}</TableCell>
                                                <TableCell><span class="text-xs text-muted-foreground">{created_label}</span></TableCell>
                                                if *tab == 0 {
                                                    <TableCell class="text-right">
                                                        <div class="flex justify-end gap-1">
                                                            <Button variant={ButtonVariant::Outline} size={ButtonSize::Sm}
                                                                onclick={Callback::from(move |_| on_app.emit(id_app.clone()))}>
                                                                {t.t("permissions.approvals.actions.approve")}
                                                            </Button>
                                                            <Button variant={ButtonVariant::Destructive} size={ButtonSize::Sm}
                                                                onclick={Callback::from(move |_| on_rej.emit(id_rej.clone()))}>
                                                                {t.t("permissions.approvals.actions.reject")}
                                                            </Button>
                                                        </div>
                                                    </TableCell>
                                                }
                                            </TableRow>
                                        }
                                    })}
                            </TableBody>
                        </Table>
                    }
                </CardBody>
            </Card>
        </div>
    }
}
