use gloo::timers::callback::{Interval, Timeout};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::components::loading::Loading;
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::card::{Card, CardContent, CardHeader, CardTitle};
use crate::hooks::use_trans::use_trans;
use crate::icons::{History, PlayCircle, RefreshCw};
use crate::pages::execution::detail::{
    parse_cast_frames, CastFrame, ExecutionDetailClient, ExecutionDetailOutput, ExecutionDetailView,
};
use crate::routes::Route;
use crate::services::{api, command};
use crate::types::Client;
use common::command::CommandLogLine;
use common::entity::execution::{ExecutionSession, ExecutionType, SessionStatus};

fn history_output_placeholder(
    session: &ExecutionSession,
    has_recording: bool,
    t: &crate::i18n::I18n,
) -> String {
    if matches!(
        session.status,
        SessionStatus::Pending | SessionStatus::Running
    ) {
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
            let ip = client
                .primary_ip
                .clone()
                .unwrap_or(client.ip_address.clone());
            (
                client.hostname.clone(),
                ip.clone(),
                format!("{} ({})", client.hostname, ip),
            )
        })
        .unwrap_or_else(|| (client_id.to_string(), String::new(), client_id.to_string()))
}

fn next_playback_elapsed(current_ms: u64, duration_ms: u64, step_ms: u64) -> u64 {
    current_ms.saturating_add(step_ms).min(duration_ms)
}

fn playback_duration_ms(frames: &[CastFrame]) -> u64 {
    frames
        .last()
        .map(|frame| (frame.elapsed_secs.max(0.0) * 1000.0).ceil() as u64)
        .unwrap_or(0)
}

