use yew::prelude::*;
use lucide_yew::LoaderCircle;

#[derive(Properties, PartialEq)]
pub struct LoadingProps {
    #[prop_or("加载中...".to_string())]
    pub message: String,
    #[prop_or("primary".to_string())]
    pub color: String,
}

#[function_component(Loading)]
pub fn loading(props: &LoadingProps) -> Html {
    html! {
        <div class="flex flex-col items-center justify-center py-10 space-y-4">
            <LoaderCircle class="h-10 w-10 animate-spin text-primary" />
            <p class="text-sm text-muted-foreground">{ &props.message }</p>
        </div>
    }
} 