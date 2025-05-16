use yew::prelude::*;
use yew_router::prelude::*;

use crate::routes::{Route, switch};
use crate::components::layout::Layout;

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
