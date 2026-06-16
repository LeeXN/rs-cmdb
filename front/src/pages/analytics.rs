use crate::components::error::ErrorDisplay;
use crate::components::loading::Loading;
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::card::{Card, CardBody, CardHeader};
use crate::hooks::use_trans::use_trans;
use crate::icons::{Activity, CircleCheck, CircleX, Monitor};
use crate::services::api::{self, ApiError};
use crate::types::DetailedStats;
use gloo_timers;
use js_sys;
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen_futures;
use web_sys;
use yew::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub enum AnalyticsAction {
    SetStats(DetailedStats),
    SetLoading(bool),
    SetError(Option<String>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnalyticsState {
    stats: DetailedStats,
    loading: bool,
    error: Option<String>,
}

impl Default for AnalyticsState {
    fn default() -> Self {
        Self {
            stats: DetailedStats::default(),
            loading: true,
            error: None,
        }
    }
}

impl Reducible for AnalyticsState {
    type Action = AnalyticsAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            AnalyticsAction::SetStats(stats) => Rc::new(Self {
                stats,
                loading: false,
                error: None,
            }),
            AnalyticsAction::SetLoading(loading) => Rc::new(Self {
                loading,
                ..(*self).clone()
            }),
            AnalyticsAction::SetError(error) => Rc::new(Self {
                error,
                loading: false,
                ..(*self).clone()
            }),
        }
    }
}

#[function_component(AnalyticsPage)]
pub fn analytics_page() -> Html {
    let t = use_trans();
    let state = use_reducer(AnalyticsState::default);

    // 加载统计数据
    {
        let state = state.clone();
        use_effect_with((), move |_| {
            state.dispatch(AnalyticsAction::SetLoading(true));

            api::get_detailed_stats(Callback::from(
                move |result: Result<DetailedStats, ApiError>| match result {
                    Ok(stats) => {
                        state.dispatch(AnalyticsAction::SetStats(stats));
                    }
                    Err(err) => {
                        state.dispatch(AnalyticsAction::SetError(Some(err.message)));
                    }
                },
            ));

            || ()
        });
    }

    // 初始化图表
    {
        let stats = state.stats.clone();
        let loading = state.loading;
        use_effect_with((stats.clone(), loading), move |(stats, loading)| {
            if !loading && stats.total_clients > 0 {
                let stats_clone = stats.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    init_all_charts(&stats_clone).await;
                });
            }
            || ()
        });
    }

    // 刷新数据
    let on_refresh = {
        let state = state.clone();
        Callback::from(move |_: MouseEvent| {
            state.dispatch(AnalyticsAction::SetLoading(true));

            api::get_detailed_stats(Callback::from({
                let state = state.clone();
                move |result: Result<DetailedStats, ApiError>| match result {
                    Ok(stats) => {
                        state.dispatch(AnalyticsAction::SetStats(stats));
                    }
                    Err(err) => {
                        state.dispatch(AnalyticsAction::SetError(Some(err.message)));
                    }
                }
            }));
        })
    };

    html! {
        <div class="container-fluid py-4">
            <div class="row">
                <div class="col-12">
                    if state.loading {
                        <Loading />
                    } else if let Some(error) = &state.error {
                        <div class="d-flex flex-column align-items-center">
                            <ErrorDisplay message={error.clone()} />
                            <Button
                                variant={ButtonVariant::Default}
                                size={ButtonSize::Sm}
                                onclick={on_refresh.clone()}
                                class="mt-3"
                            >
                                <i class="material-icons text-sm mr-1">{"refresh"}</i>
                                {t.t("analytics.retry")}
                            </Button>
                        </div>
                    } else {
                        <>
                            // 统计概览
                            { render_summary_cards(&state.stats, &t) }

                            // 详细图表
                            { render_analytics_charts(&state.stats, &t) }
                        </>
                    }
                </div>
            </div>
        </div>
    }
}

