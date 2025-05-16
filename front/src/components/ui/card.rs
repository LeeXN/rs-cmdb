use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct CardProps {
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub children: Children,
}

#[function_component(Card)]
pub fn card(props: &CardProps) -> Html {
    html! {
        <div class={classes!(
            "rounded-lg", "border", "border-border", "bg-card", "text-card-foreground", "shadow-sm", "backdrop-blur-sm", "bg-opacity-90", 
            "transition-all", "duration-300", "hover:shadow-[0_0_20px_rgba(6,182,212,0.15)]", "hover:border-primary/50",
            props.class.clone()
        )}>
            { for props.children.iter() }
        </div>
    }
}

#[function_component(CardHeader)]
pub fn card_header(props: &CardProps) -> Html {
    html! {
        <div class={classes!("flex", "flex-col", "space-y-1.5", "p-6", props.class.clone())}>
            { for props.children.iter() }
        </div>
    }
}

#[function_component(CardTitle)]
pub fn card_title(props: &CardProps) -> Html {
    html! {
        <h3 class={classes!("text-2xl", "font-semibold", "leading-none", "tracking-tight", "text-primary", props.class.clone())}>
            { for props.children.iter() }
        </h3>
    }
}

#[function_component(CardDescription)]
pub fn card_description(props: &CardProps) -> Html {
    html! {
        <p class={classes!("text-sm", "text-muted-foreground", props.class.clone())}>
            { for props.children.iter() }
        </p>
    }
}

#[function_component(CardContent)]
pub fn card_content(props: &CardProps) -> Html {
    html! {
        <div class={classes!("p-6", "pt-0", props.class.clone())}>
            { for props.children.iter() }
        </div>
    }
}

#[function_component(CardBody)]
pub fn card_body(props: &CardProps) -> Html {
    html! {
        <div class={classes!("p-6", "pt-0", props.class.clone())}>
            { for props.children.iter() }
        </div>
    }
}

#[function_component(CardFooter)]
pub fn card_footer(props: &CardProps) -> Html {
    html! {
        <div class={classes!("flex", "items-center", "p-6", "pt-0", props.class.clone())}>
            { for props.children.iter() }
        </div>
    }
}
