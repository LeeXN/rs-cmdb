use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct TableProps {
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub children: Children,
}

#[function_component(Table)]
pub fn table(props: &TableProps) -> Html {
    html! {
        <div class="relative w-full overflow-auto">
            <table class={classes!("w-full", "caption-bottom", "text-sm", props.class.clone())}>
                { for props.children.iter() }
            </table>
        </div>
    }
}

#[function_component(TableHeader)]
pub fn table_header(props: &TableProps) -> Html {
    html! {
        <thead class={classes!("[&_tr]:border-b", props.class.clone())}>
            { for props.children.iter() }
        </thead>
    }
}

#[function_component(TableBody)]
pub fn table_body(props: &TableProps) -> Html {
    html! {
        <tbody class={classes!("[&_tr:last-child]:border-0", props.class.clone())}>
            { for props.children.iter() }
        </tbody>
    }
}

#[function_component(TableFooter)]
pub fn table_footer(props: &TableProps) -> Html {
    html! {
        <tfoot class={classes!("border-t", "bg-muted/50", "font-medium", "[&>tr]:last:border-b-0", props.class.clone())}>
            { for props.children.iter() }
        </tfoot>
    }
}

#[function_component(TableRow)]
pub fn table_row(props: &TableProps) -> Html {
    html! {
        <tr class={classes!("border-b", "transition-colors", "hover:bg-muted/50", "hover:shadow-[inset_0_0_10px_rgba(6,182,212,0.05)]", "data-[state=selected]:bg-muted", props.class.clone())}>
            { for props.children.iter() }
        </tr>
    }
}

#[function_component(TableHead)]
pub fn table_head(props: &TableProps) -> Html {
    html! {
        <th class={classes!("h-12", "px-4", "text-left", "align-middle", "font-medium", "text-muted-foreground", "[&:has([role=checkbox])]:pr-0", props.class.clone())}>
            { for props.children.iter() }
        </th>
    }
}

#[function_component(TableCell)]
pub fn table_cell(props: &TableProps) -> Html {
    html! {
        <td class={classes!("p-4", "align-middle", "[&:has([role=checkbox])]:pr-0", props.class.clone())}>
            { for props.children.iter() }
        </td>
    }
}
