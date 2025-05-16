use yewdux::prelude::*;
use serde::{Deserialize, Serialize};
use crate::i18n::Language;

#[derive(Store, Debug, PartialEq, Clone, Serialize, Deserialize)]
#[store(storage = "local", storage_tab_sync)]
pub struct LanguageStore {
    pub language: Language,
}

impl Default for LanguageStore {
    fn default() -> Self {
        // Default to Chinese as requested if not found in storage
        Self {
            language: Language::ZhCn,
        }
    }
}
