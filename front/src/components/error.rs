use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use lucide_yew::{CircleAlert, RefreshCw};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ErrorProps {
    pub message: String,
    #[prop_or_default]
    pub on_retry: Option<Callback<()>>,
}

#[function_component(ErrorDisplay)]
pub fn error_display(props: &ErrorProps) -> Html {
    let on_retry = {
        let on_retry = props.on_retry.clone();
        Callback::from(move |_| {
            if let Some(retry_callback) = &on_retry {
                retry_callback.emit(());
            }
        })
    };

    html! {
        <div class="container mx-auto p-4 flex justify-center">
            <div class="rounded-lg border border-destructive/50 bg-destructive/10 p-6 text-destructive max-w-md w-full shadow-sm">
                <div class="flex items-start gap-4">
                    <CircleAlert class="h-5 w-5 mt-0.5" />
                    <div class="flex-1">
                        <h5 class="mb-1 font-medium leading-none tracking-tight">{"出错了"}</h5>
                        <div class="text-sm opacity-90 mb-4">{ &props.message }</div>

                        if props.on_retry.is_some() {
                            <Button
                                variant={ButtonVariant::Destructive}
                                size={ButtonSize::Sm}
                                onclick={on_retry}
                                class="w-full sm:w-auto"
                            >
                                <RefreshCw class="mr-2 h-4 w-4" />
                                {"重试"}
                            </Button>
                        }
                    </div>
                </div>
            </div>
        </div>
    }
}
