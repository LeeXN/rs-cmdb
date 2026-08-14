use crate::components::loading::Loading;
use crate::components::notification::{Notification, NotificationType};
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::card::{Card, CardContent, CardHeader, CardTitle};
use crate::components::ui::checkbox::Checkbox;
use crate::components::ui::input::Input;
use crate::components::ui::table::{Table, TableBody, TableCell, TableHead, TableHeader, TableRow};
use crate::hooks::use_trans::use_trans;
use crate::i18n::I18n;
use crate::services::{api, command};
use crate::types::UpdateRemoteExecConfigRequest;
use crate::types::{RemoteExecOpsOverviewResponse, TerminalSessionState, TerminalSessionSummary};
use crate::utils::format::format_datetime_with_ago;
use crate::utils::i18n_helper::translate_api_message;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

fn terminal_state_label(state: &TerminalSessionState, t: &I18n) -> (String, &'static str) {
    match state {
        TerminalSessionState::Pending => {
            (t.t("execution.ops.terminal_state.pending"), "text-warning")
        }
        TerminalSessionState::Active => {
            (t.t("execution.ops.terminal_state.active"), "text-primary")
        }
        TerminalSessionState::Closed => (
            t.t("execution.ops.terminal_state.closed"),
            "text-muted-foreground",
        ),
        TerminalSessionState::Failed => (t.t("execution.ops.terminal_state.failed"), "text-error"),
    }
}

fn terminal_mode_label(mode: &common::entity::permission::TerminalMode, t: &I18n) -> String {
    match mode {
        common::entity::permission::TerminalMode::ReadOnly => {
            t.t("execution.ops.terminal_mode.restricted")
        }
        common::entity::permission::TerminalMode::ReadWrite => {
            t.t("execution.ops.terminal_mode.standard")
        }
    }
}

fn format_optional_datetime(value: Option<&String>) -> String {
    value
        .map(|value| format_datetime_with_ago(value))
        .unwrap_or_else(|| "-".to_string())
}