// 渲染概览卡片
fn render_summary_cards(stats: &DetailedStats, t: &Rc<crate::i18n::I18n>) -> Html {
    html! {
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-6">
            <Card class="shadow-xl">
                <CardBody class="p-4">
                    <div class="flex items-center">
                        <div class="icon icon-shape bg-gradient-primary shadow text-center border-radius-md flex items-center justify-center">
                            <Monitor class="w-6 h-6 text-white" />
                        </div>
                        <div class="ml-3">
                            <p class="text-sm mb-0 text-capitalize font-weight-bold text-slate-400">{t.t("analytics.total_devices")}</p>
                            <h5 class="font-weight-bolder mb-0 text-white">
                                {stats.total_clients}
                            </h5>
                        </div>
                    </div>
                </CardBody>
            </Card>

            <Card class="shadow-xl">
                <CardBody class="p-4">
                    <div class="flex items-center">
                        <div class="icon icon-shape bg-gradient-success shadow text-center border-radius-md flex items-center justify-center">
                            <CircleCheck class="w-6 h-6 text-white" />
                        </div>
                        <div class="ml-3">
                            <p class="text-sm mb-0 text-capitalize font-weight-bold text-slate-400">{t.t("analytics.online_devices")}</p>
                            <h5 class="font-weight-bolder mb-0 text-white">
                                {stats.online_clients}
                            </h5>
                        </div>
                    </div>
                </CardBody>
            </Card>

            <Card class="shadow-xl">
                <CardBody class="p-4">
                    <div class="flex items-center">
                        <div class="icon icon-shape bg-gradient-danger shadow text-center border-radius-md flex items-center justify-center">
                            <CircleX class="w-6 h-6 text-white" />
                        </div>
                        <div class="ml-3">
                            <p class="text-sm mb-0 text-capitalize font-weight-bold text-slate-400">{t.t("analytics.offline_devices")}</p>
                            <h5 class="font-weight-bolder mb-0 text-white">
                                {stats.offline_clients}
                            </h5>
                        </div>
                    </div>
                </CardBody>
            </Card>

            <Card class="shadow-xl">
                <CardBody class="p-4">
                    <div class="flex items-center">
                        <div class="icon icon-shape bg-gradient-info shadow text-center border-radius-md flex items-center justify-center">
                            <Activity class="w-6 h-6 text-white" />
                        </div>
                        <div class="ml-3">
                            <p class="text-sm mb-0 text-capitalize font-weight-bold text-slate-400">{t.t("analytics.online_rate")}</p>
                            <h5 class="font-weight-bolder mb-0 text-white">
                                {
                                    if stats.total_clients > 0 {
                                        format!("{:.1}%", (stats.online_clients as f64 / stats.total_clients as f64) * 100.0)
                                    } else {
                                        "0%".to_string()
                                    }
                                }
                            </h5>
                        </div>
                    </div>
                </CardBody>
            </Card>
        </div>
    }
}

