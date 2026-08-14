use std::collections::HashMap;

use serde_json::json;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::components::common::pagination::Pagination;
use crate::components::loading::Loading;
use crate::components::notification::{Notification, NotificationType};
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::card::{Card, CardContent, CardHeader, CardTitle};
use crate::components::ui::input::Input;
use crate::components::ui::select::{Select, SelectOption};
use crate::components::ui::table::{Table, TableBody, TableCell, TableHead, TableHeader, TableRow};
use crate::hooks::use_trans::use_trans;
use crate::pages::execution::detail::{execution_type_badge, session_status_badge};
use crate::routes::Route;
use crate::services::{api, command};
use crate::types::{Client, User};
use crate::utils::format::format_datetime_with_ago;
use common::entity::execution::{ExecutionSession, ExecutionType, SessionStatus};

fn format_date_start(value: &str) -> Option<String> {
    if value.is_empty() {
        None
    } else {
        Some(format!("{}T00:00:00Z", value))
    }
}

fn format_date_end(value: &str) -> Option<String> {
    if value.is_empty() {
        None
    } else {
        Some(format!("{}T23:59:59Z", value))
    }
}

fn prefill_history_filters(
    query: &HashMap<String, String>,
) -> (Option<String>, Option<String>, Option<String>) {
    (
        query.get("client_id").cloned(),
        query.get("execution_type").cloned(),
        query.get("search").cloned(),
    )
}

fn rerun_target(session: &ExecutionSession) -> Option<(Route, HashMap<String, String>)> {
    if matches!(session.execution_type, ExecutionType::Terminal) {
        return None;
    }
    if matches!(session.execution_type, ExecutionType::Batch) {
        let mut query = HashMap::new();
        query.insert("client_ids".to_string(), session.client_ids.join(","));
        query.insert("command".to_string(), session.command.clone());
        return Some((Route::ExecutionBatch, query));
    }
    None
}

fn command_preview(command: &str) -> String {
    let mut lines = command.lines();
    let first = lines.next().unwrap_or_default().trim_end();
    let has_more = lines.any(|line| !line.trim().is_empty());

    let mut preview = if first.chars().count() > 72 {
        let truncated = first.chars().take(72).collect::<String>();
        format!("{}...", truncated)
    } else {
        first.to_string()
    };

    if has_more {
        if preview.is_empty() {
            preview = "...".to_string();
        }
        preview.push_str("  [+more]");
    }

    preview
}

fn client_display_label(client: &Client) -> String {
    format!(
        "{} ({})",
        client.hostname,
        client
            .primary_ip
            .clone()
            .unwrap_or(client.ip_address.clone())
    )
}

fn client_display_parts(client: &Client) -> (String, String) {
    (
        client.hostname.clone(),
        client
            .primary_ip
            .clone()
            .unwrap_or(client.ip_address.clone()),
    )
}

fn resolve_client_parts(client_id: &str, clients: &[Client]) -> (String, String) {
    clients
        .iter()
        .find(|client| client.id == client_id)
        .map(client_display_parts)
        .unwrap_or_else(|| (client_id.to_string(), String::new()))
}

fn resolve_client_label(client_id: &str, clients: &[Client]) -> String {
    clients
        .iter()
        .find(|client| client.id == client_id)
        .map(client_display_label)
        .unwrap_or_else(|| client_id.to_string())
}

fn resolve_user_label(session: &ExecutionSession, users: &[User]) -> String {
    users
        .iter()
        .find(|user| user.id == session.user_id || user.username == session.username)
        .map(|user| user.username.clone())
        .unwrap_or_else(|| {
            if session.username.is_empty() {
                session.user_id.clone()
            } else {
                session.username.clone()
            }
        })
}

#[cfg(test)]
mod tests {
    use super::{
        command_preview, format_date_end, format_date_start, prefill_history_filters, rerun_target,
    };
    use crate::routes::Route;
    use common::entity::execution::{ExecutionSession, ExecutionType, SessionStatus};
    use std::collections::HashMap;

    fn session(
        execution_type: ExecutionType,
        client_ids: Vec<&str>,
        command: &str,
    ) -> ExecutionSession {
        ExecutionSession {
            session_id: "session-1".to_string(),
            user_id: "user-1".to_string(),
            username: "tester".to_string(),
            client_ids: client_ids.into_iter().map(|id| id.to_string()).collect(),
            execution_type,
            command: command.to_string(),
            status: SessionStatus::Success,
            start_time: "2026-07-08T10:00:00Z".to_string(),
            end_time: None,
            duration_secs: None,
            cast_file_path: None,
        }
    }

