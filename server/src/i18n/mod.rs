pub mod language;
pub mod locales;

pub use language::{Language, parse_accept_language, parse_lang_param};
use locales::en_us::get_translations as get_en_translations;
use locales::zh_cn::get_translations as get_zh_translations;
use std::collections::HashMap;

pub struct I18n {
    language: Language,
    translations: HashMap<String, String>,
}

impl I18n {
    pub fn new(language: Language) -> Self {
        let translations = match language {
            Language::EnUs => get_en_translations(),
            Language::ZhCn => get_zh_translations(),
        };

        Self {
            language,
            translations,
        }
    }

    pub fn t(&self, key: &str) -> String {
        self.translations.get(key).cloned().unwrap_or_else(|| {
            eprintln!(
                "Warning: Translation key '{}' not found for language {:?}",
                key, self.language
            );
            key.to_string()
        })
    }

    pub fn t_with_args(&self, key: &str, args: &HashMap<&str, &str>) -> String {
        let mut message = self.t(key);

        for (k, v) in args {
            message = message.replace(&format!("{{{}}}", k), v);
        }

        message
    }

    pub fn language(&self) -> &Language {
        &self.language
    }
}

impl Default for I18n {
    fn default() -> Self {
        Self::new(Language::ZhCn)
    }
}
