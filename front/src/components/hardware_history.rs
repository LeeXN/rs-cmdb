use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::card::{Card, CardContent, CardHeader, CardTitle};
use crate::components::ui::table::{Table, TableBody, TableCell, TableHead, TableHeader, TableRow};
use crate::services::api::{fetch_hardware_history, ApiError};
use crate::types::Hardware;
use lucide_yew::{
    CircleAlert, CircleMinus, CirclePlus, Eye, History, Pencil, RefreshCw, TrendingDown,
    TrendingUp, X,
};
use wasm_bindgen_futures::spawn_local;
use web_sys::console;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct HardwareHistoryProps {
    pub client_id: String,
}

pub enum HardwareHistoryMsg {
    LoadHistory,
    HistoryLoaded(Result<Vec<(String, Hardware)>, ApiError>),
    SelectHistory(usize),
}

pub struct HardwareHistory {
    history: Vec<(String, Hardware)>,
    loading: bool,
    error: Option<String>,
    selected_index: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct HardwareChange {
    pub component: String,
    pub change_type: ChangeType,
    pub old_value: String,
    pub new_value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ChangeType {
    Added,
    Removed,
    Modified,
    Upgraded,
    Downgraded,
}

impl Component for HardwareHistory {
    type Message = HardwareHistoryMsg;
    type Properties = HardwareHistoryProps;

    fn create(ctx: &Context<Self>) -> Self {
        // 自动加载历史记录
        ctx.link().send_message(HardwareHistoryMsg::LoadHistory);

        Self {
            history: Vec::new(),
            loading: true,
            error: None,
            selected_index: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            HardwareHistoryMsg::LoadHistory => {
                self.loading = true;
                self.error = None;

                let client_id = ctx.props().client_id.clone();
                let link = ctx.link().clone();

                spawn_local(async move {
                    let result = fetch_hardware_history(&client_id).await;
                    link.send_message(HardwareHistoryMsg::HistoryLoaded(result));
                });

                true
            }
            HardwareHistoryMsg::HistoryLoaded(result) => {
                self.loading = false;

                match result {
                    Ok(history) => {
                        self.history = history;
                        self.error = None;
                    }
                    Err(err) => {
                        let error_message = err.message.clone();
                        self.error = Some(err.message);
                        console::error_1(&format!("加载硬件历史失败: {}", error_message).into());
                    }
                }

                true
            }
            HardwareHistoryMsg::SelectHistory(index) => {
                self.selected_index = Some(index);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <Card>
                <CardHeader>
                    <div class="flex justify-between items-center">
                        <CardTitle>{"硬件变更历史"}</CardTitle>
                        <Button
                            variant={ButtonVariant::Outline}
                            size={ButtonSize::Sm}
                            onclick={ctx.link().callback(|_| HardwareHistoryMsg::LoadHistory)}
                            disabled={self.loading}
                        >
                            {if self.loading {
                                html! { <RefreshCw class="h-4 w-4 mr-2 animate-spin" /> }
                            } else {
                                html! { <RefreshCw class="h-4 w-4 mr-2" /> }
                            }}
                            {if self.loading { "加载中..." } else { "刷新" }}
                        </Button>
                    </div>
                </CardHeader>
                <CardContent>
                    {self.render_content(ctx)}
                </CardContent>
            </Card>
        }
    }
}

impl HardwareHistory {
    fn render_content(&self, ctx: &Context<Self>) -> Html {
        if self.loading {
            return html! {
                <div class="flex flex-col items-center justify-center p-8 text-muted-foreground">
                    <RefreshCw class="h-8 w-8 animate-spin mb-2" />
                    <p>{"正在加载硬件历史..."}</p>
                </div>
            };
        }

        if let Some(error) = &self.error {
            return html! {
                <div class="rounded-md bg-destructive/15 p-4 text-destructive flex items-center gap-2 mb-4">
                    <CircleAlert class="h-5 w-5" />
                    <span>{"错误: "}{error}</span>
                </div>
            };
        }

        if self.history.is_empty() {
            return html! {
                <div class="flex flex-col items-center justify-center p-8 text-muted-foreground">
                    <History class="h-12 w-12 mb-2 opacity-50" />
                    <p>{"暂无硬件变更历史"}</p>
                </div>
            };
        }

        html! {
            <div class="space-y-6">
                <div class="rounded-md border">
                    <Table>
                        <TableHeader>
                            <TableRow>
                                <TableHead>{"时间"}</TableHead>
                                <TableHead>{"变更内容"}</TableHead>
                                <TableHead>{"变更类型"}</TableHead>
                                <TableHead>{"操作"}</TableHead>
                            </TableRow>
                        </TableHeader>
                        <TableBody>
                            {
                                self.history.iter().enumerate().map(|(index, (timestamp, hardware))| {
                                    self.render_history_row(ctx, index, timestamp, hardware)
                                }).collect::<Html>()
                            }
                        </TableBody>
                    </Table>
                </div>

                {self.render_selected_details(ctx)}
            </div>
        }
    }

    fn render_history_row(
        &self,
        ctx: &Context<Self>,
        index: usize,
        timestamp: &str,
        hardware: &Hardware,
    ) -> Html {
        let is_selected = self.selected_index == Some(index);
        let row_class = if is_selected { "bg-muted/50" } else { "" };

        // 格式化时间戳
        let formatted_time = self.format_timestamp(timestamp);

        // 计算与前一个版本的差异
        let changes = if index < self.history.len() - 1 {
            let previous_hardware = &self.history[index + 1].1;
            self.calculate_hardware_changes(previous_hardware, hardware)
        } else {
            vec![HardwareChange {
                component: "初始版本".to_string(),
                change_type: ChangeType::Added,
                old_value: "".to_string(),
                new_value: "首次记录".to_string(),
            }]
        };

        html! {
            <TableRow class={row_class}>
                <TableCell>
                    <div class="flex flex-col">
                        <span class="font-medium">{formatted_time.0}</span>
                        <span class="text-xs text-muted-foreground">{formatted_time.1}</span>
                    </div>
                </TableCell>
                <TableCell>
                    <div class="flex flex-col gap-1">
                        {
                            changes.iter().take(3).map(|change| {
                                html! {
                                    <span class="text-xs">
                                        <strong class="font-medium">{&change.component}{":"}</strong>
                                        {" "}
                                        {self.format_change_description(change)}
                                    </span>
                                }
                            }).collect::<Html>()
                        }
                        {
                            if changes.len() > 3 {
                                html! {
                                    <span class="text-xs text-muted-foreground">
                                        {format!("... 还有 {} 项变更", changes.len() - 3)}
                                    </span>
                                }
                            } else {
                                html! {}
                            }
                        }
                    </div>
                </TableCell>
                <TableCell>
                    {self.render_change_badges(&changes)}
                </TableCell>
                <TableCell>
                    <Button
                        variant={ButtonVariant::Ghost}
                        size={ButtonSize::Icon}
                        onclick={ctx.link().callback(move |_| HardwareHistoryMsg::SelectHistory(index))}
                        title="查看详情"
                    >
                        <Eye class="h-4 w-4" />
                    </Button>
                </TableCell>
            </TableRow>
        }
    }

    fn render_change_badges(&self, changes: &[HardwareChange]) -> Html {
        let mut change_types = std::collections::HashMap::new();
        for change in changes {
            *change_types.entry(&change.change_type).or_insert(0) += 1;
        }

        html! {
            <div class="flex flex-wrap gap-1">
                {
                    change_types.iter().map(|(change_type, count)| {
                        let (variant, icon, text) = match change_type {
                            ChangeType::Added => (BadgeVariant::Success, html! { <CirclePlus class="h-3 w-3 mr-1" /> }, "新增"),
                            ChangeType::Removed => (BadgeVariant::Destructive, html! { <CircleMinus class="h-3 w-3 mr-1" /> }, "移除"),
                            ChangeType::Modified => (BadgeVariant::Info, html! { <Pencil class="h-3 w-3 mr-1" /> }, "修改"),
                            ChangeType::Upgraded => (BadgeVariant::Default, html! { <TrendingUp class="h-3 w-3 mr-1" /> }, "升级"),
                            ChangeType::Downgraded => (BadgeVariant::Warning, html! { <TrendingDown class="h-3 w-3 mr-1" /> }, "降级"),
                        };

                        html! {
                            <Badge variant={variant} class="flex items-center">
                                {icon}
                                {format!("{} ({})", text, count)}
                            </Badge>
                        }
                    }).collect::<Html>()
                }
            </div>
        }
    }

    fn render_selected_details(&self, ctx: &Context<Self>) -> Html {
        if let Some(index) = self.selected_index {
            if let Some((timestamp, hardware)) = self.history.get(index) {
                let formatted_time = self.format_timestamp(timestamp);

                // 计算详细变更
                let changes = if index < self.history.len() - 1 {
                    let previous_hardware = &self.history[index + 1].1;
                    self.calculate_hardware_changes(previous_hardware, hardware)
                } else {
                    vec![]
                };

                return html! {
                    <Card class="mt-6 bg-muted/30 border-dashed">
                        <CardHeader>
                            <div class="flex justify-between items-center">
                                <CardTitle class="text-lg">{format!("硬件详情 - {}", formatted_time.0)}</CardTitle>
                                <Button
                                    variant={ButtonVariant::Ghost}
                                    size={ButtonSize::Icon}
                                    onclick={ctx.link().callback(|_| HardwareHistoryMsg::SelectHistory(usize::MAX))}
                                >
                                    <X class="h-4 w-4" />
                                </Button>
                            </div>
                        </CardHeader>
                        <CardContent>
                            {
                                if !changes.is_empty() {
                                    html! {
                                        <div class="mb-6">
                                            <h4 class="text-sm font-semibold uppercase text-muted-foreground mb-3">{"本次变更详情"}</h4>
                                            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                                {
                                                    changes.iter().map(|change| {
                                                        self.render_change_detail(change)
                                                    }).collect::<Html>()
                                                }
                                            </div>
                                            <div class="my-4 border-t"></div>
                                        </div>
                                    }
                                } else {
                                    html! {}
                                }
                            }
                            {self.render_hardware_summary(hardware)}
                        </CardContent>
                    </Card>
                };
            }
        }

        html! {}
    }

    fn render_change_detail(&self, change: &HardwareChange) -> Html {
        let (icon, bg_class, text_class) = match change.change_type {
            ChangeType::Added => (
                html! { <CirclePlus class="h-5 w-5" /> },
                "bg-green-500/10",
                "text-green-500",
            ),
            ChangeType::Removed => (
                html! { <CircleMinus class="h-5 w-5" /> },
                "bg-red-500/10",
                "text-red-500",
            ),
            ChangeType::Modified => (
                html! { <Pencil class="h-5 w-5" /> },
                "bg-blue-500/10",
                "text-blue-500",
            ),
            ChangeType::Upgraded => (
                html! { <TrendingUp class="h-5 w-5" /> },
                "bg-primary/10",
                "text-primary",
            ),
            ChangeType::Downgraded => (
                html! { <TrendingDown class="h-5 w-5" /> },
                "bg-yellow-500/10",
                "text-yellow-500",
            ),
        };

        html! {
            <div class="rounded-lg border bg-card p-4 shadow-sm">
                <div class="flex items-center gap-3">
                    <div class={format!("p-2 rounded-lg {} {}", bg_class, text_class)}>
                        {icon}
                    </div>
                    <div class="flex-grow">
                        <h6 class="font-medium text-sm">{&change.component}</h6>
                        <p class="text-xs text-muted-foreground">{self.format_change_description(change)}</p>
                    </div>
                </div>
            </div>
        }
    }

    fn render_hardware_summary(&self, hardware: &Hardware) -> Html {
        html! {
            <div>
                <h4 class="text-sm font-semibold uppercase text-muted-foreground mb-3">{"硬件配置快照"}</h4>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                    <div>
                        <h5 class="text-xs font-medium uppercase text-muted-foreground mb-2">{"处理器"}</h5>
                        <div class="rounded-lg border bg-card p-3 mb-4">
                            <div class="flex flex-col">
                                <h6 class="font-medium text-sm">{&hardware.cpu.model_name}</h6>
                                <span class="text-xs text-muted-foreground">{format!("{} | {}核{}线程 | {}MHz",
                                    hardware.cpu.vendor_id,
                                    hardware.cpu.cores,
                                    hardware.cpu.threads,
                                    hardware.cpu.speed)}</span>
                            </div>
                        </div>

                        <h5 class="text-xs font-medium uppercase text-muted-foreground mb-2">{"内存"}</h5>
                        <div class="rounded-lg border bg-card p-3 mb-4">
                            <div class="flex flex-col">
                                <h6 class="font-medium text-sm">{format!("{}GB", hardware.ram.total_size)}</h6>
                                <span class="text-xs text-muted-foreground">{format!("{} | {}根内存条 | {}MHz",
                                    hardware.ram.vendor,
                                    hardware.ram.count,
                                    hardware.ram.speed)}</span>
                            </div>
                        </div>
                    </div>

                    <div>
                        <h5 class="text-xs font-medium uppercase text-muted-foreground mb-2">{"存储设备"}</h5>
                        <div class="flex flex-col gap-2 mb-4">
                            {
                                hardware.disks.iter().take(3).map(|disk| {
                                    html! {
                                        <div class="rounded-lg border bg-card p-3">
                                            <div class="flex flex-col">
                                                <h6 class="font-medium text-sm">{&disk.model}</h6>
                                                <span class="text-xs text-muted-foreground">{format!("{} | {}GB | {:?}",
                                                    disk.vendor,
                                                    disk.size,
                                                    disk.storage_type)}</span>
                                            </div>
                                        </div>
                                    }
                                }).collect::<Html>()
                            }
                            {
                                if hardware.disks.len() > 3 {
                                    html! {
                                        <div class="text-xs text-center text-muted-foreground">
                                            {format!("... 还有 {} 个存储设备", hardware.disks.len() - 3)}
                                        </div>
                                    }
                                } else {
                                    html! {}
                                }
                            }
                        </div>

                        <h5 class="text-xs font-medium uppercase text-muted-foreground mb-2">{"显卡"}</h5>
                        <div class="flex flex-col gap-2">
                            {
                                if hardware.gpus.is_empty() {
                                    html! {
                                        <div class="text-xs text-muted-foreground p-2">
                                            {"无独立显卡"}
                                        </div>
                                    }
                                } else {
                                    hardware.gpus.iter().map(|gpu| {
                                        html! {
                                            <div class="rounded-lg border bg-card p-3">
                                                <div class="flex flex-col">
                                                    <h6 class="font-medium text-sm">{&gpu.model}</h6>
                                                    <span class="text-xs text-muted-foreground">{format!("{} | 设备ID: {}",
                                                        gpu.vendor,
                                                        gpu.device_id)}</span>
                                                </div>
                                            </div>
                                        }
                                    }).collect::<Html>()
                                }
                            }
                        </div>
                    </div>
                </div>
            </div>
        }
    }

    fn format_timestamp(&self, timestamp: &str) -> (String, String) {
        // 尝试解析时间戳
        if let Ok(ts) = timestamp.parse::<i64>() {
            if let Some(datetime) = chrono::DateTime::from_timestamp(ts, 0) {
                let local_time = datetime.with_timezone(&chrono::Local);
                let date = local_time.format("%Y-%m-%d").to_string();
                let time = local_time.format("%H:%M:%S").to_string();
                return (date, time);
            }
        }

        // 如果解析失败，返回原始时间戳
        (timestamp.to_string(), "".to_string())
    }

    fn calculate_hardware_changes(&self, old: &Hardware, new: &Hardware) -> Vec<HardwareChange> {
        let mut changes = Vec::new();

        // CPU Changes
        if old.cpu.model_name != new.cpu.model_name {
            changes.push(HardwareChange {
                component: "CPU".to_string(),
                change_type: ChangeType::Modified,
                old_value: old.cpu.model_name.clone(),
                new_value: new.cpu.model_name.clone(),
            });
        }

        // RAM Changes
        if old.ram.total_size != new.ram.total_size {
            let change_type = if new.ram.total_size > old.ram.total_size {
                ChangeType::Upgraded
            } else {
                ChangeType::Downgraded
            };
            changes.push(HardwareChange {
                component: "内存容量".to_string(),
                change_type,
                old_value: format!("{} GB", old.ram.total_size),
                new_value: format!("{} GB", new.ram.total_size),
            });
        }

        // Disk Changes (Simplified check)
        if old.disks.len() != new.disks.len() {
            let change_type = if new.disks.len() > old.disks.len() {
                ChangeType::Added
            } else {
                ChangeType::Removed
            };
            changes.push(HardwareChange {
                component: "硬盘数量".to_string(),
                change_type,
                old_value: format!("{} 个", old.disks.len()),
                new_value: format!("{} 个", new.disks.len()),
            });
        }

        // GPU Changes (Simplified check)
        if old.gpus.len() != new.gpus.len() {
            let change_type = if new.gpus.len() > old.gpus.len() {
                ChangeType::Added
            } else {
                ChangeType::Removed
            };
            changes.push(HardwareChange {
                component: "显卡数量".to_string(),
                change_type,
                old_value: format!("{} 个", old.gpus.len()),
                new_value: format!("{} 个", new.gpus.len()),
            });
        }

        changes
    }

    fn format_change_description(&self, change: &HardwareChange) -> String {
        match change.change_type {
            ChangeType::Added => format!("新增: {}", change.new_value),
            ChangeType::Removed => format!("移除: {}", change.old_value),
            ChangeType::Modified => format!("{} -> {}", change.old_value, change.new_value),
            ChangeType::Upgraded => format!("升级: {} -> {}", change.old_value, change.new_value),
            ChangeType::Downgraded => format!("降级: {} -> {}", change.old_value, change.new_value),
        }
    }
}
