use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use std::rc::Rc;

use gloo::timers::callback::Interval;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::components::notification::{Notification, NotificationType};
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::card::{Card, CardContent, CardHeader, CardTitle};
use crate::components::ui::checkbox::Checkbox;
use crate::components::ui::input::Input;
use crate::components::ui::select::{Select, SelectOption};
use crate::hooks::use_trans::use_trans;
use crate::i18n::I18n;
use crate::icons::{History, LoaderCircle, Search, Terminal};
use crate::services::api;
use crate::services::command;
use crate::types::{
    Client, CommandLogLine, CommandStatus, CommandTask, CreateCommandRequest, Project,
};
use common::entity::execution::ExecutionType;

fn command_status_badge(status: &CommandStatus, t: &I18n) -> Html {
    let (variant, label) = match status {
        CommandStatus::Pending => (BadgeVariant::Warning, t.t("execution.status.pending")),
        CommandStatus::Running => (BadgeVariant::Info, t.t("execution.status.running")),
        CommandStatus::Success => (BadgeVariant::Success, t.t("execution.status.success")),
        CommandStatus::Failed => (BadgeVariant::Destructive, t.t("execution.status.failed")),
        CommandStatus::Timeout => (BadgeVariant::Destructive, t.t("execution.status.timeout")),
        CommandStatus::Expired => (BadgeVariant::Secondary, t.t("execution.status.expired")),
    };
    html! { <Badge variant={variant}>{label}</Badge> }
}

fn is_terminal_status(status: &CommandStatus) -> bool {
    matches!(
        status,
        CommandStatus::Success
            | CommandStatus::Failed
            | CommandStatus::Timeout
            | CommandStatus::Expired
    )
}

fn is_waiting_for_output(status: Option<&CommandStatus>, active_logs: &[CommandLogLine]) -> bool {
    active_logs.is_empty()
        && matches!(
            status,
            Some(CommandStatus::Pending | CommandStatus::Running) | None
        )
}

#[derive(Clone, Copy, PartialEq)]
enum BatchOutputMode {
    Single,
    All,
}

#[derive(Clone, Default, PartialEq)]
struct BatchTasksState(Vec<CommandTask>);

impl Deref for BatchTasksState {
    type Target = [CommandTask];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

enum BatchTasksAction {
    Replace(Vec<CommandTask>),
    Updated(CommandTask),
    Clear,
}

impl Reducible for BatchTasksState {
    type Action = BatchTasksAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            BatchTasksAction::Replace(tasks) => Rc::new(Self(tasks)),
            BatchTasksAction::Updated(updated) => {
                let mut tasks = self.0.clone();
                if let Some(existing) = tasks.iter_mut().find(|task| task.id == updated.id) {
                    *existing = updated;
                }
                Rc::new(Self(tasks))
            }
            BatchTasksAction::Clear => Rc::new(Self::default()),
        }
    }
}

#[derive(Clone, Default, PartialEq)]
struct BatchLogsState {
    by_task: HashMap<String, Vec<CommandLogLine>>,
    errors: HashSet<String>,
}

enum BatchLogsAction {
    Loaded(String, Vec<CommandLogLine>),
    Failed(String),
    Reset,
}

impl Reducible for BatchLogsState {
    type Action = BatchLogsAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut next = (*self).clone();
        match action {
            BatchLogsAction::Loaded(task_id, logs) => {
                next.errors.remove(&task_id);
                next.by_task.insert(task_id, logs);
            }
            BatchLogsAction::Failed(task_id) => {
                next.errors.insert(task_id);
            }
            BatchLogsAction::Reset => return Rc::new(Self::default()),
        }
        Rc::new(next)
    }
}

fn logs_are_loading(task_id: &str, status: &CommandStatus, logs: &BatchLogsState) -> bool {
    !logs.by_task.contains_key(task_id)
        || is_waiting_for_output(
            Some(status),
            logs.by_task.get(task_id).map(Vec::as_slice).unwrap_or(&[]),
        )
}

fn fetch_logs_into_cache(task_ids: Vec<String>, logs: UseReducerHandle<BatchLogsState>) {
    for task_id in task_ids {
        let logs = logs.clone();
        spawn_local(async move {
            match command::fetch_command_logs(&task_id).await {
                Ok(lines) => logs.dispatch(BatchLogsAction::Loaded(task_id, lines)),
                Err(_) => logs.dispatch(BatchLogsAction::Failed(task_id)),
            }
        });
    }
}

