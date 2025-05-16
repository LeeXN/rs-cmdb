use yew::prelude::*;
use yew_router::prelude::*;

use crate::pages::clients::ClientsPage;
use crate::pages::client_detail::ClientDetailPage;
use crate::pages::home::HomePage;
use crate::pages::client_setup::ClientSetupPage;
use crate::pages::analytics::AnalyticsPage;
use crate::pages::not_found::NotFoundPage;
use crate::pages::login::Login;
use crate::pages::persons::Persons;
use crate::pages::projects::Projects;
use crate::pages::components::Components;
use crate::pages::dictionaries::Dictionaries;
use crate::pages::racks::Racks;
use crate::pages::settings::change_password::ChangePassword;
use crate::pages::settings::users::Users;

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
        Route::NotFound => html! { <NotFoundPage /> },
    }
}