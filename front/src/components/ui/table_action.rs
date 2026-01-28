use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use lucide_yew::{Pencil, Trash2};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct TableActionProps {
    #[prop_or_default]
    pub on_edit: Option<Callback<MouseEvent>>,
    #[prop_or_default]
    pub on_delete: Option<Callback<MouseEvent>>,
    #[prop_or_default]
    pub edit_title: Option<String>,
    #[prop_or_default]
    pub delete_title: Option<String>,
}

#[function_component(TableActions)]
pub fn table_actions(props: &TableActionProps) -> Html {
    html! {
        <div class="flex items-center justify-center gap-2">
            if let Some(on_edit) = &props.on_edit {
                <Button
                    variant={ButtonVariant::Ghost}
                    size={ButtonSize::Icon}
                    onclick={on_edit.clone()}
                    title={props.edit_title.clone().unwrap_or_else(|| "编辑".to_string())}
                    class="text-cyan-400 hover:text-cyan-300 hover:bg-cyan-950/30"
                >
                    <Pencil class="w-4 h-4" />
                </Button>
            }
            if let Some(on_delete) = &props.on_delete {
                <Button
                    variant={ButtonVariant::Ghost}
                    size={ButtonSize::Icon}
                    onclick={on_delete.clone()}
                    title={props.delete_title.clone().unwrap_or_else(|| "删除".to_string())}
                    class="text-red-500 hover:text-red-400 hover:bg-red-950/30"
                >
                    <Trash2 class="w-4 h-4" />
                </Button>
            }
        </div>
    }
}
