use yew::prelude::*;
use std::collections::HashMap;
use crate::components::ui::card::{Card, CardHeader, CardTitle, CardDescription, CardContent};
use crate::components::ui::table::{Table, TableBody, TableRow, TableCell};
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::hooks::use_trans::use_trans;
use lucide_yew::{Monitor, Server, Terminal, Command, AppWindow};

#[derive(Properties, PartialEq)]
pub struct OsDistCardProps {
    pub os_stats: HashMap<String, i32>,
    pub total_clients: usize,
}

#[function_component(OsDistCard)]
pub fn os_dist_card(props: &OsDistCardProps) -> Html {
    let os_stats = &props.os_stats;
    let total_clients = props.total_clients;
    let t = use_trans();

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
                            <Table>
                                <TableBody>
                                    {
                                        os_stats.iter().map(|(os, count)| {
                                            let percentage = if total_clients > 0 {
                                                (*count as f64 / total_clients as f64) * 100.0
                                            } else {
                                                0.0
                                            };

                                            let os_lower = os.to_lowercase();
                                            // We don't have brand icons in Lucide, so we use generic ones or text
                                            let icon = if os_lower.contains("linux") || os_lower.contains("ubuntu") || os_lower.contains("centos") || os_lower.contains("debian") {
                                                html! { <Terminal class="h-5 w-5 text-orange-500" /> }
                                            } else if os_lower.contains("windows") {
                                                html! { <AppWindow class="h-5 w-5 text-blue-500" /> }
                                            } else if os_lower.contains("mac") || os_lower.contains("darwin") {
                                                html! { <Command class="h-5 w-5 text-slate-200" /> }
                                            } else {
                                                html! { <Server class="h-5 w-5 text-gray-500" /> }
                                            };

                                            html! {
                                                <TableRow>
                                                    <TableCell>
                                                        <div class="flex items-center gap-3">
                                                            {icon}
                                                            <div class="font-medium">{os}</div>
                                                        </div>
                                                    </TableCell>
                                                    <TableCell class="text-center font-bold">
                                                        {count}{t.t("dashboard.unit_machines")}
                                                    </TableCell>
                                                    <TableCell class="w-1/3">
                                                        <div class="flex flex-col gap-1">
                                                            <span class="text-xs font-bold text-right text-muted-foreground">{format!("{:.1}%", percentage)}</span>
                                                            <div class="h-2 w-full rounded-full bg-secondary">
                                                                <div class="h-2 rounded-full bg-primary" style={format!("width: {}%", percentage)}></div>
                                                            </div>
                                                        </div>
                                                    </TableCell>
                                                </TableRow>
                                            }
                                        }).collect::<Html>()
                                    }
                                </TableBody>
                            </Table>
                        }
                    }
                }
            </CardContent>
        </Card>
    }
}
