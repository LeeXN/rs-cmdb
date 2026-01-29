#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    ZhCn,
    EnUs,
}

impl Language {
    pub fn to_str(&self) -> &'static str {
        match self {
            Language::ZhCn => "zh-CN",
            Language::EnUs => "en-US",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "zh-cn" | "zh_cn" => Some(Language::ZhCn),
            "en-us" | "en_us" => Some(Language::EnUs),
            _ => None,
        }
    }
}

impl Default for Language {
    fn default() -> Self {
        Language::ZhCn
    }
}

/// Parse Accept-Language header to determine the preferred language
/// Supports standard Accept-Language format like "en-US,en;q=0.9,zh-CN;q=0.8"
pub fn parse_accept_language(header: Option<&str>) -> Language {
    let header = match header {
        Some(h) if !h.is_empty() => h,
        _ => return Language::ZhCn,
    };

    // Parse the Accept-Language header
    // Format: "en-US,en;q=0.9,zh-CN;q=0.8" or simple "zh-CN"
    for part in header.split(',') {
        let part = part.trim();
        let lang_part = if let Some(idx) = part.find(';') {
            &part[..idx]
        } else {
            part
        }
        .trim();

        if let Some(lang) = Language::from_str(lang_part) {
            return lang;
        }
    }

    // Default to Chinese if no supported language found
    Language::ZhCn
}

/// Parse language parameter from query string or request body
/// Supports "en-US", "zh-CN" formats
pub fn parse_lang_param(param: Option<&str>) -> Option<Language> {
    param.and_then(|p| Language::from_str(p))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_from_str() {
        assert_eq!(Language::from_str("zh-CN"), Some(Language::ZhCn));
        assert_eq!(Language::from_str("zh_cn"), Some(Language::ZhCn));
        assert_eq!(Language::from_str("zh"), None); // Changed: now requires full variant
        assert_eq!(Language::from_str("en-US"), Some(Language::EnUs));
        assert_eq!(Language::from_str("en_us"), Some(Language::EnUs));
        assert_eq!(Language::from_str("en"), None); // Changed: now requires full variant
        assert_eq!(Language::from_str("fr-FR"), None);
    }

    #[test]
    fn test_to_str() {
        assert_eq!(Language::ZhCn.to_str(), "zh-CN");
        assert_eq!(Language::EnUs.to_str(), "en-US");
    }

    #[test]
    fn test_parse_accept_language() {
        assert_eq!(parse_accept_language(Some("zh-CN")), Language::ZhCn);
        assert_eq!(parse_accept_language(Some("en-US")), Language::EnUs);
        assert_eq!(
            parse_accept_language(Some("en-US,en;q=0.9,zh-CN;q=0.8")),
            Language::EnUs
        );
        assert_eq!(
            parse_accept_language(Some("zh-CN,en;q=0.9")),
            Language::ZhCn
        );
        assert_eq!(parse_accept_language(Some("fr-FR,en;q=0.9")), Language::ZhCn);
        assert_eq!(parse_accept_language(None), Language::ZhCn);
        assert_eq!(parse_accept_language(Some("")), Language::ZhCn);
    }

    #[test]
    fn test_parse_lang_param() {
        assert_eq!(parse_lang_param(Some("zh-CN")), Some(Language::ZhCn));
        assert_eq!(parse_lang_param(Some("en-US")), Some(Language::EnUs));
        assert_eq!(parse_lang_param(Some("fr-FR")), None);
        assert_eq!(parse_lang_param(None), None);
    }
}
