use yewdux::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Store)]
#[store(storage = "local", storage_tab_sync)]
pub struct ThemeStore {
    pub theme: String,
}

impl Default for ThemeStore {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
        }
    }
}

impl ThemeStore {
    #[allow(dead_code)]
    pub fn is_dark(&self) -> bool {
        self.theme == "dark"
    }
}
