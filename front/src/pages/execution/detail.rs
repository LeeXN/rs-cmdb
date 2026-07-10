use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::hooks::use_trans::use_trans;
use crate::i18n::I18n;
use crate::utils::format::format_datetime_with_ago;
use common::command::{CommandLogLine, LogStream};
use common::entity::execution::{ExecutionSession, ExecutionType, SessionStatus};
use yew::prelude::*;

pub fn session_status_badge(status: &SessionStatus, t: &I18n) -> Html {
    let (variant, key) = match status {
        SessionStatus::Pending => (BadgeVariant::Warning, "execution.status.pending"),
        SessionStatus::Running => (BadgeVariant::Info, "execution.status.running"),
        SessionStatus::Success => (BadgeVariant::Success, "execution.status.success"),
        SessionStatus::Failed => (BadgeVariant::Destructive, "execution.status.failed"),
        SessionStatus::Partial => (BadgeVariant::Warning, "execution.status.partial"),
    };
    html! { <Badge variant={variant}>{t.t(key)}</Badge> }
}

pub fn execution_type_badge(kind: &ExecutionType, t: &I18n) -> Html {
    match kind {
        ExecutionType::Terminal => {
            html! { <Badge variant={BadgeVariant::Info}>{t.t("execution.type.terminal")}</Badge> }
        }
        ExecutionType::Batch => {
            html! { <Badge variant={BadgeVariant::Secondary}>{t.t("execution.type.batch")}</Badge> }
        }
    }
}

pub fn strip_ansi_and_controls(input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let mut output = String::new();
    let mut index = 0;
    let mut last_was_newline = false;

    while index < chars.len() {
        let ch = chars[index];
        if ch == '\u{1b}' {
            if let Some(next) = chars.get(index + 1) {
                match *next {
                    '[' => {
                        index += 2;
                        while index < chars.len() {
                            let marker = chars[index];
                            if ('@'..='~').contains(&marker) {
                                index += 1;
                                break;
                            }
                            index += 1;
                        }
                        continue;
                    }
                    ']' => {
                        index += 2;
                        while index < chars.len() {
                            let marker = chars[index];
                            if marker == '\u{7}' {
                                index += 1;
                                break;
                            }
                            if marker == '\u{1b}' && chars.get(index + 1) == Some(&'\\') {
                                index += 2;
                                break;
                            }
                            index += 1;
                        }
                        continue;
                    }
                    _ => {
                        index += 2;
                        continue;
                    }
                }
            }
            index += 1;
            continue;
        }

        match ch {
            '\u{08}' => {
                output.pop();
                last_was_newline = output.ends_with('\n');
            }
            '\r' | '\n' => {
                if !last_was_newline {
                    output.push('\n');
                    last_was_newline = true;
                }
            }
            '\t' => {
                output.push(ch);
                last_was_newline = false;
            }
            c if c.is_control() => {}
            _ => {
                output.push(ch);
                last_was_newline = false;
            }
        }

        index += 1;
    }

    output
}

pub fn parse_cast_output(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes);
    let mut output = String::new();

    for (index, line) in text.lines().enumerate() {
        if index == 0 {
            continue;
        }
        let Ok(value) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        let Some(items) = value.as_array() else {
            continue;
        };
        if items.len() < 3 {
            continue;
        }
        if items.get(1).and_then(|item| item.as_str()) != Some("o") {
            continue;
        }
        if let Some(chunk) = items.get(2).and_then(|item| item.as_str()) {
            let normalized = chunk.replace("\r\n", "\n");
            output.push_str(&strip_ansi_and_controls(&normalized));
        }
    }

    output
}

fn log_stream_badge(stream: &LogStream) -> Html {
    let (variant, label) = match stream {
        LogStream::Stdout => (BadgeVariant::Success, "stdout"),
        LogStream::Stderr => (BadgeVariant::Destructive, "stderr"),
    };
    html! { <Badge variant={variant}>{label}</Badge> }
}

#[derive(Clone, PartialEq)]
pub struct ExecutionDetailClient {
    pub title: String,
    pub subtitle: String,
    pub tooltip: String,
}

#[derive(Clone, PartialEq)]
pub enum ExecutionDetailOutput {
    Replay(String),
    Logs(Vec<CommandLogLine>),
    Placeholder(String),
}

#[derive(Properties, PartialEq)]
pub struct ExecutionDetailViewProps {
    pub session: ExecutionSession,
    pub user_label: String,
    pub clients: Vec<ExecutionDetailClient>,
    pub output: ExecutionDetailOutput,
    pub output_title: String,
    pub output_description: String,
    #[prop_or_default]
    pub output_action: Html,
}

