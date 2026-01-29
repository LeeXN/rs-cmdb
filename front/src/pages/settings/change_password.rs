use crate::components::ui::button::{Button, ButtonVariant};
use crate::hooks::use_trans::use_trans;
use crate::services::api;
use crate::types::ChangePasswordRequest;
use crate::utils::i18n_helper::translate_api_message;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[function_component(ChangePassword)]
pub fn change_password() -> Html {
    let old_password = use_state(String::new);
    let new_password = use_state(String::new);
    let confirm_password = use_state(String::new);
    let error_message = use_state(|| Option::<String>::None);
    let success_message = use_state(|| Option::<String>::None);
    let loading = use_state(|| false);
    let t = use_trans();

    let onsubmit = {
        let old_password = old_password.clone();
        let new_password = new_password.clone();
        let confirm_password = confirm_password.clone();
        let error_message = error_message.clone();
        let success_message = success_message.clone();
        let loading = loading.clone();
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

            let loading = loading.clone();
            let error_message = error_message.clone();
            let success_message = success_message.clone();
            let old_password = old_password.clone();
            let new_password = new_password.clone();
            let confirm_password = confirm_password.clone();
            let t = t.clone();

            loading.set(true);
            error_message.set(None);
            success_message.set(None);

            spawn_local(async move {
                let request = ChangePasswordRequest {
                    old_password: old_pwd,
                    new_password: new_pwd,
                };

                match api::change_password(request).await {
                    Ok(_) => {
                        success_message.set(Some(t.t("password.success")));
                        old_password.set(String::new());
                        new_password.set(String::new());
                        confirm_password.set(String::new());
                    }
                    Err(err) => {
                        error_message.set(Some(translate_api_message(&err.message)));
                    }
                }
                loading.set(false);
            });
        })
    };

    html! {
        <>
            <div class="container-fluid py-4">
                <div class="row">
                    <div class="col-12 col-md-6 mx-auto">
                        <div class="card">
                            <div class="card-header p-0 position-relative mt-n4 mx-3 z-index-2">
                                <div class="bg-gradient-primary shadow-primary border-radius-lg pt-4 pb-3">
                                    <h6 class="text-white text-capitalize ps-3">{t.t("password.change_title")}</h6>
                                </div>
                            </div>
                            <div class="card-body">
                                if let Some(msg) = (*error_message).clone() {
                                    <div class="alert alert-danger text-white" role="alert">
                                        {msg}
                                    </div>
                                }
                                if let Some(msg) = (*success_message).clone() {
                                    <div class="alert alert-success text-white" role="alert">
                                        {msg}
                                    </div>
                                }
                                <form {onsubmit}>
                                    <div class="input-group input-group-outline mb-3">
                                        <label class="form-label">{t.t("password.current")}</label>
                                        <input type="password" class="form-control"
                                            value={(*old_password).clone()}
                                            oninput={Callback::from(move |e: InputEvent| {
                                                let input: HtmlInputElement = e.target_unchecked_into();
                                                old_password.set(input.value());
                                            })}
                                            required=true
                                        />
                                    </div>
                                    <div class="input-group input-group-outline mb-3">
                                        <label class="form-label">{t.t("password.new")}</label>
                                        <input type="password" class="form-control"
                                            value={(*new_password).clone()}
                                            oninput={Callback::from(move |e: InputEvent| {
                                                let input: HtmlInputElement = e.target_unchecked_into();
                                                new_password.set(input.value());
                                            })}
                                            required=true
                                        />
                                    </div>
                                    <div class="input-group input-group-outline mb-3">
                                        <label class="form-label">{t.t("password.confirm")}</label>
                                        <input type="password" class="form-control"
                                            value={(*confirm_password).clone()}
                                            oninput={Callback::from(move |e: InputEvent| {
                                                let input: HtmlInputElement = e.target_unchecked_into();
                                                confirm_password.set(input.value());
                                            })}
                                            required=true
                                        />
                                    </div>
                                    <div class="text-center">
                                        <Button
                                            variant={ButtonVariant::Default}
                                            type_="submit"
                                            class={classes!("w-full", "my-4", "mb-2")}
                                            disabled={*loading}
                                        >
                                            if *loading {
                                                <span class="loading loading-spinner loading-sm mr-2"></span>
                                                {t.t("password.submitting")}
                                            } else {
                                                {t.t("password.submit")}
                                            }
                                        </Button>
                                    </div>
                                </form>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </>
    }
}