// 渲染分析图表
fn render_analytics_charts(stats: &DetailedStats, t: &Rc<crate::i18n::I18n>) -> Html {
    // 使用effect来初始化图表
    use_effect_with(stats.clone(), {
        let stats = stats.clone();
        move |_| {
            // 在组件挂载后初始化图表
            wasm_bindgen_futures::spawn_local(async move {
                init_all_charts(&stats).await;
            });
            || ()
        }
    });

    html! {
        <>
            // 第一行：GPU相关统计
            <div class="grid grid-cols-1 lg:grid-cols-3 gap-6 mb-6">
                <Card class="h-full shadow-xl">
                    <CardHeader>
                        <h6 class="text-white text-capitalize ps-3">{t.t("analytics.gpu_vendor_distribution")}</h6>
                        <p class="text-xs text-slate-400 ps-3 mb-0">{t.t("analytics.by_machine_count")}</p>
                    </CardHeader>
                    <CardBody class="p-4">
                        <div class="flex justify-center my-4">
                            <canvas id="gpuVendorChart" width="300" height="200"></canvas>
                        </div>

                        <div class="divider my-1 border-slate-700"></div>

                        <h3 class="font-bold text-xs uppercase opacity-70 mb-2 text-slate-400">{t.t("analytics.detailed_stats")}</h3>
                        <div class="overflow-x-auto">
                            <table class="table w-full text-slate-300">
                                <thead>
                                    <tr class="text-slate-400 border-b border-slate-700">
                                        <th class="text-left p-2">{t.t("analytics.gpu_vendor")}</th>
                                        <th class="text-center p-2">{t.t("analytics.count")}</th>
                                        <th class="text-center p-2">{t.t("analytics.percentage")}</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {
                                        stats.gpu_stats.by_vendor.iter().map(|item| {
                                            html! {
                                                <tr class="border-b border-slate-800 hover:bg-slate-800/50">
                                                    <td class="p-2">{&item.name}</td>
                                                    <td class="text-center p-2">{t.t_with_args("analytics.unit_machine", &HashMap::from([("count".to_string(), item.count.to_string())]))}</td>
                                                    <td class="text-center p-2">{format!("{:.1}%", item.percentage)}</td>
                                                </tr>
                                            }
                                        }).collect::<Html>()
                                    }
                                </tbody>
                            </table>
                        </div>
                    </CardBody>
                </Card>

                <Card class="h-full shadow-xl">
                    <CardHeader>
                        <h6 class="text-white text-capitalize ps-3">{t.t("analytics.gpu_model_distribution")}</h6>
                        <p class="text-xs text-slate-400 ps-3 mb-0">{t.t("analytics.by_machine_count")}</p>
                    </CardHeader>
                    <CardBody class="p-4">
                        <div class="flex justify-center my-4">
                            <canvas id="gpuModelChart" width="300" height="200"></canvas>
                        </div>

                        <div class="divider my-1 border-slate-700"></div>

                        <h3 class="font-bold text-xs uppercase opacity-70 mb-2 text-slate-400">{t.t("analytics.detailed_stats")}</h3>
                        <div class="overflow-x-auto max-h-[300px]">
                            <table class="table w-full text-slate-300">
                                <thead>
                                    <tr class="text-slate-400 border-b border-slate-700">
                                        <th class="text-left p-2">{t.t("analytics.gpu_model")}</th>
                                        <th class="text-center p-2">{t.t("analytics.count")}</th>
                                        <th class="text-center p-2">{t.t("analytics.percentage")}</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {
                                        stats.gpu_stats.by_model.iter().map(|item| {
                                            html! {
                                                <tr class="border-b border-slate-800 hover:bg-slate-800/50">
                                                    <td class="p-2">{&item.name}</td>
                                                    <td class="text-center p-2">{t.t_with_args("analytics.unit_machine", &HashMap::from([("count".to_string(), item.count.to_string())]))}</td>
                                                    <td class="text-center p-2">{format!("{:.1}%", item.percentage)}</td>
                                                </tr>
                                            }
                                        }).collect::<Html>()
                                    }
                                </tbody>
                            </table>
                        </div>
                    </CardBody>
                </Card>

                <Card class="h-full shadow-xl">
                    <CardHeader>
                        <h6 class="text-white text-capitalize ps-3">{t.t("analytics.gpu_detailed_config")}</h6>
                        <p class="text-xs text-slate-400 ps-3 mb-0">{t.t("analytics.by_model_and_count")}</p>
                    </CardHeader>
                    <CardBody class="p-4">
                        <div class="flex justify-center my-4">
                            <canvas id="gpuDetailChart" width="300" height="200"></canvas>
                        </div>

                        <div class="divider my-1 border-slate-700"></div>

                        <h3 class="font-bold text-xs uppercase opacity-70 mb-2 text-slate-400">{t.t("analytics.detailed_stats")}</h3>
                        <div class="overflow-x-auto max-h-[300px]">
                            <table class="table w-full text-slate-300">
                                <thead>
                                    <tr class="text-slate-400 border-b border-slate-700">
                                        <th class="text-left p-2">{t.t("analytics.gpu_config")}</th>
                                        <th class="text-center p-2">{t.t("analytics.count")}</th>
                                        <th class="text-center p-2">{t.t("analytics.percentage")}</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {
                                        stats.gpu_stats.by_model_with_count.iter().map(|item| {
                                            html! {
                                                <tr class="border-b border-slate-800 hover:bg-slate-800/50">
                                                    <td class="p-2">{&item.name}</td>
                                                    <td class="text-center p-2">{t.t_with_args("analytics.unit_machine", &HashMap::from([("count".to_string(), item.count.to_string())]))}</td>
                                                    <td class="text-center p-2">{format!("{:.1}%", item.percentage)}</td>
                                                </tr>
                                            }
                                        }).collect::<Html>()
                                    }
                                </tbody>
                            </table>
                        </div>
                    </CardBody>
                </Card>
            </div>

            // 第二行：CPU和存储统计
            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-6">
                <Card class="h-full shadow-xl">
                    <CardHeader>
                        <h6 class="text-white text-capitalize ps-3">{t.t("analytics.cpu_model_distribution")}</h6>
                        <p class="text-xs text-slate-400 ps-3 mb-0">{t.t("analytics.by_machine_count")}</p>
                    </CardHeader>
                    <CardBody class="p-4">
                        <div class="flex justify-center my-4">
                            <canvas id="cpuModelChart" width="400" height="250"></canvas>
                        </div>

                        <div class="divider my-1 border-slate-700"></div>

                        <h3 class="font-bold text-xs uppercase opacity-70 mb-2 text-slate-400">{t.t("analytics.detailed_stats")}</h3>
                        <div class="overflow-x-auto max-h-[300px]">
                            <table class="table w-full text-slate-300">
                                <thead>
                                    <tr class="text-slate-400 border-b border-slate-700">
                                        <th class="text-left p-2">{t.t("analytics.cpu_model")}</th>
                                        <th class="text-center p-2">{t.t("analytics.count")}</th>
                                        <th class="text-center p-2">{t.t("analytics.percentage")}</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {
                                        stats.cpu_stats.by_model.iter().map(|item| {
                                            html! {
                                                <tr class="border-b border-slate-800 hover:bg-slate-800/50">
                                                    <td class="p-2">{&item.name}</td>
                                                    <td class="text-center p-2">{t.t_with_args("analytics.unit_machine", &HashMap::from([("count".to_string(), item.count.to_string())]))}</td>
                                                    <td class="text-center p-2">{format!("{:.1}%", item.percentage)}</td>
                                                </tr>
                                            }
                                        }).collect::<Html>()
                                    }
                                </tbody>
                            </table>
                        </div>
                    </CardBody>
                </Card>

                <Card class="h-full shadow-xl">
                    <CardHeader>
                        <h6 class="text-white text-capitalize ps-3">{t.t("analytics.storage_type_distribution")}</h6>
                        <p class="text-xs text-slate-400 ps-3 mb-0">{t.t("analytics.by_machine_count")}</p>
                    </CardHeader>
                    <CardBody class="p-4">
                        <div class="flex justify-center my-4">
                            <canvas id="storageTypeChart" width="400" height="250"></canvas>
                        </div>

                        <div class="divider my-1 border-slate-700"></div>

                        <h3 class="font-bold text-xs uppercase opacity-70 mb-2 text-slate-400">{t.t("analytics.detailed_stats")}</h3>
                        <div class="overflow-x-auto">
                            <table class="table w-full text-slate-300">
                                <thead>
                                    <tr class="text-slate-400 border-b border-slate-700">
                                        <th class="text-left p-2">{t.t("analytics.storage_type")}</th>
                                        <th class="text-center p-2">{t.t("analytics.count")}</th>
                                        <th class="text-center p-2">{t.t("analytics.percentage")}</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {
                                        stats.storage_stats.by_type.iter().map(|item| {
                                            html! {
                                                <tr class="border-b border-slate-800 hover:bg-slate-800/50">
                                                    <td class="p-2">{&item.name}</td>
                                                    <td class="text-center p-2">{t.t_with_args("analytics.unit_machine", &HashMap::from([("count".to_string(), item.count.to_string())]))}</td>
                                                    <td class="text-center p-2">{format!("{:.1}%", item.percentage)}</td>
                                                </tr>
                                            }
                                        }).collect::<Html>()
                                    }
                                </tbody>
                            </table>
                        </div>
                    </CardBody>
                </Card>
            </div>

            // 第三行：操作系统和内存统计
            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-6">
                <Card class="h-full shadow-xl">
                    <CardHeader>
                        <h6 class="text-white text-capitalize ps-3">{t.t("analytics.os_distribution")}</h6>
                        <p class="text-xs text-slate-400 ps-3 mb-0">{t.t("analytics.by_machine_count")}</p>
                    </CardHeader>
                    <CardBody class="p-4">
                        <div class="flex justify-center my-4">
                            <canvas id="osChart" width="400" height="250"></canvas>
                        </div>

                        <div class="divider my-1 border-slate-700"></div>

                        <h3 class="font-bold text-xs uppercase opacity-70 mb-2 text-slate-400">{t.t("analytics.detailed_stats")}</h3>
                        <div class="overflow-x-auto">
                            <table class="table w-full text-slate-300">
                                <thead>
                                    <tr class="text-slate-400 border-b border-slate-700">
                                        <th class="text-left p-2">{t.t("analytics.os")}</th>
                                        <th class="text-center p-2">{t.t("analytics.count")}</th>
                                        <th class="text-center p-2">{t.t("analytics.percentage")}</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {
                                        stats.os_stats.by_name.iter().map(|item| {
                                            html! {
                                                <tr class="border-b border-slate-800 hover:bg-slate-800/50">
                                                    <td class="p-2">{&item.name}</td>
                                                    <td class="text-center p-2">{t.t_with_args("analytics.unit_machine", &HashMap::from([("count".to_string(), item.count.to_string())]))}</td>
                                                    <td class="text-center p-2">{format!("{:.1}%", item.percentage)}</td>
                                                </tr>
                                            }
                                        }).collect::<Html>()
                                    }
                                </tbody>
                            </table>
                        </div>
                    </CardBody>
                </Card>

                <Card class="h-full shadow-xl">
                    <CardHeader>
                        <h6 class="text-white text-capitalize ps-3">{t.t("analytics.memory_size_distribution")}</h6>
                        <p class="text-xs text-slate-400 ps-3 mb-0">{t.t("analytics.by_machine_count")}</p>
                    </CardHeader>
                    <CardBody class="p-4">
                        <div class="flex justify-center my-4">
                            <canvas id="memoryChart" width="400" height="250"></canvas>
                        </div>

                        <div class="divider my-1 border-slate-700"></div>

                        <h3 class="font-bold text-xs uppercase opacity-70 mb-2 text-slate-400">{t.t("analytics.detailed_stats")}</h3>
                        <div class="overflow-x-auto">
                            <table class="table w-full text-slate-300">
                                <thead>
                                    <tr class="text-slate-400 border-b border-slate-700">
                                        <th class="text-left p-2">{t.t("analytics.memory_size")}</th>
                                        <th class="text-center p-2">{t.t("analytics.count")}</th>
                                        <th class="text-center p-2">{t.t("analytics.percentage")}</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {
                                        stats.memory_stats.by_capacity.iter().map(|item| {
                                            html! {
                                                <tr class="border-b border-slate-800 hover:bg-slate-800/50">
                                                    <td class="p-2">{&item.name}</td>
                                                    <td class="text-center p-2">{t.t_with_args("analytics.unit_machine", &HashMap::from([("count".to_string(), item.count.to_string())]))}</td>
                                                    <td class="text-center p-2">{format!("{:.1}%", item.percentage)}</td>
                                                </tr>
                                            }
                                        }).collect::<Html>()
                                    }
                                </tbody>
                            </table>
                        </div>
                    </CardBody>
                </Card>
            </div>

            // 第四行：网络和服务器统计
            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                <Card class="h-full shadow-xl">
                    <CardHeader>
                        <h6 class="text-white text-capitalize ps-3">{t.t("analytics.network_type_distribution")}</h6>
                        <p class="text-xs text-slate-400 ps-3 mb-0">{t.t("analytics.by_machine_count")}</p>
                    </CardHeader>
                    <CardBody class="p-4">
                        <div class="flex justify-center my-4">
                            <canvas id="networkChart" width="400" height="250"></canvas>
                        </div>

                        <div class="divider my-1 border-slate-700"></div>

                        <h3 class="font-bold text-xs uppercase opacity-70 mb-2 text-slate-400">{t.t("analytics.detailed_stats")}</h3>
                        <div class="overflow-x-auto">
                            <table class="table w-full text-slate-300">
                                <thead>
                                    <tr class="text-slate-400 border-b border-slate-700">
                                        <th class="text-left p-2">{t.t("analytics.network_type")}</th>
                                        <th class="text-center p-2">{t.t("analytics.count")}</th>
                                        <th class="text-center p-2">{t.t("analytics.percentage")}</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {
                                        stats.network_stats.by_type.iter().map(|item| {
                                            html! {
                                                <tr class="border-b border-slate-800 hover:bg-slate-800/50">
                                                    <td class="p-2">{&item.name}</td>
                                                    <td class="text-center p-2">{t.t_with_args("analytics.unit_machine", &HashMap::from([("count".to_string(), item.count.to_string())]))}</td>
                                                    <td class="text-center p-2">{format!("{:.1}%", item.percentage)}</td>
                                                </tr>
                                            }
                                        }).collect::<Html>()
                                    }
                                </tbody>
                            </table>
                        </div>
                    </CardBody>
                </Card>

                <Card class="h-full shadow-xl">
                    <CardHeader>
                        <h6 class="text-white text-capitalize ps-3">{t.t("analytics.server_model_distribution")}</h6>
                        <p class="text-xs text-slate-400 ps-3 mb-0">{t.t("analytics.by_machine_count")}</p>
                    </CardHeader>
                    <CardBody class="p-4">
                        <div class="flex justify-center my-4">
                            <canvas id="serverChart" width="400" height="250"></canvas>
                        </div>

                        <div class="divider my-1 border-slate-700"></div>

                        <h3 class="font-bold text-xs uppercase opacity-70 mb-2 text-slate-400">{t.t("analytics.detailed_stats")}</h3>
                        <div class="overflow-x-auto max-h-[300px]">
                            <table class="table w-full text-slate-300">
                                <thead>
                                    <tr class="text-slate-400 border-b border-slate-700">
                                        <th class="text-left p-2">{t.t("analytics.server_model")}</th>
                                        <th class="text-center p-2">{t.t("analytics.count")}</th>
                                        <th class="text-center p-2">{t.t("analytics.percentage")}</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {
                                        stats.server_stats.by_product_name.iter().map(|item| {
                                            html! {
                                                <tr class="border-b border-slate-800 hover:bg-slate-800/50">
                                                    <td class="p-2">{&item.name}</td>
                                                    <td class="text-center p-2">{t.t_with_args("analytics.unit_machine", &HashMap::from([("count".to_string(), item.count.to_string())]))}</td>
                                                    <td class="text-center p-2">{format!("{:.1}%", item.percentage)}</td>
                                                </tr>
                                            }
                                        }).collect::<Html>()
                                    }
                                </tbody>
                            </table>
                        </div>
                    </CardBody>
                </Card>
            </div>
        </>
    }
}