#[function_component(ExecutionDetailView)]
pub fn execution_detail_view(props: &ExecutionDetailViewProps) -> Html {
    let t = use_trans();
    let session = &props.session;

    let output_body = match &props.output {
        ExecutionDetailOutput::Replay(text) => {
            html! {
                <div class="min-h-[520px] rounded-lg border border-white/10 bg-black/90 p-4 font-mono text-xs text-green-300 overflow-y-auto">
                    <pre class="whitespace-pre-wrap break-all">{text.clone()}</pre>
                </div>
            }
        }
        ExecutionDetailOutput::Logs(logs) => {
            html! {
                <div class="max-h-[32rem] min-h-[520px] space-y-2 overflow-auto rounded-lg border border-white/10 bg-black/90 p-4 font-mono text-xs">
                    {for logs.iter().map(|log| html! {
                        <div class="space-y-1 border-b border-white/5 pb-2 last:border-b-0 last:pb-0">
                            <div class="flex flex-wrap items-center gap-2 text-[11px] text-muted-foreground">
                                {log_stream_badge(&log.stream)}
                                <span>{format_datetime_with_ago(&log.timestamp)}</span>
                                <span>{format!("#{}", log.seq)}</span>
                            </div>
                            <pre class={classes!("whitespace-pre-wrap", "break-all", match log.stream {
                                LogStream::Stdout => "text-green-300",
                                LogStream::Stderr => "text-amber-300",
                            })}>{log.line.clone()}</pre>
                        </div>
                    })}
                </div>
            }
        }
        ExecutionDetailOutput::Placeholder(message) => {
            html! {
                <div class="flex min-h-[520px] items-center justify-center rounded-lg border border-dashed border-white/10 bg-black/80 p-6 text-center text-sm text-muted-foreground">
                    {message.clone()}
                </div>
            }
        }
    };

    html! {
        <div class="grid gap-5 xl:grid-cols-[1.35fr_0.95fr]">
            <div class="space-y-4">
                <div class="rounded-xl border border-border/70 bg-muted/20 p-4 shadow-sm">
                    <div class="mb-3 flex items-center justify-between gap-3">
                        <div>
                            <div class="text-xs uppercase tracking-wide text-muted-foreground">{t.t("execution.replay.command")}</div>
                            <div class="mt-1 text-sm font-medium text-foreground">{t.t("execution.history.details_title")}</div>
                        </div>
                        <div class="flex flex-wrap items-center gap-2">
                            {execution_type_badge(&session.execution_type, t.as_ref())}
                            {session_status_badge(&session.status, t.as_ref())}
                        </div>
                    </div>
                    <pre class="overflow-x-auto whitespace-pre-wrap rounded-lg border border-border/70 bg-black/80 p-4 font-mono text-xs text-green-300">{session.command.clone()}</pre>
                </div>

                <div class="rounded-xl border border-border/70 bg-black/95 p-4 shadow-sm">
                    <div class="mb-3 flex items-start justify-between gap-3">
                        <div>
                            <div class="text-sm font-medium text-foreground">{props.output_title.clone()}</div>
                            <p class="text-sm text-muted-foreground">{props.output_description.clone()}</p>
                        </div>
                        {props.output_action.clone()}
                    </div>
                    {output_body}
                </div>
            </div>

            <div class="grid gap-4 text-sm content-start">
                <div class="rounded-xl border border-border/70 bg-muted/20 p-4 shadow-sm">
                    <div class="text-xs uppercase tracking-wide text-muted-foreground">{t.t("execution.history.status_summary")}</div>
                    <div class="mt-3 space-y-3 text-muted-foreground">
                        <div>
                            <div class="text-xs uppercase tracking-wide text-muted-foreground">{t.t("execution.history.table.user")}</div>
                            <div class="mt-1 text-sm text-foreground">{props.user_label.clone()}</div>
                        </div>
                        <div>
                            <div class="text-xs uppercase tracking-wide text-muted-foreground">{t.t("execution.history.started")}</div>
                            <div class="mt-1 text-sm text-foreground">{format_datetime_with_ago(&session.start_time)}</div>
                        </div>
                        <div>
                            <div class="text-xs uppercase tracking-wide text-muted-foreground">{t.t("execution.history.ended")}</div>
                            <div class="mt-1 text-sm text-foreground">{session.end_time.as_ref().map(|value| format_datetime_with_ago(value)).unwrap_or_else(|| "-".to_string())}</div>
                        </div>
                        <div>
                            <div class="text-xs uppercase tracking-wide text-muted-foreground">{t.t("execution.history.table.duration")}</div>
                            <div class="mt-1 text-sm text-foreground">{session.duration_secs.map(|secs| format!("{}s", secs)).unwrap_or_else(|| "-".to_string())}</div>
                        </div>
                        <div>
                            <div class="text-xs uppercase tracking-wide text-muted-foreground">{"Session ID"}</div>
                            <div class="mt-1 break-all font-mono text-xs text-foreground">{session.session_id.clone()}</div>
                        </div>
                        <div>
                            <div class="text-xs uppercase tracking-wide text-muted-foreground">{t.t("execution.history.recording")}</div>
                            <div class="mt-1 text-sm text-foreground">{if session.cast_file_path.is_some() { t.t("execution.history.recording_available") } else { t.t("execution.history.recording_missing") }}</div>
                        </div>
                    </div>
                </div>

                <div class="rounded-xl border border-border/70 bg-muted/20 p-4 shadow-sm">
                    <div class="mb-3 flex items-center justify-between gap-3">
                        <div class="text-xs uppercase tracking-wide text-muted-foreground">{t.t("execution.history.table.targets")}</div>
                        <Badge variant={BadgeVariant::Outline}>{props.clients.len().to_string()}</Badge>
                    </div>
                    <div class="grid gap-2">
                        {for props.clients.iter().map(|client| html! {
                            <div class="rounded-lg border border-border/70 bg-background/60 px-3 py-2" title={client.tooltip.clone()}>
                                <div class="text-sm font-medium text-foreground">{client.title.clone()}</div>
                                if !client.subtitle.is_empty() {
                                    <div class="text-xs text-muted-foreground">{client.subtitle.clone()}</div>
                                }
                            </div>
                        })}
                    </div>
                </div>
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::strip_ansi_and_controls;

    #[test]
    fn strip_ansi_and_controls_removes_escape_sequences() {
        assert_eq!(
            strip_ansi_and_controls("\u{1b}[31merror\u{1b}[0m\r\nnext"),
            "error\nnext"
        );
    }

    #[test]
    fn strip_ansi_and_controls_applies_backspace() {
        assert_eq!(strip_ansi_and_controls("ab\u{08}c"), "ac");
    }
}
