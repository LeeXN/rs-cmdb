use yew::prelude::*;

#[derive(Clone, PartialEq, Default)]
pub enum BadgeVariant {
    #[default]
    Default,
    Secondary,
    Destructive,
    Outline,
    Success,
    Warning,
    Info,
}

impl BadgeVariant {
    pub fn as_class(&self) -> &'static str {
        match self {
            BadgeVariant::Default => "border-transparent bg-primary text-primary-foreground hover:bg-primary/80",
            BadgeVariant::Secondary => "border-transparent bg-secondary text-secondary-foreground hover:bg-secondary/80",
            BadgeVariant::Destructive => "border-transparent bg-destructive text-destructive-foreground hover:bg-destructive/80",
            BadgeVariant::Outline => "text-foreground",
            BadgeVariant::Success => "border-transparent bg-green-500/15 text-green-500 hover:bg-green-500/25",
            BadgeVariant::Warning => "border-transparent bg-amber-500/15 text-amber-500 hover:bg-amber-500/25",
            BadgeVariant::Info => "border-transparent bg-blue-500/15 text-blue-500 hover:bg-blue-500/25",
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct BadgeProps {
    #[prop_or_default]
    pub variant: BadgeVariant,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub children: Children,
}

#[function_component(Badge)]
pub fn badge(props: &BadgeProps) -> Html {
    html! {
        <div class={classes!(
            "inline-flex", "items-center", "rounded-full", "border", "px-2.5", "py-0.5", "text-xs", "font-semibold", "transition-colors", "focus:outline-none", "focus:ring-2", "focus:ring-ring", "focus:ring-offset-2",
            props.variant.as_class(),
            props.class.clone()
        )}>
            { for props.children.iter() }
        </div>
    }
}
