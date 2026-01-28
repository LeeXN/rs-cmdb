use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::card::{Card, CardContent, CardDescription, CardHeader, CardTitle};
use crate::components::ui::table::{Table, TableBody, TableCell, TableHead, TableHeader, TableRow};
use crate::hooks::use_trans::use_trans;
use crate::routes::Route;
use crate::types::Client;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ClientStatusListProps {
    pub clients: Vec<Client>,
    pub total_clients: usize,
}

#[function_component(ClientStatusList)]
pub fn client_status_list(props: &ClientStatusListProps) -> Html {
    let clients = &props.clients;
    let total_clients = props.total_clients;
    let t = use_trans();

    html! {
        <Card class="h-full">
            <CardHeader>
                <div class="flex justify-between items-center">
                    <div>
                        <CardTitle>{t.t("dashboard.client_status_list")}</CardTitle>
                        <CardDescription>{format!("{} {} {}", t.t("dashboard.total"), total_clients, t.t("dashboard.managed_nodes"))}</CardDescription>
                    </div>
                    <Link<Route> to={Route::Clients}>
                        <Button variant={ButtonVariant::Ghost} size={ButtonSize::Sm} class="text-primary">
                            {t.t("dashboard.view_all")}
                        </Button>
                    </Link<Route>>
                </div>
            </CardHeader>
            <CardContent>
                <Table>
                    <TableHeader>
                        <TableRow>
                            <TableHead>{t.t("dashboard.host")}</TableHead>
                            <TableHead>{t.t("dashboard.system")}</TableHead>
                            <TableHead class="text-center">{t.t("dashboard.config")}</TableHead>
                            <TableHead class="text-center">{t.t("dashboard.status")}</TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        {
                            clients.iter().take(6).map(|client| {
                                let is_online = client.last_seen.as_ref()
                                    .and_then(|last_seen| chrono::DateTime::parse_from_rfc3339(last_seen).ok())
                                    .map(|dt| {
                                        let now = chrono::Utc::now();
                                        let duration = now.signed_duration_since(dt.with_timezone(&chrono::Utc));
                                        duration.num_minutes() <= 5
                                    })
                                    .unwrap_or(false);

                                let (status_label, variant) = if is_online {
                                    (t.t("dashboard.online"), BadgeVariant::Default)
                                } else {
                                    (t.t("dashboard.offline"), BadgeVariant::Secondary)
                                };

                                html! {
                                    <TableRow>
                                        <TableCell>
                                            <div class="flex flex-col">
                                                <div class="font-medium">{client.hostname.clone()}</div>
                                                <div class="text-xs text-muted-foreground">{client.ip_address.clone()}</div>
                                            </div>
                                        </TableCell>
                                        <TableCell>
                                            <div class="font-medium text-xs">{client.os.clone().unwrap_or_else(|| t.t("unknown_system"))}</div>
                                            <div class="text-xs text-muted-foreground">{client.product_name.clone().unwrap_or_else(|| t.t("unknown_model"))}</div>
                                        </TableCell>
                                        <TableCell class="text-center text-xs font-medium text-muted-foreground">
                                            {client.serial_number.clone().unwrap_or_else(|| t.t("unknown"))}
                                        </TableCell>
                                        <TableCell class="text-center">
                                            <Badge variant={variant}>{status_label}</Badge>
                                        </TableCell>
                                    </TableRow>
                                }
                            }).collect::<Html>()
                        }
                    </TableBody>
                </Table>
            </CardContent>
        </Card>
    }
}