fn visible_cast_output(frames: &[CastFrame], elapsed_secs: f64, separate_frames: bool) -> String {
    let mut output = String::new();
    for frame in frames
        .iter()
        .take_while(|frame| frame.elapsed_secs <= elapsed_secs)
    {
        output.push_str(&frame.output);
        if separate_frames && !frame.output.ends_with('\r') && !frame.output.ends_with('\n') {
            output.push('\n');
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::{next_playback_elapsed, playback_duration_ms, visible_cast_output};
    use crate::pages::execution::detail::CastFrame;

    #[test]
    fn playback_progress_advances_from_the_latest_position() {
        assert_eq!(next_playback_elapsed(0, 1_000, 50), 50);
        assert_eq!(next_playback_elapsed(50, 1_000, 50), 100);
        assert_eq!(next_playback_elapsed(975, 1_000, 50), 1_000);
    }

    #[test]
    fn batch_replay_restores_line_boundaries_between_legacy_frames() {
        let frames = vec![
            CastFrame {
                elapsed_secs: 0.1,
                output: "first".into(),
            },
            CastFrame {
                elapsed_secs: 0.2,
                output: "second\n".into(),
            },
        ];

        assert_eq!(visible_cast_output(&frames, 1.0, true), "first\nsecond\n");
        assert_eq!(visible_cast_output(&frames, 1.0, false), "firstsecond\n");
    }

    #[test]
    fn playback_duration_rounds_up_so_fractional_tail_frames_are_visible() {
        let frames = vec![CastFrame {
            elapsed_secs: 0.044_614,
            output: "tail".into(),
        }];

        let duration_ms = playback_duration_ms(&frames);
        assert_eq!(duration_ms, 45);
        assert_eq!(
            visible_cast_output(&frames, duration_ms as f64 / 1000.0, false),
            "tail"
        );
    }
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
    let cast_frames = use_state(Vec::<CastFrame>::new);
    let playback_elapsed_ms = use_state(|| 0u64);
    let is_playing = use_state(|| false);
    let command_logs = use_state(Vec::<CommandLogLine>::new);
    let cast_error = use_state(|| Option::<String>::None);
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
        let cast_frames = cast_frames.clone();
        let playback_elapsed_ms = playback_elapsed_ms.clone();
        let is_playing = is_playing.clone();
        let command_logs = command_logs.clone();
        let cast_error = cast_error.clone();
        let loading = loading.clone();
        let error = error.clone();
        let t = t.clone();
        use_effect_with((), move |_| {
            let session_id = session_id.clone();
            let session = session.clone();
            let cast_frames = cast_frames.clone();
            let playback_elapsed_ms = playback_elapsed_ms.clone();
            let is_playing = is_playing.clone();
            let command_logs = command_logs.clone();
            let cast_error = cast_error.clone();
            let loading = loading.clone();
            let error = error.clone();
            let t = t.clone();
            spawn_local(async move {
                match command::fetch_session(&session_id).await {
                    Ok(s) => {
                        let has_recording = s.cast_file_path.is_some();
                        session.set(Some(s));

                        if has_recording {
                            match command::fetch_session_cast(&session_id).await {
                                Ok(bytes) => {
                                    let frames = parse_cast_frames(&bytes);
                                    playback_elapsed_ms.set(0);
                                    is_playing.set(!frames.is_empty());
                                    cast_frames.set(frames);
                                }
                                Err(e) => {
                                    cast_error.set(Some(format!(
                                        "{}: {}",
                                        t.t("execution.replay.load_cast_error"),
                                        e.message
                                    )));
                                }
                            }
                        }

                        if let Ok(logs) = command::fetch_command_logs(&session_id).await {
                            command_logs.set(logs);
                        }
                    }
                    Err(e) => {
                        error.set(Some(format!(
                            "{}: {}",
                            t.t("execution.replay.load_session_error"),
                            e.message
                        )));
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
        let cast_frames = cast_frames.clone();
        let playback_elapsed_ms = playback_elapsed_ms.clone();
        let is_playing = is_playing.clone();
        use_effect_with(
            (*is_playing, *playback_elapsed_ms, cast_frames.len()),
            move |_| {
                if !*is_playing || cast_frames.is_empty() {
                    return Box::new(|| ()) as Box<dyn FnOnce()>;
                }
                let duration_ms = playback_duration_ms(cast_frames.as_ref());
                let playback_elapsed_ms = playback_elapsed_ms.clone();
                let is_playing = is_playing.clone();
                let current_ms = *playback_elapsed_ms;
                let timeout = Timeout::new(50, move || {
                    let next = next_playback_elapsed(current_ms, duration_ms, 50);
                    playback_elapsed_ms.set(next);
                    if next >= duration_ms {
                        is_playing.set(false);
                    }
                });
                Box::new(move || drop(timeout)) as Box<dyn FnOnce()>
            },
        );
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
                    let (title, subtitle, tooltip) =
                        resolve_client_parts(client_id, clients.as_ref());
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
        if !cast_frames.is_empty() {
            let elapsed_secs = *playback_elapsed_ms as f64 / 1000.0;
            let output = visible_cast_output(
                cast_frames.as_ref(),
                elapsed_secs,
                sess.execution_type == ExecutionType::Batch,
            );
            ExecutionDetailOutput::Replay(output)
        } else if !command_logs.is_empty() {
            ExecutionDetailOutput::Logs((*command_logs).clone())
        } else if let Some(message) = (*cast_error).clone() {
            ExecutionDetailOutput::Placeholder(message)
        } else {
            ExecutionDetailOutput::Placeholder(history_output_placeholder(
                sess,
                has_recording,
                t.as_ref(),
            ))
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
                        if !cast_frames.is_empty() {
                            <Button
                                variant={ButtonVariant::Outline}
                                size={ButtonSize::Sm}
                                onclick={
                                    let is_playing = is_playing.clone();
                                    Callback::from(move |_| is_playing.set(!*is_playing))
                                }
                            >
                                <PlayCircle class="mr-1 h-4 w-4" />
                                {if *is_playing { t.t("execution.replay.pause") } else { t.t("execution.replay.play") }}
                            </Button>
                            <Button
                                variant={ButtonVariant::Ghost}
                                size={ButtonSize::Sm}
                                title={Some(t.t("execution.replay.restart"))}
                                onclick={
                                    let playback_elapsed_ms = playback_elapsed_ms.clone();
                                    let is_playing = is_playing.clone();
                                    Callback::from(move |_| {
                                        playback_elapsed_ms.set(0);
                                        is_playing.set(true);
                                    })
                                }
                            >
                                <RefreshCw class="h-4 w-4" />
                            </Button>
                        }
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
                            />
                        }
                    }
                </CardContent>
            </Card>
        </div>
    }
}
