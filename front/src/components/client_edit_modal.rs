use yew::prelude::*;
use web_sys::{HtmlInputElement, HtmlSelectElement};
use wasm_bindgen_futures::spawn_local;
use crate::types::{Client, Person, Project, ClientStatus, Environment, Rack};
use crate::services::api;
use crate::components::ui::modal::{Modal, ModalContent, ModalHeader, ModalFooter, ModalTitle, ModalDescription};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::input::Input;
use lucide_yew::X;
use crate::hooks::use_trans::use_trans;

#[derive(Properties, PartialEq)]
pub struct ClientEditModalProps {
    pub client: Client,
    pub on_save: Callback<Client>,
    pub on_cancel: Callback<()>,
}

#[function_component(ClientEditModal)]
pub fn client_edit_modal(props: &ClientEditModalProps) -> Html {
    let t = use_trans();
    let location = use_state(|| props.client.location.clone().unwrap_or_default());
    let rack = use_state(|| props.client.rack.clone().unwrap_or_default());
    let unit_position = use_state(|| props.client.unit_position.clone().unwrap_or_default());
    let u_height = use_state(|| props.client.u_height.unwrap_or(1));
    let power_consumption = use_state(|| props.client.power_consumption);
    let comment = use_state(|| props.client.comment.clone().unwrap_or_default());
    
    let owner_id = use_state(|| props.client.owner_id.clone().unwrap_or_default());
    let project_id = use_state(|| props.client.project_id.clone().unwrap_or_default());
    
    let status = use_state(|| props.client.status.clone());
    let environment = use_state(|| props.client.environment.clone());
    
    let asset_tag = use_state(|| props.client.asset_tag.clone().unwrap_or_default());
    let warranty_expiration = use_state(|| props.client.warranty_expiration.clone().unwrap_or_default());
    let supplier = use_state(|| props.client.supplier.clone().unwrap_or_default());

    let persons = use_state(|| Vec::<Person>::new());
    let projects = use_state(|| Vec::<Project>::new());
    let racks = use_state(|| Vec::<Rack>::new());

    {
        let persons = persons.clone();
        let projects = projects.clone();
        let racks = racks.clone();
        use_effect_with((), move |_| {
            let persons = persons.clone();
            let projects = projects.clone();
            let racks = racks.clone();
            spawn_local(async move {
                if let Ok(data) = api::fetch_persons(1, 1000, None, None).await {
                    persons.set(data.items);
                }
            });
            spawn_local(async move {
                if let Ok(data) = api::fetch_projects(1, 1000, None, None).await {
                    projects.set(data.items);
                }
            });
            spawn_local(async move {
                if let Ok(data) = api::fetch_racks(1, 1000, None, None).await {
                    racks.set(data.items);
                }
            });
            || ()
        });
    }

    let onsubmit = {
        let location = location.clone();
        let rack = rack.clone();
        let unit_position = unit_position.clone();
        let u_height = u_height.clone();
        let power_consumption = power_consumption.clone();
        let comment = comment.clone();
        let owner_id = owner_id.clone();
        let project_id = project_id.clone();
        let status = status.clone();
        let environment = environment.clone();
        let asset_tag = asset_tag.clone();
        let warranty_expiration = warranty_expiration.clone();
        let supplier = supplier.clone();
        
        let client = props.client.clone();
        let on_save = props.on_save.clone();
        
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let mut updated_client = client.clone();
            updated_client.location = if location.is_empty() { None } else { Some((*location).clone()) };
            updated_client.rack = if rack.is_empty() { None } else { Some((*rack).clone()) };
            updated_client.unit_position = if unit_position.is_empty() { None } else { Some((*unit_position).clone()) };
            updated_client.u_height = Some(*u_height);
            updated_client.power_consumption = *power_consumption;
            updated_client.comment = if comment.is_empty() { None } else { Some((*comment).clone()) };
            
            updated_client.owner_id = if owner_id.is_empty() { None } else { Some((*owner_id).clone()) };
            updated_client.project_id = if project_id.is_empty() { None } else { Some((*project_id).clone()) };
            
            updated_client.status = (*status).clone();
            updated_client.environment = (*environment).clone();
            
            updated_client.asset_tag = if asset_tag.is_empty() { None } else { Some((*asset_tag).clone()) };
            updated_client.warranty_expiration = if warranty_expiration.is_empty() { None } else { Some((*warranty_expiration).clone()) };
            updated_client.supplier = if supplier.is_empty() { None } else { Some((*supplier).clone()) };
            
            on_save.emit(updated_client);
        })
    };

    let select_class = "flex h-9 w-full items-center justify-between rounded-md border border-input bg-slate-950 px-3 py-2 text-sm text-slate-200 shadow-sm ring-offset-background placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-ring disabled:cursor-not-allowed disabled:opacity-50 [&>span]:line-clamp-1";
    let label_class = "text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70 mb-2 block";

    html! {
        <Modal open={true} on_open_change={Callback::from(|_| {})}>
            <ModalContent class="sm:max-w-[800px] max-h-[90vh] overflow-y-auto">
                <ModalHeader>
                    <ModalTitle>{t.t("client_edit.title")}</ModalTitle>
                    <ModalDescription>{t.t("client_edit.description")}</ModalDescription>
                    <Button 
                        variant={ButtonVariant::Ghost} 
                        size={crate::components::ui::button::ButtonSize::Icon}
                        class="absolute right-4 top-4 rounded-sm opacity-70 ring-offset-background transition-opacity hover:opacity-100 focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:pointer-events-none data-[state=open]:bg-accent data-[state=open]:text-muted-foreground"
                        onclick={props.on_cancel.reform(|_| ())}
                    >
                        <X class="h-4 w-4" />
                        <span class="sr-only">{"Close"}</span>
                    </Button>
                </ModalHeader>
                
                <form {onsubmit} class="space-y-6 py-4">
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                        // Basic Info
                        <div class="space-y-2">
                            <label class={label_class}>{t.t("client_detail.location")}</label>
                            <Input 
                                value={(*location).clone()}
                                oninput={
                                    let location = location.clone();
                                    Callback::from(move |val: String| {
                                        location.set(val);
                                    })
                                }
                            />
                        </div>
                        <div class="space-y-2">
                            <label class={label_class}>{t.t("client_detail.rack")}</label>
                            <select class={select_class}
                                onchange={
                                    let rack = rack.clone();
                                    let location = location.clone();
                                    let racks = racks.clone();
                                    Callback::from(move |e: Event| {
                                        let input: HtmlSelectElement = e.target_unchecked_into();
                                        let selected_rack_id = input.value();
                                        rack.set(selected_rack_id.clone());
                                        
                                        // Auto-fill location if a rack is selected
                                        if !selected_rack_id.is_empty() {
                                            if let Some(selected_rack) = racks.iter().find(|r| r.id == selected_rack_id) {
                                                if let Some(loc) = &selected_rack.location {
                                                    location.set(loc.clone());
                                                }
                                            }
                                        }
                                    })
                                }
                            >
                                <option value="" selected={rack.is_empty()}>{t.t("client_edit.unassigned")}</option>
                                {
                                    racks.iter().map(|r| {
                                        html! {
                                            <option value={r.id.clone()} selected={*rack == r.id}>{&r.name}</option>
                                        }
                                    }).collect::<Html>()
                                }
                            </select>
                        </div>
                        <div class="space-y-2">
                            <label class={label_class}>{t.t("client_detail.unit_position")}</label>
                            <Input 
                                value={(*unit_position).clone()}
                                oninput={Callback::from(move |val: String| {
                                    unit_position.set(val);
                                })}
                            />
                        </div>
                        <div class="space-y-2">
                            <label class={label_class}>{t.t("client_detail.u_height")}</label>
                            <Input 
                                type_="number"
                                value={(*u_height).to_string()}
                                min="1"
                                max="42"
                                oninput={Callback::from(move |val: String| {
                                    if let Ok(v) = val.parse::<u32>() {
                                        u_height.set(v);
                                    }
                                })}
                            />
                        </div>
                        <div class="space-y-2">
                            <label class={label_class}>{t.t("client_detail.power")}</label>
                            <Input 
                                type_="number"
                                value={(*power_consumption).map(|v| v.to_string()).unwrap_or_default()}
                                min="0"
                                oninput={Callback::from(move |val: String| {
                                    if let Ok(v) = val.parse::<u32>() {
                                        power_consumption.set(Some(v));
                                    } else if val.is_empty() {
                                        power_consumption.set(None);
                                    }
                                })}
                            />
                        </div>

                        // Associations
                        <div class="space-y-2">
                            <label class={label_class}>{t.t("client_detail.owner")}</label>
                            <select class={select_class}
                                onchange={
                                    let owner_id = owner_id.clone();
                                    Callback::from(move |e: Event| {
                                        let input: HtmlSelectElement = e.target_unchecked_into();
                                        owner_id.set(input.value());
                                    })
                                }
                            >
                                <option value="" selected={owner_id.is_empty()}>{t.t("client_edit.unassigned")}</option>
                                {
                                    persons.iter().map(|p| {
                                        html! {
                                            <option value={p.id.clone()} selected={*owner_id == p.id}>{&p.name}</option>
                                        }
                                    }).collect::<Html>()
                                }
                            </select>
                        </div>
                        <div class="space-y-2">
                            <label class={label_class}>{t.t("client_detail.project")}</label>
                            <select class={select_class}
                                onchange={
                                    let project_id = project_id.clone();
                                    Callback::from(move |e: Event| {
                                        let input: HtmlSelectElement = e.target_unchecked_into();
                                        project_id.set(input.value());
                                    })
                                }
                            >
                                <option value="" selected={project_id.is_empty()}>{t.t("client_edit.unassigned")}</option>
                                {
                                    projects.iter().map(|p| {
                                        html! {
                                            <option value={p.id.clone()} selected={*project_id == p.id}>{&p.name}</option>
                                        }
                                    }).collect::<Html>()
                                }
                            </select>
                        </div>

                        // Status & Environment
                        <div class="space-y-2">
                            <label class={label_class}>{t.t("client_detail.status")}</label>
                            <select class={select_class}
                                onchange={
                                    let status = status.clone();
                                    Callback::from(move |e: Event| {
                                        let input: HtmlSelectElement = e.target_unchecked_into();
                                        let val = input.value();
                                        let new_status = match val.as_str() {
                                            "Active" => Some(ClientStatus::Active),
                                            "Maintenance" => Some(ClientStatus::Maintenance),
                                            "InStock" => Some(ClientStatus::InStock),
                                            "Decommissioned" => Some(ClientStatus::Decommissioned),
                                            _ => None,
                                        };
                                        status.set(new_status);
                                    })
                                }
                            >
                                <option value="" selected={status.is_none()}>{t.t("unknown")}</option>
                                <option value="Active" selected={matches!(*status, Some(ClientStatus::Active))}>{t.t("client_status.active")}</option>
                                <option value="Maintenance" selected={matches!(*status, Some(ClientStatus::Maintenance))}>{t.t("client_status.maintenance")}</option>
                                <option value="InStock" selected={matches!(*status, Some(ClientStatus::InStock))}>{t.t("client_status.in_stock")}</option>
                                <option value="Decommissioned" selected={matches!(*status, Some(ClientStatus::Decommissioned))}>{t.t("client_status.decommissioned")}</option>
                            </select>
                        </div>
                        <div class="space-y-2">
                            <label class={label_class}>{t.t("client_detail.environment")}</label>
                            <select class={select_class}
                                onchange={
                                    let environment = environment.clone();
                                    Callback::from(move |e: Event| {
                                        let input: HtmlSelectElement = e.target_unchecked_into();
                                        let val = input.value();
                                        let new_env = match val.as_str() {
                                            "Prod" => Some(Environment::Prod),
                                            "Dev" => Some(Environment::Dev),
                                            "Test" => Some(Environment::Test),
                                            "Staging" => Some(Environment::Staging),
                                            _ => None,
                                        };
                                        environment.set(new_env);
                                    })
                                }
                            >
                                <option value="" selected={environment.is_none()}>{t.t("unknown")}</option>
                                <option value="Prod" selected={matches!(*environment, Some(Environment::Prod))}>{t.t("environment.prod")}</option>
                                <option value="Dev" selected={matches!(*environment, Some(Environment::Dev))}>{t.t("environment.dev")}</option>
                                <option value="Test" selected={matches!(*environment, Some(Environment::Test))}>{t.t("environment.test")}</option>
                                <option value="Staging" selected={matches!(*environment, Some(Environment::Staging))}>{t.t("environment.staging")}</option>
                            </select>
                        </div>

                        // Asset Info
                        <div class="space-y-2">
                            <label class={label_class}>{t.t("client_detail.asset_tag")}</label>
                            <Input 
                                value={(*asset_tag).clone()}
                                oninput={Callback::from(move |val: String| {
                                    asset_tag.set(val);
                                })}
                            />
                        </div>
                        <div class="space-y-2">
                            <label class={label_class}>{t.t("client_detail.warranty")}</label>
                            <Input 
                                type_="date"
                                value={(*warranty_expiration).clone()}
                                oninput={Callback::from(move |val: String| {
                                    warranty_expiration.set(val);
                                })}
                            />
                        </div>
                        <div class="space-y-2">
                            <label class={label_class}>{t.t("client_detail.supplier")}</label>
                            <Input 
                                value={(*supplier).clone()}
                                oninput={Callback::from(move |val: String| {
                                    supplier.set(val);
                                })}
                            />
                        </div>
                    </div>

                    <div class="space-y-2">
                        <label class={label_class}>{t.t("client_detail.comment")}</label>
                        <textarea 
                            class="flex min-h-[80px] w-full rounded-md border border-input bg-transparent px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 bg-slate-950/50"
                            value={(*comment).clone()}
                            oninput={Callback::from(move |e: InputEvent| {
                                let input: HtmlInputElement = e.target_unchecked_into();
                                comment.set(input.value());
                            })}
                        ></textarea>
                    </div>

                    <ModalFooter>
                        <Button variant={ButtonVariant::Outline} onclick={props.on_cancel.reform(|_| ())}>{t.t("client_edit.cancel")}</Button>
                        <Button type_="submit">{t.t("client_edit.save")}</Button>
                    </ModalFooter>
                </form>
            </ModalContent>
        </Modal>
    }
}