fn task_logs_panel(task: &CommandTask, logs: &BatchLogsState, t: &I18n, compact: bool) -> Html {
    let task_logs = logs.by_task.get(&task.id).map(Vec::as_slice).unwrap_or(&[]);
    let min_height = if compact {
        "min-h-[120px]"
    } else {
        "min-h-[420px]"
    };
    let content_height = if compact {
        "min-h-[80px]"
    } else {
        "min-h-[360px]"
    };

    html! {
        <div class={classes!(min_height, "rounded-lg", "border", "border-border", "bg-black/90", "p-4", "font-mono", "text-xs", "text-green-300", "overflow-auto")}>
            if logs.errors.contains(&task.id) && !logs.by_task.contains_key(&task.id) {
                <div class={classes!(content_height, "flex", "items-center", "justify-center", "text-center", "text-sm", "text-red-300")}>
                    {t.t("execution.batch.output_load_failed")}
                </div>
            } else if logs_are_loading(&task.id, &task.status, logs) {
                <div class={classes!(content_height, "flex", "items-center", "justify-center", "gap-3", "text-muted-foreground")}>
                    <LoaderCircle class="h-4 w-4 animate-spin" />
                    {t.t("execution.batch.waiting_output")}
                </div>
            } else if task_logs.is_empty() {
                <div class={classes!(content_height, "flex", "items-center", "justify-center", "text-center", "text-sm", "text-muted-foreground")}>
                    {t.t("execution.batch.no_output_terminal")}
                </div>
            } else {
                { for task_logs.iter().map(|line| {
                    let color = match line.stream {
                        common::command::LogStream::Stdout => "text-green-300",
                        common::command::LogStream::Stderr => "text-red-300",
                    };
                    html! {
                        <div class={classes!("whitespace-pre-wrap", color)}>{line.line.clone()}</div>
                    }
                }) }
            }
        </div>
    }
}

fn parse_client_ids(query: &HashMap<String, String>) -> HashSet<String> {
    let mut ids = HashSet::new();
    if let Some(client_id) = query.get("client_id") {
        if !client_id.trim().is_empty() {
            ids.insert(client_id.trim().to_string());
        }
    }
    if let Some(client_ids) = query.get("client_ids") {
        for client_id in client_ids.split(',') {
            let trimmed = client_id.trim();
            if !trimmed.is_empty() {
                ids.insert(trimmed.to_string());
            }
        }
    }
    ids
}

fn preloaded_command(query: &HashMap<String, String>) -> Option<String> {
    query.get("command").cloned()
}

fn rack_scope_key(client: &Client) -> Option<String> {
    let rack = client.rack.as_deref().unwrap_or("").trim();
    let location = client.location.as_deref().unwrap_or("").trim();
    if rack.is_empty() && location.is_empty() {
        None
    } else {
        Some(format!("{}||{}", rack, location))
    }
}

fn rack_scope_label(client: &Client) -> Option<String> {
    let rack = client.rack.as_deref().unwrap_or("").trim();
    let location = client.location.as_deref().unwrap_or("").trim();
    match (rack.is_empty(), location.is_empty()) {
        (true, true) => None,
        (false, true) => Some(rack.to_string()),
        (true, false) => Some(location.to_string()),
        (false, false) => Some(format!("{} / {}", location, rack)),
    }
}

fn matches_batch_filters(
    client: &Client,
    query: &str,
    active_project: &str,
    active_rack: &str,
) -> bool {
    let query = query.trim().to_lowercase();
    let text_match = query.is_empty()
        || client.hostname.to_lowercase().contains(&query)
        || client.ip_address.to_lowercase().contains(&query)
        || client.id.to_lowercase().contains(&query)
        || client
            .primary_ip
            .as_ref()
            .map(|ip| ip.to_lowercase().contains(&query))
            .unwrap_or(false)
        || client
            .serial_number
            .as_ref()
            .map(|serial| serial.to_lowercase().contains(&query))
            .unwrap_or(false)
        || client
            .location
            .as_ref()
            .map(|location| location.to_lowercase().contains(&query))
            .unwrap_or(false)
        || client
            .rack
            .as_ref()
            .map(|rack| rack.to_lowercase().contains(&query))
            .unwrap_or(false);

    let project_match =
        active_project.is_empty() || client.project_id.as_deref() == Some(active_project);
    let rack_match =
        active_rack.is_empty() || rack_scope_key(client).as_deref() == Some(active_rack);

    text_match && project_match && rack_match
}

#[cfg(test)]
mod tests {
    use super::{
        matches_batch_filters, parse_client_ids, preloaded_command, rack_scope_key,
        rack_scope_label,
    };
    use crate::types::{Client, CommandLogLine};
    use std::collections::{HashMap, HashSet};

    fn client() -> Client {
        Client {
            id: "client-1".to_string(),
            hostname: "app-node-01".to_string(),
            ip_address: "10.0.0.12".to_string(),
            primary_ip: Some("172.16.1.20".to_string()),
            serial_number: Some("SN-001".to_string()),
            location: Some("dc-a".to_string()),
            rack: Some("rack-7".to_string()),
            project_id: Some("project-a".to_string()),
            ..Default::default()
        }
    }

    #[test]
    fn parse_client_ids_combines_single_and_multi_values() {
        let mut query = HashMap::new();
        query.insert("client_id".to_string(), "client-a".to_string());
        query.insert(
            "client_ids".to_string(),
            "client-b, client-c , ,client-a".to_string(),
        );

        let ids = parse_client_ids(&query);
        let expected = HashSet::from([
            "client-a".to_string(),
            "client-b".to_string(),
            "client-c".to_string(),
        ]);

        assert_eq!(ids, expected);
    }

    #[test]
    fn parse_client_ids_ignores_blank_values() {
        let mut query = HashMap::new();
        query.insert("client_id".to_string(), "   ".to_string());
        query.insert("client_ids".to_string(), " , , ".to_string());

        let ids = parse_client_ids(&query);

        assert!(ids.is_empty());
    }

