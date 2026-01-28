use web_sys::HtmlSelectElement;
use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
}

#[derive(Properties, PartialEq)]
pub struct SelectProps {
    #[prop_or_default]
    pub value: String,
    #[prop_or_default]
    pub onchange: Callback<String>,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub options: Vec<SelectOption>,
    #[prop_or_default]
    pub children: Children,
    #[prop_or_default]
    pub name: Option<String>,
    #[prop_or_default]
    pub id: Option<String>,
}

#[function_component(Select)]
pub fn select(props: &SelectProps) -> Html {
    let onchange = {
        let onchange = props.onchange.clone();
        Callback::from(move |e: Event| {
            let select: HtmlSelectElement = e.target_unchecked_into();
            onchange.emit(select.value());
        })
    };

    let classes = classes!(
        "flex",
        "h-10",
        "w-full",
        "items-center",
        "justify-between",
        "rounded-md",
        "border",
        "border-slate-700",
        "bg-slate-950",
        "px-3",
        "py-2",
        "text-sm",
        "text-slate-200",
        "ring-offset-slate-950",
        "placeholder:text-slate-500",
        "focus:outline-none",
        "focus:ring-2",
        "focus:ring-cyan-500/50",
        "focus:ring-offset-2",
        "focus:border-cyan-500",
        "disabled:cursor-not-allowed",
        "disabled:opacity-50",
        "transition-all",
        "duration-200",
        "shadow-sm",
        props.class.clone()
    );

    html! {
        <select
            class={classes}
            value={props.value.clone()}
            onchange={onchange}
            disabled={props.disabled}
            name={props.name.clone()}
            id={props.id.clone()}
        >
            {
                if !props.options.is_empty() {
                    html! {
                        for props.options.iter().map(|option| {
                            html! {
                                <option value={option.value.clone()} selected={option.value == props.value}>
                                    { &option.label }
                                </option>
                            }
                        })
                    }
                } else {
                    html! { for props.children.iter() }
                }
            }
        </select>
    }
}
