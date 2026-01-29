use crate::components::chart::{get_chart_colors, ChartData, PieChart};
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::card::{Card, CardContent, CardDescription, CardHeader, CardTitle};
use crate::hooks::use_trans::use_trans;
use lucide_yew::Monitor;
use std::collections::HashMap;
use std::rc::Rc;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct OsDistCardProps {
    pub os_stats: HashMap<String, i32>,
    pub total_clients: usize,
}

#[function_component(OsDistCard)]
pub fn os_dist_card(props: &OsDistCardProps) -> Html {
    let os_stats = &props.os_stats;
    let t = use_trans();

    let chart_data: Rc<Vec<ChartData>> = use_memo((os_stats.clone(),), |(stats,)| {
        let colors = get_chart_colors();
        let mut data: Vec<ChartData> = stats
            .iter()
            .enumerate()
            .map(|(i, (label, value))| ChartData {
                label: label.clone(),
                value: *value as f64,
                color: colors[i % colors.len()].clone(),
            })
            .collect();
        // Sort by value descending for better visualization
        data.sort_by(|a, b| {
            b.value
                .partial_cmp(&a.value)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        data
    });

    html! {
        <Card class="h-full">
            <CardHeader>
                <div class="flex justify-between items-center">
                    <div>
                        <CardTitle>{t.t("dashboard.os_dist_title")}</CardTitle>
                        <CardDescription>{t.t("dashboard.os_dist_desc")}</CardDescription>
                    </div>
                    <Badge variant={BadgeVariant::Default}>{t.t("dashboard.realtime")}</Badge>
                </div>
            </CardHeader>
            <CardContent>
                {
                    if os_stats.is_empty() {
                        html! {
                            <div class="flex flex-col items-center justify-center py-10 text-muted-foreground">
                                <Monitor class="h-12 w-12 opacity-50" />
                                <p class="mt-2">{t.t("dashboard.no_data")}</p>
                            </div>
                        }
                    } else {
                        html! {
                            <div class="flex justify-center items-center py-4">
                                <PieChart
                                    data={(*chart_data).clone()}
                                    width={300}
                                    height={300}
                                />
                            </div>
                        }
                    }
                }
            </CardContent>
        </Card>
    }
}
