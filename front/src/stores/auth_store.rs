use yewdux::prelude::*;
use serde::{Serialize, Deserialize};
use crate::types::User;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Store)]
#[store(storage = "local", storage_tab_sync)]
pub struct AuthStore {
    pub token: Option<String>,
    pub user: Option<User>,
    pub is_authenticated: bool,
}

impl Default for AuthStore {
    fn default() -> Self {
        Self {
            token: None,
            user: None,
            is_authenticated: false,
        }
    }
}

impl AuthStore {
    pub fn logout(dispatch: Dispatch<AuthStore>) {
        dispatch.reduce(|_| AuthStore::default().into());
    }

    pub fn login(dispatch: Dispatch<AuthStore>, token: String, user: User) {
        dispatch.reduce(|_| AuthStore {
            token: Some(token),
            user: Some(user),
            is_authenticated: true,
        }.into());
    }
}
