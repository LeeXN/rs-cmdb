use yew::prelude::*;

#[derive(Clone, PartialEq, Default)]
#[allow(dead_code)]
pub enum ButtonVariant {
    #[default]
    Default,
    Destructive,
    Outline,
    Secondary,
    Ghost,
    Link,
}

impl ButtonVariant {
    pub fn as_class(&self) -> &'static str {
        match self {
            ButtonVariant::Default => "bg-cyan-600 text-black hover:bg-cyan-500 shadow-[0_0_10px_rgba(8,145,178,0.5)] hover:shadow-[0_0_20px_rgba(8,145,178,0.8)] transition-all duration-300 font-bold",
            ButtonVariant::Destructive => "bg-red-600 text-white hover:bg-red-700 shadow-sm hover:shadow-[0_0_15px_rgba(220,38,38,0.4)] transition-all duration-300",
            ButtonVariant::Outline => "border border-slate-700 bg-transparent text-slate-300 hover:bg-slate-800 hover:text-white hover:border-slate-600 transition-all duration-300",
            ButtonVariant::Secondary => "bg-slate-700 text-slate-200 hover:bg-slate-600",
            ButtonVariant::Ghost => "hover:bg-slate-800/50 transition-colors duration-200",
            ButtonVariant::Link => "text-cyan-500 underline-offset-4 hover:underline",
        }
    }
}

#[derive(Clone, PartialEq, Default)]
#[allow(dead_code)]
pub enum ButtonSize {
    #[default]
    Default,
    Sm,
    Lg,
    Icon,
}

impl ButtonSize {
    pub fn as_class(&self) -> &'static str {
        match self {
            ButtonSize::Default => "h-10 px-4 py-2",
            ButtonSize::Sm => "h-8 rounded-md px-3 text-xs",
            ButtonSize::Lg => "h-11 rounded-md px-8",
            ButtonSize::Icon => "h-8 w-8 p-0",
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct ButtonProps {
    #[prop_or_default]
    pub variant: ButtonVariant,
    #[prop_or_default]
    pub size: ButtonSize,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub onclick: Callback<MouseEvent>,
    #[prop_or_default]
    pub children: Children,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub type_: Option<String>,
    #[prop_or_default]
    pub title: Option<String>,
}

#[function_component(Button)]
pub fn button(props: &ButtonProps) -> Html {
    let type_ = props.type_.clone().unwrap_or_else(|| "button".to_string());
    let classes = classes!(
        "inline-flex",
        "items-center",
        "justify-center",
        "whitespace-nowrap",
        "rounded-md",
        "text-sm",
        "font-medium",
        "ring-offset-background",
        "transition-colors",
        "focus-visible:outline-none",
        "focus-visible:ring-2",
        "focus-visible:ring-ring",
        "focus-visible:ring-offset-2",
        "disabled:pointer-events-none",
        "disabled:opacity-50",
        props.variant.as_class(),
        props.size.as_class(),
        props.class.clone(),
    );

    html! {
        <button
            type={type_}
            class={classes}
            onclick={props.onclick.clone()}
            disabled={props.disabled}
            title={props.title.clone()}
        >
            { for props.children.iter() }
        </button>
    }
}