// 图表初始化函数
async fn init_all_charts(stats: &DetailedStats) {
    // Neon theme colors
    let colors = vec![
        "#06b6d4", // Cyan 500
        "#8b5cf6", // Violet 500
        "#3b82f6", // Blue 500
        "#ec4899", // Pink 500
        "#10b981", // Emerald 500
        "#f59e0b", // Amber 500
        "#6366f1", // Indigo 500
        "#14b8a6", // Teal 500
    ];

    // 给页面更多时间渲染完成，并等待Chart.js加载
    gloo_timers::future::TimeoutFuture::new(500).await;

    // 检查Chart.js是否已加载
    if let Ok(window) = web_sys::window().ok_or("window不存在") {
        if let Ok(chart_obj) =
            js_sys::Reflect::get(&window, &wasm_bindgen::JsValue::from_str("Chart"))
        {
            if chart_obj.is_undefined() {
                return;
            }
        }
    }

    // GPU厂商饼图
    if let Some(canvas) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id("gpuVendorChart"))
    {
        if !stats.gpu_stats.by_vendor.is_empty() {
            create_pie_chart_js(&canvas, &stats.gpu_stats.by_vendor, &colors).await;
        }
    }

    // GPU型号柱状图
    if let Some(canvas) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id("gpuModelChart"))
    {
        if !stats.gpu_stats.by_model.is_empty() {
            create_bar_chart_js(&canvas, &stats.gpu_stats.by_model, &colors).await;
        }
    }

    // GPU详细配置柱状图
    if let Some(canvas) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id("gpuDetailChart"))
    {
        if !stats.gpu_stats.by_model_with_count.is_empty() {
            create_bar_chart_js(&canvas, &stats.gpu_stats.by_model_with_count, &colors).await;
        }
    }

    // CPU型号柱状图
    if let Some(canvas) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id("cpuModelChart"))
    {
        if !stats.cpu_stats.by_model.is_empty() {
            create_bar_chart_js(&canvas, &stats.cpu_stats.by_model, &colors).await;
        }
    }

    // 存储类型饼图
    if let Some(canvas) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id("storageTypeChart"))
    {
        if !stats.storage_stats.by_type.is_empty() {
            create_pie_chart_js(&canvas, &stats.storage_stats.by_type, &colors).await;
        }
    }

    // 操作系统饼图
    if let Some(canvas) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id("osChart"))
    {
        if !stats.os_stats.by_name.is_empty() {
            create_pie_chart_js(&canvas, &stats.os_stats.by_name, &colors).await;
        }
    }

    // 内存饼图
    if let Some(canvas) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id("memoryChart"))
    {
        if !stats.memory_stats.by_capacity.is_empty() {
            create_pie_chart_js(&canvas, &stats.memory_stats.by_capacity, &colors).await;
        }
    }

    // 网络类型饼图
    if let Some(canvas) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id("networkChart"))
    {
        if !stats.network_stats.by_type.is_empty() {
            create_pie_chart_js(&canvas, &stats.network_stats.by_type, &colors).await;
        }
    }

    // 服务器型号柱状图
    if let Some(canvas) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id("serverChart"))
    {
        if !stats.server_stats.by_product_name.is_empty() {
            create_bar_chart_js(&canvas, &stats.server_stats.by_product_name, &colors).await;
        }
    }
}

