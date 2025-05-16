use yew::prelude::*;
use yewdux::prelude::*;
use std::rc::Rc;
use crate::stores::language_store::LanguageStore;
use crate::i18n::I18n;

/// A hook that provides translation capabilities and triggers re-renders on language change.
#[hook]
pub fn use_trans() -> Rc<I18n> {
    let (store, _) = use_store::<LanguageStore>();
    // Create a new I18n instance based on the current store language.
    // Since I18n creation is cheap (hashmap lookup), we can do it here.
    // Optimization: We could cache the I18n instance in the store or a context if needed,
    // but for now, creating it on the fly ensures reactivity.
    Rc::new(I18n::new(store.language.clone()))
}
