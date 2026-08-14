use std::collections::HashMap;

use js_sys::{Array, Function, Object, Reflect};
use serde_json::json;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use wasm_bindgen_futures::spawn_local;
use web_sys::window;
use web_sys::{CloseEvent, Event, HtmlElement, MessageEvent, WebSocket};
use yew::prelude::*;
use yew_router::prelude::*;

use crate::components::loading::Loading;
use crate::components::notification::{Notification, NotificationType};
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::card::{Card, CardContent, CardHeader, CardTitle};
use crate::components::ui::input::Input;
use crate::hooks::use_trans::use_trans;
use crate::i18n::I18n;
use crate::icons::{History, LoaderCircle, RefreshCw, Server, Terminal};
use crate::routes::Route;
use crate::services::api;
use crate::services::command;
use crate::types::{
    Client, CreateTerminalSessionRequest, ResizeTerminalRequest, TerminalSessionState,
    TerminalSessionSummary,
};
use crate::utils::format::format_datetime_with_ago;
use common::entity::permission::TerminalMode;

const TERMINAL_STATUS_IDLE: &str = "idle";
const TERMINAL_STATUS_CONNECTING: &str = "connecting";
const TERMINAL_STATUS_CONNECTED: &str = "connected";
const TERMINAL_STATUS_DISCONNECTED: &str = "disconnected";
const INPUT_FLUSH_DELAY_MS: i32 = 40;

struct TerminalBindings {
    on_data: Option<Closure<dyn FnMut(String)>>,
    on_message: Option<Closure<dyn FnMut(MessageEvent)>>,
    on_open: Option<Closure<dyn FnMut(Event)>>,
    on_close: Option<Closure<dyn FnMut(CloseEvent)>>,
    on_error: Option<Closure<dyn FnMut(Event)>>,
}

impl Default for TerminalBindings {
    fn default() -> Self {
        Self {
            on_data: None,
            on_message: None,
            on_open: None,
            on_close: None,
            on_error: None,
        }
    }
}

impl TerminalBindings {
    fn clear(&mut self) {
        self.on_data = None;
        self.on_message = None;
        self.on_open = None;
        self.on_close = None;
        self.on_error = None;
    }
}

fn terminal_state_badge(state: &TerminalSessionState, t: &I18n) -> Html {
    let (variant, label) = match state {
        TerminalSessionState::Pending => (BadgeVariant::Warning, t.t("execution.status.pending")),
        TerminalSessionState::Active => (BadgeVariant::Info, t.t("execution.status.running")),
        TerminalSessionState::Closed => (
            BadgeVariant::Secondary,
            t.t("execution.terminal.state_closed"),
        ),
        TerminalSessionState::Failed => (BadgeVariant::Destructive, t.t("execution.status.failed")),
    };
    html! { <Badge variant={variant}>{label}</Badge> }
}

fn terminal_mode_badge(mode: &TerminalMode, t: &I18n) -> Html {
    let (variant, key) = match mode {
        TerminalMode::ReadOnly => (BadgeVariant::Warning, "execution.terminal.mode_restricted"),
        TerminalMode::ReadWrite => (BadgeVariant::Success, "execution.terminal.mode_standard"),
    };
    html! { <Badge variant={variant}>{t.t(key)}</Badge> }
}

fn format_optional_datetime(value: Option<&String>) -> String {
    value
        .map(|value| format_datetime_with_ago(value))
        .unwrap_or_else(|| "-".to_string())
}

fn preloaded_terminal_command(query: &HashMap<String, String>) -> Option<String> {
    query.get("command").cloned()
}

fn js_error(err: JsValue, fallback: &str) -> String {
    err.as_string().unwrap_or_else(|| fallback.to_string())
}

fn js_method(target: &JsValue, name: &str) -> Result<Function, String> {
    Reflect::get(target, &JsValue::from_str(name))
        .map_err(|err| js_error(err, "missing js method"))?
        .dyn_into::<Function>()
        .map_err(|_| format!("{} is not a function", name))
}

fn js_call0(target: &JsValue, name: &str) -> Result<JsValue, String> {
    js_method(target, name)?
        .call0(target)
        .map_err(|err| js_error(err, "js call failed"))
}

fn js_call1(target: &JsValue, name: &str, arg: &JsValue) -> Result<JsValue, String> {
    js_method(target, name)?
        .call1(target, arg)
        .map_err(|err| js_error(err, "js call failed"))
}

