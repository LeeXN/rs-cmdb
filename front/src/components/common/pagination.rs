use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::input::Input;
use crate::hooks::use_trans::use_trans;
use std::collections::HashMap;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct PaginationProps {
    pub total_pages: usize,
    pub current_page: usize,
    pub page_size: usize,
    pub total_items: usize,
    pub on_page_change: Callback<usize>,
    pub on_page_size_change: Callback<usize>,
}

#[function_component(Pagination)]
pub fn pagination(props: &PaginationProps) -> Html {
    let t = use_trans();
    let jump_page_input = use_node_ref();
    let jump_page_value = use_state(String::new);

    let total_pages = props.total_pages;
    let current_page = props.current_page;
    let page_size = props.page_size;
    let total_items = props.total_items;
    let on_page_change = props.on_page_change.clone();
    let on_page_size_change = props.on_page_size_change.clone();

    let on_jump_page = {
        let jump_page_input = jump_page_input.clone();
        let jump_page_value = jump_page_value.clone();
        let on_page_change = on_page_change.clone();

        Callback::from(move |_: MouseEvent| {
            if let Some(input) = jump_page_input.cast::<HtmlInputElement>() {
                if let Ok(page) = input.value().parse::<usize>() {
                    if page >= 1 && page <= total_pages {
                        on_page_change.emit(page);
                        input.set_value("");
                        jump_page_value.set(String::new());
                    }
                }
            }
        })
    };

    let on_input_change = {
        let jump_page_value = jump_page_value.clone();
        Callback::from(move |val: String| {
            jump_page_value.set(val);
        })
    };

    let on_key_down = {
        let on_jump_page = on_jump_page.clone();
        Callback::from(move |e: KeyboardEvent| {
            if e.key() == "Enter" {
                on_jump_page.emit(MouseEvent::new("click").unwrap());
            }
        })
    };

    let select_class = "flex h-8 w-full items-center justify-between rounded-md border border-input bg-slate-950 px-2 py-1 text-xs text-slate-200 shadow-sm ring-offset-background placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-ring disabled:cursor-not-allowed disabled:opacity-50";

    if total_pages <= 1 && total_items <= page_size {
        return html! {
            <div class="flex justify-between items-center mt-4 p-4 bg-card/50 rounded-lg border border-border backdrop-blur-sm">
                <div class="flex items-center gap-2">
                    <span class="text-sm text-muted-foreground">{t.t_with_args("pagination.total_items", &HashMap::from([("count".to_string(), total_items.to_string())]))}</span>
                    <span class="text-muted-foreground">{"•"}</span>
                    <span class="text-sm text-muted-foreground">{t.t("pagination.per_page")}</span>
                    <select
                        class={select_class}
                        value={page_size.to_string()}
                        onchange={
                            Callback::from(move |e: Event| {
                                let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
                                if let Ok(size) = select.value().parse::<usize>() {
                                    on_page_size_change.emit(size);
                                }
                            })
                        }
                    >
                        <option value="10" selected={page_size == 10}>{"10"}</option>
                        <option value="20" selected={page_size == 20}>{"20"}</option>
                        <option value="50" selected={page_size == 50}>{"50"}</option>
                        <option value="100" selected={page_size == 100}>{"100"}</option>
                    </select>
                    <span class="text-sm text-muted-foreground">{t.t("pagination.unit")}</span>
                </div>
            </div>
        };
    }

    html! {
        <div class="flex flex-col md:flex-row justify-between items-center mt-4 p-4 bg-card/50 rounded-lg gap-4 border border-border backdrop-blur-sm">
            // 左侧：统计信息和每页条数
            <div class="flex items-center gap-2">
                <span class="text-sm text-muted-foreground">{t.t_with_args("pagination.total_items", &HashMap::from([("count".to_string(), total_items.to_string())]))}</span>
                <span class="text-muted-foreground">{"•"}</span>
                <select
                    class={select_class}
                    value={page_size.to_string()}
                    onchange={
                        let on_page_size_change = on_page_size_change.clone();
                        Callback::from(move |e: Event| {
                            let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
                            if let Ok(size) = select.value().parse::<usize>() {
                                on_page_size_change.emit(size);
                            }
                        })
                    }
                >
                    <option value="10" selected={page_size == 10}>{"10"}</option>
                    <option value="20" selected={page_size == 20}>{"20"}</option>
                    <option value="50" selected={page_size == 50}>{"50"}</option>
                    <option value="100" selected={page_size == 100}>{"100"}</option>
                </select>
                <span class="text-sm text-muted-foreground">{t.t("pagination.items_per_page")}</span>
            </div>

            // 中间：分页导航
            <div class="flex items-center gap-1">
                // 首页
                <Button
                    variant={ButtonVariant::Outline}
                    size={ButtonSize::Sm}
                    class="h-8 w-8 p-0"
                    onclick={
                        let on_page_change = on_page_change.clone();
                        Callback::from(move |_: MouseEvent| {
                            if current_page > 1 {
                                on_page_change.emit(1);
                            }
                        })
                    }
                    disabled={current_page == 1}
                >
                    {"‹‹"}
                </Button>

                // 上一页
                <Button
                    variant={ButtonVariant::Outline}
                    size={ButtonSize::Sm}
                    class="h-8 w-8 p-0"
                    onclick={
                        let on_page_change = on_page_change.clone();
                        Callback::from(move |_: MouseEvent| {
                            if current_page > 1 {
                                on_page_change.emit(current_page - 1);
                            }
                        })
                    }
                    disabled={current_page == 1}
                >
                    {"‹"}
                </Button>

                // 页码
                {render_page_numbers(total_pages, current_page, on_page_change.clone())}

                // 下一页
                <Button
                    variant={ButtonVariant::Outline}
                    size={ButtonSize::Sm}
                    class="h-8 w-8 p-0"
                    onclick={
                        let on_page_change = on_page_change.clone();
                        Callback::from(move |_: MouseEvent| {
                            if current_page < total_pages {
                                on_page_change.emit(current_page + 1);
                            }
                        })
                    }
                    disabled={current_page == total_pages}
                >
                    {"›"}
                </Button>

                // 尾页
                <Button
                    variant={ButtonVariant::Outline}
                    size={ButtonSize::Sm}
                    class="h-8 w-8 p-0"
                    onclick={
                        let on_page_change = on_page_change.clone();
                        Callback::from(move |_: MouseEvent| {
                            if current_page < total_pages {
                                on_page_change.emit(total_pages);
                            }
                        })
                    }
                    disabled={current_page == total_pages}
                >
                    {"››"}
                </Button>
            </div>

            // 右侧：当前页信息和快速跳转
            <div class="flex items-center gap-2">
                <span class="text-sm text-muted-foreground">{format!("{}/{}", current_page, total_pages)}</span>
                <span class="text-muted-foreground">{"•"}</span>
                <span class="text-xs text-muted-foreground">{t.t("pagination.jump_to")}</span>
                <Input
                    node_ref={jump_page_input}
                    type_="number"
                    class="h-8 w-16 text-center"
                    min={Some("1".to_string())}
                    max={Some(total_pages.to_string())}
                    placeholder={current_page.to_string()}
                    value={(*jump_page_value).clone()}
                    oninput={on_input_change}
                    onkeydown={on_key_down}
                />
                <Button
                    variant={ButtonVariant::Outline}
                    size={ButtonSize::Sm}
                    class="h-8 px-2 text-xs"
                    onclick={on_jump_page}
                >
                    {t.t("pagination.go")}
                </Button>
            </div>
        </div>
    }
}

