use gloo::timers::callback::Interval;
use lucide_yew::{Database, Settings, Signal, Zap};
use std::rc::Rc;
use yew::prelude::*;

use crate::components::dashboard::client_status_list::ClientStatusList;
use crate::components::dashboard::os_dist_card::OsDistCard;
use crate::components::dashboard::recent_clients_card::RecentClientsCard;
use crate::components::dashboard::stats_card::StatsCard;
use crate::components::dashboard::system_status_card::SystemStatusCard;
use crate::components::error::ErrorDisplay;
use crate::components::loading::Loading;
use crate::hooks::use_trans::use_trans;
use crate::services::api::{self, ApiError};
use crate::types::Client;

pub enum HomeAction {
    SetClients(Vec<Client>),
    SetError(Option<String>),
    RefreshData,
}

pub struct HomeState {
    clients: Vec<Client>,
    loading: bool,
    error: Option<String>,
}

impl Default for HomeState {
    fn default() -> Self {
        Self {
            clients: Vec::new(),
            loading: true,
            error: None,
        }
    }
}

impl Reducible for HomeState {
    type Action = HomeAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            HomeAction::SetClients(clients) => Rc::new(Self {
                clients,
                loading: false,
                error: None,
            }),
            HomeAction::SetError(error) => Rc::new(Self {
                clients: self.clients.clone(),
                loading: false,
                error,
            }),
            HomeAction::RefreshData => Rc::new(Self {
                clients: self.clients.clone(),
                loading: true,
                error: None,
            }),
        }
    }
}

#[function_component(HomePage)]
pub fn home_page() -> Html {
    let state = use_reducer(HomeState::default);
    let t = use_trans();

    // 设置定时刷新
    let state_for_interval = state.clone();
    use_effect_with((), move |_| {
        // 每60秒刷新一次数据
        let interval = Interval::new(60000, move || {
            state_for_interval.dispatch(HomeAction::RefreshData);
        });

        || drop(interval) // cleanup
    });

    // 加载数据
    {
        let state = state.clone();
        use_effect_with(state.loading, move |loading| {
            if *loading {
                let state_clone = state.clone();
                api::get_clients(
                    1,
                    1000,
                    None,
                    None,
                    None,
                    Callback::from(
                        move |result: Result<crate::types::PaginatedResult<Client>, ApiError>| {
                            match result {
                                Ok(paginated) => {
                                    state_clone.dispatch(HomeAction::SetClients(paginated.items));
                                }
                                Err(err) => {
                                    state_clone.dispatch(HomeAction::SetError(Some(err.message)));
                                }
                            }
                        },
                    ),
                );
            }

            || ()
        });
    }

    // 计算统计数据
    let total_clients = state.clients.len();
    let online_clients = state
        .clients
        .iter()
        .filter(|client| {
            client
                .last_seen
                .as_ref()
                .and_then(|last_seen| chrono::DateTime::parse_from_rfc3339(last_seen).ok())
                .map(|dt| {
                    let now = chrono::Utc::now();
                    let duration = now.signed_duration_since(dt.with_timezone(&chrono::Utc));
                    duration.num_minutes() <= 5
                })
                .unwrap_or(false)
        })
        .count();

    let new_clients_today = state
        .clients
        .iter()
        .filter(|client| {
            client
                .registered_at
                .as_ref()
                .and_then(|registered_at| chrono::DateTime::parse_from_rfc3339(registered_at).ok())
                .map(|dt| {
                    let now = chrono::Utc::now();
                    let duration = now.signed_duration_since(dt.with_timezone(&chrono::Utc));
                    duration.num_hours() <= 24
                })
                .unwrap_or(false)
        })
        .count();

    let offline_clients = total_clients.saturating_sub(online_clients);
    let online_ratio = if total_clients > 0 {
        (online_clients as f64 / total_clients as f64) * 100.0
    } else {
        0.0
    };
    let online_rate_display = format!("{:.1}%", online_ratio);

    // 计算操作系统分布
    let mut os_stats = std::collections::HashMap::new();
    for client in &state.clients {
        if let Some(os) = &client.os {
            *os_stats.entry(os.clone()).or_insert(0) += 1;
        }
    }

    // 获取最近注册的客户端
    let mut recent_clients = state.clients.clone();
    recent_clients.sort_by(|a, b| match (&b.registered_at, &a.registered_at) {
        (Some(b_time), Some(a_time)) => b_time.cmp(a_time),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    });
    recent_clients.truncate(5);

    let _on_refresh = {
        let state = state.clone();
        Callback::from(move |_: MouseEvent| {
            state.dispatch(HomeAction::RefreshData);
        })
    };

    let on_retry = {
        let state = state.clone();
        Callback::from(move |_| {
            state.dispatch(HomeAction::RefreshData);
        })
    };

    if state.loading && state.clients.is_empty() {
        return html! { <Loading message={t.t("dashboard.loading")} /> };
    }

    if let Some(error) = &state.error {
        return html! { <ErrorDisplay message={error.clone()} on_retry={on_retry} /> };
    }

    html! {
        <div class="flex flex-col gap-6">

            // Row 1: Summary Cards
            <div class="grid grid-cols-1 sm:grid-cols-2 xl:grid-cols-4 gap-6">
                <StatsCard
                    title={t.t("dashboard.total_clients")}
                    value={total_clients.to_string()}
                    subtext={t.t("dashboard.registered_nodes")}
                    icon={html! { <Database class="h-full w-full" /> }}
                />

                <StatsCard
                    title={t.t("dashboard.online_rate")}
                    value={online_rate_display.clone()}
                    subtext={format!("{} {}", format!("{}/{}", online_clients, total_clients), t.t("dashboard.online"))}
                    icon={html! { <Signal class="h-full w-full" /> }}
                />

                <StatsCard
                    title={t.t("dashboard.new_today")}
                    value={new_clients_today.to_string()}
                    subtext={t.t("dashboard.24h_registered")}
                    icon={html! { <Zap class="h-full w-full" /> }}
                />

                <StatsCard
                    title={t.t("dashboard.system_types")}
                    value={os_stats.len().to_string()}
                    subtext={t.t("dashboard.diverse_os")}
                    icon={html! { <Settings class="h-full w-full" /> }}
                />
            </div>

            // Row 2: 操作系统分布和在线状态图表
            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                <OsDistCard
                    os_stats={os_stats.clone()}
                    total_clients={total_clients}
                />

                <SystemStatusCard
                    online_rate_display={online_rate_display.clone()}
                    online_clients={online_clients}
                    offline_clients={offline_clients}
                />
            </div>

            // Row 3: 最近注册的客户端
            <div class="grid grid-cols-1 lg:grid-cols-12 gap-6">
                <div class="lg:col-span-5">
                    <RecentClientsCard recent_clients={recent_clients.clone()} />
                </div>

                <div class="lg:col-span-7">
                    <ClientStatusList
                        clients={state.clients.clone()}
                        total_clients={total_clients}
                    />
                </div>
            </div>
        </div>
    }
}