    #[test]
    fn format_date_range_wraps_day_boundaries() {
        assert_eq!(
            format_date_start("2026-07-08"),
            Some("2026-07-08T00:00:00Z".to_string())
        );
        assert_eq!(
            format_date_end("2026-07-08"),
            Some("2026-07-08T23:59:59Z".to_string())
        );
    }

    #[test]
    fn format_date_range_returns_none_for_empty_values() {
        assert_eq!(format_date_start(""), None);
        assert_eq!(format_date_end(""), None);
    }

    #[test]
    fn prefill_history_filters_reads_client_and_execution_type() {
        let mut query = HashMap::new();
        query.insert("client_id".to_string(), "client-7".to_string());
        query.insert("execution_type".to_string(), "terminal".to_string());

        assert_eq!(
            prefill_history_filters(&query),
            (
                Some("client-7".to_string()),
                Some("terminal".to_string()),
                None
            )
        );
    }

    #[test]
    fn rerun_target_skips_terminal_sessions() {
        let session = session(ExecutionType::Terminal, vec!["client-1"], "ls -la");

        assert!(rerun_target(&session).is_none());
    }

    #[test]
    fn rerun_target_routes_batch_sessions_back_to_batch_workspace() {
        let session = session(
            ExecutionType::Batch,
            vec!["client-1", "client-2"],
            "uname -a",
        );

        let (route, query) = rerun_target(&session).expect("batch rerun target");

        assert_eq!(route, Route::ExecutionBatch);
        assert_eq!(
            query.get("client_ids"),
            Some(&"client-1,client-2".to_string())
        );
        assert_eq!(query.get("command"), Some(&"uname -a".to_string()));
    }

    #[test]
    fn rerun_target_skips_terminal_session_without_client() {
        let session = session(ExecutionType::Terminal, vec![], "pwd");

        assert!(rerun_target(&session).is_none());
    }

    #[test]
    fn command_preview_marks_multiline_commands() {
        assert_eq!(
            command_preview("ls -l\ndf -h\nhostname -i"),
            "ls -l  [+more]"
        );
        assert_eq!(command_preview("uname -a"), "uname -a");
    }
}

