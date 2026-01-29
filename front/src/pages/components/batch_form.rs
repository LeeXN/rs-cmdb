use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::card::{Card, CardBody, CardHeader};
use crate::hooks::use_trans::use_trans;
use common::models::Component;
use web_sys::{HtmlInputElement, InputEvent, SubmitEvent};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct BatchCreateFormProps {
    pub on_save: Callback<Vec<Component>>,
    pub on_cancel: Callback<()>,
}

#[function_component(BatchCreateForm)]
pub fn batch_create_form(props: &BatchCreateFormProps) -> Html {
    let t = use_trans();
    let json_content = use_state(|| "".to_string());
    let error_msg = use_state(|| None::<String>);

    let onsubmit = {
        let json_content = json_content.clone();
        let error_msg = error_msg.clone();
        let on_save = props.on_save.clone();
        let t = t.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            match serde_json::from_str::<Vec<Component>>(&json_content) {
                Ok(components) => {
                    on_save.emit(components);
                }
                Err(e) => {
                    error_msg.set(Some(
                        t.t("components.json_parse_error")
                            .replace("{error}", &e.to_string()),
                    ));
                }
            }
        })
    };

    html! {
            <Card class="shadow-xl">
                <CardHeader>
                    <h6 class="text-white text-capitalize ps-3">{t.t("components.batch_create_json")}</h6>
                </CardHeader>
                <CardBody>
                    if let Some(err) = &*error_msg {
                        <div class="alert alert-danger text-white" role="alert">
                            {err}
                        </div>
                    }
                    <p class="text-sm text-slate-400 mb-2">
                        {t.t("components.json_input_hint")}
                        <pre class="bg-slate-800 p-2 border-radius-md text-slate-300 mt-2 overflow-auto">
    {r#"[
  {
    "id": "uuid-1",
    "serial_number": "SN123456",
    "model": "RTX 4090",
    "component_type": "GPU",
    "vendor": "NVIDIA",
    "status": "InStock",
    "purchase_date": "2023-01-01",
    "warranty_expiration": "2026-01-01"
  }
]"#}
                        </pre>
                    </p>
                    <form {onsubmit}>
                        <div class="mb-4">
                            <textarea class="textarea textarea-bordered w-full bg-slate-800 text-slate-200 border-slate-700 focus:border-blue-500 font-mono" rows="10"
                                value={(*json_content).clone()}
                                oninput={Callback::from(move |e: InputEvent| {
                                    let input: HtmlInputElement = e.target_unchecked_into();
                                    json_content.set(input.value());
                                })}
                            ></textarea>
                        </div>
                        <div class="flex justify-end gap-4 mt-6">
                            <Button type_="button" variant={ButtonVariant::Outline} onclick={props.on_cancel.reform(|_| ())}>{t.t("common.cancel")}</Button>
                            <Button type_="submit" variant={ButtonVariant::Default}>{t.t("components.batch_create")}</Button>
                        </div>
                    </form>
                </CardBody>
            </Card>
        }
}