// 使用Chart.js创建饼图
async fn create_pie_chart_js(
    canvas: &web_sys::Element,
    data: &[crate::types::StatItem],
    colors: &[&str],
) {
    use wasm_bindgen::JsValue;

    if data.is_empty() {
        return;
    }

    let labels: Vec<JsValue> = data
        .iter()
        .take(10)
        .map(|item| JsValue::from_str(&item.name))
        .collect();

    let values: Vec<JsValue> = data
        .iter()
        .take(10)
        .map(|item| JsValue::from_f64(item.count as f64))
        .collect();

    let chart_colors: Vec<JsValue> = colors
        .iter()
        .take(data.len().min(10))
        .map(|color| JsValue::from_str(color))
        .collect();

    let config = js_sys::Object::new();

    // type
    js_sys::Reflect::set(
        &config,
        &JsValue::from_str("type"),
        &JsValue::from_str("pie"),
    )
    .unwrap();

    // data
    let chart_data = js_sys::Object::new();
    js_sys::Reflect::set(
        &chart_data,
        &JsValue::from_str("labels"),
        &js_sys::Array::from_iter(labels),
    )
    .unwrap();

    let datasets = js_sys::Array::new();
    let dataset = js_sys::Object::new();
    js_sys::Reflect::set(
        &dataset,
        &JsValue::from_str("data"),
        &js_sys::Array::from_iter(values),
    )
    .unwrap();
    js_sys::Reflect::set(
        &dataset,
        &JsValue::from_str("backgroundColor"),
        &js_sys::Array::from_iter(chart_colors),
    )
    .unwrap();
    js_sys::Reflect::set(
        &dataset,
        &JsValue::from_str("borderWidth"),
        &JsValue::from_f64(0.0),
    )
    .unwrap(); // No border for cleaner look
    datasets.push(&dataset);

    js_sys::Reflect::set(&chart_data, &JsValue::from_str("datasets"), &datasets).unwrap();
    js_sys::Reflect::set(&config, &JsValue::from_str("data"), &chart_data).unwrap();

    // options - 限制饼图大小
    let options = js_sys::Object::new();

    // 设置响应式和维持长宽比
    js_sys::Reflect::set(
        &options,
        &JsValue::from_str("responsive"),
        &JsValue::from_bool(true),
    )
    .unwrap();
    js_sys::Reflect::set(
        &options,
        &JsValue::from_str("maintainAspectRatio"),
        &JsValue::from_bool(true),
    )
    .unwrap();
    js_sys::Reflect::set(
        &options,
        &JsValue::from_str("aspectRatio"),
        &JsValue::from_f64(1.5),
    )
    .unwrap();

    // 设置饼图大小限制
    let elements = js_sys::Object::new();
    let arc = js_sys::Object::new();
    js_sys::Reflect::set(
        &arc,
        &JsValue::from_str("borderWidth"),
        &JsValue::from_f64(0.0),
    )
    .unwrap();
    js_sys::Reflect::set(&elements, &JsValue::from_str("arc"), &arc).unwrap();
    js_sys::Reflect::set(&options, &JsValue::from_str("elements"), &elements).unwrap();

    let plugins = js_sys::Object::new();
    let legend = js_sys::Object::new();
    js_sys::Reflect::set(
        &legend,
        &JsValue::from_str("display"),
        &JsValue::from_bool(true),
    )
    .unwrap();
    js_sys::Reflect::set(
        &legend,
        &JsValue::from_str("position"),
        &JsValue::from_str("right"),
    )
    .unwrap();

    // Legend labels color
    let legend_labels = js_sys::Object::new();
    js_sys::Reflect::set(
        &legend_labels,
        &JsValue::from_str("color"),
        &JsValue::from_str("#94a3b8"),
    )
    .unwrap(); // slate-400
    js_sys::Reflect::set(&legend, &JsValue::from_str("labels"), &legend_labels).unwrap();

    js_sys::Reflect::set(&plugins, &JsValue::from_str("legend"), &legend).unwrap();

    let chart_title = js_sys::Object::new();
    js_sys::Reflect::set(
        &chart_title,
        &JsValue::from_str("display"),
        &JsValue::from_bool(false),
    )
    .unwrap();
    js_sys::Reflect::set(&plugins, &JsValue::from_str("title"), &chart_title).unwrap();

    // 添加布局边距来进一步控制大小
    let layout = js_sys::Object::new();
    let padding = js_sys::Object::new();
    js_sys::Reflect::set(
        &padding,
        &JsValue::from_str("top"),
        &JsValue::from_f64(10.0),
    )
    .unwrap();
    js_sys::Reflect::set(
        &padding,
        &JsValue::from_str("bottom"),
        &JsValue::from_f64(10.0),
    )
    .unwrap();
    js_sys::Reflect::set(
        &padding,
        &JsValue::from_str("left"),
        &JsValue::from_f64(10.0),
    )
    .unwrap();
    js_sys::Reflect::set(
        &padding,
        &JsValue::from_str("right"),
        &JsValue::from_f64(10.0),
    )
    .unwrap();
    js_sys::Reflect::set(&layout, &JsValue::from_str("padding"), &padding).unwrap();
    js_sys::Reflect::set(&options, &JsValue::from_str("layout"), &layout).unwrap();

    js_sys::Reflect::set(&options, &JsValue::from_str("plugins"), &plugins).unwrap();
    js_sys::Reflect::set(&config, &JsValue::from_str("options"), &options).unwrap();

    // 调用Chart.js
    if let Ok(window) = web_sys::window().ok_or("window不存在") {
        if let Ok(chart_constructor) = js_sys::Reflect::get(&window, &JsValue::from_str("Chart")) {
            if !chart_constructor.is_undefined() {
                js_sys::Reflect::construct(
                    &chart_constructor.into(),
                    &js_sys::Array::of2(canvas, &config),
                )
                .ok();
            }
        }
    }
}

