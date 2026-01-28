use crate::hooks::use_trans::use_trans;
use gloo::net::http::Request;
use serde::Deserialize;
use std::collections::HashMap;
use std::rc::Rc;
use yew::prelude::*;

use crate::components::error::ErrorDisplay;
use crate::components::loading::Loading;
use crate::components::ui::card::{Card, CardBody, CardHeader};
use crate::components::ui::select::{Select, SelectOption};
use lucide_yew::{CircleCheck, Download, FileText, Info};
use wasm_bindgen_futures;

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ClientInfo {
    pub server_url: String,
    pub download_url: String,
    pub install_script: String,
    pub systemd_service: String,
    pub config_template: String,
}

pub enum ClientSetupAction {
    SetClientInfo(ClientInfo),
    SetLoading(bool),
    SetError(Option<String>),
    SetPlatform(String),
    SetArch(String),
}

pub struct ClientSetupState {
    client_info: Option<ClientInfo>,
    loading: bool,
    error: Option<String>,
    platform: String,
    arch: String,
}

impl Default for ClientSetupState {
    fn default() -> Self {
        Self {
            client_info: None,
            loading: true,
            error: None,
            platform: "linux".to_string(),
            arch: "x86_64".to_string(),
        }
    }
}

impl Reducible for ClientSetupState {
    type Action = ClientSetupAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            ClientSetupAction::SetClientInfo(info) => Rc::new(Self {
                client_info: Some(info),
                loading: false,
                error: None,
                platform: self.platform.clone(),
                arch: self.arch.clone(),
            }),
            ClientSetupAction::SetLoading(loading) => Rc::new(Self {
                client_info: self.client_info.clone(),
                loading,
                error: if loading { None } else { self.error.clone() },
                platform: self.platform.clone(),
                arch: self.arch.clone(),
            }),
            ClientSetupAction::SetError(error) => Rc::new(Self {
                client_info: self.client_info.clone(),
                loading: false,
                error,
                platform: self.platform.clone(),
                arch: self.arch.clone(),
            }),
            ClientSetupAction::SetPlatform(platform) => Rc::new(Self {
                client_info: None,
                loading: true,
                error: None,
                platform,
                arch: self.arch.clone(),
            }),
            ClientSetupAction::SetArch(arch) => Rc::new(Self {
                client_info: None,
                loading: true,
                error: None,
                platform: self.platform.clone(),
                arch,
            }),
        }
    }
}

