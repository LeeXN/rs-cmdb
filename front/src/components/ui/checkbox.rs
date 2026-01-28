use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct CheckboxProps {
    #[prop_or_default]
    pub checked: bool,
    #[prop_or_default]
    pub onchange: Callback<bool>,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub id: Option<String>,
}

#[function_component(Checkbox)]
pub fn checkbox(props: &CheckboxProps) -> Html {
    let onchange = {
        let onchange = props.onchange.clone();
        Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            onchange.emit(input.checked());
        })
    };

    let classes = classes!(
        "h-4",
        "w-4",
        "rounded",
        "border-cyan-500/50",
        "text-cyan-600",
        "shadow-[0_0_5px_rgba(6,182,212,0.3)]",
        "focus:ring-cyan-500/30",
        "bg-slate-950",
        "accent-cyan-600",
        "transition-all",
        "duration-200",
        "cursor-pointer",
        "disabled:cursor-not-allowed",
        "disabled:opacity-50",
        props.class.clone()
    );

    html! {
        <input
            type="checkbox"
            class={classes}
            checked={props.checked}
            onchange={onchange}
            disabled={props.disabled}
            id={props.id.clone()}
        />
    }
}