#[function_component(RemoteExec)]
pub fn remote_exec() -> Html {
    let t = use_trans();
    let enabled = use_state(|| false);
    let loading = use_state(|| false);
    let saving = use_state(|| false);
    let error_message = use_state(|| Option::<String>::None);
    let success_message = use_state(|| Option::<String>::None);
    let fetched = use_state(|| false);
    let ops_overview = use_state(RemoteExecOpsOverviewResponse::default);
    let terminal_sessions = use_state(Vec::<TerminalSessionSummary>::new);
    let retention_days = use_state(|| "30".to_string());

    {
        let enabled = enabled.clone();
        let loading = loading.clone();
        let error_message = error_message.clone();
        let fetched = fetched.clone();
        let ops_overview = ops_overview.clone();
        let terminal_sessions = terminal_sessions.clone();

        use_effect_with((), move |_| {
            if *fetched {
                return;
            }
            loading.set(true);
            let enabled = enabled.clone();
            let loading = loading.clone();
            let error_message = error_message.clone();
            let fetched = fetched.clone();
            let ops_overview = ops_overview.clone();
            let terminal_sessions = terminal_sessions.clone();

            spawn_local(async move {
                match api::fetch_remote_exec_config().await {
                    Ok(config) => {
                        enabled.set(config.enabled);
                        fetched.set(true);
                    }
                    Err(err) => {
                        error_message.set(Some(translate_api_message(&err.message)));
                    }
                }
                if let Ok(overview) = command::fetch_remote_exec_ops_overview().await {
                    ops_overview.set(overview);
                }
                if let Ok(sessions) = command::list_all_terminal_sessions().await {
                    terminal_sessions.set(sessions);
                }
                loading.set(false);
            });
        });
    }

    let on_toggle = {
        let enabled = enabled.clone();
        Callback::from(move |_| {
            let new_val = !*enabled;
            enabled.set(new_val);
        })
    };

    let on_save = {
        let enabled = enabled.clone();
        let saving = saving.clone();
        let error_message = error_message.clone();
        let success_message = success_message.clone();
        let t = t.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let current = *enabled;
            let saving = saving.clone();
            let error_message = error_message.clone();
            let success_message = success_message.clone();
            let t = t.clone();

            saving.set(true);
            error_message.set(None);
            success_message.set(None);

            spawn_local(async move {
                let req = UpdateRemoteExecConfigRequest { enabled: current };
                match api::update_remote_exec_config(&req).await {
                    Ok(_) => {
                        success_message.set(Some(t.t("execution.settings.updated")));
                    }
                    Err(err) => {
                        error_message.set(Some(translate_api_message(&err.message)));
                    }
                }
                saving.set(false);
            });
        })
    };

    let close_error = {
        let error_message = error_message.clone();
        Callback::from(move |_| error_message.set(None))
    };

    let close_success = {
        let success_message = success_message.clone();
        Callback::from(move |_| success_message.set(None))
    };

    let refresh_ops = {
        let ops_overview = ops_overview.clone();
        let terminal_sessions = terminal_sessions.clone();
        let error_message = error_message.clone();
        Callback::from(move |_: ()| {
            let ops_overview = ops_overview.clone();
            let terminal_sessions = terminal_sessions.clone();
            let error_message = error_message.clone();
            spawn_local(async move {
                match command::fetch_remote_exec_ops_overview().await {
                    Ok(overview) => ops_overview.set(overview),
                    Err(err) => error_message.set(Some(translate_api_message(&err.message))),
                }
                match command::list_all_terminal_sessions().await {
                    Ok(sessions) => terminal_sessions.set(sessions),
                    Err(err) => error_message.set(Some(translate_api_message(&err.message))),
                }
            });
        })
    };

    let refresh_ops_click = {
        let refresh_ops = refresh_ops.clone();
        Callback::from(move |_| refresh_ops.emit(()))
    };

    let on_cleanup = {
        let retention_days = retention_days.clone();
        let success_message = success_message.clone();
        let error_message = error_message.clone();
        let refresh_ops = refresh_ops.clone();
        let t = t.clone();
        Callback::from(move |_| {
            let retention_days = retention_days.clone();
            let success_message = success_message.clone();
            let error_message = error_message.clone();
            let refresh_ops = refresh_ops.clone();
            let t = t.clone();
            spawn_local(async move {
                let days = retention_days.parse::<u64>().unwrap_or_default();
                match command::cleanup_cast_history(days).await {
                    Ok(result) => {
                        success_message.set(Some(format!(
                            "{} {} {}",
                            t.t("execution.ops.messages.cast_cleanup_prefix"),
                            result.deleted_files,
                            t.t("execution.ops.messages.cast_cleanup_suffix")
                        )));
                        refresh_ops.emit(());
                    }
                    Err(err) => error_message.set(Some(translate_api_message(&err.message))),
                }
            });
        })
    };

    let on_close_session = {
        let success_message = success_message.clone();
        let error_message = error_message.clone();
        let refresh_ops = refresh_ops.clone();
        let t = t.clone();
        Callback::from(move |session_id: String| {
            let success_message = success_message.clone();
            let error_message = error_message.clone();
            let refresh_ops = refresh_ops.clone();
            let t = t.clone();
            spawn_local(async move {
                match command::close_terminal_session(&session_id).await {
                    Ok(_) => {
                        success_message.set(Some(format!(
                            "{} {} {}",
                            t.t("execution.ops.messages.session_closed_prefix"),
                            session_id,
                            t.t("execution.ops.messages.session_closed_suffix")
                        )));
                        refresh_ops.emit(());
                    }
                    Err(err) => error_message.set(Some(translate_api_message(&err.message))),
                }
            });
        })
    };

    html! {
        <div class="space-y-6 p-4">
            if let Some(msg) = (*error_message).clone() {
                <Notification notification_type={NotificationType::Error} message={msg} show={true} on_close={close_error} />
            }
            if let Some(msg) = (*success_message).clone() {
                <Notification notification_type={NotificationType::Success} message={msg} show={true} on_close={close_success} />
            }

            <Card class="mx-auto max-w-3xl">
                <CardHeader class="gap-4 md:flex-row md:items-start md:justify-between md:space-y-0">
                    <div class="space-y-2">
                        <CardTitle class="text-xl">{t.t("execution.settings.title")}</CardTitle>
                        <p class="text-sm text-muted-foreground">
                            {t.t("execution.settings.description")}
                        </p>
                    </div>
                    <Badge variant={if *enabled { BadgeVariant::Success } else { BadgeVariant::Secondary }}>
                        {if *enabled { t.t("execution.settings.enabled") } else { t.t("execution.settings.disabled") }}
                    </Badge>
                </CardHeader>
                <CardContent>
                    if *loading {
                        <Loading message={t.t("execution.settings.loading")} />
                    } else {
                        <form onsubmit={on_save} class="space-y-6">
                            <div class="rounded-lg border border-border/70 bg-muted/20 p-4">
                                <div class="flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
                                    <div class="space-y-1">
                                        <div class="text-base font-medium text-foreground">{t.t("execution.settings.toggle_title")}</div>
                                        <p class="text-sm text-muted-foreground">
                                            {t.t("execution.settings.toggle_description")}
                                        </p>
                                    </div>
                                    <label class="flex items-center gap-3 text-sm text-foreground">
                                        <Checkbox
                                            checked={*enabled}
                                            onchange={Callback::from(move |_| on_toggle.emit(()))}
                                            id={Some("remote-exec-toggle".to_string())}
                                        />
                                        {if *enabled { t.t("execution.settings.allow") } else { t.t("execution.settings.block") }}
                                    </label>
                                </div>
                            </div>

                            <div class="rounded-lg border border-cyan-500/20 bg-cyan-500/10 p-4 text-sm text-cyan-100">
                                {t.t("execution.settings.warning")}
                            </div>

                            <div class="flex justify-end">
                                <Button type_="submit" variant={ButtonVariant::Default} disabled={*saving}>
                                    {if *saving { t.t("execution.settings.saving") } else { t.t("execution.settings.save") }}
                                </Button>
                            </div>
                        </form>
                    }
                </CardContent>
            </Card>

            <div class="grid gap-4 xl:grid-cols-4 md:grid-cols-2">
                <Card><CardContent class="pt-6"><div class="text-xs text-muted-foreground">{t.t("execution.ops.summary.exec_policies")}</div><div class="text-2xl font-semibold">{ops_overview.exec_policies_count}</div></CardContent></Card>
                <Card><CardContent class="pt-6"><div class="text-xs text-muted-foreground">{t.t("execution.ops.summary.web_terminal_policies")}</div><div class="text-2xl font-semibold">{ops_overview.web_terminal_policies_count}</div></CardContent></Card>
                <Card><CardContent class="pt-6"><div class="text-xs text-muted-foreground">{t.t("execution.ops.summary.pending_approvals")}</div><div class="text-2xl font-semibold text-warning">{ops_overview.approval_summary.pending}</div></CardContent></Card>
                <Card><CardContent class="pt-6"><div class="text-xs text-muted-foreground">{t.t("execution.ops.summary.active_terminal_sessions")}</div><div class="text-2xl font-semibold text-primary">{ops_overview.terminal_summary.active_sessions}</div></CardContent></Card>
            </div>

            <Card>
                <CardHeader class="flex flex-row items-center justify-between">
                    <div>
                        <CardTitle class="text-lg">{t.t("execution.ops.overview.title")}</CardTitle>
                        <p class="text-sm text-muted-foreground">{t.t("execution.ops.overview.description")}</p>
                    </div>
                    <Button variant={ButtonVariant::Outline} onclick={refresh_ops_click}>{t.t("execution.ops.actions.refresh")}</Button>
                </CardHeader>
                <CardContent class="space-y-4 text-sm">
                    <div class="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
                        <div class="rounded-lg border border-border/70 p-4">
                            <div class="text-muted-foreground">{t.t("execution.ops.overview.total_sessions")}</div>
                            <div class="mt-2 text-2xl font-semibold">{ops_overview.terminal_summary.total_sessions}</div>
                        </div>
                        <div class="rounded-lg border border-border/70 p-4">
                            <div class="text-muted-foreground">{t.t("execution.ops.overview.active_clients")}</div>
                            <div class="mt-2 text-2xl font-semibold">{ops_overview.terminal_summary.active_clients}</div>
                        </div>
                        <div class="rounded-lg border border-border/70 p-4">
                            <div class="text-muted-foreground">{t.t("execution.ops.overview.pending_active")}</div>
                            <div class="mt-2 text-2xl font-semibold">{format!("{}/{}", ops_overview.terminal_summary.pending_sessions, ops_overview.terminal_summary.active_sessions)}</div>
                        </div>
                        <div class="rounded-lg border border-border/70 p-4">
                            <div class="text-muted-foreground">{t.t("execution.ops.overview.stale_threshold")}</div>
                            <div class="mt-2 text-2xl font-semibold">{format!("{}s", ops_overview.terminal_summary.stale_session_threshold_secs)}</div>
                        </div>
                    </div>
                    <div class="rounded-lg border border-border/70 bg-muted/20 p-4">
                        <div class="text-xs uppercase tracking-wide text-muted-foreground">{t.t("execution.ops.overview.cast_storage_dir")}</div>
                        <div class="mt-2 break-all font-mono text-xs">{ops_overview.cast_storage_dir.clone()}</div>
                    </div>
                    <div class="flex flex-col gap-3 rounded-lg border border-border/70 p-4 md:flex-row md:items-end">
                        <div class="w-full md:max-w-xs">
                            <label class="mb-1 block text-sm font-medium">{t.t("execution.ops.cleanup.retention_days")}</label>
                            <Input
                                type_="number"
                                value={(*retention_days).clone()}
                                min={Some("1".to_string())}
                                oninput={let retention_days = retention_days.clone(); Callback::from(move |value| retention_days.set(value))}
                            />
                        </div>
                        <Button variant={ButtonVariant::Default} onclick={on_cleanup}>{t.t("execution.ops.cleanup.run")}</Button>
                    </div>
                </CardContent>
            </Card>

            <Card>
                <CardHeader>
                    <CardTitle class="text-lg">{t.t("execution.ops.terminal_governance.title")}</CardTitle>
                </CardHeader>
                <CardContent>
                    if terminal_sessions.is_empty() {
                        <div class="py-6 text-center text-muted-foreground">{t.t("execution.ops.terminal_governance.empty")}</div>
                    } else {
                        <Table>
                            <TableHeader>
                                <TableRow>
                                    <TableHead>{t.t("execution.ops.table.session")}</TableHead>
                                    <TableHead>{t.t("execution.ops.table.client")}</TableHead>
                                    <TableHead>{t.t("execution.ops.table.user")}</TableHead>
                                    <TableHead>{t.t("execution.ops.table.mode")}</TableHead>
                                    <TableHead>{t.t("execution.ops.table.state")}</TableHead>
                                    <TableHead>{t.t("execution.ops.table.last_activity")}</TableHead>
                                    <TableHead>{t.t("execution.ops.table.last_heartbeat")}</TableHead>
                                    <TableHead>{t.t("execution.ops.table.close_reason")}</TableHead>
                                    <TableHead class="text-right">{t.t("execution.ops.table.operations")}</TableHead>
                                </TableRow>
                            </TableHeader>
                            <TableBody>
                                {for terminal_sessions.iter().map(|session| {
                                        let session_id = session.session_id.clone();
                                        let can_close = matches!(session.state, TerminalSessionState::Pending | TerminalSessionState::Active);
                                        let (state_label, state_class) = terminal_state_label(&session.state, t.as_ref());
                                        html! {
                                            <TableRow>
                                                <TableCell class="font-mono text-xs">{session.session_id.clone()}</TableCell>
                                                <TableCell class="font-mono text-xs">{session.client_id.clone()}</TableCell>
                                                <TableCell>{session.username.clone()}</TableCell>
                                                <TableCell>{terminal_mode_label(&session.mode, t.as_ref())}</TableCell>
                                                <TableCell><span class={classes!("text-sm", state_class)}>{state_label}</span></TableCell>
                                                <TableCell class="text-xs text-muted-foreground">{format_optional_datetime(session.last_activity_at.as_ref())}</TableCell>
                                                <TableCell class="text-xs text-muted-foreground">{format_optional_datetime(session.last_heartbeat_at.as_ref())}</TableCell>
                                                <TableCell class="text-xs text-muted-foreground">{session.close_reason.clone().unwrap_or_else(|| t.t("execution.ops.none"))}</TableCell>
                                                <TableCell class="text-right">
                                                    if can_close {
                                                        <Button variant={ButtonVariant::Outline} onclick={{ let on_close_session = on_close_session.clone(); Callback::from(move |_| on_close_session.emit(session_id.clone())) }}>
                                                            {t.t("execution.ops.actions.force_close")}
                                                        </Button>
                                                    } else {
                                                        <span class="text-xs text-muted-foreground">{t.t("execution.ops.terminal_governance.ended")}</span>
                                                    }
                                                </TableCell>
                                            </TableRow>
                                        }
                                    })}
                            </TableBody>
                        </Table>
                    }
                </CardContent>
            </Card>
        </div>
    }
}
