use gloo::timers::callback::Interval;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::components::loading::Loading;
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::card::{Card, CardContent, CardHeader, CardTitle};
use crate::hooks::use_trans::use_trans;
use crate::icons::History;
use crate::pages::execution::detail::{
    parse_cast_output, ExecutionDetailClient, ExecutionDetailOutput, ExecutionDetailView,
};
use crate::routes::Route;
use crate::services::{api, command};
use crate::types::Client;
use common::command::CommandLogLine;
use common::entity::execution::{ExecutionSession, ExecutionType, SessionStatus};

fn history_output_placeholder(session: &ExecutionSession, has_recording: bool, t: &crate::i18n::I18n) -> String {
    if matches!(session.status, SessionStatus::Pending | SessionStatus::Running) {
        return t.t("execution.history.output_pending");
    }

    if matches!(session.execution_type, ExecutionType::Terminal) && has_recording {
        return t.t("execution.history.output_replay_hint");
    }

    t.t("execution.history.output_empty")
}

fn resolve_client_parts(client_id: &str, clients: &[Client]) -> (String, String, String) {
    clients
        .iter()
        .find(|client| client.id == client_id)
        .map(|client| {
            let ip = client.primary_ip.clone().unwrap_or(client.ip_address.clone());
            (
                client.hostname.clone(),
                ip.clone(),
                format!("{} ({})", client.hostname, ip),
            )
        })
        .unwrap_or_else(|| (client_id.to_string(), String::new(), client_id.to_string()))
}

#[derive(Properties, PartialEq)]
pub struct ReplayPageProps {
    pub session_id: String,
}

#[function_component(ReplayPage)]
pub fn replay_page(props: &ReplayPageProps) -> Html {
    let t = use_trans();
    let navigator = use_navigator();
    let session = use_state(|| None::<ExecutionSession>);
    let clients = use_state(Vec::<Client>::new);
    let cast_text = use_state(String::new);
    let command_logs = use_state(Vec::<CommandLogLine>::new);
    let loading = use_state(|| true);
    let error = use_state(|| Option::<String>::None);

    let on_back = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            if let Some(navigator) = &navigator {
                navigator.push(&Route::ExecutionHistory);
            }
        })
    };

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
        let session_id = props.session_id.clone();
        let session = session.clone();
        let cast_text = cast_text.clone();
        let command_logs = command_logs.clone();
        let loading = loading.clone();
        let error = error.clone();
        let t = t.clone();
        use_effect_with((), move |_| {
            let session_id = session_id.clone();
            let session = session.clone();
            let cast_text = cast_text.clone();
            let command_logs = command_logs.clone();
            let loading = loading.clone();
            let error = error.clone();
            let t = t.clone();
            spawn_local(async move {
                match command::fetch_session(&session_id).await {
                    Ok(s) => {
                        let has_recording = s.cast_file_path.is_some();
                        session.set(Some(s));

                        if has_recording {
                            if let Ok(bytes) = command::fetch_session_cast(&session_id).await {
                                cast_text.set(parse_cast_output(&bytes));
                            }
                        }

                        if let Ok(logs) = command::fetch_command_logs(&session_id).await {
                            command_logs.set(logs);
                        }
                    }
                    Err(e) => {
                        error.set(Some(format!("{}: {}", t.t("execution.replay.load_session_error"), e.message)));
                        loading.set(false);
                        return;
                    }
                }

                loading.set(false);
            });
            || ()
        });
    }

    {
        let session_id = props.session_id.clone();
        let session = session.clone();
        use_effect_with((), move |_| {
            let interval = Interval::new(3000, move || {
                let session_id = session_id.clone();
                let session = session.clone();
                spawn_local(async move {
                    if let Ok(s) = command::fetch_session(&session_id).await {
                        session.set(Some(s));
                    }
                });
            });
            Box::new(move || drop(interval)) as Box<dyn FnOnce()>
        });
    }

    let detail_session = (*session).clone();
    let detail_clients = detail_session
        .as_ref()
        .map(|sess| {
            sess.client_ids
                .iter()
                .map(|client_id| {
                    let (title, subtitle, tooltip) = resolve_client_parts(client_id, clients.as_ref());
                    ExecutionDetailClient {
                        title,
                        subtitle,
                        tooltip,
                    }
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let detail_output = detail_session.as_ref().map(|sess| {
        let has_recording = sess.cast_file_path.is_some();
        if !cast_text.is_empty() {
            ExecutionDetailOutput::Replay((*cast_text).clone())
        } else if !command_logs.is_empty() {
            ExecutionDetailOutput::Logs((*command_logs).clone())
        } else {
            ExecutionDetailOutput::Placeholder(history_output_placeholder(sess, has_recording, t.as_ref()))
        }
    });

    html! {
        <div class="space-y-6 p-4">
            <Card>
                <CardHeader class="gap-4 md:flex-row md:items-start md:justify-between md:space-y-0">
                    <div class="space-y-2">
                        <div class="flex items-center gap-3">
                            <History class="h-6 w-6 text-primary" />
                            <div>
                                <CardTitle class="text-xl">{t.t("execution.replay.title")}</CardTitle>
                                <p class="text-sm text-muted-foreground">
                                    {t.t("execution.replay.description")}
                                </p>
                            </div>
                        </div>
                        <Badge variant={BadgeVariant::Outline} class="font-mono text-xs">{props.session_id.clone()}</Badge>
                    </div>
                    <div class="flex items-center gap-2">
                        <Button variant={ButtonVariant::Outline} size={ButtonSize::Sm} onclick={on_back}>
                            {t.t("execution.replay.back")}
                        </Button>
                    </div>
                </CardHeader>
                <CardContent class="space-y-6">
                    if *loading {
                        <Loading message={t.t("execution.replay.loading")} />
                    } else if let Some(err) = &*error {
                        <div class="rounded-lg border border-destructive/40 bg-destructive/10 p-4 text-sm text-destructive">
                            {err}
                        </div>
                    } else {
                        if let Some(sess) = detail_session {
                            <ExecutionDetailView
                                session={sess.clone()}
                                user_label={sess.username.clone()}
                                clients={detail_clients.clone()}
                                output={detail_output.clone().unwrap_or_else(|| ExecutionDetailOutput::Placeholder(String::new()))}
                                output_title={t.t("execution.replay.output_title")}
                                output_description={t.t("execution.replay.output_description")}
                                output_action={html! {
                                    <Button variant={ButtonVariant::Outline} size={ButtonSize::Sm} disabled={sess.cast_file_path.is_none()}>
                                        {t.t("execution.actions.replay")}
                                    </Button>
                                }}
                            />
                        }
                    }
                </CardContent>
            </Card>
        </div>
    }
}