    #[test]
    fn preloaded_command_preserves_exact_query_value() {
        let mut query = HashMap::new();
        query.insert(
            "command".to_string(),
            "  ls -la /tmp\ncat test.txt  ".to_string(),
        );

        assert_eq!(
            preloaded_command(&query),
            Some("  ls -la /tmp\ncat test.txt  ".to_string())
        );
    }

    #[test]
    fn rack_scope_helpers_build_stable_key_and_label() {
        let client = client();

        assert_eq!(rack_scope_key(&client), Some("rack-7||dc-a".to_string()));
        assert_eq!(rack_scope_label(&client), Some("dc-a / rack-7".to_string()));
    }

    #[test]
    fn matches_batch_filters_checks_text_project_and_rack() {
        let client = client();

        assert!(matches_batch_filters(
            &client,
            "172.16",
            "project-a",
            "rack-7||dc-a"
        ));
        assert!(!matches_batch_filters(
            &client,
            "172.16",
            "project-b",
            "rack-7||dc-a"
        ));
        assert!(!matches_batch_filters(
            &client,
            "172.16",
            "project-a",
            "rack-8||dc-a"
        ));
        assert!(!matches_batch_filters(
            &client,
            "missing",
            "project-a",
            "rack-7||dc-a"
        ));
    }

    #[test]
    fn waiting_for_output_only_applies_to_live_tasks_without_logs() {
        assert!(super::is_waiting_for_output(
            Some(&super::CommandStatus::Pending),
            &[]
        ));
        assert!(super::is_waiting_for_output(
            Some(&super::CommandStatus::Running),
            &[]
        ));
        assert!(!super::is_waiting_for_output(
            Some(&super::CommandStatus::Failed),
            &[]
        ));
        assert!(!super::is_waiting_for_output(
            Some(&super::CommandStatus::Success),
            &[]
        ));
        assert!(!super::is_waiting_for_output(
            Some(&super::CommandStatus::Failed),
            &[CommandLogLine {
                seq: 1,
                line: "boom".to_string(),
                stream: common::command::LogStream::Stderr,
                timestamp: "2026-01-01T00:00:00Z".to_string(),
            }]
        ));
    }
}

