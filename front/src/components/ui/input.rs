use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct InputProps {
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub type_: String,
    #[prop_or_default]
    pub placeholder: String,
    #[prop_or_default]
    pub value: String,
    #[prop_or_default]
    pub oninput: Callback<String>,
    #[prop_or_default]
    pub onchange: Callback<String>,
    #[prop_or_default]
    pub onkeydown: Callback<KeyboardEvent>,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub name: String,
    #[prop_or_default]
    pub id: String,
    #[prop_or_default]
    pub min: Option<String>,
    #[prop_or_default]
    pub max: Option<String>,
    #[prop_or_default]
    pub required: bool,
    #[prop_or_default]
    pub node_ref: NodeRef,
}

#[function_component(Input)]
pub fn input(props: &InputProps) -> Html {
    let type_ = if props.type_.is_empty() {
        "text".to_string()
    } else {
        props.type_.clone()
    };

    let oninput = {
        let oninput = props.oninput.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            oninput.emit(input.value());
        })
    };

    let onchange = {
        let onchange = props.onchange.clone();
        Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            onchange.emit(input.value());
        })
    };

    html! {
        <input
            type={type_}
            ref={props.node_ref.clone()}
            class={classes!(
                "flex", "h-10", "w-full", "rounded-md", "border", "border-input", "bg-background", "px-3", "py-2", "text-sm", "ring-offset-background",
                "file:border-0", "file:bg-transparent", "file:text-sm", "file:font-medium", "placeholder:text-muted-foreground",
                "focus-visible:outline-none", "focus-visible:ring-2", "focus-visible:ring-ring", "focus-visible:ring-offset-2", "focus-visible:shadow-[0_0_10px_rgba(6,182,212,0.3)]",
                "disabled:cursor-not-allowed", "disabled:opacity-50", "bg-slate-950/50", "transition-all", "duration-200",
                props.class.clone()
            )}
            placeholder={props.placeholder.clone()}
            value={props.value.clone()}
            oninput={oninput}
            onchange={onchange}
            onkeydown={props.onkeydown.clone()}
            disabled={props.disabled}
            name={props.name.clone()}
            id={props.id.clone()}
            min={props.min.clone()}
            max={props.max.clone()}
            required={props.required}
        />
    }
}