#[function_component(ClientSetupPage)]
pub fn client_setup_page() -> Html {
    let t = use_trans();
    let state = use_reducer(ClientSetupState::default);

    // 加载客户端信息
    {
        let state = state.clone();
        let t = t.clone();
        use_effect_with(
            (state.loading, state.platform.clone(), state.arch.clone()),
            move |(loading, platform, arch)| {
                if *loading {
                    let state_clone = state.clone();
                    let platform = platform.clone();
                    let arch = arch.clone();
                    let t = t.clone();

                    wasm_bindgen_futures::spawn_local(async move {
                        let url =
                            format!("/api/v1/download/info?platform={}&arch={}", platform, arch);

                        match Request::get(&url).send().await {
                            Ok(response) => {
                                if response.ok() {
                                    match response.json::<ClientInfo>().await {
                                        Ok(info) => {
                                            state_clone
                                                .dispatch(ClientSetupAction::SetClientInfo(info));
                                        }
                                        Err(e) => {
                                            state_clone.dispatch(ClientSetupAction::SetError(
                                                Some(t.t_with_args(
                                                    "client_setup.parse_error",
                                                    &HashMap::from([(
                                                        "error".to_string(),
                                                        e.to_string(),
                                                    )]),
                                                )),
                                            ));
                                        }
                                    }
                                } else {
                                    state_clone.dispatch(ClientSetupAction::SetError(Some(
                                        t.t_with_args(
                                            "client_setup.request_failed",
                                            &HashMap::from([(
                                                "status".to_string(),
                                                response.status().to_string(),
                                            )]),
                                        ),
                                    )));
                                }
                            }
                            Err(e) => {
                                state_clone.dispatch(ClientSetupAction::SetError(Some(
                                    t.t_with_args(
                                        "client_setup.network_error",
                                        &HashMap::from([("error".to_string(), e.to_string())]),
                                    ),
                                )));
                            }
                        }
                    });
                }

                || ()
            },
        );
    }

    let on_platform_change = {
        let state = state.clone();
        Callback::from(move |val: String| {
            state.dispatch(ClientSetupAction::SetPlatform(val));
        })
    };

    let on_arch_change = {
        let state = state.clone();
        Callback::from(move |val: String| {
            state.dispatch(ClientSetupAction::SetArch(val));
        })
    };

    let on_retry = {
        let state = state.clone();
        Callback::from(move |_: ()| {
            state.dispatch(ClientSetupAction::SetLoading(true));
        })
    };

    let platform_options = vec![SelectOption {
        value: "linux".to_string(),
        label: "Linux".to_string(),
    }];

    let arch_options = vec![
        SelectOption {
            value: "x86_64".to_string(),
            label: "x86_64".to_string(),
        },
        SelectOption {
            value: "aarch64".to_string(),
            label: "aarch64".to_string(),
        },
    ];

    if state.loading {
        return html! { <Loading /> };
    }

    if let Some(error) = &state.error {
        return html! { <ErrorDisplay message={error.clone()} on_retry={on_retry} /> };
    }

    let client_info = match &state.client_info {
        Some(info) => info,
        None => return html! { <div class="text-white">{t.t("client_setup.load_failed")}</div> },
    };

    html! {
        <div class="container-fluid py-4">
            <div class="row">
                <div class="col-12">
                    <Card class="shadow-xl">
                        <CardHeader>
                            <div class="d-flex align-items-center">
                                <div class="icon icon-shape bg-gradient-primary shadow text-center border-radius-md me-3 flex items-center justify-center">
                                    <Download class="w-6 h-6 text-white" />
                                </div>
                                <div>
                                    <h6 class="text-white text-capitalize mb-0">{t.t("client_setup.guide_title")}</h6>
                                    <p class="text-xs text-slate-400 mb-0">{t.t("client_setup.guide_subtitle")}</p>
                                </div>
                            </div>
                        </CardHeader>
                        <CardBody class="p-4">
                            // Platform Selection
                            <div class="row mb-4">
                                <div class="col-md-6 mb-3">
                                    <label class="block text-sm font-bold text-slate-400 mb-1">{t.t("client_setup.select_platform")}</label>
                                    <Select
                                        options={platform_options}
                                        value={state.platform.clone()}
                                        onchange={on_platform_change}
                                    />
                                </div>
                                <div class="col-md-6 mb-3">
                                    <label class="block text-sm font-bold text-slate-400 mb-1">{t.t("client_setup.select_arch")}</label>
                                    <Select
                                        options={arch_options}
                                        value={state.arch.clone()}
                                        onchange={on_arch_change}
                                    />
                                </div>
                            </div>

                            <hr class="horizontal dark my-4" />

                            // Download Info
                            <div class="mb-4">
                                <h6 class="text-white mb-3">{t.t("client_setup.step1_download")}</h6>
                                <div class="alert alert-info text-white" role="alert">
                                    <div class="d-flex flex-column">
                                        <p class="mb-1">
                                            <span class="font-weight-bold">{t.t("client_setup.download_url")}</span>
                                            <a href={client_info.download_url.clone()} target="_blank" class="text-white text-decoration-underline font-monospace">
                                                {client_info.download_url.clone()}
                                            </a>
                                        </p>
                                        <p class="mb-0">
                                            <span class="font-weight-bold">{t.t("client_setup.server_url")}</span>
                                            <span class="font-monospace">{client_info.server_url.clone()}</span>
                                        </p>
                                    </div>
                                </div>
                            </div>

                            // Quick Install
                            {
                                if state.platform == "linux" {
                                    html! {
                                        <div class="mb-4">
                                            <h6 class="text-white mb-3">{t.t("client_setup.step2_quick_install")}</h6>
                                            <p class="text-sm text-slate-400 mb-2">{t.t("client_setup.copy_command")}</p>
                                            <div class="bg-slate-950 rounded p-3 border border-slate-800">
                                                <code class="text-success font-monospace">{format!("curl -fsSL {}/install.sh | bash", client_info.server_url)}</code>
                                            </div>
                                            <p class="text-xs text-slate-500 mt-2">{t.t("client_setup.quick_install_desc")}</p>
                                        </div>
                                    }
                                } else {
                                    html! {}
                                }
                            }

                            // Manual Install Script
                            <div class="mb-4">
                                <h6 class="text-white mb-3">{if state.platform == "linux" { t.t("client_setup.step3_manual_install") } else { t.t("client_setup.step2_install_script") }}</h6>
                                <p class="text-sm text-slate-400 mb-2">{t.t("client_setup.save_script")}</p>
                                <div class="bg-slate-950 rounded border border-slate-800">
                                    <div class="p-3 border-b border-slate-800 d-flex justify-content-between align-items-center">
                                        <span class="text-sm text-slate-300 font-monospace">
                                            {match state.platform.as_str() {
                                                "linux" => "install.sh",
                                                "windows" => "install.bat",
                                                _ => "install script",
                                            }}
                                        </span>
                                    </div>
                                    <div class="p-3 overflow-auto" style="max-height: 300px;">
                                        <pre class="text-xs text-slate-300 font-monospace m-0">
                                            {client_info.install_script.clone()}
                                        </pre>
                                    </div>
                                </div>
                            </div>

                            // Config Template
                            <div class="mb-4">
                                <h6 class="text-white mb-3">{if state.platform == "linux" { t.t("client_setup.step4_config") } else { t.t("client_setup.step3_config") }}</h6>
                                <p class="text-sm text-slate-400 mb-2">{t.t("client_setup.config_template_desc")}</p>
                                <div class="bg-slate-950 rounded border border-slate-800">
                                    <div class="p-3 border-b border-slate-800 d-flex justify-content-between align-items-center">
                                        <span class="text-sm text-slate-300 font-monospace">{"client.toml"}</span>
                                    </div>
                                    <div class="p-3 overflow-auto" style="max-height: 300px;">
                                        <pre class="text-xs text-slate-300 font-monospace m-0">
                                            {client_info.config_template.clone()}
                                        </pre>
                                    </div>
                                </div>
                            </div>

                            // Systemd Service (Linux only)
                            {
                                if state.platform == "linux" {
                                    html! {
                                        <div class="mb-4">
                                            <h6 class="text-white mb-3">{t.t("client_setup.step5_systemd")}</h6>
                                            <p class="text-sm text-slate-400 mb-2">{t.t("client_setup.systemd_desc")}</p>
                                            <div class="bg-slate-950 rounded border border-slate-800">
                                                <div class="p-3 border-b border-slate-800 d-flex justify-content-between align-items-center">
                                                    <span class="text-sm text-slate-300 font-monospace">{"rs-cmdb-client.service"}</span>
                                                </div>
                                                <div class="p-3 overflow-auto">
                                                    <pre class="text-xs text-slate-300 font-monospace m-0">
                                                        {client_info.systemd_service.clone()}
                                                    </pre>
                                                </div>
                                            </div>
                                        </div>
                                    }
                                } else {
                                    html! {}
                                }
                            }

                            // Verify Install
                            <div>
                                <h6 class="text-white mb-3">{if state.platform == "linux" { t.t("client_setup.step6_verify") } else { t.t("client_setup.step4_verify") }}</h6>
                                <div class="alert alert-success text-white" role="alert">
                                    <div class="d-flex align-items-center mb-2">
                                        <CircleCheck class="w-5 h-5 me-2" />
                                        <span class="font-weight-bold">{t.t("client_setup.check_status")}</span>
                                    </div>
                                    {
                                        if state.platform == "linux" {
                                            html! {
                                                <div class="bg-slate-900 rounded p-2 mb-3">
                                                    <code class="text-white font-monospace">{"sudo systemctl status rs-cmdb-client"}</code>
                                                </div>
                                            }
                                        } else {
                                            html! {
                                                <p class="mb-3 text-sm">{t.t("client_setup.manual_run_check")}</p>
                                            }
                                        }
                                    }

                                    <div class="d-flex align-items-center mb-2">
                                        <FileText class="w-5 h-5 me-2" />
                                        <span class="font-weight-bold">{t.t("client_setup.check_logs")}</span>
                                    </div>
                                    {
                                        if state.platform == "linux" {
                                            html! {
                                                <div class="bg-slate-900 rounded p-2">
                                                    <code class="text-white font-monospace">{"sudo journalctl -u rs-cmdb-client -f"}</code>
                                                </div>
                                            }
                                        } else {
                                            html! {
                                                <p class="mb-0 text-sm">{t.t("client_setup.check_logs_dir")}</p>
                                            }
                                        }
                                    }
                                </div>

                                <div class="alert alert-secondary text-white mt-3 flex items-center" role="alert">
                                    <Info class="w-4 h-4 me-2" />
                                    <span class="text-sm">{t.t("client_setup.install_complete_prefix")}</span>
                                    <a href="/clients" class="text-white font-weight-bold text-decoration-underline">{t.t("client_setup.client_list")}</a>
                                    <span class="text-sm">{t.t("client_setup.install_complete_suffix")}</span>
                                </div>
                            </div>
                        </CardBody>
                    </Card>
                </div>
            </div>
        </div>
    }
}
