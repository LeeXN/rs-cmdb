use crate::stores::auth_store::AuthStore;
use crate::types::Role;
use yew::prelude::*;
use yewdux::prelude::*;

#[derive(Properties, PartialEq)]
pub struct PermissionGuardProps {
    #[prop_or_default]
    pub children: Children,
    #[prop_or(Role::Viewer)]
    pub min_role: Role,
}

#[function_component(PermissionGuard)]
pub fn permission_guard(props: &PermissionGuardProps) -> Html {
    let (auth_store, _) = use_store::<AuthStore>();

    let has_permission = if let Some(user) = &auth_store.user {
        user.role >= props.min_role
    } else {
        false
    };

    if has_permission {
        html! {
            <>
                { for props.children.iter() }
            </>
        }
    } else {
        html! {
            <></>
        }
    }
}
