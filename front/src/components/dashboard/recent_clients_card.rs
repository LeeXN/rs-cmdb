use yew::prelude::*;
use crate::types::Client;
use crate::utils::format::format_time_ago;
use crate::components::ui::card::{Card, CardHeader, CardTitle, CardDescription, CardContent};
use crate::components::ui::table::{Table, TableBody, TableRow, TableCell};
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::hooks::use_trans::use_trans;
use lucide_yew::{Monitor, Server};

#[derive(Properties, PartialEq)]
pub struct RecentClientsCardProps {
    pub recent_clients: Vec<Client>,
}

#[function_component(RecentClientsCard)]
pub fn recent_clients_card(props: &RecentClientsCardProps) -> Html {
    let recent_clients = &props.recent_clients;
    let t = use_trans();

    html! {
        <Card class="h-full">
            <CardHeader>
                <div class="flex justify-between items-center">
                    <div>
                        <CardTitle>{t.t("dashboard.recent_active_title")}</CardTitle>
                        <CardDescription>{t.t("dashboard.recent_active_desc")}</CardDescription>
                    </div>
                    <Badge variant={BadgeVariant::Default} class="bg-blue-500 hover:bg-blue-600">{t.t("dashboard.realtime")}</Badge>
                </div>
            </CardHeader>
            <CardContent>
                {
                    if recent_clients.is_empty() {
                        html! {
                            <div class="flex flex-col items-center justify-center py-10 text-muted-foreground">
                                <Monitor class="h-12 w-12 opacity-50" />
                                <p class="mt-2">{t.t("dashboard.no_clients_registered")}</p>
                            </div>
                        }
                    } else {
                        html! {
                            <Table>
                                <TableBody>
                                    {
                                        recent_clients.iter().map(|client| {
                                            let is_online = client.last_seen.as_ref()
                                                .and_then(|last_seen| chrono::DateTime::parse_from_rfc3339(last_seen).ok())
                                                .map(|dt| {
                                                    let now = chrono::Utc::now();
                                                    let duration = now.signed_duration_since(dt.with_timezone(&chrono::Utc));
                                                    duration.num_minutes() <= 5
                                                })
                                                .unwrap_or(false);

                                            let (variant, status_text) = if is_online {
                                                (BadgeVariant::Default, t.t("dashboard.online")) // Default is primary (cyan/green)
                                            } else {
                                                (BadgeVariant::Secondary, t.t("dashboard.offline"))
                                            };

                                            html! {
                                                <TableRow>
                                                    <TableCell>
                                                        <div class="flex items-center gap-3">
                                                            <div class="flex h-10 w-10 items-center justify-center rounded-full bg-secondary text-secondary-foreground">
                                                                <Server class="h-5 w-5" />
                                                            </div>
                                                            <div>
                                                                <div class="font-medium whitespace-nowrap">{client.hostname.clone()}</div>
                                                                <div class="text-xs text-muted-foreground">{client.ip_address.clone()}</div>
                                                            </div>
                                                        </div>
                                                    </TableCell>
                                                    <TableCell>
                                                        <Badge variant={variant}>{status_text}</Badge>
                                                    </TableCell>
                                                    <TableCell class="text-right text-xs text-muted-foreground whitespace-nowrap">
                                                        {client.registered_at.as_ref().map(|t| format_time_ago(t)).unwrap_or(t.t("unknown"))}
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
