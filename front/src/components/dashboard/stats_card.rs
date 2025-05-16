use yew::prelude::*;
use crate::components::ui::card::{Card, CardContent};

#[derive(Properties, PartialEq)]
pub struct StatsCardProps {
    pub title: String,
    pub value: String,
    pub subtext: String,
    pub icon: Html,
}

#[function_component(StatsCard)]
pub fn stats_card(props: &StatsCardProps) -> Html {
    html! {
        <Card class="hover:shadow-md transition-shadow duration-200 border-primary/20 shadow-[0_0_10px_rgba(6,182,212,0.1)]">
            <CardContent class="p-6 flex flex-row items-center justify-between space-y-0">
                <div class="space-y-1">
                    <p class="text-sm font-medium text-muted-foreground">{&props.title}</p>
                    <div class="text-2xl font-bold text-primary">{&props.value}</div>
                    <p class="text-xs text-muted-foreground">{&props.subtext}</p>
                </div>
                <div class="h-8 w-8 text-primary opacity-80">
                    { props.icon.clone() }
                </div>
            </CardContent>
        </Card>
    }
}
