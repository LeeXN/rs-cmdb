use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::card::{Card, CardContent};
use crate::components::ui::input::Input;
use lucide_yew::{Search, X};
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub struct SearchFilter {
    pub search_text: String,
    pub os_filter: String,
    pub status_filter: String,
    pub auto_refresh: bool,
    pub refresh_interval: u32, // 秒
}

impl Default for SearchFilter {
    fn default() -> Self {
        Self {
            search_text: String::new(),
            os_filter: "all".to_string(),
            status_filter: "all".to_string(),
            auto_refresh: false,
            refresh_interval: 60,
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct SearchFilterProps {
    pub filter: SearchFilter,
    pub on_filter_change: Callback<SearchFilter>,
    pub os_options: Vec<String>,
    pub total_count: usize,
    pub filtered_count: usize,
}

#[function_component(SearchFilterComponent)]
pub fn search_filter_component(props: &SearchFilterProps) -> Html {
    let search_input_ref = use_node_ref();

    let on_search_input = {
        let filter = props.filter.clone();
        let on_filter_change = props.on_filter_change.clone();

        Callback::from(move |val: String| {
            let new_filter = SearchFilter {
                search_text: val,
                os_filter: filter.os_filter.clone(),
                status_filter: filter.status_filter.clone(),
                auto_refresh: filter.auto_refresh,
                refresh_interval: filter.refresh_interval,
            };
            on_filter_change.emit(new_filter);
        })
    };

    let on_os_change = {
        let filter = props.filter.clone();
        let on_filter_change = props.on_filter_change.clone();

        Callback::from(move |e: Event| {
            if let Some(select) = e.target_dyn_into::<web_sys::HtmlSelectElement>() {
                let os_filter = select.value();
                let new_filter = SearchFilter {
                    search_text: filter.search_text.clone(),
                    os_filter,
                    status_filter: filter.status_filter.clone(),
                    auto_refresh: filter.auto_refresh,
                    refresh_interval: filter.refresh_interval,
                };
                on_filter_change.emit(new_filter);
            }
        })
    };

    let on_status_change = {
        let filter = props.filter.clone();
        let on_filter_change = props.on_filter_change.clone();

        Callback::from(move |e: Event| {
            if let Some(select) = e.target_dyn_into::<web_sys::HtmlSelectElement>() {
                let status_filter = select.value();
                let new_filter = SearchFilter {
                    search_text: filter.search_text.clone(),
                    os_filter: filter.os_filter.clone(),
                    status_filter,
                    auto_refresh: filter.auto_refresh,
                    refresh_interval: filter.refresh_interval,
                };
                on_filter_change.emit(new_filter);
            }
        })
    };

    let on_clear_filters = {
        let on_filter_change = props.on_filter_change.clone();
        let search_input_ref = search_input_ref.clone();

        Callback::from(move |_: MouseEvent| {
            // 清空搜索框
            if let Some(input) = search_input_ref.cast::<HtmlInputElement>() {
                input.set_value("");
            }

            let new_filter = SearchFilter::default();
            on_filter_change.emit(new_filter);
        })
    };

    let on_auto_refresh_change = {
        let filter = props.filter.clone();
        let on_filter_change = props.on_filter_change.clone();

        Callback::from(move |e: Event| {
            if let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                let auto_refresh = input.checked();
                let new_filter = SearchFilter {
                    search_text: filter.search_text.clone(),
                    os_filter: filter.os_filter.clone(),
                    status_filter: filter.status_filter.clone(),
                    auto_refresh,
                    refresh_interval: filter.refresh_interval,
                };
                on_filter_change.emit(new_filter);
            }
        })
    };

    let on_refresh_interval_change = {
        let filter = props.filter.clone();
        let on_filter_change = props.on_filter_change.clone();

        Callback::from(move |e: Event| {
            if let Some(select) = e.target_dyn_into::<web_sys::HtmlSelectElement>() {
                if let Ok(refresh_interval) = select.value().parse::<u32>() {
                    let new_filter = SearchFilter {
                        search_text: filter.search_text.clone(),
                        os_filter: filter.os_filter.clone(),
                        status_filter: filter.status_filter.clone(),
                        auto_refresh: filter.auto_refresh,
                        refresh_interval,
                    };
                    on_filter_change.emit(new_filter);
                }
            }
        })
    };

    let has_active_filters = !props.filter.search_text.is_empty()
        || props.filter.os_filter != "all"
        || props.filter.status_filter != "all";

    let select_class = "flex h-9 w-full items-center justify-between rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-sm ring-offset-background placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-ring disabled:cursor-not-allowed disabled:opacity-50 [&>span]:line-clamp-1 bg-slate-950/50";

    html! {
        <Card class="mb-6">
            <CardContent class="p-4 space-y-4">
                <div class="grid grid-cols-1 md:grid-cols-12 gap-4 items-end">
                    // Search Input
                    <div class="md:col-span-4 relative">
                        <Search class="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
                        <Input
                            node_ref={search_input_ref}
                            class="pl-9"
                            value={props.filter.search_text.clone()}
                            oninput={on_search_input}
                            placeholder="搜索主机名、IP、操作系统..."
                        />
                    </div>

                    // OS Filter
                    <div class="md:col-span-2">
                        <label class="text-xs font-medium text-muted-foreground mb-1.5 block">{"操作系统"}</label>
                        <select
                            class={select_class}
                            value={props.filter.os_filter.clone()}
                            onchange={on_os_change}
                        >
                            <option value="all">{"全部"}</option>
                            {
                                for props.os_options.iter().map(|os| {
                                    html! { <option value={os.clone()}>{ os }</option> }
                                })
                            }
                        </select>
                    </div>

                    // Status Filter
                    <div class="md:col-span-2">
                        <label class="text-xs font-medium text-muted-foreground mb-1.5 block">{"状态"}</label>
                        <select
                            class={select_class}
                            value={props.filter.status_filter.clone()}
                            onchange={on_status_change}
                        >
                            <option value="all">{"所有状态"}</option>
                            <option value="online">{"在线"}</option>
                            <option value="offline">{"离线"}</option>
                        </select>
                    </div>

                    // Auto Refresh
                    <div class="md:col-span-4 flex items-center justify-end space-x-4 pb-1">
                        <div class="flex items-center space-x-2">
                            <label class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">
                                {"自动刷新"}
                            </label>
                            <input
                                type="checkbox"
                                class="h-4 w-4 rounded border-gray-300 text-cyan-600 focus:ring-cyan-500 bg-slate-950"
                                checked={props.filter.auto_refresh}
                                onchange={on_auto_refresh_change}
                            />
                        </div>

                        {
                            if props.filter.auto_refresh {
                                html! {
                                    <select
                                        class={format!("{} w-24", select_class)}
                                        value={props.filter.refresh_interval.to_string()}
                                        onchange={on_refresh_interval_change}
                                    >
                                        <option value="30">{"30秒"}</option>
                                        <option value="60">{"1分钟"}</option>
                                        <option value="120">{"2分钟"}</option>
                                        <option value="300">{"5分钟"}</option>
                                        <option value="600">{"10分钟"}</option>
                                    </select>
                                }
                            } else {
                                html! {}
                            }
                        }
                    </div>
                </div>

                <div class="flex items-center justify-between pt-2 border-t border-border">
                    <div class="text-sm text-muted-foreground">
                        {"显示 "}<span class="text-foreground font-medium">{props.filtered_count}</span>{" / " }{props.total_count}{" 台设备"}
                    </div>

                    <div class="flex items-center space-x-2">
                        {
                            if has_active_filters {
                                html! {
                                    <>
                                        <div class="flex items-center space-x-2 mr-4">
                                            <span class="text-xs text-muted-foreground">{"当前筛选:"}</span>
                                            {
                                                if !props.filter.search_text.is_empty() {
                                                    html! {
                                                        <Badge variant={BadgeVariant::Secondary} class="text-xs">
                                                            {"搜索: "}{props.filter.search_text.clone()}
                                                        </Badge>
                                                    }
                                                } else {
                                                    html! {}
                                                }
                                            }
                                            {
                                                if props.filter.os_filter != "all" {
                                                    html! {
                                                        <Badge variant={BadgeVariant::Outline} class="text-xs">
                                                            {"系统: "}{props.filter.os_filter.clone()}
                                                        </Badge>
                                                    }
                                                } else {
                                                    html! {}
                                                }
                                            }
                                            {
                                                if props.filter.status_filter != "all" {
                                                    html! {
                                                        <Badge variant={BadgeVariant::Outline} class="text-xs">
                                                            {"状态: "}{
                                                                match props.filter.status_filter.as_str() {
                                                                    "online" => "在线",
                                                                    "offline" => "离线",
                                                                    _ => &props.filter.status_filter,
                                                                }
                                                            }
                                                        </Badge>
                                                    }
                                                } else {
                                                    html! {}
                                                }
                                            }
                                        </div>
                                        <Button
                                            variant={ButtonVariant::Ghost}
                                            size={ButtonSize::Sm}
                                            onclick={on_clear_filters}
                                            class="h-8 px-2 lg:px-3"
                                        >
                                            <X class="mr-2 h-4 w-4" />
                                            {"清除筛选"}
                                        </Button>
                                    </>
                                }
                            } else {
                                html! {}
                            }
                        }
                    </div>
                </div>
            </CardContent>
        </Card>
    }
}