fn create_terminal_runtime(
    host: &HtmlElement,
    cols: u16,
    rows: u16,
) -> Result<(JsValue, Option<JsValue>), String> {
    let global = js_sys::global();
    let terminal_ctor = Reflect::get(&global, &JsValue::from_str("Terminal"))
        .map_err(|err| js_error(err, "xterm.js not loaded"))?
        .dyn_into::<Function>()
        .map_err(|_| "Terminal constructor unavailable".to_string())?;

    let options = Object::new();
    Reflect::set(&options, &JsValue::from_str("cursorBlink"), &JsValue::TRUE)
        .map_err(|err| js_error(err, "failed to set terminal option"))?;
    Reflect::set(&options, &JsValue::from_str("convertEol"), &JsValue::TRUE)
        .map_err(|err| js_error(err, "failed to set terminal option"))?;
    Reflect::set(
        &options,
        &JsValue::from_str("cols"),
        &JsValue::from_f64(cols as f64),
    )
    .map_err(|err| js_error(err, "failed to set terminal option"))?;
    Reflect::set(
        &options,
        &JsValue::from_str("rows"),
        &JsValue::from_f64(rows as f64),
    )
    .map_err(|err| js_error(err, "failed to set terminal option"))?;
    Reflect::set(
        &options,
        &JsValue::from_str("fontFamily"),
        &JsValue::from_str("JetBrains Mono, monospace"),
    )
    .map_err(|err| js_error(err, "failed to set terminal option"))?;
    let theme = js_sys::Object::from_entries(&Array::of2(
        &Array::of2(
            &JsValue::from_str("background"),
            &JsValue::from_str("#020617"),
        ),
        &Array::of2(
            &JsValue::from_str("foreground"),
            &JsValue::from_str("#86efac"),
        ),
    ))
    .map_err(|err| js_error(err, "failed to set terminal theme"))?;
    Reflect::set(&options, &JsValue::from_str("theme"), &JsValue::from(theme))
        .map_err(|err| js_error(err, "failed to set terminal option"))?;

    let terminal = Reflect::construct(&terminal_ctor, &Array::of1(&options.into()))
        .map_err(|err| js_error(err, "failed to create terminal"))?;
    js_call1(&terminal, "open", host.as_ref())?;

    let fit_addon = Reflect::get(&global, &JsValue::from_str("FitAddon"))
        .ok()
        .and_then(|namespace| Reflect::get(&namespace, &JsValue::from_str("FitAddon")).ok())
        .and_then(|value| value.dyn_into::<Function>().ok())
        .and_then(|ctor| Reflect::construct(&ctor, &Array::new()).ok());

    if let Some(addon) = fit_addon.as_ref() {
        let _ = js_call1(&terminal, "loadAddon", addon);
        let _ = js_call0(addon, "fit");
    }

    let _ = js_call0(&terminal, "focus");
    Ok((terminal, fit_addon))
}

fn dispose_terminal(terminal: &JsValue) {
    let _ = js_call0(terminal, "dispose");
}

fn clear_terminal(terminal: &JsValue) {
    let _ = js_call0(terminal, "clear");
}

fn write_terminal(terminal: &JsValue, data: &str) {
    let _ = js_call1(terminal, "write", &JsValue::from_str(data));
}

fn fit_terminal(terminal: &JsValue, fit_addon: Option<&JsValue>) -> Option<(u16, u16)> {
    if let Some(addon) = fit_addon {
        let _ = js_call0(addon, "fit");
    }
    let cols = Reflect::get(terminal, &JsValue::from_str("cols"))
        .ok()?
        .as_f64()? as u16;
    let rows = Reflect::get(terminal, &JsValue::from_str("rows"))
        .ok()?
        .as_f64()? as u16;
    Some((cols, rows))
}

#[cfg(test)]
mod tests {
    use super::preloaded_terminal_command;
    use std::collections::HashMap;

    #[test]
    fn preloaded_terminal_command_preserves_exact_query_value() {
        let mut query = HashMap::new();
        query.insert("command".to_string(), "echo hi && pwd\nls -l".to_string());

        assert_eq!(
            preloaded_terminal_command(&query),
            Some("echo hi && pwd\nls -l".to_string())
        );
    }
}

#[derive(Properties, PartialEq)]
pub struct TerminalPageProps {
    pub client_id: String,
}