#[function_component(HistoryPage)]
pub fn history_page() -> Html {
    let t = use_trans();
    let navigator = use_navigator();
    let location = use_location();

    let sessions = use_state(Vec::<ExecutionSession>::new);
    let clients = use_state(Vec::<Client>::new);
    let users = use_state(Vec::<User>::new);
    let loading = use_state(|| true);
    let page = use_state(|| 1usize);
    let page_size = use_state(|| 20usize);
    let total_items = use_state(|| 0usize);
    let total_pages = use_state(|| 0usize);
    let client_filter = use_state(String::new);
    let search_filter = use_state(String::new);
    let status_filter = use_state(|| "all".to_string());
    let execution_type_filter = use_state(|| "all".to_string());
    let from_filter = use_state(String::new);
    let to_filter = use_state(String::new);
    let notification = use_state(|| None::<(NotificationType, String)>);

    let query_params = location
        .as_ref()
        .and_then(|loc| loc.query::<HashMap<String, String>>().ok())
        .unwrap_or_default();

    {
        let client_filter = client_filter.clone();
        let search_filter = search_filter.clone();
        let execution_type_filter = execution_type_filter.clone();
        let query_params = query_params.clone();
        use_effect_with(query_params.clone(), move |_| {
            let (client_id, execution_type, search) = prefill_history_filters(&query_params);
            if let Some(client_id) = client_id {
                client_filter.set(client_id);
            }
            if let Some(execution_type) = execution_type {
                execution_type_filter.set(execution_type);
            }
            if let Some(search) = search {
                search_filter.set(search);
            }
            || ()
        });
    }

    {
        let clients = clients.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                if let Ok(page) = api::fetch_clients(1, 1000, None, None, None).await {
                    clients.set(page.items);
                }
            });
            || ()
        });
    }

    {
        let users = users.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                if let Ok(items) = api::fetch_users().await {
                    users.set(items);
                }
            });
            || ()
        });
    }

    {
        let sessions = sessions.clone();
        let loading = loading.clone();
        let total_items = total_items.clone();
        let total_pages = total_pages.clone();
        let notification = notification.clone();
        let deps = (
            *page,
            *page_size,
            (*client_filter).clone(),
            (*search_filter).clone(),
            (*status_filter).clone(),
            (*execution_type_filter).clone(),
            (*from_filter).clone(),
            (*to_filter).clone(),
        );
        use_effect_with(
            deps,
            move |(
                current_page,
                current_page_size,
                client_id,
                search,
                status,
                execution_type,
                from,
                to,
            )| {
                loading.set(true);
                let current_page = *current_page;
                let current_page_size = *current_page_size;
                let client_id = client_id.clone();
                let search = search.clone();
                let status = status.clone();
                let execution_type = execution_type.clone();
                let from_owned = from.clone();
                let to_owned = to.clone();
                let from_value = format_date_start(&from_owned);
                let to_value = format_date_end(&to_owned);
                spawn_local(async move {
                    match command::fetch_sessions(
                        if search.is_empty() {
                            None
                        } else {
                            Some(search.as_str())
                        },
                        if client_id.is_empty() {
                            None
                        } else {
                            Some(client_id.as_str())
                        },
                        None,
                        if status == "all" {
                            None
                        } else {
                            Some(status.as_str())
                        },
                        if execution_type == "all" {
                            None
                        } else {
                            Some(execution_type.as_str())
                        },
                        from_value.as_deref(),
                        to_value.as_deref(),
                        current_page,
                        current_page_size,
                    )
                    .await
                    {
                        Ok(result) => {
                            notification.set(None);
                            total_items.set(result.total);
                            total_pages.set(result.total_pages);
                            sessions.set(result.items);
                        }
                        Err(err) => {
                            total_items.set(0);
                            total_pages.set(0);
                            sessions.set(Vec::new());
                            notification.set(Some((NotificationType::Error, err.message)));
                        }
                    }
                    loading.set(false);
                });
                || ()
            },
        );
    }

    let client_options = std::iter::once(SelectOption {
        value: "".into(),
        label: t.t("execution.history.all_clients"),
    })
    .chain(clients.iter().map(|client| SelectOption {
        value: client.id.clone(),
        label: format!(
                "{} ({})",
                client.hostname,
                client
                    .primary_ip
                    .clone()
                    .unwrap_or(client.ip_address.clone())
            ),
    }))
    .collect::<Vec<_>>();

    let status_options = vec![
        SelectOption {
            value: "all".into(),
            label: t.t("execution.history.all_statuses"),
        },
        SelectOption {
            value: "pending".into(),
            label: t.t("execution.status.pending"),
        },
        SelectOption {
            value: "running".into(),
            label: t.t("execution.status.running"),
        },
        SelectOption {
            value: "success".into(),
            label: t.t("execution.status.success"),
        },
        SelectOption {
            value: "failed".into(),
            label: t.t("execution.status.failed"),
        },
        SelectOption {
            value: "partial".into(),
            label: t.t("execution.status.partial"),
        },
    ];

    let execution_type_options = vec![
        SelectOption {
            value: "all".into(),
            label: t.t("execution.history.all_execution_types"),
        },
        SelectOption {
            value: "terminal".into(),
            label: t.t("execution.type.terminal"),
        },
        SelectOption {
            value: "batch".into(),
            label: t.t("execution.type.batch"),
        },
    ];

    let close_notification = {
        let notification = notification.clone();
        Callback::from(move |_| notification.set(None))
    };

    html! {
        <div class="space-y-6 p-4">
            if let Some((type_, message)) = (*notification).clone() {
                <Notification notification_type={type_} message={message} show={true} on_close={close_notification} />
            }
            <Card>
                <CardHeader class="gap-4 md:flex-row md:items-start md:justify-between md:space-y-0">
                    <div class="space-y-2">
                        <CardTitle class="text-xl">{t.t("execution.history.title")}</CardTitle>
                        <p class="text-sm text-muted-foreground">
                            {t.t("execution.history.description")}
                        </p>
                    </div>
                    <div class="flex flex-wrap items-center gap-2">
                        <Badge variant={BadgeVariant::Outline}>
                            {t.t_with_args(
                                "execution.history.records_badge",
                                &HashMap::from([("count".to_string(), total_items.to_string())]),
                            )}
                        </Badge>
                    </div>
                </CardHeader>
                <CardContent class="space-y-6">
                    <div class="grid gap-4 md:grid-cols-2 xl:grid-cols-6">
                        <div class="space-y-2 xl:col-span-2">
                            <label class="text-sm font-medium text-foreground">{t.t("execution.history.search_label")}</label>
                            <Input
                                value={(*search_filter).clone()}
                                oninput={{
                                    let search_filter = search_filter.clone();
                                    let page = page.clone();
                                    Callback::from(move |value: String| {
                                        search_filter.set(value);
                                        page.set(1);
                                    })
                                }}
                                placeholder={t.t("execution.history.search_placeholder")}
                            />
                        </div>
                        <div class="space-y-2">
                            <label class="text-sm font-medium text-foreground">{t.t("execution.history.client_scope")}</label>
                            <Select
                                options={client_options}
                                value={(*client_filter).clone()}
                                onchange={{
                                    let client_filter = client_filter.clone();
                                    let page = page.clone();
                                    Callback::from(move |value: String| {
                                        client_filter.set(value);
                                        page.set(1);
                                    })
                                }}
                            />
                        </div>
                        <div class="space-y-2">
                            <label class="text-sm font-medium text-foreground">{t.t("execution.history.status")}</label>
                            <Select
                                options={status_options}
                                value={(*status_filter).clone()}
                                onchange={{
                                    let status_filter = status_filter.clone();
                                    let page = page.clone();
                                    Callback::from(move |value: String| {
                                        status_filter.set(value);
                                        page.set(1);
                                    })
                                }}
                            />
                        </div>
                        <div class="space-y-2">
                            <label class="text-sm font-medium text-foreground">{t.t("execution.history.execution_type")}</label>
                            <Select
                                options={execution_type_options}
                                value={(*execution_type_filter).clone()}
                                onchange={{
                                    let execution_type_filter = execution_type_filter.clone();
                                    let page = page.clone();
                                    Callback::from(move |value: String| {
                                        execution_type_filter.set(value);
                                        page.set(1);
                                    })
                                }}
                            />
                        </div>
                        <div class="space-y-2">
                            <label class="text-sm font-medium text-foreground">{t.t("execution.history.from")}</label>
                            <Input
                                type_="date"
                                value={(*from_filter).clone()}
                                oninput={{
                                    let from_filter = from_filter.clone();
                                    let page = page.clone();
                                    Callback::from(move |value: String| {
                                        from_filter.set(value);
                                        page.set(1);
                                    })
                                }}
                            />
                        </div>
                        <div class="space-y-2">
                            <label class="text-sm font-medium text-foreground">{t.t("execution.history.to")}</label>
                            <Input
                                type_="date"
                                value={(*to_filter).clone()}
                                oninput={{
                                    let to_filter = to_filter.clone();
                                    let page = page.clone();
                                    Callback::from(move |value: String| {
                                        to_filter.set(value);
                                        page.set(1);
                                    })
                                }}
                            />
                        </div>
                    </div>

                    <div class="flex flex-wrap gap-2">
                        <Button variant={ButtonVariant::Outline} size={ButtonSize::Sm} onclick={{
                            let client_filter = client_filter.clone();
                            let search_filter = search_filter.clone();
                            let status_filter = status_filter.clone();
                            let execution_type_filter = execution_type_filter.clone();
                            let from_filter = from_filter.clone();
                            let to_filter = to_filter.clone();
                            let page = page.clone();
                            Callback::from(move |_| {
                                client_filter.set(String::new());
                                search_filter.set(String::new());
                                status_filter.set("all".to_string());
                                execution_type_filter.set("all".to_string());
                                from_filter.set(String::new());
                                to_filter.set(String::new());
                                page.set(1);
                            })
                        }}>
                            {t.t("execution.actions.reset_filters")}
                        </Button>
                    </div>

                    if *loading {
                        <Loading message={t.t("execution.history.loading")} />
                    } else if sessions.is_empty() {
                        <div class="rounded-lg border border-dashed border-border/70 p-10 text-center text-muted-foreground">
                            {t.t("execution.history.empty")}
                        </div>
                    } else {
                        <div class="rounded-lg border border-border/70">
                            <Table>
                                <TableHeader>
                                    <TableRow>
                                        <TableHead>{t.t("execution.history.table.time")}</TableHead>
                                        <TableHead>{t.t("execution.history.table.type")}</TableHead>
                                        <TableHead>{t.t("execution.history.table.targets")}</TableHead>
                                        <TableHead>{t.t("execution.history.table.command")}</TableHead>
                                        <TableHead>{t.t("execution.history.table.status")}</TableHead>
                                        <TableHead>{t.t("execution.history.table.duration")}</TableHead>
                                        <TableHead>{t.t("execution.history.table.user")}</TableHead>
                                        <TableHead class="text-right">{t.t("execution.history.table.actions")}</TableHead>
                                    </TableRow>
                                </TableHeader>
                                <TableBody>
                                    {for sessions.iter().map(|session| {
                                        let session_id = session.session_id.clone();
                                        let rerun_session = session.clone();
                                        let command_preview = command_preview(&session.command);
                                        let client_labels = session.client_ids.iter().map(|client_id| resolve_client_label(client_id, clients.as_ref())).collect::<Vec<_>>();
                                        let user_label = resolve_user_label(session, users.as_ref());
                                        html! {
                                            <TableRow>
                                                <TableCell class="text-sm text-muted-foreground">{format_datetime_with_ago(&session.start_time)}</TableCell>
                                                <TableCell>{execution_type_badge(&session.execution_type, t.as_ref())}</TableCell>
                                                <TableCell>
                                                    <div class="flex max-w-80 flex-wrap gap-2" title={client_labels.join(", ")}>
                                                        {for session.client_ids.iter().take(3).map(|client_id| {
                                                            let (hostname, ip) = resolve_client_parts(client_id, clients.as_ref());
                                                            let full_label = resolve_client_label(client_id, clients.as_ref());
                                                            html! {
                                                                <div class="min-w-[9rem] rounded-md border border-border/70 bg-muted/20 px-2.5 py-1.5 text-xs" title={full_label}>
                                                                    <div class="truncate font-medium text-foreground">{hostname}</div>
                                                                    if !ip.is_empty() {
                                                                        <div class="truncate text-muted-foreground">{ip}</div>
                                                                    }
                                                                </div>
                                                            }
                                                        })}
                                                        if session.client_ids.len() > 3 {
                                                            <Badge variant={BadgeVariant::Outline}>{format!("+{}", session.client_ids.len() - 3)}</Badge>
                                                        }
                                                    </div>
                                                </TableCell>
                                                <TableCell>
                                                    <div class="max-w-md truncate font-mono text-xs text-foreground whitespace-pre-wrap" title={session.command.clone()}>
                                                        {command_preview}
                                                    </div>
                                                </TableCell>
                                                <TableCell>{session_status_badge(&session.status, t.as_ref())}</TableCell>
                                                <TableCell class="text-sm text-muted-foreground">
                                                    {session.duration_secs.map(|secs| format!("{}s", secs)).unwrap_or_else(|| "-".to_string())}
                                                </TableCell>
                                                <TableCell class="text-sm text-muted-foreground">{user_label}</TableCell>
                                                <TableCell class="text-right">
                                                    <div class="flex justify-end gap-2">
                                                        <Button variant={ButtonVariant::Ghost} size={ButtonSize::Sm} onclick={{
                                                            let navigator = navigator.clone();
                                                            let session_id = session_id.clone();
                                                            Callback::from(move |_| {
                                                                if let Some(navigator) = &navigator {
                                                                    navigator.push(&Route::ExecutionReplay { id: session_id.clone() });
                                                                }
                                                            })
                                                        }}>
                                                            {t.t("execution.actions.replay")}
                                                        </Button>
                                                        if let Some((route, query)) = rerun_target(&rerun_session) {
                                                            <Button variant={ButtonVariant::Ghost} size={ButtonSize::Sm} onclick={{
                                                                let navigator = navigator.clone();
                                                                Callback::from(move |_| {
                                                                    if let Some(navigator) = &navigator {
                                                                        let _ = navigator.push_with_query(&route, &json!(query));
                                                                    }
                                                                })
                                                            }}>
                                                                {t.t("execution.actions.rerun")}
                                                            </Button>
                                                        }
                                                    </div>
                                                </TableCell>
                                            </TableRow>
                                        }
                                    })}
                                </TableBody>
                            </Table>
                        </div>
                    }

                    <Pagination
                        total_pages={*total_pages}
                        current_page={*page}
                        page_size={*page_size}
                        total_items={*total_items}
                        on_page_change={{
                            let page = page.clone();
                            Callback::from(move |value: usize| page.set(value))
                        }}
                        on_page_size_change={{
                            let page_size = page_size.clone();
                            let page = page.clone();
                            Callback::from(move |value: usize| {
                                page_size.set(value);
                                page.set(1);
                            })
                        }}
                    />
                </CardContent>
            </Card>
        </div>
    }
}
