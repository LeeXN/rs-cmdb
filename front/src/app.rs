use yew::prelude::*;
use yew_router::prelude::*;

use crate::components::layout::Layout;
use crate::routes::{switch, Route};

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <BrowserRouter>
            <Layout>
                <Switch<Route> render={switch} />
            </Layout>
        </BrowserRouter>
    }
}
