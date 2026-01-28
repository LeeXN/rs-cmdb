use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod en_us;
pub mod zh_cn;

/// 支持的语言
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum Language {
    #[default]
    ZhCn,
    EnUs,
}

impl Language {
    #[allow(dead_code)]
    pub fn code(&self) -> &'static str {
        match self {
            Language::ZhCn => "zh-CN",
            Language::EnUs => "en-US",
        }
    }

    #[allow(dead_code)]
    pub fn from_code(code: &str) -> Self {
        match code {
            "en-US" | "en" => Language::EnUs,
            _ => Language::ZhCn,
        }
    }
}

/// 国际化翻译器
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct I18n {
    current_language: Language,
    translations: HashMap<String, String>,
}

impl I18n {
    #[allow(dead_code)]
    pub fn new(language: Language) -> Self {
        let translations = match language {
            Language::ZhCn => zh_cn::get_translations(),
            Language::EnUs => en_us::get_translations(),
        };

        Self {
            current_language: language,
            translations,
        }
    }

    #[allow(dead_code)]
    pub fn t(&self, key: &str) -> String {
        self.translations.get(key).cloned().unwrap_or_else(|| {
            web_sys::console::warn_1(&format!("Missing translation for key: {}", key).into());
            key.to_string()
        })
    }

    #[allow(dead_code)]
    pub fn t_with_args(&self, key: &str, args: &HashMap<String, String>) -> String {
        let mut text = self.t(key);
        for (k, v) in args {
            text = text.replace(&format!("{{{}}}", k), v);
        }
        text
    }

    #[allow(dead_code)]
    pub fn current_language(&self) -> &Language {
        &self.current_language
    }

    #[allow(dead_code)]
    pub fn set_language(&mut self, language: Language) {
        if self.current_language != language {
            self.current_language = language.clone();
            self.translations = match language {
                Language::ZhCn => zh_cn::get_translations(),
                Language::EnUs => en_us::get_translations(),
            };
        }
    }
}

impl Default for I18n {
    fn default() -> Self {
        Self::new(Language::default())
    }
}