fn render_page_numbers(
    total_pages: usize,
    current_page: usize,
    on_page_change: Callback<usize>,
) -> Html {
    // 智能分页：显示当前页前后几页
    let start_page = if current_page > 3 {
        current_page - 2
    } else {
        1
    };
    let end_page = std::cmp::min(start_page + 4, total_pages);
    let actual_start = if end_page == total_pages && total_pages > 5 {
        std::cmp::max(1, total_pages - 4)
    } else {
        start_page
    };

    let mut pages = Vec::new();

    // 如果开始页不是1，显示省略号
    if actual_start > 2 {
        pages.push(html! {
                <Button variant={ButtonVariant::Ghost} size={ButtonSize::Sm} class="h-8 w-8 p-0" disabled={true} key="start-ellipsis">
                    {"⋯"}
                </Button>
            });
    }

    // 页码
    for page in actual_start..=end_page {
        let is_current = page == current_page;
        let variant = if is_current {
            ButtonVariant::Default
        } else {
            ButtonVariant::Outline
        };

        pages.push(html! {
            <Button
                variant={variant}
                size={ButtonSize::Sm}
                class="h-8 w-8 p-0"
                key={page}
                onclick={
                    let on_page_change = on_page_change.clone();
                    Callback::from(move |_: MouseEvent| {
                        on_page_change.emit(page);
                    })
                }
            >
                {page}
            </Button>
        });
    }

    // 如果结束页不是最后一页，显示省略号
    if end_page < total_pages && end_page < total_pages - 1 {
        pages.push(html! {
                <Button variant={ButtonVariant::Ghost} size={ButtonSize::Sm} class="h-8 w-8 p-0" disabled={true} key="end-ellipsis">
                    {"⋯"}
                </Button>
            });
    }

    html! { for pages }
}
