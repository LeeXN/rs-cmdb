use yew::prelude::*;
use yew_router::prelude::*;

use crate::routes::Route;

#[function_component(NotFoundPage)]
pub fn not_found_page() -> Html {
    html! {
        <div class="hero min-h-[70vh] bg-base-200 rounded-box">
            <div class="hero-content text-center">
                <div class="max-w-md">
                    <div class="text-9xl font-bold text-base-300 mb-4">
                        <i class="fas fa-map-signs"></i>
                    </div>
                    <h1 class="text-5xl font-bold text-primary mb-4">{"404"}</h1>
                    <h2 class="text-2xl font-semibold mb-6">{"页面未找到"}</h2>
                    <p class="py-6 text-base-content/70">
                        {"您请求的页面不存在或已被移除。请检查URL是否正确，或返回首页。"}
                    </p>
                    <Link<Route> to={Route::Home} classes="btn btn-primary">
                        <i class="fas fa-home mr-2"></i>
                        {"返回首页"}
                    </Link<Route>>
                </div>
            </div>
        </div>
    }
}
