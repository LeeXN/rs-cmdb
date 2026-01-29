use crate::components::chart::{ChartData, PieChart};
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::card::{Card, CardContent, CardDescription, CardHeader, CardTitle};
use crate::hooks::use_trans::use_trans;
use lucide_yew::RefreshCw;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct SystemStatusCardProps {
    pub online_rate_display: String,
    pub online_clients: usize,
    pub offline_clients: usize,
}

#[function_component(SystemStatusCard)]
pub fn system_status_card(props: &SystemStatusCardProps) -> Html {
    let t = use_trans();

    let chart_data = vec![
        ChartData {
            label: t.t("dashboard.online_clients"),
            value: props.online_clients as f64,
            color: "#22c55e".to_string(), // green-500
        },
        ChartData {
            label: t.t("dashboard.offline_clients"),
            value: props.offline_clients as f64,
            color: "#64748b".to_string(), // slate-500
        },
    ];

    html! {
        <Card class="h-full">
            <CardHeader>
                <div class="flex justify-between items-center">
                    <div>
                        <CardTitle>{t.t("dashboard.system_status_title")}</CardTitle>
                        <CardDescription>{t.t("dashboard.realtime_refresh")}</CardDescription>
                    </div>
                    <Badge variant={BadgeVariant::Default} class="bg-green-500 hover:bg-green-600">
                        {format!("{} {}", t.t("dashboard.online_rate"), props.online_rate_display)}
                    </Badge>
                </div>
            </CardHeader>
            <CardContent>
                <div class="grid grid-cols-2 gap-4">
                    <div class="flex flex-col items-center justify-center rounded-lg border border-border bg-secondary/50 p-4">
                        <div class="text-sm font-medium text-muted-foreground">{t.t("dashboard.online_clients")}</div>
                        <div class="text-2xl font-bold text-green-500">{props.online_clients}</div>
                    </div>
                    <div class="flex flex-col items-center justify-center rounded-lg border border-border bg-secondary/50 p-4">
                        <div class="text-sm font-medium text-muted-foreground">{t.t("dashboard.offline_clients")}</div>
                        <div class="text-2xl font-bold text-muted-foreground">{props.offline_clients}</div>
                    </div>
                </div>

                <div class="mt-6 flex justify-center h-48">
                    <PieChart
                        data={chart_data}
                        width={200}
                        height={200}
                    />
                </div>

                <div class="flex items-center gap-2 mt-4 text-xs text-muted-foreground">
                    <RefreshCw class="h-3 w-3 animate-spin" />
                    <span>{t.t("dashboard.realtime_update")}</span>
                </div>
            </CardContent>
        </Card>
    }
}
