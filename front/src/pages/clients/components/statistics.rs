use yew::prelude::*;
use crate::types::Client;
use crate::pages::clients::utils::{calculate_os_counts, calculate_vendor_counts};
use crate::components::ui::card::{Card, CardContent};
use crate::hooks::use_trans::use_trans;
use lucide_yew::{Monitor, Search, Cpu, Building2};

#[derive(Properties, PartialEq)]
pub struct StatisticsProps {
    pub total_db_items: usize,
    pub filtered_total: usize,
    pub filtered_clients: Vec<Client>,
}

#[function_component(Statistics)]
pub fn statistics(props: &StatisticsProps) -> Html {
    let all_count = props.total_db_items;
    let filtered_count = props.filtered_total;
    let os_counts = calculate_os_counts(&props.filtered_clients);
    let vendor_counts = calculate_vendor_counts(&props.filtered_clients);
    let t = use_trans();

    html! {
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mb-6">
            <Card>
                <CardContent class="flex items-center justify-between p-6">
                    <div>
                        <p class="text-sm font-medium text-muted-foreground">{t.t("clients.stats.total_devices")}</p>
                        <div class="text-2xl font-bold text-primary">{all_count}</div>
                    </div>
                    <Monitor class="h-8 w-8 text-primary opacity-50" />
                </CardContent>
            </Card>

            <Card>
                <CardContent class="flex items-center justify-between p-6">
                    <div>
                        <p class="text-sm font-medium text-muted-foreground">{t.t("clients.stats.filtered_results")}</p>
                        <div class="text-2xl font-bold text-info">{filtered_count}</div>
                    </div>
                    <Search class="h-8 w-8 text-info opacity-50" />
                </CardContent>
            </Card>

            <Card>
                <CardContent class="flex items-center justify-between p-6">
                    <div>
                        <p class="text-sm font-medium text-muted-foreground">{t.t("clients.stats.os_types")}</p>
                        <div class="text-2xl font-bold text-success">{os_counts.len()}</div>
                    </div>
                    <Cpu class="h-8 w-8 text-success opacity-50" />
                </CardContent>
            </Card>

            <Card>
                <CardContent class="flex items-center justify-between p-6">
                    <div>
                        <p class="text-sm font-medium text-muted-foreground">{t.t("clients.stats.vendor_count")}</p>
                        <div class="text-2xl font-bold text-warning">{vendor_counts.len()}</div>
                    </div>
                    <Building2 class="h-8 w-8 text-warning opacity-50" />
                </CardContent>
            </Card>
        </div>
    }
} 