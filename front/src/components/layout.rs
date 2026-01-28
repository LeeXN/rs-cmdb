use crate::components::change_password_modal::ChangePasswordModal;
use crate::components::ui::input::Input;
use crate::hooks::use_trans::use_trans;
use crate::i18n::Language;
use crate::routes::Route;
use crate::stores::auth_store::AuthStore;
use crate::stores::language_store::LanguageStore;
use crate::stores::theme::ThemeStore;
use crate::types::Role;
use lucide_yew::{
    Bell, ChartBar, Code, Cpu, Download, Folder, HardDrive, Key, Languages, LayoutDashboard, List,
    LogOut, Menu, Search, Server, Shield, User, Users,
};
use web_sys::window;
use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

#[derive(Properties, PartialEq)]
pub struct LayoutProps {
    pub children: Children,
}

#[function_component(Layout)]
pub fn layout(props: &LayoutProps) -> Html {
    let current_route = use_route::<Route>();
    let sidebar_open = use_state(|| false);
    let change_password_modal_open = use_state(|| false);
    let (theme_store, _theme_dispatch) = use_store::<ThemeStore>();
    let (auth_store, auth_dispatch) = use_store::<AuthStore>();
    let (language_store, language_dispatch) = use_store::<LanguageStore>();
    let navigator = use_navigator().unwrap();
    let t = use_trans();

    // Sync global i18n for helpers
    {
        let language = language_store.language.clone();
        use_effect_with(language, move |language| {
            crate::utils::i18n_helper::init_i18n(language.clone());
            || ()
        });
    }

    let toggle_language = {
        let language_dispatch = language_dispatch.clone();
        let current_language = language_store.language.clone();
        Callback::from(move |_| {
            let new_language = match current_language {
                Language::ZhCn => Language::EnUs,
                Language::EnUs => Language::ZhCn,
            };
            language_dispatch.reduce_mut(|store| store.language = new_language);
        })
    };

    // Auth check
    {
        let auth_store = auth_store.clone();
        let navigator = navigator.clone();
        let current_route = current_route.clone();
        use_effect_with(
            (auth_store, current_route),
            move |(auth_store, current_route)| {
                if !auth_store.is_authenticated {
                    if let Some(route) = current_route.as_ref() {
                        if *route != Route::Login {
                            navigator.push(&Route::Login);
                        }
                    }
                }
                || ()
            },
        );
    }

    if let Some(Route::Login) = current_route.as_ref() {
        return html! {
            <div class="min-h-screen bg-background text-foreground font-mono">
                { for props.children.iter() }
            </div>
        };
    }

    let on_logout = {
        let auth_dispatch = auth_dispatch.clone();
        let navigator = navigator.clone();
        Callback::from(move |_| {
            AuthStore::logout(auth_dispatch.clone());
            navigator.push(&Route::Login);
        })
    };

    // Theme effect (keep existing logic)
    {
        let theme = theme_store.theme.clone();
        use_effect_with(theme, move |theme| {
            if let Some(window) = window() {
                if let Some(document) = window.document() {
                    if let Some(element) = document.document_element() {
                        let _ = element.set_attribute("data-theme", theme);
                    }
                }
            }
            || ()
        });
    }

    let toggle_sidebar = {
        let sidebar_open = sidebar_open.clone();
        Callback::from(move |_| sidebar_open.set(!*sidebar_open))
    };

    let close_sidebar = {
        let sidebar_open = sidebar_open.clone();
        Callback::from(move |_| sidebar_open.set(false))
    };

    let open_change_password_modal = {
        let change_password_modal_open = change_password_modal_open.clone();
        Callback::from(move |_| change_password_modal_open.set(true))
    };

    let close_change_password_modal = {
        let change_password_modal_open = change_password_modal_open.clone();
        Callback::from(move |_| change_password_modal_open.set(false))
    };

    let search_query = use_state(String::new);
    let on_search_input = {
        let search_query = search_query.clone();
        Callback::from(move |val: String| {
            search_query.set(val);
        })
    };
    let on_search_keypress = {
        let search_query = search_query.clone();
        let navigator = navigator.clone();
        Callback::from(move |e: KeyboardEvent| {
            if e.key() == "Enter" {
                let query = (*search_query).clone();
                let _ =
                    navigator.push_with_query(&Route::Clients, &serde_json::json!({"q": query}));
            }
        })
    };

    // Sidebar Items
    let sidebar_item = |route: Route, icon: Html, label: &str| {
        let is_active = current_route.as_ref() == Some(&route);
        let active_class = if is_active {
            "bg-primary/10 text-primary border-r-2 border-primary shadow-[inset_0_0_10px_rgba(6,182,212,0.1)]"
        } else {
            "text-muted-foreground hover:bg-accent hover:text-accent-foreground hover:shadow-[inset_0_0_10px_rgba(6,182,212,0.05)]"
        };

        html! {
            <Link<Route> to={route} classes={classes!("flex", "items-center", "gap-3", "px-3", "py-2", "text-sm", "font-medium", "transition-all", "duration-200", active_class)}>
                {icon}
                {label}
            </Link<Route>>
        }
    };

    html! {
        <div class="min-h-screen bg-background font-mono text-foreground flex">
            // Mobile Sidebar Overlay
            if *sidebar_open {
                <div class="fixed inset-0 z-40 bg-background/80 backdrop-blur-sm lg:hidden" onclick={close_sidebar.clone()} />
            }

            // Sidebar
            <aside class={classes!(
                "fixed", "inset-y-0", "left-0", "z-50", "w-64", "border-r", "border-border", "bg-card/50", "backdrop-blur-xl", "transition-transform", "lg:static", "lg:translate-x-0",
                if *sidebar_open { "translate-x-0" } else { "-translate-x-full" }
            )}>
                <div class="flex h-14 items-center border-b border-border px-6">
                    <div class="flex items-center gap-2 font-bold text-lg tracking-wider text-primary">
                        <Server class="h-6 w-6" />
                        <span>{"CMDB"}<span class="text-xs text-muted-foreground ml-1">{"PRO"}</span></span>
                    </div>
                </div>

                <div class="flex-1 overflow-y-auto py-4">
                    <nav class="grid gap-1 px-2">
                        { sidebar_item(Route::Home, html!{ <LayoutDashboard class="h-4 w-4" /> }, &t.t("menu.dashboard")) }

                        <div class="mt-4 mb-2 px-4 text-xs font-semibold uppercase text-muted-foreground tracking-wider">{t.t("menu.assets")}</div>
                        { sidebar_item(Route::Clients, html!{ <Server class="h-4 w-4" /> }, &t.t("menu.clients")) }
                        { sidebar_item(Route::Racks, html!{ <HardDrive class="h-4 w-4" /> }, &t.t("menu.racks")) }
                        { sidebar_item(Route::Components, html!{ <Cpu class="h-4 w-4" /> }, &t.t("menu.components")) }

                        <div class="mt-4 mb-2 px-4 text-xs font-semibold uppercase text-muted-foreground tracking-wider">{t.t("menu.organization")}</div>
                        { sidebar_item(Route::Persons, html!{ <Users class="h-4 w-4" /> }, &t.t("menu.users")) }
                        { sidebar_item(Route::Projects, html!{ <Folder class="h-4 w-4" /> }, &t.t("menu.projects")) }

                        <div class="mt-4 mb-2 px-4 text-xs font-semibold uppercase text-muted-foreground tracking-wider">{t.t("menu.system")}</div>
                        { sidebar_item(Route::Analytics, html!{ <ChartBar class="h-4 w-4" /> }, &t.t("menu.analytics")) }
                        { sidebar_item(Route::ClientSetup, html!{ <Download class="h-4 w-4" /> }, &t.t("menu.setup_guide")) }
                        { sidebar_item(Route::BaseData, html!{ <List class="h-4 w-4" /> }, &t.t("menu.base_data")) }

                        if let Some(user) = &auth_store.user {
                            if user.role == Role::Admin {
                                { sidebar_item(Route::Accounts, html!{ <Shield class="h-4 w-4" /> }, &t.t("menu.accounts")) }
                            }
                        }
                    </nav>
                </div>

                <div class="border-t border-border p-4">
                    <a href="https://github.com/LeeXN/rs-cmdb" target="_blank" class="flex items-center gap-2 text-xs text-muted-foreground hover:text-primary transition-colors">
                        <Code class="h-4 w-4" />
                        {t.t("menu.source_code")}
                    </a>
                </div>
            </aside>

            // Main Content
            <div class="flex flex-1 flex-col overflow-hidden">
                // Header
                <header class="flex h-14 items-center gap-4 border-b border-border bg-background/50 backdrop-blur-md px-6 sticky top-0 z-30">
                    <button class="lg:hidden text-muted-foreground hover:text-foreground" onclick={toggle_sidebar}>
                        <Menu class="h-6 w-6" />
                    </button>

                    <div class="w-full flex-1">
                        <div class="relative w-full max-w-sm">
                            <Search class="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground z-10" />
                            <Input
                                type_="search"
                                placeholder={t.t("header.search_placeholder")}
                                class="pl-9"
                                value={(*search_query).clone()}
                                oninput={on_search_input}
                                onkeydown={on_search_keypress}
                            />
                        </div>
                    </div>

                    <div class="flex items-center gap-4">
                        <button onclick={toggle_language} class="text-muted-foreground hover:text-foreground" title={t.t("header.switch_language")}>
                            <Languages class="h-5 w-5" />
                        </button>
                        <button class="text-muted-foreground hover:text-foreground relative">
                            <Bell class="h-5 w-5" />
                            <span class="absolute -top-1 -right-1 h-2 w-2 rounded-full bg-primary animate-pulse"></span>
                        </button>

                        if auth_store.is_authenticated {
                            <div class="flex items-center gap-2 text-sm font-medium text-muted-foreground">
                                <User class="h-5 w-5" />
                                <span class="hidden md:inline">{ auth_store.user.as_ref().map(|u| u.username.clone()).unwrap_or_default() }</span>
                                <button onclick={open_change_password_modal} class="ml-2 text-muted-foreground hover:text-primary" title={t.t("header.change_password")}>
                                    <Key class="h-5 w-5" />
                                </button>
                                <button onclick={on_logout} class="ml-2 text-destructive hover:text-destructive/80" title={t.t("header.logout")}>
                                    <LogOut class="h-5 w-5" />
                                </button>
                            </div>
                        }
                    </div>
                </header>

                // Page Content
                <main class="flex-1 overflow-y-auto p-6">
                    { for props.children.iter() }
                </main>
            </div>

            <ChangePasswordModal
                is_open={*change_password_modal_open}
                on_close={close_change_password_modal}
            />
        </div>
    }
}