#[function_component(BatchExecPage)]
pub fn batch_exec_page() -> Html {
    let t = use_trans();
    let location = use_location();

    let clients = use_state(Vec::<Client>::new);
    let projects = use_state(Vec::<Project>::new);
    let search_query = use_state(String::new);
    let project_filter = use_state(String::new);
    let rack_filter = use_state(String::new);
    let selected_ids = use_state(HashSet::<String>::new);
    let command_text = use_state(String::new);
    let submitting = use_state(|| false);
    let show_results = use_state(|| false);
    let tasks = use_reducer(BatchTasksState::default);
    let active_task_id = use_state(|| None::<String>);
    let output_mode = use_state(|| BatchOutputMode::Single);
    let logs = use_reducer(BatchLogsState::default);
    let notification = use_state(|| None::<(NotificationType, String)>);

    let query_params = location
        .as_ref()
        .and_then(|loc| loc.query::<HashMap<String, String>>().ok())
        .unwrap_or_default();

    {
        let selected_ids = selected_ids.clone();
        let command_text = command_text.clone();
        let query_params = query_params.clone();
        use_effect_with(query_params.clone(), move |_| {
            let ids = parse_client_ids(&query_params);
            if !ids.is_empty() {
                selected_ids.set(ids);
            }
            if let Some(command) = preloaded_command(&query_params) {
                command_text.set(command);
            }
            || ()
        });
    }

    {
        let clients = clients.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                if let Ok(page) =
                    api::fetch_clients(1, 1000, None, None, Some("online".to_string())).await
                {
                    clients.set(page.items);
                }
            });
            || ()
        });
    }

    {
        let projects = projects.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                if let Ok(page) = api::fetch_projects(1, 1000, None, None).await {
                    projects.set(page.items);
                }
            });
            || ()
        });
    }

    {
        let tasks = tasks.clone();
        let show_results = show_results.clone();
        let task_ids = tasks.iter().map(|task| task.id.clone()).collect::<Vec<_>>();
        use_effect_with((*show_results, task_ids.clone()), move |_| {
            if !*show_results {
                return Box::new(|| ()) as Box<dyn FnOnce()>;
            }
            let tasks = tasks.clone();
            let refresh = move || {
                for task_id in task_ids.clone() {
                    let tasks = tasks.clone();
                    spawn_local(async move {
                        if let Ok(updated) = command::fetch_command(&task_id).await {
                            tasks.dispatch(BatchTasksAction::Updated(updated));
                        }
                    });
                }
            };
            refresh();
            let interval = Interval::new(2000, refresh);
            Box::new(move || drop(interval)) as Box<dyn FnOnce()>
        });
    }

    {
        let show_results = show_results.clone();
        let logs = logs.clone();
        let active_task_id = (*active_task_id).clone();
        let output_mode = *output_mode;
        let task_states = tasks
            .iter()
            .map(|task| (task.id.clone(), task.status.clone()))
            .collect::<Vec<_>>();
        use_effect_with(
            (
                *show_results,
                output_mode,
                active_task_id.clone(),
                task_states.clone(),
            ),
            move |_| {
                if !*show_results {
                    return Box::new(|| ()) as Box<dyn FnOnce()>;
                }

                let visible_ids = match output_mode {
                    BatchOutputMode::Single => {
                        active_task_id.clone().into_iter().collect::<Vec<_>>()
                    }
                    BatchOutputMode::All => task_states.iter().map(|(id, _)| id.clone()).collect(),
                };
                fetch_logs_into_cache(visible_ids, logs.clone());

                let live_ids = task_states
                    .iter()
                    .filter(|(id, status)| {
                        matches!(status, CommandStatus::Pending | CommandStatus::Running)
                            && (output_mode == BatchOutputMode::All
                                || active_task_id.as_deref() == Some(id.as_str()))
                    })
                    .map(|(id, _)| id.clone())
                    .collect::<Vec<_>>();
                let interval = if live_ids.is_empty() {
                    None
                } else {
                    let logs = logs.clone();
                    Some(Interval::new(2000, move || {
                        fetch_logs_into_cache(live_ids.clone(), logs.clone());
                    }))
                };
                Box::new(move || drop(interval)) as Box<dyn FnOnce()>
            },
        );
    }

    let filtered_clients = {
        let query = (*search_query).trim().to_lowercase();
        let active_project = (*project_filter).clone();
        let active_rack = (*rack_filter).clone();
        if query.is_empty() {
            clients
                .iter()
                .filter(|client| {
                    (active_project.is_empty()
                        || client.project_id.as_deref() == Some(active_project.as_str()))
                        && (active_rack.is_empty()
                            || rack_scope_key(client).as_deref() == Some(active_rack.as_str()))
                })
                .cloned()
                .collect::<Vec<_>>()
        } else {
            clients
                .iter()
                .filter(|client| {
                    matches_batch_filters(client, &query, &active_project, &active_rack)
                })
                .cloned()
                .collect::<Vec<_>>()
        }
    };

    let project_options = {
        let mut options = vec![SelectOption {
            value: String::new(),
            label: t.t("execution.batch.all_projects"),
        }];
        let mut project_items = (*projects).clone();
        project_items.sort_by(|a, b| a.name.cmp(&b.name));
        options.extend(project_items.into_iter().map(|project| {
            let code = project.code.unwrap_or_default();
            let label = if code.trim().is_empty() {
                project.name.clone()
            } else {
                format!("{} ({})", project.name, code)
            };
            SelectOption {
                value: project.id,
                label,
            }
        }));
        options
    };

    let rack_options = {
        let mut rack_items = clients
            .iter()
            .filter_map(|client| rack_scope_key(client).zip(rack_scope_label(client)))
            .collect::<Vec<_>>();
        rack_items.sort_by(|a, b| a.1.cmp(&b.1));
        rack_items.dedup_by(|a, b| a.0 == b.0);

        let mut options = vec![SelectOption {
            value: String::new(),
            label: t.t("execution.batch.all_racks"),
        }];
        options.extend(
            rack_items
                .into_iter()
                .map(|(value, label)| SelectOption { value, label }),
        );
        options
    };

    let selected_clients = clients
        .iter()
        .filter(|client| selected_ids.contains(&client.id))
        .cloned()
        .collect::<Vec<_>>();
    let selected_count = selected_clients.len();
    let over_limit = selected_count > 50;

    let close_notification = {
        let notification = notification.clone();
        Callback::from(move |_| notification.set(None))
    };

    let on_submit = {
        let command_text = command_text.clone();
        let selected_ids = selected_ids.clone();
        let tasks = tasks.clone();
        let active_task_id = active_task_id.clone();
        let show_results = show_results.clone();
        let output_mode = output_mode.clone();
        let logs = logs.clone();
        let notification = notification.clone();
        let submitting = submitting.clone();
        let t = t.clone();
        Callback::from(move |_| {
            let command_value = (*command_text).clone();
            if command_value.trim().is_empty() {
                notification.set(Some((
                    NotificationType::Warning,
                    t.t("execution.batch.validation.command_required"),
                )));
                return;
            }

            let ids = (*selected_ids).iter().cloned().collect::<Vec<_>>();
            if ids.is_empty() {
                notification.set(Some((
                    NotificationType::Warning,
                    t.t("execution.batch.validation.targets_required"),
                )));
                return;
            }

            if ids.len() > 50 {
                notification.set(Some((
                    NotificationType::Error,
                    t.t("execution.batch.validation.limit"),
                )));
                return;
            }

            submitting.set(true);
            let tasks = tasks.clone();
            let active_task_id = active_task_id.clone();
            let show_results = show_results.clone();
            let output_mode = output_mode.clone();
            let logs = logs.clone();
            let notification = notification.clone();
            let submitting = submitting.clone();
            let t = t.clone();

            spawn_local(async move {
                let mut created_tasks = Vec::new();
                let mut failures = 0usize;
                let mut approvals = 0usize;

                for client_id in ids {
                    let request = CreateCommandRequest {
                        client_id: client_id.clone(),
                        command: command_value.clone(),
                        shell_mode: true,
                        args: None,
                        execution_type: Some(ExecutionType::Batch),
                        timeout_secs: Some(60),
                        force: false,
                    };

                    match command::create_command(&request).await {
                        Ok(response) => {
                            if let Some(task_id) = response.task_id {
                                created_tasks.push(CommandTask {
                                    id: task_id,
                                    client_id,
                                    command: command_value.clone(),
                                    status: CommandStatus::Pending,
                                    ..Default::default()
                                });
                            } else if response.approval_id.is_some() {
                                approvals += 1;
                            }
                        }
                        Err(_) => failures += 1,
                    }
                }

                if created_tasks.is_empty() && approvals == 0 {
                    notification.set(Some((
                        NotificationType::Error,
                        t.t("execution.batch.notifications.no_tasks"),
                    )));
                } else {
                    let first_task_id = created_tasks.first().map(|task| task.id.clone());
                    let has_tasks = !created_tasks.is_empty();
                    tasks.dispatch(BatchTasksAction::Replace(created_tasks));
                    active_task_id.set(first_task_id);
                    output_mode.set(BatchOutputMode::Single);
                    logs.dispatch(BatchLogsAction::Reset);
                    show_results.set(has_tasks);
                    if approvals > 0 {
                        let args = HashMap::from([("count".to_string(), approvals.to_string())]);
                        notification.set(Some((
                            NotificationType::Info,
                            t.t_with_args("execution.batch.notifications.approvals_created", &args),
                        )));
                    } else if failures > 0 {
                        let args = HashMap::from([("count".to_string(), failures.to_string())]);
                        notification.set(Some((
                            NotificationType::Warning,
                            t.t_with_args("execution.batch.notifications.partial_failures", &args),
                        )));
                    } else {
                        notification.set(Some((
                            NotificationType::Success,
                            t.t("execution.batch.notifications.started"),
                        )));
                    }
                }

                submitting.set(false);
            });
        })
    };

    let selected_task = (*active_task_id)
        .as_ref()
        .and_then(|task_id| tasks.iter().find(|task| &task.id == task_id).cloned());

    let completed_count = tasks
        .iter()
        .filter(|task| is_terminal_status(&task.status))
        .count();
    let running_count = tasks
        .iter()
        .filter(|task| matches!(task.status, CommandStatus::Pending | CommandStatus::Running))
        .count();

    html! {
        <div class="space-y-6 p-4">
            if let Some((type_, message)) = (*notification).clone() {
                <Notification notification_type={type_} message={message} show={true} on_close={close_notification.clone()} />
            }

            <Card>
                                <CardHeader class="gap-4 md:flex-row md:items-start md:justify-between md:space-y-0">
                    <div class="space-y-2">
                        <div class="flex items-center gap-3">
                            <Terminal class="h-6 w-6 text-primary" />
                            <div>
                                <CardTitle class="text-xl">{t.t("execution.batch.title")}</CardTitle>
                                <p class="text-sm text-muted-foreground">
                                    {t.t("execution.batch.description")}
                                </p>
                            </div>
                        </div>
                        <div class="flex flex-wrap items-center gap-2">
                            <Badge variant={BadgeVariant::Outline}>
                                {t.t_with_args("execution.batch.selected_badge", &HashMap::from([("count".to_string(), selected_count.to_string())]))}
                            </Badge>
                            if over_limit {
                                <Badge variant={BadgeVariant::Destructive}>{t.t("execution.batch.limit_badge")}</Badge>
                            }
                            if query_params.contains_key("command") {
                                <Badge variant={BadgeVariant::Info}>{t.t("execution.batch.preloaded_badge")}</Badge>
                            }
                        </div>
                    </div>
                    <div class="flex gap-2">
                        <Button variant={ButtonVariant::Outline} size={ButtonSize::Sm} onclick={{
                            let show_results = show_results.clone();
                            let tasks = tasks.clone();
                            let output_mode = output_mode.clone();
                            let logs = logs.clone();
                            let active_task_id = active_task_id.clone();
                            Callback::from(move |_| {
                                show_results.set(false);
                                tasks.dispatch(BatchTasksAction::Clear);
                                output_mode.set(BatchOutputMode::Single);
                                logs.dispatch(BatchLogsAction::Reset);
                                active_task_id.set(None);
                            })
                        }} disabled={!*show_results}>
                            {t.t("execution.batch.back_to_workspace")}
                        </Button>
                    </div>
                </CardHeader>
                <CardContent>
                    if *show_results {
                        <div class="grid gap-6 xl:grid-cols-[minmax(320px,380px)_1fr]">
                            <Card class="border-border/70">
                                <CardHeader>
                                    <CardTitle class="text-lg">{t.t("execution.batch.targets")}</CardTitle>
                                </CardHeader>
                                <CardContent class="space-y-4">
                                    <div class="grid grid-cols-2 gap-3 text-sm">
                                        <div class="rounded-lg border border-border/70 bg-muted/20 p-3">
                                            <div class="text-xs uppercase tracking-wide text-muted-foreground">{t.t("execution.batch.running")}</div>
                                            <div class="mt-1 text-xl font-semibold text-foreground">{running_count}</div>
                                        </div>
                                        <div class="rounded-lg border border-border/70 bg-muted/20 p-3">
                                            <div class="text-xs uppercase tracking-wide text-muted-foreground">{t.t("execution.batch.completed")}</div>
                                            <div class="mt-1 text-xl font-semibold text-foreground">{completed_count}</div>
                                        </div>
                                    </div>
                                    <div class="max-h-[560px] space-y-2 overflow-auto pr-1">
                                        {for tasks.iter().map(|task| {
                                            let is_active = (*active_task_id).as_deref() == Some(task.id.as_str());
                                            let task_id = task.id.clone();
                                            let task_clone = task.clone();
                                            html! {
                                                <button
                                                    class={classes!(
                                                        "w-full",
                                                        "rounded-lg",
                                                        "border",
                                                        "p-3",
                                                        "text-left",
                                                        "transition-colors",
                                                        if is_active {
                                                            "border-cyan-500/40 bg-cyan-500/10"
                                                        } else {
                                                            "border-border/70 bg-muted/20 hover:bg-muted/40"
                                                        }
                                                    )}
                                                    onclick={{
                                                        let active_task_id = active_task_id.clone();
                                                        let output_mode = output_mode.clone();
                                                        Callback::from(move |_| {
                                                            active_task_id.set(Some(task_id.clone()));
                                                            output_mode.set(BatchOutputMode::Single);
                                                        })
                                                    }}
                                                >
                                                    <div class="flex items-start justify-between gap-3">
                                                        <div class="min-w-0">
                                                            <div class="truncate font-medium text-foreground">{task_clone.client_id.clone()}</div>
                                                            <div class="mt-1 truncate font-mono text-xs text-muted-foreground">{task_clone.command.clone()}</div>
                                                        </div>
                                                        {command_status_badge(&task_clone.status, t.as_ref())}
                                                    </div>
                                                </button>
                                            }
                                        })}
                                    </div>
                                </CardContent>
                            </Card>

                            <Card>
                                <CardHeader class="gap-3 md:flex-row md:items-center md:justify-between md:space-y-0">
                                    <div>
                                        <CardTitle class="text-lg">
                                            {if *output_mode == BatchOutputMode::All {
                                                t.t("execution.batch.all_output_title")
                                            } else {
                                                t.t("execution.batch.output_title")
                                            }}
                                        </CardTitle>
                                        <p class="text-sm text-muted-foreground">
                                            {if *output_mode == BatchOutputMode::All {
                                                t.t("execution.batch.all_output_description")
                                            } else {
                                                t.t("execution.batch.output_description")
                                            }}
                                        </p>
                                    </div>
                                    <div class="flex flex-wrap items-center gap-2">
                                        <Button
                                            variant={if *output_mode == BatchOutputMode::Single { ButtonVariant::Default } else { ButtonVariant::Outline }}
                                            size={ButtonSize::Sm}
                                            onclick={{
                                                let output_mode = output_mode.clone();
                                                Callback::from(move |_| output_mode.set(BatchOutputMode::Single))
                                            }}
                                        >
                                            {t.t("execution.batch.single_output")}
                                        </Button>
                                        <Button
                                            variant={if *output_mode == BatchOutputMode::All { ButtonVariant::Default } else { ButtonVariant::Outline }}
                                            size={ButtonSize::Sm}
                                            onclick={{
                                                let output_mode = output_mode.clone();
                                                Callback::from(move |_| output_mode.set(BatchOutputMode::All))
                                            }}
                                        >
                                            {t.t("execution.batch.all_output")}
                                        </Button>
                                        if *output_mode == BatchOutputMode::Single {
                                            if let Some(task) = &selected_task {
                                                {command_status_badge(&task.status, t.as_ref())}
                                            }
                                        }
                                    </div>
                                </CardHeader>
                                <CardContent class="space-y-4">
                                    if *output_mode == BatchOutputMode::All {
                                        <div class="max-h-[680px] space-y-4 overflow-auto pr-1">
                                            {for tasks.iter().map(|task| html! {
                                                <section class="space-y-3 rounded-lg border border-border/70 bg-muted/10 p-3">
                                                    <div class="flex flex-wrap items-start justify-between gap-3">
                                                        <div class="min-w-0">
                                                            <div class="break-all font-medium text-foreground">{task.client_id.clone()}</div>
                                                            <div class="mt-1 break-all font-mono text-[11px] text-muted-foreground">
                                                                {format!("{}: {}", t.t("execution.batch.task_id"), task.id)}
                                                            </div>
                                                        </div>
                                                        {command_status_badge(&task.status, t.as_ref())}
                                                    </div>
                                                    {task_logs_panel(task, &logs, t.as_ref(), true)}
                                                </section>
                                            })}
                                        </div>
                                    } else if let Some(task) = &selected_task {
                                        <div class="grid gap-4 md:grid-cols-3">
                                            <div class="rounded-lg border border-border/70 bg-muted/20 p-3 md:col-span-2">
                                                <div class="text-xs uppercase tracking-wide text-muted-foreground">{t.t("execution.batch.command_label")}</div>
                                                <pre class="mt-2 overflow-x-auto whitespace-pre-wrap rounded-md border border-border/70 bg-black/70 p-3 font-mono text-xs text-green-300">{task.command.clone()}</pre>
                                            </div>
                                            <div class="rounded-lg border border-border/70 bg-muted/20 p-3 text-sm text-muted-foreground">
                                                <div class="text-xs uppercase tracking-wide">{t.t("execution.batch.target_client")}</div>
                                                <div class="mt-1 break-all text-foreground">{task.client_id.clone()}</div>
                                                <div class="mt-4 text-xs uppercase tracking-wide">{t.t("execution.batch.task_id")}</div>
                                                <div class="mt-1 break-all font-mono text-xs text-foreground">{task.id.clone()}</div>
                                            </div>
                                        </div>
                                        {task_logs_panel(task, &logs, t.as_ref(), false)}
                                    } else {
                                        <div class="flex min-h-[420px] items-center justify-center rounded-lg border border-border/70 bg-muted/20 text-muted-foreground">
                                            {t.t("execution.batch.select_target_prompt")}
                                        </div>
                                    }
                                </CardContent>
                            </Card>
                        </div>
                    } else {
                        <div class="grid gap-6 xl:grid-cols-[minmax(360px,420px)_1fr]">
                            <Card class="border-border/70">
                                <CardHeader>
                                    <CardTitle class="text-lg">{t.t("execution.batch.select_targets")}</CardTitle>
                                </CardHeader>
                                <CardContent class="space-y-4">
                                    <div class="grid gap-4 md:grid-cols-2">
                                        <div class="space-y-2 md:col-span-2">
                                            <label class="text-sm font-medium text-foreground">{t.t("execution.batch.search_label")}</label>
                                            <div class="relative">
                                                <Search class="pointer-events-none absolute left-3 top-3 h-4 w-4 text-muted-foreground" />
                                                <Input
                                                    class="pl-9"
                                                    value={(*search_query).clone()}
                                                    oninput={{
                                                        let search_query = search_query.clone();
                                                        Callback::from(move |value: String| search_query.set(value))
                                                    }}
                                                    placeholder={t.t("execution.batch.search_placeholder")}
                                                />
                                            </div>
                                        </div>
                                        <div class="space-y-2">
                                            <label class="text-sm font-medium text-foreground">{t.t("execution.batch.project_filter")}</label>
                                            <Select
                                                value={(*project_filter).clone()}
                                                options={project_options.clone()}
                                                onchange={{
                                                    let project_filter = project_filter.clone();
                                                    Callback::from(move |value: String| project_filter.set(value))
                                                }}
                                            />
                                        </div>
                                        <div class="space-y-2">
                                            <label class="text-sm font-medium text-foreground">{t.t("execution.batch.rack_filter")}</label>
                                            <Select
                                                value={(*rack_filter).clone()}
                                                options={rack_options.clone()}
                                                onchange={{
                                                    let rack_filter = rack_filter.clone();
                                                    Callback::from(move |value: String| rack_filter.set(value))
                                                }}
                                            />
                                        </div>
                                    </div>
                                    <div class="flex flex-wrap items-center gap-2 text-sm text-muted-foreground">
                                        <Button variant={ButtonVariant::Outline} size={ButtonSize::Sm} onclick={{
                                            let selected_ids = selected_ids.clone();
                                            let filtered_clients = filtered_clients.clone();
                                            Callback::from(move |_| {
                                                let mut next = (*selected_ids).clone();
                                                for client in &filtered_clients {
                                                    next.insert(client.id.clone());
                                                }
                                                selected_ids.set(next);
                                            })
                                        }}>
                                            {t.t("execution.batch.select_visible")}
                                        </Button>
                                        <Button variant={ButtonVariant::Outline} size={ButtonSize::Sm} onclick={{
                                            let selected_ids = selected_ids.clone();
                                            Callback::from(move |_| selected_ids.set(HashSet::new()))
                                        }}>
                                            {t.t("execution.batch.clear_all")}
                                        </Button>
                                        <span>{t.t_with_args("execution.batch.matches", &HashMap::from([("count".to_string(), filtered_clients.len().to_string())]))}</span>
                                    </div>
                                    <div class="max-h-[520px] space-y-2 overflow-auto pr-1">
                                        {for filtered_clients.iter().map(|client| {
                                            let checked = selected_ids.contains(&client.id);
                                            let client_id = client.id.clone();
                                            let hostname = client.hostname.clone();
                                            let ip = client.primary_ip.clone().unwrap_or(client.ip_address.clone());
                                            let serial = client.serial_number.clone().unwrap_or_default();
                                            let project_name = client
                                                .project_id
                                                .as_ref()
                                                .and_then(|project_id| projects.iter().find(|project| project.id == *project_id))
                                                .map(|project| project.name.clone());
                                            let rack_label = rack_scope_label(client);
                                            html! {
                                                <label class="flex cursor-pointer items-start gap-3 rounded-lg border border-border/70 bg-muted/20 px-3 py-3 transition-colors hover:bg-muted/40">
                                                    <Checkbox
                                                        checked={checked}
                                                        onchange={{
                                                            let selected_ids = selected_ids.clone();
                                                            Callback::from(move |is_checked: bool| {
                                                                let mut next = (*selected_ids).clone();
                                                                if is_checked {
                                                                    next.insert(client_id.clone());
                                                                } else {
                                                                    next.remove(&client_id);
                                                                }
                                                                selected_ids.set(next);
                                                            })
                                                        }}
                                                    />
                                                    <div class="min-w-0 flex-1">
                                                        <div class="font-medium text-foreground">{hostname}</div>
                                                        <div class="mt-1 text-xs text-muted-foreground">{ip}</div>
                                                        <div class="mt-2 flex flex-wrap gap-2">
                                                            if let Some(project_name) = project_name {
                                                                <Badge variant={BadgeVariant::Outline}>{project_name}</Badge>
                                                            }
                                                            if let Some(rack_label) = rack_label {
                                                                <Badge variant={BadgeVariant::Secondary}>{rack_label}</Badge>
                                                            }
                                                        </div>
                                                        if !serial.is_empty() {
                                                            <div class="mt-1 font-mono text-xs text-muted-foreground">{serial}</div>
                                                        }
                                                    </div>
                                                </label>
                                            }
                                        })}
                                    </div>
                                </CardContent>
                            </Card>

                            <div class="space-y-6">
                                <Card>
                                    <CardHeader>
                                        <CardTitle class="text-lg">{t.t("execution.batch.summary_title")}</CardTitle>
                                    </CardHeader>
                                    <CardContent class="space-y-4">
                                        <div class="flex flex-wrap items-center gap-2">
                                            <Badge variant={BadgeVariant::Outline}>
                                                {t.t_with_args("execution.batch.selected_badge", &HashMap::from([("count".to_string(), selected_count.to_string())]))}
                                            </Badge>
                                            if over_limit {
                                                <Badge variant={BadgeVariant::Destructive}>{t.t("execution.batch.reduce_limit")}</Badge>
                                            }
                                        </div>
                                        if selected_clients.is_empty() {
                                            <div class="rounded-lg border border-dashed border-border/70 p-6 text-sm text-muted-foreground">
                                                {t.t("execution.batch.summary_empty")}
                                            </div>
                                        } else {
                                            <div class="flex flex-wrap gap-2">
                                                {for selected_clients.iter().map(|client| html! {
                                                    <Badge variant={BadgeVariant::Secondary}>{format!("{} ({})", client.hostname, client.primary_ip.clone().unwrap_or(client.ip_address.clone()))}</Badge>
                                                })}
                                            </div>
                                        }
                                    </CardContent>
                                </Card>

                                <Card>
                                    <CardHeader>
                                        <CardTitle class="text-lg">{t.t("execution.batch.editor_title")}</CardTitle>
                                    </CardHeader>
                                    <CardContent class="space-y-4">
                                        <div class="rounded-lg border border-cyan-500/20 bg-cyan-500/10 p-3 text-sm text-cyan-100">
                                            {t.t("execution.batch.editor_help")}
                                        </div>
                                        <textarea
                                            class="min-h-56 w-full rounded-md border border-input bg-slate-950/70 px-3 py-3 font-mono text-sm text-green-300 placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                                            value={(*command_text).clone()}
                                            oninput={{
                                                let command_text = command_text.clone();
                                                Callback::from(move |event: InputEvent| {
                                                    let value = event.target_unchecked_into::<web_sys::HtmlTextAreaElement>().value();
                                                    command_text.set(value);
                                                })
                                            }}
                                            onkeydown={{
                                                let on_submit = on_submit.clone();
                                                Callback::from(move |event: KeyboardEvent| {
                                                    if event.ctrl_key() && event.key() == "Enter" {
                                                        event.prevent_default();
                                                        on_submit.emit(());
                                                    }
                                                })
                                            }}
                                            placeholder={t.t("execution.batch.editor_placeholder")}
                                            disabled={*submitting}
                                        />
                                        <div class="flex items-center justify-between gap-3 text-sm text-muted-foreground">
                                            <div class="flex flex-wrap gap-2">
                                                <Badge variant={BadgeVariant::Info}>{t.t("execution.batch.shortcut_newline")}</Badge>
                                                <Badge variant={BadgeVariant::Info}>{t.t("execution.batch.shortcut_execute")}</Badge>
                                            </div>
                                            <Button
                                                disabled={*submitting || selected_count == 0 || over_limit || (*command_text).trim().is_empty()}
                                                onclick={{
                                                    let on_submit = on_submit.clone();
                                                    Callback::from(move |_| on_submit.emit(()))
                                                }}
                                            >
                                                if *submitting {
                                                    <LoaderCircle class="mr-2 h-4 w-4 animate-spin" />
                                                    {t.t("execution.batch.submitting")}
                                                } else {
                                                    <Terminal class="mr-2 h-4 w-4" />
                                                    {t.t("execution.batch.execute")}
                                                }
                                            </Button>
                                        </div>
                                    </CardContent>
                                </Card>
                            </div>
                        </div>
                    }
                </CardContent>
            </Card>
        </div>
    }
}
