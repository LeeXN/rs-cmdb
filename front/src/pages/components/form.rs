use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::card::{Card, CardBody, CardHeader};
use crate::components::ui::input::Input;
use crate::components::ui::select::{Select, SelectOption};
use crate::hooks::use_trans::use_trans;
use common::models::{Component, ComponentStatus, ComponentType};
use std::collections::HashMap;
use web_sys::SubmitEvent;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ComponentFormProps {
    pub component: Component,
    pub on_save: Callback<Component>,
    pub on_cancel: Callback<()>,
    #[prop_or(false)]
    pub is_new: bool,
}

#[function_component(ComponentForm)]
pub fn component_form(props: &ComponentFormProps) -> Html {
    let t = use_trans();
    let status = use_state(|| props.component.status.clone());
    let location = use_state(|| props.component.location.clone().unwrap_or_default());
    let purchase_date = use_state(|| props.component.purchase_date.clone().unwrap_or_default());
    let warranty_expiration = use_state(|| {
        props
            .component
            .warranty_expiration
            .clone()
            .unwrap_or_default()
    });

    // New fields for creation
    let component_type = use_state(|| props.component.component_type.clone());
    let vendor = use_state(|| props.component.vendor.clone().unwrap_or_default());
    let model = use_state(|| props.component.model.clone());
    let serial_number = use_state(|| props.component.serial_number.clone());

    let onsubmit = {
        let status = status.clone();
        let location = location.clone();
        let purchase_date = purchase_date.clone();
        let warranty_expiration = warranty_expiration.clone();
        let component_type = component_type.clone();
        let vendor = vendor.clone();
        let model = model.clone();
        let serial_number = serial_number.clone();
        let props_component = props.component.clone();
        let on_save = props.on_save.clone();
        let is_new = props.is_new;

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let mut component = props_component.clone();
            component.status = (*status).clone();
            component.location = if (*location).is_empty() {
                None
            } else {
                Some((*location).clone())
            };
            component.purchase_date = if (*purchase_date).is_empty() {
                None
            } else {
                Some((*purchase_date).clone())
            };
            component.warranty_expiration = if (*warranty_expiration).is_empty() {
                None
            } else {
                Some((*warranty_expiration).clone())
            };

            if is_new {
                component.component_type = (*component_type).clone();
                component.vendor = if (*vendor).is_empty() {
                    None
                } else {
                    Some((*vendor).clone())
                };
                component.model = (*model).clone();
                component.serial_number = (*serial_number).clone();
            }

            on_save.emit(component);
        })
    };

    let type_options = vec![
        SelectOption {
            value: "Other".to_string(),
            label: t.t("components.type_other"),
        },
        SelectOption {
            value: "GPU".to_string(),
            label: t.t("components.type_gpu"),
        },
        SelectOption {
            value: "CPU".to_string(),
            label: t.t("components.type_cpu"),
        },
        SelectOption {
            value: "Memory".to_string(),
            label: t.t("components.type_memory"),
        },
        SelectOption {
            value: "Disk".to_string(),
            label: t.t("components.type_disk"),
        },
        SelectOption {
            value: "NetworkCard".to_string(),
            label: t.t("components.type_network_card"),
        },
        SelectOption {
            value: "Motherboard".to_string(),
            label: t.t("components.type_motherboard"),
        },
        SelectOption {
            value: "PowerSupply".to_string(),
            label: t.t("components.type_power_supply"),
        },
    ];

    let status_options = vec![
        SelectOption {
            value: "InStock".to_string(),
            label: t.t("components.status_in_stock"),
        },
        SelectOption {
            value: "InUse".to_string(),
            label: t.t("components.status_in_use"),
        },
        SelectOption {
            value: "LentOut".to_string(),
            label: t.t("components.status_lent_out"),
        },
        SelectOption {
            value: "Faulty".to_string(),
            label: t.t("components.status_faulty"),
        },
        SelectOption {
            value: "Decommissioned".to_string(),
            label: t.t("components.status_decommissioned"),
        },
        SelectOption {
            value: "Unknown".to_string(),
            label: t.t("components.status_unknown"),
        },
    ];

    html! {
        <Card class="shadow-xl">
            <CardHeader>
                <h6 class="text-white text-capitalize ps-3">
                    if props.is_new {
                        {t.t("components.new_component")}
                    } else {
                        {t.t_with_args("components.edit_component", &HashMap::from([
                            ("model".to_string(), props.component.model.clone()),
                            ("sn".to_string(), props.component.serial_number.clone())
                        ]))}
                    }
                </h6>
            </CardHeader>
            <CardBody>
                <form {onsubmit}>
                    if props.is_new {
                        <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mb-4">
                            <div>
                                <label class="block text-sm font-bold text-slate-400 mb-1">{t.t("components.serial_number")}</label>
                                <Input
                                    value={(*serial_number).clone()}
                                    oninput={Callback::from(move |val: String| serial_number.set(val))}
                                    required=true
                                />
                            </div>
                            <div>
                                <label class="block text-sm font-bold text-slate-400 mb-1">{t.t("components.model")}</label>
                                <Input
                                    value={(*model).clone()}
                                    oninput={Callback::from(move |val: String| model.set(val))}
                                    required=true
                                />
                            </div>
                        </div>
                    }

                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mb-4">
                        <div>
                            <label class="block text-sm font-bold text-slate-400 mb-1">{t.t("components.type")}</label>
                            if props.is_new {
                                <Select
                                    options={type_options}
                                    value={format!("{:?}", *component_type)}
                                    onchange={
                                        let component_type = component_type.clone();
                                        Callback::from(move |val: String| {
                                            let new_type = match val.as_str() {
                                                "GPU" => ComponentType::GPU,
                                                "CPU" => ComponentType::CPU,
                                                "Memory" => ComponentType::Memory,
                                                "Disk" => ComponentType::Disk,
                                                "NetworkCard" => ComponentType::NetworkCard,
                                                "Motherboard" => ComponentType::Motherboard,
                                                "PowerSupply" => ComponentType::PowerSupply,
                                                _ => ComponentType::Other,
                                            };
                                            component_type.set(new_type);
                                        })
                                    }
                                />
                            } else {
                                <Input value={format!("{:?}", props.component.component_type)} disabled=true />
                            }
                        </div>
                        <div>
                            <label class="block text-sm font-bold text-slate-400 mb-1">{t.t("components.vendor")}</label>
                            if props.is_new {
                                <Input
                                    value={(*vendor).clone()}
                                    oninput={Callback::from(move |val: String| vendor.set(val))}
                                />
                            } else {
                                <Input value={props.component.vendor.clone().unwrap_or_default()} disabled=true />
                            }
                        </div>
                    </div>

                    <div class="mb-4">
                        <label class="block text-sm font-bold text-slate-400 mb-1">{t.t("components.status")}</label>
                        <Select
                            options={status_options}
                            value={format!("{:?}", *status)}
                            onchange={
                                let status = status.clone();
                                Callback::from(move |val: String| {
                                    let new_status = match val.as_str() {
                                        "InStock" => ComponentStatus::InStock,
                                        "InUse" => ComponentStatus::InUse,
                                        "LentOut" => ComponentStatus::LentOut,
                                        "Faulty" => ComponentStatus::Faulty,
                                        "Decommissioned" => ComponentStatus::Decommissioned,
                                        _ => ComponentStatus::Unknown,
                                    };
                                    status.set(new_status);
                                })
                            }
                        />
                    </div>

                    <div class="mb-4">
                        <label class="block text-sm font-bold text-slate-400 mb-1">{t.t("components.location")}</label>
                        <Input
                            value={(*location).clone()}
                            oninput={Callback::from(move |val: String| location.set(val))}
                        />
                    </div>

                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mb-4">
                        <div>
                            <label class="block text-sm font-bold text-slate-400 mb-1">{t.t("components.purchase_date")}</label>
                            <Input
                                type_="date"
                                value={(*purchase_date).clone()}
                                oninput={Callback::from(move |val: String| purchase_date.set(val))}
                            />
                        </div>
                        <div>
                            <label class="block text-sm font-bold text-slate-400 mb-1">{t.t("components.warranty_expiration")}</label>
                            <Input
                                type_="date"
                                value={(*warranty_expiration).clone()}
                                oninput={Callback::from(move |val: String| warranty_expiration.set(val))}
                            />
                        </div>
                    </div>

                    <div class="flex justify-end gap-4 mt-6">
                        <Button type_="button" variant={ButtonVariant::Outline} onclick={props.on_cancel.reform(|_| ())}>{t.t("common.cancel")}</Button>
                        <Button type_="submit" variant={ButtonVariant::Default}>{t.t("common.save")}</Button>
                    </div>
                </form>
            </CardBody>
        </Card>
    }
}
