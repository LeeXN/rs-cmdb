use yew::prelude::*;
use crate::components::ui::modal::Modal;
use crate::components::ui::button::{Button, ButtonVariant};
use crate::hooks::use_trans::use_trans;

#[derive(Properties, PartialEq)]
pub struct ConfirmModalProps {
    pub is_open: bool,
    pub title: String,
    pub message: String,
    pub on_confirm: Callback<()>,
    pub on_cancel: Callback<()>,
    #[prop_or_default]
    pub error_message: Option<String>,
}

#[function_component(ConfirmModal)]
pub fn confirm_modal(props: &ConfirmModalProps) -> Html {
    let t = use_trans();
    let on_confirm = {
        let on_confirm = props.on_confirm.clone();
        Callback::from(move |_: MouseEvent| {
            on_confirm.emit(());
        })
    };

    let on_cancel_click = {
        let on_cancel = props.on_cancel.clone();
        Callback::from(move |_: MouseEvent| {
            on_cancel.emit(());
        })
    };

    let on_modal_close = {
        let on_cancel = props.on_cancel.clone();
        Callback::from(move |_: ()| {
            on_cancel.emit(());
        })
    };

    let footer = html! {
        <div class="flex justify-end gap-2">
            <Button variant={ButtonVariant::Outline} onclick={on_cancel_click}>{t.t("common.cancel")}</Button>
            <Button variant={ButtonVariant::Destructive} onclick={on_confirm}>{t.t("common.delete")}</Button>
        </div>
    };

    html! {
        <Modal 
            title={props.title.clone()} 
            is_open={props.is_open} 
            on_close={on_modal_close}
            footer={footer}
        >
            <p class="text-sm text-slate-300">
                { &props.message }
            </p>
            if let Some(err) = &props.error_message {
                <div class="mt-4 p-3 bg-red-500/10 border border-red-500/20 rounded text-red-400 text-sm">
                    { err }
                </div>
            }
        </Modal>
    }
}
