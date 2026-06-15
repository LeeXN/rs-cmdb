use chrono::Utc;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::card::{Card, CardContent, CardFooter, CardHeader};
use crate::icons::{ArrowRight, Calendar, Clock, Monitor};
use crate::routes::Route;
use crate::types::Client;
use crate::utils::format::{format_datetime, format_time_ago};

#[derive(Properties, PartialEq)]
pub struct ClientCardProps {
    #[prop_or_default]
    pub client: Client,
}

#[function_component(ClientCard)]
pub fn client_card(props: &ClientCardProps) -> Html {
    let last_seen_ago = props
        .client
        .last_seen
        .as_ref()
        .map_or("未知".to_string(), |time| format_time_ago(time));
    let last_seen_date = props
        .client
        .last_seen
        .as_ref()
        .map_or("未知".to_string(), |time| format_datetime(time));
    let registered_date = props
        .client
        .registered_at
        .as_ref()
        .map_or("未知".to_string(), |time| format_datetime(time));

    let is_online = props
        .client
        .last_seen
        .as_ref()
        .and_then(|time| chrono::DateTime::parse_from_rfc3339(time).ok())
        .map(|dt| {
            let now = Utc::now();
            let duration = now.signed_duration_since(dt.with_timezone(&Utc));
            duration.num_minutes() <= 5
        })
        .unwrap_or(false);

    let online_status = if is_online {
        html! {
            <Badge variant={BadgeVariant::Outline} class="animate-pulse bg-emerald-500/10 text-emerald-500 border-emerald-500/20">{"在线"}</Badge>
        }
    } else {
        html! {
            <Badge variant={BadgeVariant::Secondary}>{"离线"}</Badge>
        }
    };

    html! {
        <Card class="hover:border-cyan-500/50 transition-colors duration-300">
            <CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
                <div class="flex flex-col space-y-1">
                    <Link<Route> to={Route::ClientDetail { id: props.client.id.clone() }} classes="font-semibold text-lg hover:text-cyan-400 transition-colors">
                        { &props.client.hostname }
                    </Link<Route>>
                    <div class="flex items-center text-sm text-muted-foreground">
                        <Monitor class="mr-1 h-3 w-3" />
                        { props.client.primary_ip.as_ref().unwrap_or(&props.client.ip_address) }
                    </div>
                </div>
                <div>
                    { online_status }
                </div>
            </CardHeader>

            <CardContent class="pb-2">
                <div class="grid grid-cols-2 gap-4 text-sm">
                    <div class="flex flex-col space-y-1">
                        <span class="text-xs text-muted-foreground flex items-center">
                            <Calendar class="mr-1 h-3 w-3" />
                            {"注册时间"}
                        </span>
                        <span title={registered_date.clone()}>{ registered_date }</span>
                    </div>
                    <div class="flex flex-col space-y-1">
                        <span class="text-xs text-muted-foreground flex items-center">
                            <Clock class="mr-1 h-3 w-3" />
                            {"最后在线"}
                        </span>
                        <span title={last_seen_date.clone()}>{ last_seen_ago }</span>
                    </div>
                </div>
            </CardContent>

            <CardFooter class="pt-2">
                <Link<Route> to={Route::ClientDetail { id: props.client.id.clone() }} classes="w-full">
                    <Button variant={ButtonVariant::Outline} size={ButtonSize::Sm} class="w-full group">
                        {"查看详情"}
                        <ArrowRight class="ml-2 h-4 w-4 transition-transform group-hover:translate-x-1" />
                    </Button>
                </Link<Route>>
            </CardFooter>
        </Card>
    }
}