// 使用Chart.js创建柱状图
async fn create_bar_chart_js(
    canvas: &web_sys::Element,
    data: &[crate::types::StatItem],
    colors: &[&str],
) {
    use wasm_bindgen::JsValue;

    if data.is_empty() {
        return;
    }

    let display_data = data.iter().take(8).collect::<Vec<_>>();

    let labels: Vec<JsValue> = display_data
        .iter()
        .map(|item| JsValue::from_str(&item.name))
        .collect();

    let values: Vec<JsValue> = display_data
        .iter()
        .map(|item| JsValue::from_f64(item.count as f64))
        .collect();

    let chart_colors: Vec<JsValue> = colors
        .iter()
        .take(display_data.len())
        .map(|color| JsValue::from_str(color))
        .collect();

    let config = js_sys::Object::new();

    // type
    js_sys::Reflect::set(
        &config,
        &JsValue::from_str("type"),
        &JsValue::from_str("bar"),
    )
    .unwrap();

    // data
    let chart_data = js_sys::Object::new();
    js_sys::Reflect::set(
        &chart_data,
        &JsValue::from_str("labels"),
        &js_sys::Array::from_iter(labels),
    )
    .unwrap();

    let datasets = js_sys::Array::new();
    let dataset = js_sys::Object::new();
    js_sys::Reflect::set(
        &dataset,
        &JsValue::from_str("data"),
        &js_sys::Array::from_iter(values),
    )
    .unwrap();
    js_sys::Reflect::set(
        &dataset,
        &JsValue::from_str("backgroundColor"),
        &js_sys::Array::from_iter(chart_colors),
    )
    .unwrap();
    js_sys::Reflect::set(
        &dataset,
        &JsValue::from_str("borderWidth"),
        &JsValue::from_f64(0.0),
    )
    .unwrap();
    datasets.push(&dataset);

    js_sys::Reflect::set(&chart_data, &JsValue::from_str("datasets"), &datasets).unwrap();
    js_sys::Reflect::set(&config, &JsValue::from_str("data"), &chart_data).unwrap();

    // options
    let options = js_sys::Object::new();
    let responsive = JsValue::from_bool(true);
    js_sys::Reflect::set(&options, &JsValue::from_str("responsive"), &responsive).unwrap();

    let plugins = js_sys::Object::new();
    let legend = js_sys::Object::new();
    js_sys::Reflect::set(
        &legend,
        &JsValue::from_str("display"),
        &JsValue::from_bool(false),
    )
    .unwrap();
    js_sys::Reflect::set(&plugins, &JsValue::from_str("legend"), &legend).unwrap();

    let chart_title = js_sys::Object::new();
    js_sys::Reflect::set(
        &chart_title,
        &JsValue::from_str("display"),
        &JsValue::from_bool(false),
    )
    .unwrap();
    js_sys::Reflect::set(&plugins, &JsValue::from_str("title"), &chart_title).unwrap();

    js_sys::Reflect::set(&options, &JsValue::from_str("plugins"), &plugins).unwrap();

    let scales = js_sys::Object::new();

    // Y Axis
    let y_axis = js_sys::Object::new();
    js_sys::Reflect::set(
        &y_axis,
        &JsValue::from_str("beginAtZero"),
        &JsValue::from_bool(true),
    )
    .unwrap();

    let grid = js_sys::Object::new();
    js_sys::Reflect::set(
        &grid,
        &JsValue::from_str("color"),
        &JsValue::from_str("#334155"),
    )
    .unwrap(); // slate-700
    js_sys::Reflect::set(&y_axis, &JsValue::from_str("grid"), &grid).unwrap();

    let ticks = js_sys::Object::new();
    js_sys::Reflect::set(
        &ticks,
        &JsValue::from_str("color"),
        &JsValue::from_str("#94a3b8"),
    )
    .unwrap(); // slate-400
    js_sys::Reflect::set(&y_axis, &JsValue::from_str("ticks"), &ticks).unwrap();

    js_sys::Reflect::set(&scales, &JsValue::from_str("y"), &y_axis).unwrap();

    // X Axis
    let x_axis = js_sys::Object::new();
    let x_grid = js_sys::Object::new();
    js_sys::Reflect::set(
        &x_grid,
        &JsValue::from_str("display"),
        &JsValue::from_bool(false),
    )
    .unwrap();
    js_sys::Reflect::set(&x_axis, &JsValue::from_str("grid"), &x_grid).unwrap();

    let x_ticks = js_sys::Object::new();
    js_sys::Reflect::set(
        &x_ticks,
        &JsValue::from_str("color"),
        &JsValue::from_str("#94a3b8"),
    )
    .unwrap(); // slate-400
    js_sys::Reflect::set(&x_axis, &JsValue::from_str("ticks"), &x_ticks).unwrap();

    js_sys::Reflect::set(&scales, &JsValue::from_str("x"), &x_axis).unwrap();

    js_sys::Reflect::set(&options, &JsValue::from_str("scales"), &scales).unwrap();

    js_sys::Reflect::set(&config, &JsValue::from_str("options"), &options).unwrap();

    // 调用Chart.js
    if let Ok(window) = web_sys::window().ok_or("window不存在") {
        if let Ok(chart_constructor) = js_sys::Reflect::get(&window, &JsValue::from_str("Chart")) {
            if !chart_constructor.is_undefined() {
                js_sys::Reflect::construct(
                    &chart_constructor.into(),
                    &js_sys::Array::of2(canvas, &config),
                )
                .ok();
            }
        }
    }
}