#[function_component(TerminalPage)]
pub fn terminal_page(props: &TerminalPageProps) -> Html {
    let t = use_trans();
    let navigator = use_navigator();
    let location = use_location();

    let client = use_state(|| None::<Client>);
    let client_loading = use_state(|| true);
    let config_enabled = use_state(|| false);
    let config_loading = use_state(|| true);
    let cols = use_state(|| 120u16);
    let rows = use_state(|| 32u16);
    let current_session = use_state(|| None::<TerminalSessionSummary>);
    let opening_session = use_state(|| false);
    let terminal_bootstrapped = use_state(|| false);
    let notification = use_state(|| None::<(NotificationType, String)>);
    let terminal_nonce = use_state(|| 0u64);
    let terminal_status = use_state(|| TERMINAL_STATUS_IDLE.to_string());
    let pending_preload = use_state(|| {
        location
            .as_ref()
            .and_then(|loc| loc.query::<HashMap<String, String>>().ok())
            .and_then(|query| preloaded_terminal_command(&query))
    });

    let terminal_host = use_node_ref();
    let terminal_ref = use_mut_ref(|| None::<JsValue>);
    let fit_addon_ref = use_mut_ref(|| None::<JsValue>);
    let websocket_ref = use_mut_ref(|| None::<WebSocket>);
    let bindings_ref = use_mut_ref(TerminalBindings::default);
    let pending_input_ref = use_mut_ref(String::new);
    let flush_timeout_ref = use_mut_ref(|| None::<i32>);

    {
        let client = client.clone();
        let client_loading = client_loading.clone();
        let client_id = props.client_id.clone();
        use_effect_with(props.client_id.clone(), move |_| {
            client_loading.set(true);
            spawn_local(async move {
                match api::fetch_client(&client_id).await {
                    Ok(data) => client.set(Some(data)),
                    Err(_) => client.set(None),
                }
                client_loading.set(false);
            });
            || ()
        });
    }

    {
        let config_enabled = config_enabled.clone();
        let config_loading = config_loading.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                if let Ok(cfg) = command::fetch_remote_exec_config().await {
                    config_enabled.set(cfg.enabled);
                }
                config_loading.set(false);
            });
            || ()
        });
    }

    {
        let terminal_bootstrapped = terminal_bootstrapped.clone();
        let current_session = current_session.clone();
        let notification = notification.clone();
        use_effect_with(props.client_id.clone(), move |_| {
            terminal_bootstrapped.set(false);
            current_session.set(None);
            notification.set(None);
            || ()
        });
    }

    {
        let current_session = current_session.clone();
        let terminal_bootstrapped = terminal_bootstrapped.clone();
        let opening_session = opening_session.clone();
        let notification = notification.clone();
        let client_id = props.client_id.clone();
        let cols = cols.clone();
        let rows = rows.clone();
        let terminal_nonce = terminal_nonce.clone();
        let t = t.clone();
        let config_enabled = config_enabled.clone();
        let config_loading = config_loading.clone();

        use_effect_with(
            (
                props.client_id.clone(),
                (*config_enabled),
                (*config_loading),
                (*opening_session),
                (*terminal_bootstrapped),
                (*current_session)
                    .as_ref()
                    .map(|session| session.session_id.clone()),
            ),
            move |_| -> Box<dyn FnOnce()> {
                if *config_loading || *opening_session || *terminal_bootstrapped {
                    return Box::new(|| ());
                }

                if !*config_enabled {
                    terminal_bootstrapped.set(true);
                    return Box::new(|| ());
                }

                opening_session.set(true);
                let current_session = current_session.clone();
                let terminal_bootstrapped = terminal_bootstrapped.clone();
                let opening_session = opening_session.clone();
                let notification = notification.clone();
                let client_id = client_id.clone();
                let cols_value = *cols;
                let rows_value = *rows;
                let terminal_nonce = terminal_nonce.clone();
                let t = t.clone();
                spawn_local(async move {
                    match command::create_terminal_session(&CreateTerminalSessionRequest {
                        client_id: client_id.clone(),
                        shell: Some("/bin/bash".to_string()),
                        cols: Some(cols_value),
                        rows: Some(rows_value),
                    })
                    .await
                    {
                        Ok(resp) => {
                            cols.set(resp.session.cols);
                            rows.set(resp.session.rows);
                            current_session.set(Some(resp.session.clone()));
                            terminal_nonce.set(*terminal_nonce + 1);
                            notification.set(Some((
                                NotificationType::Success,
                                t.t("execution.terminal.notifications.opened"),
                            )));
                        }
                        Err(err) => {
                            notification.set(Some((NotificationType::Error, err.message)));
                        }
                    }
                    opening_session.set(false);
                    terminal_bootstrapped.set(true);
                });

                Box::new(|| ())
            },
        );
    }

    {
        let current_session = current_session.clone();
        let notification = notification.clone();
        let cols_value = *cols;
        let rows_value = *rows;
        use_effect_with(
            (
                (*current_session)
                    .as_ref()
                    .map(|session| session.session_id.clone()),
                cols_value,
                rows_value,
            ),
            move |(session_id, cols_value, rows_value)| {
                if let Some(session_id) = session_id.clone() {
                    let current_session = current_session.clone();
                    let notification = notification.clone();
                    let session_snapshot = (*current_session).clone();
                    let cols_value = *cols_value;
                    let rows_value = *rows_value;
                    if let Some(session) = session_snapshot {
                        if session.cols != cols_value || session.rows != rows_value {
                            spawn_local(async move {
                                if let Err(err) = command::resize_terminal_session(
                                    &session_id,
                                    &ResizeTerminalRequest {
                                        cols: cols_value,
                                        rows: rows_value,
                                    },
                                )
                                .await
                                {
                                    notification.set(Some((NotificationType::Error, err.message)));
                                    return;
                                }
                                if let Ok(updated) =
                                    command::fetch_terminal_session(&session_id).await
                                {
                                    current_session.set(Some(updated));
                                }
                            });
                        }
                    }
                }
                || ()
            },
        );
    }

    {
        let session_key = (*current_session)
            .as_ref()
            .map(|session| session.session_id.clone());
        use_effect_with(session_key, move |session_key| {
            let Some(session_id) = session_key.clone() else {
                return Box::new(|| ()) as Box<dyn FnOnce()>;
            };

            Box::new(move || {
                spawn_local(async move {
                    let _ = command::close_terminal_session(&session_id).await;
                });
            }) as Box<dyn FnOnce()>
        });
    }

    {
        let session_key = (*current_session)
            .as_ref()
            .map(|session| session.session_id.clone());
        let terminal_nonce = *terminal_nonce;
        let terminal_host = terminal_host.clone();
        let terminal_ref = terminal_ref.clone();
        let fit_addon_ref = fit_addon_ref.clone();
        let websocket_ref = websocket_ref.clone();
        let bindings_ref = bindings_ref.clone();
        let pending_input_ref = pending_input_ref.clone();
        let flush_timeout_ref = flush_timeout_ref.clone();
        let notification = notification.clone();
        let terminal_status = terminal_status.clone();
        let t = t.clone();
        let cols = cols.clone();
        let rows = rows.clone();
        let pending_preload = pending_preload.clone();

        use_effect_with((session_key, terminal_nonce), move |(session_key, _)| {
            if let Some(ws) = websocket_ref.borrow_mut().take() {
                let _ = ws.close();
            }
            if let Some(timeout_id) = flush_timeout_ref.borrow_mut().take() {
                if let Some(win) = window() {
                    win.clear_timeout_with_handle(timeout_id);
                }
            }
            pending_input_ref.borrow_mut().clear();
            bindings_ref.borrow_mut().clear();
            if let Some(terminal) = terminal_ref.borrow_mut().take() {
                dispose_terminal(&terminal);
            }
            fit_addon_ref.borrow_mut().take();

            let Some(session_id) = session_key.clone() else {
                terminal_status.set(TERMINAL_STATUS_IDLE.to_string());
                return Box::new(|| ()) as Box<dyn FnOnce()>;
            };

            let Some(host) = terminal_host.cast::<HtmlElement>() else {
                terminal_status.set(TERMINAL_STATUS_DISCONNECTED.to_string());
                return Box::new(|| ()) as Box<dyn FnOnce()>;
            };

            terminal_status.set(TERMINAL_STATUS_CONNECTING.to_string());
            let initial_cols = *cols;
            let initial_rows = *rows;

            let (terminal, fit_addon) =
                match create_terminal_runtime(&host, initial_cols, initial_rows) {
                    Ok(runtime) => runtime,
                    Err(message) => {
                        notification.set(Some((NotificationType::Error, message)));
                        terminal_status.set(TERMINAL_STATUS_DISCONNECTED.to_string());
                        return Box::new(|| ()) as Box<dyn FnOnce()>;
                    }
                };
            clear_terminal(&terminal);

            let ws_url = match command::terminal_ws_url(&session_id) {
                Ok(url) => url,
                Err(err) => {
                    notification.set(Some((NotificationType::Error, err.message)));
                    terminal_status.set(TERMINAL_STATUS_DISCONNECTED.to_string());
                    return Box::new(|| ()) as Box<dyn FnOnce()>;
                }
            };

            let websocket = match WebSocket::new(&ws_url) {
                Ok(ws) => ws,
                Err(err) => {
                    notification.set(Some((
                        NotificationType::Error,
                        js_error(err, "failed to open websocket"),
                    )));
                    terminal_status.set(TERMINAL_STATUS_DISCONNECTED.to_string());
                    return Box::new(|| ()) as Box<dyn FnOnce()>;
                }
            };

            let terminal_for_input = terminal.clone();
            let websocket_for_input = websocket.clone();
            let pending_input_for_input = pending_input_ref.clone();
            let flush_timeout_for_input = flush_timeout_ref.clone();
            let on_data = Closure::<dyn FnMut(String)>::wrap(Box::new(move |data: String| {
                pending_input_for_input.borrow_mut().push_str(&data);
                let flush_immediately =
                    data.contains('\n') || data.contains('\r') || data.contains('\t');

                let do_flush = {
                    let pending_input_for_input = pending_input_for_input.clone();
                    let websocket_for_input = websocket_for_input.clone();
                    let flush_timeout_for_input = flush_timeout_for_input.clone();
                    move || {
                        flush_timeout_for_input.borrow_mut().take();
                        let payload = std::mem::take(&mut *pending_input_for_input.borrow_mut());
                        if !payload.is_empty() {
                            let _ = websocket_for_input.send_with_str(&payload);
                        }
                    }
                };

                if flush_immediately {
                    if let Some(timeout_id) = flush_timeout_for_input.borrow_mut().take() {
                        if let Some(win) = window() {
                            win.clear_timeout_with_handle(timeout_id);
                        }
                    }
                    do_flush();
                } else if flush_timeout_for_input.borrow().is_none() {
                    let callback = Closure::<dyn FnMut()>::once(do_flush);
                    if let Some(win) = window() {
                        if let Ok(timeout_id) = win
                            .set_timeout_with_callback_and_timeout_and_arguments_0(
                                callback.as_ref().unchecked_ref(),
                                INPUT_FLUSH_DELAY_MS,
                            )
                        {
                            *flush_timeout_for_input.borrow_mut() = Some(timeout_id);
                            callback.forget();
                        }
                    }
                }
                let _ = js_call0(&terminal_for_input, "focus");
            }));
            let _ = js_call1(&terminal, "onData", on_data.as_ref().unchecked_ref());

            let terminal_for_message = terminal.clone();
            let on_message =
                Closure::<dyn FnMut(MessageEvent)>::wrap(Box::new(move |event: MessageEvent| {
                    if let Some(text) = event.data().as_string() {
                        write_terminal(&terminal_for_message, &text);
                    }
                }));
            websocket.set_onmessage(Some(on_message.as_ref().unchecked_ref()));

            let fit_for_open = fit_addon.clone();
            let terminal_for_open = terminal.clone();
            let session_for_open = session_id.clone();
            let notification_for_open = notification.clone();
            let terminal_status_for_open = terminal_status.clone();
            let cols_for_open = cols.clone();
            let rows_for_open = rows.clone();
            let pending_preload_for_open = pending_preload.clone();
            let websocket_for_open = websocket.clone();
            let on_open = Closure::<dyn FnMut(Event)>::wrap(Box::new(move |_| {
                terminal_status_for_open.set(TERMINAL_STATUS_CONNECTED.to_string());
                if let Some((actual_cols, actual_rows)) =
                    fit_terminal(&terminal_for_open, fit_for_open.as_ref())
                {
                    cols_for_open.set(actual_cols);
                    rows_for_open.set(actual_rows);
                    let notification = notification_for_open.clone();
                    let session_id = session_for_open.clone();
                    spawn_local(async move {
                        if let Err(err) = command::resize_terminal_session(
                            &session_id,
                            &ResizeTerminalRequest {
                                cols: actual_cols,
                                rows: actual_rows,
                            },
                        )
                        .await
                        {
                            notification.set(Some((NotificationType::Error, err.message)));
                        }
                    });
                }
                if let Some(command) = (*pending_preload_for_open).clone() {
                    let payload = if command.ends_with('\n') {
                        command
                    } else {
                        format!("{}\n", command)
                    };
                    let _ = websocket_for_open.send_with_str(&payload);
                    pending_preload_for_open.set(None);
                }
            }));
            websocket.set_onopen(Some(on_open.as_ref().unchecked_ref()));

            let terminal_status_for_close = terminal_status.clone();
            let on_close = Closure::<dyn FnMut(CloseEvent)>::wrap(Box::new(move |_| {
                terminal_status_for_close.set(TERMINAL_STATUS_DISCONNECTED.to_string());
            }));
            websocket.set_onclose(Some(on_close.as_ref().unchecked_ref()));

            let notification_for_error = notification.clone();
            let terminal_status_for_error = terminal_status.clone();
            let t_for_error = t.clone();
            let on_error = Closure::<dyn FnMut(Event)>::wrap(Box::new(move |_| {
                terminal_status_for_error.set(TERMINAL_STATUS_DISCONNECTED.to_string());
                notification_for_error.set(Some((
                    NotificationType::Error,
                    t_for_error.t("execution.terminal.websocket_error"),
                )));
            }));
            websocket.set_onerror(Some(on_error.as_ref().unchecked_ref()));

            *terminal_ref.borrow_mut() = Some(terminal.clone());
            *fit_addon_ref.borrow_mut() = fit_addon.clone();
            *websocket_ref.borrow_mut() = Some(websocket.clone());
            *bindings_ref.borrow_mut() = TerminalBindings {
                on_data: Some(on_data),
                on_message: Some(on_message),
                on_open: Some(on_open),
                on_close: Some(on_close),
                on_error: Some(on_error),
            };

            Box::new(move || {
                if let Some(ws) = websocket_ref.borrow_mut().take() {
                    let _ = ws.close();
                }
                if let Some(timeout_id) = flush_timeout_ref.borrow_mut().take() {
                    if let Some(win) = window() {
                        win.clear_timeout_with_handle(timeout_id);
                    }
                }
                pending_input_ref.borrow_mut().clear();
                bindings_ref.borrow_mut().clear();
                if let Some(terminal) = terminal_ref.borrow_mut().take() {
                    dispose_terminal(&terminal);
                }
                fit_addon_ref.borrow_mut().take();
            }) as Box<dyn FnOnce()>
        });
    }

    let close_notification = {
        let notification = notification.clone();
        Callback::from(move |_| notification.set(None))
    };

    let on_history = {
        let navigator = navigator.clone();
        let client_id = props.client_id.clone();
        Callback::from(move |_| {
            if let Some(navigator) = &navigator {
                let _ = navigator.push_with_query(
                    &Route::ExecutionHistory,
                    &json!({ "client_id": client_id, "execution_type": "terminal" }),
                );
            }
        })
    };

    let on_refresh_output = {
        let terminal_nonce = terminal_nonce.clone();
        Callback::from(move |_| {
            terminal_nonce.set(*terminal_nonce + 1);
        })
    };

    let current_session_value = (*current_session).clone();

    let terminal_status_text = match terminal_status.as_str() {
        TERMINAL_STATUS_CONNECTING => t.t("execution.terminal.connecting"),
        TERMINAL_STATUS_CONNECTED => t.t("execution.terminal.connected"),
        _ if *opening_session => t.t("execution.terminal.opening_terminal"),
        TERMINAL_STATUS_DISCONNECTED => t.t("execution.terminal.disconnected"),
        _ => t.t("execution.terminal.live_terminal_empty"),
    };

    html! {
        <div class="space-y-6 p-4">
            if let Some((type_, message)) = (*notification).clone() {
                <Notification notification_type={type_} message={message} show={true} on_close={close_notification.clone()} />
            }

            if *client_loading || *config_loading {
                <Loading message={t.t("execution.terminal.loading")} />
            } else if let Some(client) = &*client {
                <div class="space-y-6">
                    <Card>
                        <CardHeader class="gap-4 md:flex-row md:items-start md:justify-between md:space-y-0">
                            <div class="space-y-3">
                                <div class="flex items-center gap-3">
                                    <Terminal class="h-6 w-6 text-primary" />
                                    <div>
                                        <CardTitle class="text-xl">{t.t("execution.terminal.title")}</CardTitle>
                                        <p class="text-sm text-muted-foreground">
                                            {t.t("execution.terminal.description")}
                                        </p>
                                    </div>
                                </div>
                                <div class="flex flex-wrap items-center gap-2">
                                    <Badge variant={BadgeVariant::Outline} class="border-cyan-500/30 bg-cyan-500/10 text-cyan-300">
                                        {client.hostname.clone()}
                                    </Badge>
                                    <Badge variant={BadgeVariant::Secondary}>{client.primary_ip.clone().unwrap_or(client.ip_address.clone())}</Badge>
                                    if *config_enabled {
                                        <Badge variant={BadgeVariant::Success}>{t.t("execution.terminal.enabled")}</Badge>
                                    } else {
                                        <Badge variant={BadgeVariant::Destructive}>{t.t("execution.terminal.disabled")}</Badge>
                                    }
                                    <Badge variant={BadgeVariant::Outline}>{terminal_status_text.clone()}</Badge>
                                </div>
                            </div>
                             <div class="flex flex-wrap items-center gap-2">
                                 <Button variant={ButtonVariant::Outline} size={ButtonSize::Sm} onclick={on_history}>
                                     <History class="mr-2 h-4 w-4" />
                                    {t.t("execution.terminal.view_history")}
                                </Button>
                                <Button
                                    variant={ButtonVariant::Outline}
                                    size={ButtonSize::Sm}
                                    onclick={on_refresh_output}
                                    disabled={current_session_value.is_none()}
                                >
                                     <RefreshCw class="mr-2 h-4 w-4" />
                                     {t.t("execution.terminal.reconnect")}
                                 </Button>
                             </div>
                         </CardHeader>
                         <CardContent>
                             <div class="rounded-lg border border-border/70 bg-muted/20 p-4">
                                 <div class="mb-2 flex items-center gap-2 text-sm font-medium text-foreground">
                                     <Server class="h-4 w-4 text-muted-foreground" />
                                     {t.t("execution.terminal.target_client")}
                                </div>
                                <div class="grid gap-2 text-sm text-muted-foreground md:grid-cols-3">
                                    <div>
                                        <div class="text-xs uppercase tracking-wide">{t.t("execution.terminal.hostname")}</div>
                                        <div class="mt-1 text-foreground">{client.hostname.clone()}</div>
                                    </div>
                                    <div>
                                        <div class="text-xs uppercase tracking-wide">{t.t("execution.terminal.ip")}</div>
                                        <div class="mt-1 text-foreground">{client.primary_ip.clone().unwrap_or(client.ip_address.clone())}</div>
                                    </div>
                                     <div>
                                         <div class="text-xs uppercase tracking-wide">{t.t("execution.terminal.os")}</div>
                                         <div class="mt-1 text-foreground">{client.os.clone().unwrap_or_else(|| "-".to_string())}</div>
                                     </div>
                                 </div>
                                 <p class="mt-3 text-sm text-muted-foreground">
                                     {t.t("execution.terminal.auto_create_hint")}
                                  </p>
                              </div>
                          </CardContent>
                     </Card>

                    <Card>
                        <CardHeader class="gap-3 md:flex-row md:items-center md:justify-between md:space-y-0">
                            <div>
                                <CardTitle class="text-lg">{t.t("execution.terminal.live_terminal")}</CardTitle>
                            </div>
                            <div class="flex flex-wrap items-center gap-2">
                                if let Some(session) = &current_session_value {
                                    {terminal_state_badge(&session.state, t.as_ref())}
                                    {terminal_mode_badge(&session.mode, t.as_ref())}
                                }
                                <Badge variant={BadgeVariant::Outline}>{terminal_status_text.clone()}</Badge>
                            </div>
                        </CardHeader>
                        <CardContent class="space-y-4">
                            if !*config_enabled {
                                <div class="rounded-lg border border-amber-500/30 bg-amber-500/10 p-3 text-sm text-amber-200">
                                    {t.t("execution.terminal.disabled_message")}
                                </div>
                            }
                            if let Some(command) = (*pending_preload).clone() {
                                <div class="rounded-lg border border-primary/20 bg-primary/10 p-3 text-xs text-primary whitespace-pre-wrap break-all">
                                    <div class="mb-1 font-medium">{t.t("execution.terminal.preloaded_command")}</div>
                                    {command}
                                </div>
                            }
                            <div class="grid gap-4 xl:grid-cols-[1.5fr_0.7fr]">
                                <div class="rounded-lg border border-border bg-black/95 shadow-inner overflow-hidden order-2 xl:order-1">
                                    <div class="flex items-center justify-between border-b border-white/10 px-4 py-2 text-xs text-muted-foreground">
                                        <span>{t.t("execution.terminal.live_shell")}</span>
                                        <span>{terminal_status_text.clone()}</span>
                                    </div>
                                    <div class="min-h-[520px] p-2">
                                        if current_session_value.is_none() {
                                            <div class="flex min-h-[500px] items-center justify-center text-sm text-muted-foreground">
                                                {t.t("execution.terminal.live_terminal_empty")}
                                            </div>
                                        } else {
                                            <div ref={terminal_host} class="h-[520px] w-full overflow-hidden rounded-md bg-black/95" />
                                        }
                                    </div>
                                </div>
                                <div class="space-y-4 order-1 xl:order-2">
                                    <div class="rounded-lg border border-border/70 bg-muted/20 p-4 space-y-4">
                                     <div>
                                         <div class="text-sm font-medium text-foreground">{t.t("execution.terminal.console_title")}</div>
                                             <p class="mt-1 text-xs text-muted-foreground">{t.t("execution.terminal.console_new_session_description")}</p>
                                             <p class="mt-2 text-xs text-muted-foreground">{t.t("execution.terminal.mode_hint")}</p>
                                          </div>
                                     <div class="grid gap-3 sm:grid-cols-2 xl:grid-cols-1">
                                         <div class="space-y-2">
                                                <label class="text-sm font-medium text-foreground">{t.t("execution.terminal.cols")}</label>
                                                <Input
                                                    type_="number"
                                                    value={cols.to_string()}
                                                    oninput={{
                                                        let cols = cols.clone();
                                                        Callback::from(move |value: String| {
                                                            if let Ok(parsed) = value.parse::<u16>() {
                                                                cols.set(parsed.max(40));
                                                            }
                                                        })
                                                    }}
                                                    disabled={*opening_session}
                                                />
                                            </div>
                                            <div class="space-y-2">
                                                <label class="text-sm font-medium text-foreground">{t.t("execution.terminal.rows")}</label>
                                                <Input
                                                    type_="number"
                                                    value={rows.to_string()}
                                                    oninput={{
                                                        let rows = rows.clone();
                                                        Callback::from(move |value: String| {
                                                            if let Ok(parsed) = value.parse::<u16>() {
                                                                rows.set(parsed.max(10));
                                                            }
                                                        })
                                                    }}
                                                     disabled={*opening_session}
                                                 />
                                             </div>
                                         </div>
                                         if *opening_session {
                                             <div class="flex items-center gap-2 rounded-lg border border-primary/20 bg-primary/10 px-3 py-2 text-sm text-primary">
                                                 <LoaderCircle class="h-4 w-4 animate-spin" />
                                                 <span>{t.t("execution.terminal.opening_terminal")}</span>
                                             </div>
                                          }
                                          if let Some(session) = &current_session_value {
                                              <div class="rounded-lg border border-border/70 bg-background/60 px-3 py-2 text-xs text-muted-foreground">
                                                  {format!("{} {}", t.t("execution.terminal.session_id"), session.session_id.get(..8).unwrap_or(&session.session_id))}
                                              </div>
                                          }
                                      </div>
                                     <div class="rounded-lg border border-border/70 bg-muted/20 p-4 text-sm">
                                         <div class="mb-2 font-medium text-foreground">{t.t("execution.terminal.current_session")}</div>
                                         if let Some(session) = &current_session_value {
                                            <div class="space-y-3 text-muted-foreground">
                                                <div class="flex flex-wrap items-center gap-2">
                                                    {terminal_state_badge(&session.state, t.as_ref())}
                                                    {terminal_mode_badge(&session.mode, t.as_ref())}
                                                </div>
                                                <div>
                                                    <div class="text-xs uppercase tracking-wide">{t.t("execution.terminal.session_id")}</div>
                                                    <div class="mt-1 font-mono text-xs text-foreground break-all">{session.session_id.clone()}</div>
                                                </div>
                                                <div class="grid gap-2 sm:grid-cols-2 xl:grid-cols-1">
                                                    <div>
                                                        <div class="text-xs uppercase tracking-wide">{t.t("execution.terminal.shell")}</div>
                                                        <div class="mt-1 text-foreground">{session.shell.clone()}</div>
                                                    </div>
                                                    <div>
                                                        <div class="text-xs uppercase tracking-wide">{t.t("execution.terminal.size")}</div>
                                                        <div class="mt-1 text-foreground">{format!("{}x{}", session.cols, session.rows)}</div>
                                                    </div>
                                                 <div>
                                                     <div class="text-xs uppercase tracking-wide">{t.t("execution.terminal.last_activity")}</div>
                                                     <div class="mt-1 text-foreground break-all">{format_optional_datetime(session.last_activity_at.as_ref())}</div>
                                                 </div>
                                                 <div>
                                                     <div class="text-xs uppercase tracking-wide">{t.t("execution.terminal.last_heartbeat")}</div>
                                                     <div class="mt-1 text-foreground break-all">{format_optional_datetime(session.last_heartbeat_at.as_ref())}</div>
                                                 </div>
                                             </div>
                                             <div>
                                                 <div class="text-xs uppercase tracking-wide">{t.t("execution.terminal.created_at")}</div>
                                                 <div class="mt-1 text-foreground break-all">{format_datetime_with_ago(&session.created_at)}</div>
                                             </div>
                                             if let Some(reason) = &session.close_reason {
                                                 <div>
                                                     <div class="text-xs uppercase tracking-wide">{t.t("execution.terminal.close_reason")}</div>
                                                        <div class="mt-1 text-foreground whitespace-pre-wrap break-all">{reason.clone()}</div>
                                                    </div>
                                                }
                                            </div>
                                        } else {
                                            <p class="text-muted-foreground">{t.t("execution.terminal.no_session")}</p>
                                        }
                                    </div>
                                </div>
                            </div>
                        </CardContent>
                    </Card>
                </div>
            } else {
                <Card>
                    <CardContent class="py-12">
                        <div class="text-center text-muted-foreground">
                            {t.t("execution.terminal.client_load_failed")}
                        </div>
                    </CardContent>
                </Card>
            }
        </div>
    }
}
