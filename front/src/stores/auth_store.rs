use crate::types::User;
use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};
use yewdux::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Store)]
#[store(storage = "local", storage_tab_sync)]
#[derive(Default)]
pub struct AuthStore {
    pub token: Option<String>,
    #[serde(default)]
    pub refresh_token: Option<String>,
    pub user: Option<User>,
    pub is_authenticated: bool,
}

impl AuthStore {
    pub fn logout(dispatch: Dispatch<AuthStore>) {
        LocalStorage::delete("auth_store");
        LocalStorage::delete("AuthStore");
        dispatch.reduce(|_| AuthStore::default().into());
    }

    pub fn login(dispatch: Dispatch<AuthStore>, token: String, refresh_token: String, user: User) {
        dispatch.reduce(|_| {
            AuthStore {
                token: Some(token),
                refresh_token: Some(refresh_token),
                user: Some(user),
                is_authenticated: true,
            }
            .into()
        });
    }
}

#[cfg(test)]
mod tests {
    use super::AuthStore;

    #[test]
    fn old_persisted_sessions_without_refresh_token_still_deserialize() {
        let json = r#"{"token":"access","user":null,"is_authenticated":true}"#;
        let store: AuthStore = serde_json::from_str(json).unwrap();

        assert_eq!(store.token.as_deref(), Some("access"));
        assert_eq!(store.refresh_token, None);
        assert!(store.is_authenticated);
    }
}
