use crate::components::error::ErrorDisplay;
use crate::components::loading::Loading;
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::card::{Card, CardBody, CardHeader, CardTitle};
use crate::components::ui::checkbox::Checkbox;
use crate::components::ui::confirm_modal::ConfirmModal;
use crate::components::ui::input::Input;
use crate::components::ui::select::{Select, SelectOption};
use crate::components::ui::table::{Table, TableBody, TableCell, TableHead, TableHeader, TableRow};
use crate::components::ui::table_action::TableActions;
use crate::hooks::use_trans::use_trans;
use crate::routes::Route;
use crate::services::api;
use crate::stores::auth_store::AuthStore;
use crate::types::{CreateUserRequest, Role, UpdateUserRequest, User};
use crate::utils::i18n_helper::translate_api_message;
use gloo::utils::document;
use lucide_yew::{Plus, Shield, User as UserIcon, X};
use wasm_bindgen_futures::spawn_local;
use yew::create_portal;
use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

#[function_component(Users)]
pub fn users() -> Html {
    let (auth_store, _) = use_store::<AuthStore>();
    let navigator = use_navigator().unwrap();
    let t = use_trans();

    {
        let auth_store = auth_store.clone();
        let navigator = navigator.clone();
        use_effect_with(auth_store, move |auth_store| {
            if let Some(user) = &auth_store.user {
                if user.role != Role::Admin {
                    navigator.push(&Route::Home);
                }
            }
            || ()
        });
    }

    let users = use_state(Vec::<User>::new);
    let error_message = use_state(|| Option::<String>::None);
    let success_message = use_state(|| Option::<String>::None);
    let loading = use_state(|| false);

    // Modal state
    let show_modal = use_state(|| false);
    let is_edit = use_state(|| false);
    let current_user_id = use_state(String::new);
    let delete_modal_open = use_state(|| false);
    let user_to_delete = use_state(|| None::<String>);

    // Form state
    let form_username = use_state(String::new);
    let form_password = use_state(String::new);
    let form_role = use_state(|| Role::Viewer);
    let form_is_active = use_state(|| true);

    let fetch_users = {
        let users = users.clone();
        let error_message = error_message.clone();
        let loading = loading.clone();

        Callback::from(move |_| {
            let users = users.clone();
            let error_message = error_message.clone();
            let loading = loading.clone();

            loading.set(true);
            spawn_local(async move {
                match api::fetch_users().await {
                    Ok(data) => users.set(data),
                    Err(err) => error_message.set(Some(translate_api_message(&err.message))),
                }
                loading.set(false);
            });
        })
    };

    // Initial fetch
    {
        let fetch_users = fetch_users.clone();
        use_effect_with((), move |_| {
            fetch_users.emit(());
            || ()
        });
    }

    let open_create_modal = {
        let show_modal = show_modal.clone();
        let is_edit = is_edit.clone();
        let form_username = form_username.clone();
        let form_password = form_password.clone();
        let form_role = form_role.clone();
        let form_is_active = form_is_active.clone();
        let error_message = error_message.clone();
        let success_message = success_message.clone();

        Callback::from(move |_| {
            is_edit.set(false);
            form_username.set(String::new());
            form_password.set(String::new());
            form_role.set(Role::Viewer);
            form_is_active.set(true);
            error_message.set(None);
            success_message.set(None);
            show_modal.set(true);
        })
    };

    let open_edit_modal = {
        let show_modal = show_modal.clone();
        let is_edit = is_edit.clone();
        let current_user_id = current_user_id.clone();
        let form_username = form_username.clone();
        let form_password = form_password.clone();
        let form_role = form_role.clone();
        let form_is_active = form_is_active.clone();
        let error_message = error_message.clone();
        let success_message = success_message.clone();

        Callback::from(move |user: User| {
            is_edit.set(true);
            current_user_id.set(user.id);
            form_username.set(user.username);
            form_password.set(String::new()); // Don't show password
            form_role.set(user.role);
            form_is_active.set(user.is_active);
            error_message.set(None);
            success_message.set(None);
            show_modal.set(true);
        })
    };

    let close_modal = {
        let show_modal = show_modal.clone();
        Callback::from(move |_| {
            show_modal.set(false);
        })
    };

    let on_submit = {
        let is_edit = is_edit.clone();
        let current_user_id = current_user_id.clone();
        let form_username = form_username.clone();
        let form_password = form_password.clone();
        let form_role = form_role.clone();
        let form_is_active = form_is_active.clone();
        let show_modal = show_modal.clone();
        let fetch_users = fetch_users.clone();
        let error_message = error_message.clone();
        let success_message = success_message.clone();
        let t = t.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            let is_edit_mode = *is_edit;
            let id = (*current_user_id).clone();
            let username = (*form_username).clone();
            let password = (*form_password).clone();
            let role = (*form_role).clone();
            let is_active = *form_is_active;

            let show_modal = show_modal.clone();
            let fetch_users = fetch_users.clone();
            let error_message = error_message.clone();
            let success_message = success_message.clone();
            let t = t.clone();

            spawn_local(async move {
                if is_edit_mode {
                    let request = UpdateUserRequest {
                        role: Some(role),
                        is_active: Some(is_active),
                        password: if password.is_empty() {
                            None
                        } else {
                            Some(password)
                        },
                    };
                    match api::update_user(&id, request).await {
                        Ok(_) => {
                            success_message.set(Some(t.t("users.update_success")));
                            show_modal.set(false);
                            fetch_users.emit(());
                        }
                        Err(err) => error_message.set(Some(translate_api_message(&err.message))),
                    }
                } else {
                    let request = CreateUserRequest {
                        username,
                        password,
                        role: Some(role),
                    };
                    match api::create_user(request).await {
                        Ok(_) => {
                            success_message.set(Some(t.t("users.create_success")));
                            show_modal.set(false);
                            fetch_users.emit(());
                        }
                        Err(err) => error_message.set(Some(translate_api_message(&err.message))),
                    }
                }
            });
        })
    };

    let on_delete = {
        let delete_modal_open = delete_modal_open.clone();
        let user_to_delete = user_to_delete.clone();

        Callback::from(move |id: String| {
            user_to_delete.set(Some(id));
            delete_modal_open.set(true);
        })
    };

    let on_confirm_delete = {
        let fetch_users = fetch_users.clone();
        let delete_modal_open = delete_modal_open.clone();
        let user_to_delete = user_to_delete.clone();
        let success_message = success_message.clone();
        let error_message = error_message.clone();
        let t = t.clone();

        Callback::from(move |_| {
            let fetch_users = fetch_users.clone();
            let delete_modal_open = delete_modal_open.clone();
            let user_to_delete = user_to_delete.clone();
            let success_message = success_message.clone();
            let error_message = error_message.clone();
            let t = t.clone();

            if let Some(id) = (*user_to_delete).clone() {
                spawn_local(async move {
                    match api::delete_user(&id).await {
                        Ok(_) => {
                            success_message.set(Some(t.t("users.delete_success")));
                            delete_modal_open.set(false);
                            user_to_delete.set(None);
                            fetch_users.emit(());
                        }
                        Err(err) => {
                            error_message.set(Some(translate_api_message(&err.message)));
                            delete_modal_open.set(false); // Close modal on error too? Or keep open? Usually close or show error in modal.
                                                          // For now, close and show error in main page as before
                        }
                    }
                });
            }
        })
    };

    let on_cancel_delete = {
        let delete_modal_open = delete_modal_open.clone();
        let user_to_delete = user_to_delete.clone();
        Callback::from(move |_| {
            delete_modal_open.set(false);
            user_to_delete.set(None);
        })
    };

    let role_options = vec![
        SelectOption {
            value: "Viewer".to_string(),
            label: t.t("users.role_viewer"),
        },
        SelectOption {
            value: "User".to_string(),
            label: t.t("users.role_user"),
        },
        SelectOption {
            value: "Admin".to_string(),
            label: t.t("users.role_admin"),
        },
    ];

    html! {
        <>
            <div class="container-fluid py-4">
                <div class="row">
                    <div class="col-12">
                        <Card class="my-4 shadow-xl">
                            <CardHeader class="flex flex-row justify-start items-center">
                                <div class="flex gap-2">
                                    <Button
                                        variant={ButtonVariant::Default}
                                        size={ButtonSize::Sm}
                                        onclick={open_create_modal}
                                    >
                                        <Plus class="h-4 w-4 mr-1" /> {t.t("users.create_user")}
                                    </Button>
                                </div>
                            </CardHeader>
                            <CardBody class="px-0 pb-2">
                                if let Some(msg) = (*error_message).clone() {
                                    <ErrorDisplay message={msg} />
                                }
                                if let Some(msg) = (*success_message).clone() {
                                    <div class="alert alert-success text-white mx-3 mb-3" role="alert">
                                        {msg}
                                    </div>
                                }
                                if *loading {
                                    <Loading />
                                } else {
                                    <div class="table-responsive p-0">
                                        <Table>
                                            <TableHeader>
                                                <TableRow>
                                                    <TableHead>{t.t("users.username")}</TableHead>
                                                    <TableHead>{t.t("users.role")}</TableHead>
                                                    <TableHead class="text-center">{t.t("users.status")}</TableHead>
                                                    <TableHead class="text-center">{t.t("users.last_login")}</TableHead>
                                                    <TableHead class="text-center">{t.t("users.actions")}</TableHead>
                                                </TableRow>
                                            </TableHeader>
                                            <TableBody>
                                                {
                                                    for users.iter().map(|user| {
                                                        let user_clone = user.clone();
                                                        let id = user.id.clone();
                                                        let on_edit_click = {
                                                            let open_edit_modal = open_edit_modal.clone();
                                                            let u = user_clone.clone();
                                                            Callback::from(move |e: MouseEvent| {
                                                                e.prevent_default();
                                                                open_edit_modal.emit(u.clone())
                                                            })
                                                        };
                                                        let on_delete_click = {
                                                            let on_delete = on_delete.clone();
                                                            let i = id.clone();
                                                            Callback::from(move |e: MouseEvent| {
                                                                e.prevent_default();
                                                                on_delete.emit(i.clone())
                                                            })
                                                        };

                                                        html! {
                                                            <TableRow class="hover:bg-slate-800/50 transition-colors">
                                                                <TableCell>
                                                                    <div class="flex items-center px-2 py-1">
                                                                        <div class="mr-2 text-slate-400">
                                                                            <UserIcon class="h-8 w-8 p-1 bg-slate-800 rounded-full" />
                                                                        </div>
                                                                        <div class="flex flex-col justify-center">
                                                                            <h6 class="mb-0 text-sm font-medium text-slate-200">{&user.username}</h6>
                                                                        </div>
                                                                    </div>
                                                                </TableCell>
                                                                <TableCell>
                                                                    <div class="flex items-center">
                                                                        <Shield class="h-4 w-4 mr-2 text-slate-500" />
                                                                        <span class="text-xs font-bold text-slate-300">{user.role.to_string()}</span>
                                                                    </div>
                                                                </TableCell>
                                                                <TableCell class="align-middle text-center text-sm">
                                                                    if user.is_active {
                                                                        <Badge variant={BadgeVariant::Outline} class="bg-emerald-500/10 text-emerald-500 border-emerald-500/20">{t.t("users.active")}</Badge>
                                                                    } else {
                                                                        <Badge variant={BadgeVariant::Secondary}>{t.t("users.inactive")}</Badge>
                                                                    }
                                                                </TableCell>
                                                                <TableCell class="align-middle text-center">
                                                                    <span class="text-slate-400 text-xs font-bold">
                                                                        {user.last_login.clone().unwrap_or_else(|| "-".to_string())}
                                                                    </span>
                                                                </TableCell>
                                                                <TableCell class="align-middle text-center">
                                                                    <TableActions
                                                                        on_edit={on_edit_click}
                                                                        on_delete={on_delete_click}
                                                                    />
                                                                </TableCell>
                                                            </TableRow>
                                                        }
                                                    })
                                                }
                                            </TableBody>
                                        </Table>
                                    </div>
                                }
                            </CardBody>
                        </Card>
                    </div>
                </div>
            </div>

            <ConfirmModal
                is_open={*delete_modal_open}
                title={t.t("common.confirm_delete")}
                message={t.t("users.delete_confirm")}
                on_confirm={on_confirm_delete}
                on_cancel={on_cancel_delete}
            />

            // Modal
            {
                if *show_modal {
                    create_portal(
                        html! {
                            <div class="fixed inset-0 z-[9999] flex items-center justify-center bg-black/50 backdrop-blur-sm">
                                <Card class="w-11/12 max-w-md shadow-2xl">
                                    <CardHeader class="flex flex-row justify-between items-center border-b border-slate-800 pb-4">
                                        <CardTitle>
                                            { if *is_edit { t.t("users.edit_user") } else { t.t("users.create_user") } }
                                        </CardTitle>
                                        <Button
                                            variant={ButtonVariant::Ghost}
                                            size={ButtonSize::Icon}
                                            onclick={close_modal.clone()}
                                        >
                                            <X class="h-4 w-4" />
                                        </Button>
                                    </CardHeader>
                                    <CardBody class="pt-4">
                                        <form onsubmit={on_submit} class="flex flex-col gap-4">
                                            <div class="space-y-2">
                                                <label class="text-sm font-medium text-slate-300">{t.t("users.username")}</label>
                                                <Input
                                                    value={(*form_username).clone()}
                                                    oninput={Callback::from(move |val: String| form_username.set(val))}
                                                    disabled={*is_edit}
                                                    required=true
                                                    placeholder={t.t("users.username_placeholder")}
                                                />
                                            </div>

                                            <div class="space-y-2">
                                                <label class="text-sm font-medium text-slate-300">
                                                    { if *is_edit { t.t("users.password_placeholder_edit") } else { t.t("users.password") } }
                                                </label>
                                                <Input
                                                    type_="password"
                                                    value={(*form_password).clone()}
                                                    oninput={Callback::from(move |val: String| form_password.set(val))}
                                                    required={!*is_edit}
                                                    placeholder={t.t("users.password_placeholder")}
                                                />
                                            </div>

                                            <div class="space-y-2">
                                                <label class="text-sm font-medium text-slate-300">{t.t("users.role")}</label>
                                                <Select
                                                    value={form_role.to_string()}
                                                    options={role_options}
                                                    onchange={
                                                        let form_role = form_role.clone();
                                                        Callback::from(move |val: String| {
                                                            match val.as_str() {
                                                                "Admin" => form_role.set(Role::Admin),
                                                                "User" => form_role.set(Role::User),
                                                                _ => form_role.set(Role::Viewer),
                                                            }
                                                        })
                                                    }
                                                />
                                            </div>

                                            if *is_edit {
                                                <div class="flex items-center space-x-2 pt-2">
                                                    <Checkbox
                                                        checked={*form_is_active}
                                                        onchange={Callback::from(move |checked: bool| form_is_active.set(checked))}
                                                        id="active-check"
                                                    />
                                                    <label for="active-check" class="text-sm font-medium text-slate-300 cursor-pointer">{t.t("users.enable_account")}</label>
                                                </div>
                                            }

                                            <div class="flex justify-end gap-2 mt-6">
                                                <Button
                                                    variant={ButtonVariant::Outline}
                                                    type_="button"
                                                    onclick={close_modal}
                                                >
                                                    {t.t("users.cancel")}
                                                </Button>
                                                <Button
                                                    variant={ButtonVariant::Default}
                                                    type_="submit"
                                                >
                                                    {t.t("users.save")}
                                                </Button>
                                            </div>
                                        </form>
                                    </CardBody>
                                </Card>
                            </div>
                        },
                        document().body().unwrap().into()
                    )
                } else {
                    html! {}
                }
            }
        </>
    }
}
