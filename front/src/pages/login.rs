use gloo_storage::Storage;
use js_sys::Date;
use log::error;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

use crate::components::ui::button::{Button, ButtonVariant};
use crate::hooks::use_trans::use_trans;
use crate::routes::Route;
use crate::services::api::login as api_login;
use crate::stores::auth_store::AuthStore;
use crate::types::LoginRequest;
use crate::utils::i18n_helper::translate_api_message;

#[function_component(Login)]
pub fn login() -> Html {
    let username = use_state(String::new);
    let password = use_state(String::new);
    let error_message = use_state(|| Option::<String>::None);
    let loading = use_state(|| false);

    let navigator = use_navigator().unwrap();
    let (_, dispatch) = use_store::<AuthStore>();
    let t = use_trans();

    let onsubmit = {
        let username = username.clone();
        let password = password.clone();
        let error_message = error_message.clone();
        let loading = loading.clone();
        let dispatch = dispatch.clone();
        let navigator = navigator.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            let username = username.clone();
            let password = password.clone();
            let error_message = error_message.clone();
            let loading = loading.clone();
            let dispatch = dispatch.clone();
            let navigator = navigator.clone();

            loading.set(true);
            error_message.set(None);

            wasm_bindgen_futures::spawn_local(async move {
                let request = LoginRequest {
                    username: (*username).clone(),
                    password: (*password).clone(),
                };

                match api_login(request).await {
                    Ok(response) => {
                        // Manually save to LocalStorage to ensure api.rs can find it immediately
                        // We use "auth_store" as the key because api.rs checks it first
                        let store = AuthStore {
                            token: Some(response.token.clone()),
                            user: Some(response.user.clone()),
                            is_authenticated: true,
                        };
                        if let Err(e) = gloo_storage::LocalStorage::set("auth_store", &store) {
                            error!("Failed to save auth token to LocalStorage: {}", e);
                        }

                        AuthStore::login(dispatch, response.token, response.user);
                        navigator.push(&Route::Home);
                    }
                    Err(err) => {
                        error_message.set(Some(translate_api_message(&err.message)));
                        loading.set(false);
                    }
                }
            });
        })
    };

    let oninput_username = {
        let username = username.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            username.set(input.value());
        })
    };

    let oninput_password = {
        let password = password.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            password.set(input.value());
        })
    };

    html! {
        <main class="min-h-screen flex items-center justify-center bg-slate-950 relative overflow-hidden">
            // Background effects
            <div class="absolute top-0 left-0 w-full h-full overflow-hidden z-0">
                <div class="absolute top-[-10%] left-[-10%] w-[40%] h-[40%] rounded-full bg-blue-600/10 blur-[100px]"></div>
                <div class="absolute bottom-[-10%] right-[-10%] w-[40%] h-[40%] rounded-full bg-purple-600/10 blur-[100px]"></div>
            </div>

            <div class="card w-full max-w-md bg-slate-900/80 backdrop-blur-xl shadow-2xl border border-slate-800 z-10 m-4">
                <div class="card-body p-8">
                    <div class="text-center mb-8">
                        <div class="w-16 h-16 rounded-2xl bg-gradient-to-br from-blue-600 to-purple-600 flex items-center justify-center shadow-lg shadow-blue-500/30 mx-auto mb-4">
                            <i class="material-icons text-3xl text-white">{"hub"}</i>
                        </div>
                        <h2 class="text-2xl font-bold text-white">{t.t("auth.login_title")}</h2>
                        <p class="text-slate-400 text-sm mt-2">{t.t("auth.login_title")}</p>
                    </div>

                    <form onsubmit={onsubmit} class="space-y-4">
                        <div class="form-control">
                            <label class="label">
                                <span class="label-text text-slate-300">{t.t("auth.username")}</span>
                            </label>
                            <div class="relative">
                                <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                                    <i class="fas fa-user text-slate-500"></i>
                                </div>
                                <input
                                    type="text"
                                    class="input input-bordered w-full pl-10 bg-slate-950/50 border-slate-700 focus:border-blue-500 focus:ring-1 focus:ring-blue-500 text-slate-200 placeholder-slate-500 transition-all"
                                    placeholder={t.t("auth.username")}
                                    oninput={oninput_username}
                                    value={(*username).clone()}
                                    required=true
                                />
                            </div>
                        </div>

                        <div class="form-control">
                            <label class="label">
                                <span class="label-text text-slate-300">{t.t("auth.password")}</span>
                            </label>
                            <div class="relative">
                                <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                                    <i class="fas fa-lock text-slate-500"></i>
                                </div>
                                <input
                                    type="password"
                                    class="input input-bordered w-full pl-10 bg-slate-950/50 border-slate-700 focus:border-blue-500 focus:ring-1 focus:ring-blue-500 text-slate-200 placeholder-slate-500 transition-all"
                                    placeholder={t.t("auth.password")}
                                    oninput={oninput_password}
                                    value={(*password).clone()}
                                    required=true
                                />
                            </div>
                        </div>

                        <div class="form-control">
                            <label class="label cursor-pointer justify-start gap-3">
                                <input type="checkbox" class="checkbox checkbox-primary checkbox-sm border-slate-600" id="rememberMe" checked=true />
                                <span class="label-text text-slate-400">{"Remember me"}</span>
                            </label>
                        </div>

                        if let Some(msg) = (*error_message).clone() {
                            <div class="alert alert-error shadow-lg text-sm py-2">
                                <i class="fas fa-exclamation-circle"></i>
                                <span>{msg}</span>
                            </div>
                        }

                        <Button type_="submit" variant={ButtonVariant::Default} class="w-full mt-6" disabled={*loading}>
                            if *loading {
                                <span class="loading loading-spinner loading-sm mr-2"></span>
                                {t.t("auth.logging_in")}
                            } else {
                                {t.t("auth.login_button")}
                            }
                        </Button>
                    </form>
                </div>
            </div>

            <footer class="absolute bottom-4 w-full text-center z-10">
                <p class="text-slate-500 text-xs">
                    {"© "} {Date::new_0().get_full_year()} {", made with "} <i class="fa fa-heart text-red-500 mx-1" aria-hidden="true"></i> {" by Rust CMDB Team."}
                </p>
            </footer>
        </main>
    }
}
