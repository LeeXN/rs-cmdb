use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::card::{Card, CardContent};
use crate::hooks::use_trans::use_trans;
use crate::types::FilterCriteria;
use lucide_yew::{Download, RefreshCw};
use yew::prelude::*;

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct StatsFilterProps {
    pub filter: FilterCriteria,
    pub on_filter_change: Callback<FilterCriteria>,
    pub on_reset: Callback<()>,
    pub on_export: Callback<()>,
}

#[function_component(StatsFilter)]
pub fn stats_filter(props: &StatsFilterProps) -> Html {
    let filter = props.filter.clone();
    let t = use_trans();

    // CPU厂商筛选
    let on_cpu_vendor_change = {
        let filter = filter.clone();
        let on_filter_change = props.on_filter_change.clone();
        Callback::from(move |e: Event| {
            let target = e.target_dyn_into::<web_sys::HtmlSelectElement>().unwrap();
            let value = target.value();
            let mut new_filter = filter.clone();
            new_filter.cpu_vendor = if value.is_empty() { None } else { Some(value) };
            on_filter_change.emit(new_filter);
        })
    };

    // 内存容量筛选
    let on_memory_capacity_change = {
        let filter = filter.clone();
        let on_filter_change = props.on_filter_change.clone();
        Callback::from(move |e: Event| {
            let target = e.target_dyn_into::<web_sys::HtmlSelectElement>().unwrap();
            let value = target.value();
            let mut new_filter = filter.clone();
            match value.as_str() {
                "8gb" => {
                    new_filter.memory_capacity_min = Some(7);
                    new_filter.memory_capacity_max = Some(9);
                }
                "16gb" => {
                    new_filter.memory_capacity_min = Some(15);
                    new_filter.memory_capacity_max = Some(17);
                }
                "32gb" => {
                    new_filter.memory_capacity_min = Some(31);
                    new_filter.memory_capacity_max = Some(33);
                }
                "64gb" => {
                    new_filter.memory_capacity_min = Some(63);
                    new_filter.memory_capacity_max = Some(65);
                }
                _ => {
                    new_filter.memory_capacity_min = None;
                    new_filter.memory_capacity_max = None;
                }
            }
            on_filter_change.emit(new_filter);
        })
    };

    // GPU厂商筛选
    let on_gpu_vendor_change = {
        let filter = filter.clone();
        let on_filter_change = props.on_filter_change.clone();
        Callback::from(move |e: Event| {
            let target = e.target_dyn_into::<web_sys::HtmlSelectElement>().unwrap();
            let value = target.value();
            let mut new_filter = filter.clone();
            new_filter.gpu_vendor = if value.is_empty() { None } else { Some(value) };
            on_filter_change.emit(new_filter);
        })
    };

    // OS筛选
    let on_os_change = {
        let filter = filter.clone();
        let on_filter_change = props.on_filter_change.clone();
        Callback::from(move |e: Event| {
            let target = e.target_dyn_into::<web_sys::HtmlSelectElement>().unwrap();
            let value = target.value();
            let mut new_filter = filter.clone();
            new_filter.os_name = if value.is_empty() { None } else { Some(value) };
            on_filter_change.emit(new_filter);
        })
    };

    // 重置筛选
    let on_reset_click = {
        let on_reset = props.on_reset.clone();
        Callback::from(move |_: MouseEvent| {
            on_reset.emit(());
        })
    };

    // 导出数据
    let on_export_click = {
        let on_export = props.on_export.clone();
        Callback::from(move |_: MouseEvent| {
            on_export.emit(());
        })
    };

    let select_class = "flex h-9 w-full items-center justify-between rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-sm ring-offset-background placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-ring disabled:cursor-not-allowed disabled:opacity-50 [&>span]:line-clamp-1 bg-slate-950/50";

    html! {
        <Card class="mb-6">
            <CardContent class="p-4">
                <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-4">
                    <div>
                        <select
                            class={select_class}
                            onchange={on_cpu_vendor_change}
                            value={filter.cpu_vendor.unwrap_or_default()}
                        >
                            <option value="">{t.t("stats.filter.cpu_vendor")}</option>
                            <option value="Intel">{"Intel"}</option>
                            <option value="AMD">{"AMD"}</option>
                            <option value="ARM">{"ARM"}</option>
                        </select>
                    </div>
                    <div>
                        <select
                            class={select_class}
                            onchange={on_memory_capacity_change}
                        >
                            <option value="">{t.t("stats.filter.memory_capacity")}</option>
                            <option value="8gb">{"8GB"}</option>
                            <option value="16gb">{"16GB"}</option>
                            <option value="32gb">{"32GB"}</option>
                            <option value="64gb">{"64GB+"}</option>
                        </select>
                    </div>
                    <div>
                        <select
                            class={select_class}
                            onchange={on_gpu_vendor_change}
                            value={filter.gpu_vendor.unwrap_or_default()}
                        >
                            <option value="">{t.t("stats.filter.gpu_vendor")}</option>
                            <option value="NVIDIA">{"NVIDIA"}</option>
                            <option value="AMD">{"AMD"}</option>
                            <option value="Intel">{"Intel"}</option>
                        </select>
                    </div>
                    <div>
                        <select
                            class={select_class}
                            onchange={on_os_change}
                            value={filter.os_name.unwrap_or_default()}
                        >
                            <option value="">{t.t("stats.filter.os")}</option>
                            <option value="Linux">{"Linux"}</option>
                            <option value="Windows">{"Windows"}</option>
                            <option value="macOS">{"macOS"}</option>
                        </select>
                    </div>
                    <div>
                        <Button
                            variant={ButtonVariant::Outline}
                            size={ButtonSize::Sm}
                            class="w-full"
                            onclick={on_reset_click}
                        >
                            <RefreshCw class="mr-2 h-4 w-4" />
                            {t.t("common.reset")}
                        </Button>
                    </div>
                    <div>
                        <Button
                            variant={ButtonVariant::Default}
                            size={ButtonSize::Sm}
                            class="w-full"
                            onclick={on_export_click}
                        >
                            <Download class="mr-2 h-4 w-4" />
                            {t.t("common.export")}
                        </Button>
                    </div>
                </div>
            </CardContent>
        </Card>
    }
}
