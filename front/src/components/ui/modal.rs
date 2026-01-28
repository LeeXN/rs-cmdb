use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ModalProps {
    #[prop_or_default]
    pub title: String, // Optional now, as we might use ModalTitle
    #[prop_or_default]
    pub is_open: bool,
    #[prop_or_default]
    pub on_close: Callback<()>,
    #[prop_or_default]
    pub on_open_change: Callback<bool>, // Add this to support the pattern seen in errors
    #[prop_or_default]
    pub children: Children,
    #[prop_or_default]
    pub footer: Option<Html>,
    #[prop_or_default]
    pub open: bool, // Alias for is_open to support the pattern seen in errors
}

#[function_component(Modal)]
pub fn modal(props: &ModalProps) -> Html {
    let is_open = props.is_open || props.open;

    if !is_open {
        return html! {};
    }

    let on_close = props.on_close.clone();
    let on_open_change = props.on_open_change.clone();

    let on_backdrop_click = Callback::from(move |_e: MouseEvent| {
        on_close.emit(());
        on_open_change.emit(false);
    });

    let on_content_click = Callback::from(|e: MouseEvent| {
        e.stop_propagation();
    });

    html! {
        <div class="fixed inset-0 z-50 bg-background/80 backdrop-blur-sm flex items-center justify-center" onclick={on_backdrop_click}>
            <div class="z-50 grid w-full max-w-lg gap-4 border bg-background p-6 shadow-lg sm:rounded-lg border-primary/20 shadow-[0_0_20px_rgba(6,182,212,0.2)]" onclick={on_content_click}>
                if !props.title.is_empty() {
                    <div class="flex flex-col space-y-1.5 text-center sm:text-left">
                        <h2 class="text-lg font-semibold leading-none tracking-tight text-primary">{ &props.title }</h2>
                    </div>
                }
                { for props.children.iter() }
                if let Some(footer) = &props.footer {
                    <div class="flex flex-col-reverse sm:flex-row sm:justify-end sm:space-x-2">
                        { footer.clone() }
                    </div>
                }
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct ModalSubProps {
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub children: Children,
}

#[function_component(ModalHeader)]
pub fn modal_header(props: &ModalSubProps) -> Html {
    html! {
        <div class={classes!("flex", "flex-col", "space-y-1.5", "text-center", "sm:text-left", props.class.clone())}>
            { for props.children.iter() }
        </div>
    }
}

#[function_component(ModalFooter)]
pub fn modal_footer(props: &ModalSubProps) -> Html {
    html! {
        <div class={classes!("flex", "flex-col-reverse", "sm:flex-row", "sm:justify-end", "sm:space-x-2", props.class.clone())}>
            { for props.children.iter() }
        </div>
    }
}

#[function_component(ModalTitle)]
pub fn modal_title(props: &ModalSubProps) -> Html {
    html! {
        <h2 class={classes!("text-lg", "font-semibold", "leading-none", "tracking-tight", props.class.clone())}>
            { for props.children.iter() }
        </h2>
    }
}

#[function_component(ModalDescription)]
pub fn modal_description(props: &ModalSubProps) -> Html {
    html! {
        <p class={classes!("text-sm", "text-muted-foreground", props.class.clone())}>
            { for props.children.iter() }
        </p>
    }
}

#[function_component(ModalContent)]
pub fn modal_content(props: &ModalSubProps) -> Html {
    html! {
        <div class={classes!("grid", "gap-4", "py-4", props.class.clone())}>
             { for props.children.iter() }
        </div>
    }
}
