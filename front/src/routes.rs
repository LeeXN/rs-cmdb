use yew::prelude::*;
use yew_router::prelude::*;

use crate::pages::analytics::AnalyticsPage;
use crate::pages::client_detail::ClientDetailPage;
use crate::pages::client_setup::ClientSetupPage;
use crate::pages::clients::ClientsPage;
use crate::pages::components::Components;
use crate::pages::dictionaries::Dictionaries;
use crate::pages::home::HomePage;
use crate::pages::login::Login;
use crate::pages::not_found::NotFoundPage;
use crate::pages::persons::Persons;
use crate::pages::projects::Projects;
use crate::pages::racks::Racks;
use crate::pages::settings::change_password::ChangePassword;
use crate::pages::settings::users::Users;
use crate::pages::terminal::TerminalPage;
use crate::pages::execution::batch::BatchExecPage;
use crate::pages::execution::history::HistoryPage;
use crate::pages::execution::replay::ReplayPage;
use crate::pages::permissions::exec_policies::ExecPoliciesPage;
use crate::pages::permissions::web_terminal_policies::WebTerminalPoliciesPage;
use crate::pages::permissions::approvals::ApprovalsPage;
use crate::pages::permissions::manage::PermissionManagePage;
use crate::pages::settings::remote_exec::RemoteExec;

#[derive(Debug, Clone, PartialEq, Routable)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/login")]
    Login,
    #[at("/clients")]
    Clients,
    #[at("/clients/:id")]
    ClientDetail { id: String },
    #[at("/client-setup")]
    ClientSetup,
    #[at("/analytics")]
    Analytics,
    #[at("/users")]
    Persons,
    #[at("/projects")]
    Projects,
    #[at("/racks")]
    Racks,
    #[at("/components")]
    Components,
    #[at("/base-data")]
    BaseData,
    #[at("/accounts")]
    Accounts,
    #[at("/settings/change-password")]
    ChangePassword,
    #[at("/execution/batch")]
    ExecutionBatch,
    #[at("/execution/history")]
    ExecutionHistory,
    #[at("/execution/replay/:id")]
    ExecutionReplay { id: String },
    #[at("/settings/remote-exec")]
    SettingsRemoteExec,
    #[at("/clients/:id/terminal")]
    Terminal { id: String },
    #[at("/permissions/exec-policies")]
    ExecPolicies,
    #[at("/permissions/web-terminal-policies")]
    WebTerminalPolicies,
    #[at("/permissions/approvals")]
    Approvals,
    #[at("/permissions/manage")]
    PermissionManage,
    #[not_found]
    #[at("/404")]
    NotFound,
}

pub fn switch(route: Route) -> Html {
    match route {
        Route::Home => html! { <HomePage /> },
        Route::Login => html! { <Login /> },
        Route::Clients => html! { <ClientsPage /> },
        Route::ClientDetail { id } => html! { <ClientDetailPage client_id={id} /> },
        Route::ClientSetup => html! { <ClientSetupPage /> },
        Route::Analytics => html! { <AnalyticsPage /> },
        Route::Persons => html! { <Persons /> },
        Route::Projects => html! { <Projects /> },
        Route::Racks => html! { <Racks /> },
        Route::Components => html! { <Components /> },
        Route::BaseData => html! { <Dictionaries /> },
        Route::Accounts => html! { <Users /> },
        Route::ChangePassword => html! { <ChangePassword /> },
        Route::ExecutionBatch => html! { <BatchExecPage /> },
        Route::ExecutionHistory => html! { <HistoryPage /> },
        Route::ExecutionReplay { id } => html! { <ReplayPage session_id={id} /> },
        Route::SettingsRemoteExec => html! { <RemoteExec /> },
        Route::Terminal { id } => html! { <TerminalPage client_id={id} /> },
        Route::ExecPolicies => html! { <ExecPoliciesPage /> },
        Route::WebTerminalPolicies => html! { <WebTerminalPoliciesPage /> },
        Route::Approvals => html! { <ApprovalsPage /> },
        Route::PermissionManage => html! { <PermissionManagePage /> },
        Route::NotFound => html! { <NotFoundPage /> },
    }
}
