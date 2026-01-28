use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::input::Input;
use crate::hooks::use_trans::use_trans;
use crate::routes::Route;
use crate::services::api::change_password;
use crate::stores::auth_store::AuthStore;
use crate::types::ChangePasswordRequest;
use crate::utils::i18n_helper::translate_api_message;
use lucide_yew::{Eye, EyeOff, X};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ChangePasswordModalProps {
    pub is_open: bool,
    pub on_close: Callback<()>,
}

#[function_component(ChangePasswordModal)]
pub fn change_password_modal(props: &ChangePasswordModalProps) -> Html {
    let old_password = use_state(String::new);
    let new_password = use_state(String::new);
    let confirm_password = use_state(String::new);
    let show_old_password = use_state(|| false);
    let show_new_password = use_state(|| false);
    let show_confirm_password = use_state(|| false);
    let error_message = use_state(|| Option::<String>::None);
    let success_message = use_state(|| Option::<String>::None);
    let is_loading = use_state(|| false);

    let navigator = use_navigator().unwrap();
    let (_, auth_dispatch) = use_store::<AuthStore>();
    let t = use_trans();

    let reset_and_close = {
        let on_close = props.on_close.clone();
        let old_password = old_password.clone();
        let new_password = new_password.clone();
        let confirm_password = confirm_password.clone();
        let show_old_password = show_old_password.clone();
        let show_new_password = show_new_password.clone();
        let show_confirm_password = show_confirm_password.clone();
        let error_message = error_message.clone();
        let success_message = success_message.clone();

        Callback::from(move |_: ()| {
            old_password.set(String::new());
            new_password.set(String::new());
            confirm_password.set(String::new());
            show_old_password.set(false);
            show_new_password.set(false);
            show_confirm_password.set(false);
            error_message.set(None);
            success_message.set(None);
            on_close.emit(());
        })
    };

    let on_close_click = {
        let reset_and_close = reset_and_close.clone();
        Callback::from(move |_: MouseEvent| {
            reset_and_close.emit(());
        })
    };

    let toggle_old_password = {
        let show = show_old_password.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            show.set(!*show);
        })
    };

    let toggle_new_password = {
        let show = show_new_password.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            show.set(!*show);
        })
    };

    let toggle_confirm_password = {
        let show = show_confirm_password.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            show.set(!*show);
        })
    };

    let on_submit = {
        let old_password = old_password.clone();
        let new_password = new_password.clone();
        let confirm_password = confirm_password.clone();
        let error_message = error_message.clone();
        let success_message = success_message.clone();
        let is_loading = is_loading.clone();
        let reset_and_close = reset_and_close.clone();
        let navigator = navigator.clone();
        let auth_dispatch = auth_dispatch.clone();
        let t = t.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            let old_pwd = (*old_password).clone();
            let new_pwd = (*new_password).clone();
            let confirm_pwd = (*confirm_password).clone();

            if new_pwd != confirm_pwd {
                error_message.set(Some(t.t("password.mismatch")));
                return;
            }

            if new_pwd.len() < 6 {
                error_message.set(Some(t.t("password.too_short")));
                return;
            }

            let is_loading = is_loading.clone();
            let error_message = error_message.clone();
            let success_message = success_message.clone();
            let reset_and_close = reset_and_close.clone();
            let navigator = navigator.clone();
            let auth_dispatch = auth_dispatch.clone();
            let t = t.clone();

            is_loading.set(true);
            error_message.set(None);

            spawn_local(async move {
                let request = ChangePasswordRequest {
                    old_password: old_pwd,
                    new_password: new_pwd,
                };

                match change_password(request).await {
                    Ok(_) => {
                        success_message.set(Some(t.t("password.success")));
                        is_loading.set(false);

                        // Wait a bit then logout and redirect
                        gloo_timers::future::TimeoutFuture::new(1500).await;

                        AuthStore::logout(auth_dispatch);
                        reset_and_close.emit(());
                        navigator.push(&Route::Login);
                    }
                    Err(err) => {
                        error_message.set(Some(translate_api_message(&err.message)));
                        is_loading.set(false);
                    }
                }
            });
        })
    };

    if !props.is_open {
        return html! {};
    }

    html! {
        <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
            <div class="w-full max-w-md rounded-lg border border-border bg-card p-6 shadow-lg animate-in fade-in zoom-in duration-200">
                <div class="flex items-center justify-between mb-6">
                    <h2 class="text-lg font-semibold">{t.t("password.change_title")}</h2>
                    <button onclick={on_close_click.clone()} class="text-muted-foreground hover:text-foreground">
                        <X class="h-5 w-5" />
                    </button>
                </div>

                if let Some(msg) = (*error_message).as_ref() {
                    <div class="mb-4 rounded-md bg-destructive/15 p-3 text-sm text-destructive">
                        {msg}
                    </div>
                }

                if let Some(msg) = (*success_message).as_ref() {
                    <div class="mb-4 rounded-md bg-green-500/15 p-3 text-sm text-green-500">
                        {msg}
                    </div>
                }

                <form onsubmit={on_submit} class="space-y-4">
                    <div class="space-y-2">
                        <label class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">
                            {t.t("password.current")}
                        </label>
                        <div class="relative">
                            <Input
                                type_={if *show_old_password { "text" } else { "password" }}
                                value={(*old_password).clone()}
                                oninput={Callback::from(move |val| old_password.set(val))}
                                required=true
                                class="pr-10"
                            />
                            <button
                                type="button"
                                onclick={toggle_old_password}
                                class="absolute right-0 top-0 h-full px-3 py-2 hover:bg-transparent text-muted-foreground hover:text-foreground"
                            >
                                if *show_old_password {
                                    <EyeOff class="h-4 w-4" />
                                } else {
                                    <Eye class="h-4 w-4" />
                                }
                            </button>
                        </div>
                    </div>

                    <div class="space-y-2">
                        <label class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">
                            {t.t("password.new")}
                        </label>
                        <div class="relative">
                            <Input
                                type_={if *show_new_password { "text" } else { "password" }}
                                value={(*new_password).clone()}
                                oninput={Callback::from(move |val| new_password.set(val))}
                                required=true
                                class="pr-10"
                            />
                            <button
                                type="button"
                                onclick={toggle_new_password}
                                class="absolute right-0 top-0 h-full px-3 py-2 hover:bg-transparent text-muted-foreground hover:text-foreground"
                            >
                                if *show_new_password {
                                    <EyeOff class="h-4 w-4" />
                                } else {
                                    <Eye class="h-4 w-4" />
                                }
                            </button>
                        </div>
                    </div>

                    <div class="space-y-2">
                        <label class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">
                            {t.t("password.confirm")}
                        </label>
                        <div class="relative">
                            <Input
                                type_={if *show_confirm_password { "text" } else { "password" }}
                                value={(*confirm_password).clone()}
                                oninput={Callback::from(move |val| confirm_password.set(val))}
                                required=true
                                class="pr-10"
                            />
                            <button
                                type="button"
                                onclick={toggle_confirm_password}
                                class="absolute right-0 top-0 h-full px-3 py-2 hover:bg-transparent text-muted-foreground hover:text-foreground"
                            >
                                if *show_confirm_password {
                                    <EyeOff class="h-4 w-4" />
                                } else {
                                    <Eye class="h-4 w-4" />
                                }
                            </button>
                        </div>
                    </div>

                    <div class="flex justify-end gap-2 pt-2">
                        <Button
                            variant={ButtonVariant::Outline}
                            type_="button"
                            onclick={on_close_click}
                            disabled={*is_loading}
                        >
                            {"Cancel"}
                        </Button>
                        <Button
                            type_="submit"
                            disabled={*is_loading}
                        >
                            if *is_loading {
                                <span class="mr-2 h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent" />
                            }
                            {t.t("password.submit")}
                        </Button>
                    </div>
                </form>
            </div>
        </div>
    }
}
