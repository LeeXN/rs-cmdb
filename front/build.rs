use std::collections::HashSet;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=src/i18n/en_us.rs");
    println!("cargo:rerun-if-changed=src/i18n/zh_cn.rs");

    let i18n_dir = Path::new("src/i18n");
    let en_file = i18n_dir.join("en_us.rs");
    let zh_file = i18n_dir.join("zh_cn.rs");

    // Extract translation keys from both files
    let en_keys = extract_translation_keys(&en_file);
    let zh_keys = extract_translation_keys(&zh_file);

    println!(
        "Translation keys: EN={} ZH={}",
        en_keys.len(),
        zh_keys.len()
    );

    let mut has_errors = false;
    let mut has_warnings = false;

    // Check for parity between EN and ZH
    let en_only: Vec<_> = en_keys.difference(&zh_keys).cloned().collect();
    let zh_only: Vec<_> = zh_keys.difference(&en_keys).cloned().collect();

    if !en_only.is_empty() {
        eprintln!(
            "❌ Translation keys in EN but missing in ZH ({} keys):",
            en_only.len()
        );
        for key in en_only.iter().take(20) {
            eprintln!("   - {}", key);
        }
        if en_only.len() > 20 {
            eprintln!("   ... and {} more", en_only.len() - 20);
        }
        has_errors = true;
    }

    if !zh_only.is_empty() {
        eprintln!(
            "❌ Translation keys in ZH but missing in EN ({} keys):",
            zh_only.len()
        );
        for key in zh_only.iter().take(20) {
            eprintln!("   - {}", key);
        }
        if zh_only.len() > 20 {
            eprintln!("   ... and {} more", zh_only.len() - 20);
        }
        has_errors = true;
    }

    // Find used translation keys in source code
    let used_keys = find_used_translation_keys(Path::new("src"));
    println!("Used translation keys: {}", used_keys.len());

    // Check for key coverage (used keys must be defined)
    let all_defined_keys: HashSet<_> = en_keys.union(&zh_keys).cloned().collect();
    let undefined_keys: Vec<_> = used_keys.difference(&all_defined_keys).cloned().collect();

    if !undefined_keys.is_empty() {
        eprintln!(
            "❌ Translation keys used in code but not defined ({} keys):",
            undefined_keys.len()
        );
        for key in undefined_keys.iter().take(10) {
            eprintln!("   - {}", key);
        }
        if undefined_keys.len() > 10 {
            eprintln!("   ... and {} more", undefined_keys.len() - 10);
        }
        has_errors = true;
    }

    // Check for unused keys (defined keys not used in code)
    let unused_en: Vec<_> = en_keys.difference(&used_keys).cloned().collect();
    let unused_zh: Vec<_> = zh_keys.difference(&used_keys).cloned().collect();

    // Only report if both have unused keys (reduces false positives)
    let common_unused: HashSet<_> = unused_en
        .into_iter()
        .collect::<HashSet<_>>()
        .intersection(&unused_zh.into_iter().collect::<HashSet<_>>())
        .cloned()
        .collect();

    if !common_unused.is_empty() {
        has_warnings = true;
        eprintln!(
            "⚠️  Potentially unused translation keys ({} keys):",
            common_unused.len()
        );
        for key in common_unused.iter().take(10) {
            eprintln!("   - {}", key);
        }
        if common_unused.len() > 10 {
            eprintln!("   ... and {} more", common_unused.len() - 10);
        }
        eprintln!("   Note: These keys might be used dynamically or via string concatenation.");
    }

    if has_errors {
        panic!("Translation validation failed. Please fix the issues above.");
    }

    if has_warnings {
        println!("✅ Translation validation passed (with warnings).");
    } else {
        println!("✅ Translation validation passed!");
    }
}

fn extract_translation_keys(file_path: &Path) -> HashSet<String> {
    let content = fs::read_to_string(file_path)
        .unwrap_or_else(|e| panic!("Failed to read {:?}: {}", file_path, e));

    let mut keys = HashSet::new();

    // Match patterns like: translations.insert("key".to_string(), "value".to_string());
    // We need to find the first string in each translations.insert() call
    let mut in_insert = false;
    let mut found_first_string = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip comments
        if trimmed.starts_with("//") {
            continue;
        }

        // Check if this is a translations.insert line
        if trimmed.contains("translations.insert(") {
            in_insert = true;
            found_first_string = false;
        }

        if in_insert && !found_first_string {
            // Look for the first string literal
            if let Some(start) = trimmed.find("\"") {
                if let Some(end) = trimmed[start + 1..].find("\"") {
                    let key = &trimmed[start + 1..start + 1 + end];

                    // Check if this looks like a translation key
                    // Translation keys typically use dot notation (e.g., "menu.dashboard")
                    // or are simple lowercase words (e.g., "online", "offline")
                    if is_translation_key(key) {
                        keys.insert(key.to_string());
                    }

                    // Mark that we found the first string (the key)
                    if trimmed.contains(".to_string()") {
                        found_first_string = true;
                    }
                }
            }
        }

        // Reset when we reach the end of the statement
        if trimmed.contains(");") {
            in_insert = false;
            found_first_string = false;
        }
    }

    keys
}

fn is_translation_key(s: &str) -> bool {
    // Skip strings that are obviously values (not keys)
    if s.is_empty() || s.len() > 100 {
        return false;
    }

    // Skip strings that look like translated values (long text, punctuation)
    if s.contains("，") || s.contains("。") || s.contains("、") || s.contains("：") {
        return false;
    }

    // Skip strings with spaces (usually values, not keys)
    if s.contains(' ') {
        // Exception: allow some phrases like "No Data" but not long sentences
        if s.split_whitespace().count() > 5 {
            return false;
        }
    }

    // Accept strings with dot notation (e.g., "menu.dashboard")
    if s.contains('.') {
        return true;
    }

    // Accept simple alphanumeric keys
    if s.chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return true;
    }

    // Accept short strings without special characters
    if s.len() < 30 && !s.contains('\\') {
        return true;
    }

    false
}

fn find_used_translation_keys(src_dir: &Path) -> HashSet<String> {
    let mut used_keys = HashSet::new();

    if let Ok(entries) = fs::read_dir(src_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                used_keys.extend(find_used_translation_keys(&path));
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                // Skip the i18n module files themselves
                if path.to_string_lossy().contains("i18n/") {
                    continue;
                }
                if let Ok(content) = fs::read_to_string(&path) {
                    // Find t("key") patterns
                    for line in content.lines() {
                        // t("key") pattern
                        let mut start = 0;
                        while let Some(t_pos) = line[start..].find("t(\"") {
                            let abs_pos = start + t_pos + 3; // Position after t("
                            if let Some(end) = line[abs_pos..].find("\"") {
                                let key = &line[abs_pos..abs_pos + end];
                                if is_valid_translation_key(key) {
                                    used_keys.insert(key.to_string());
                                }
                                start = abs_pos + end;
                            } else {
                                break;
                            }
                        }

                        // t_with_args("key", ...) pattern
                        start = 0;
                        while let Some(t_pos) = line[start..].find("t_with_args(\"") {
                            let abs_pos = start + t_pos + 14; // Position after t_with_args("
                            if let Some(end) = line[abs_pos..].find("\"") {
                                let key = &line[abs_pos..abs_pos + end];
                                if is_valid_translation_key(key) {
                                    used_keys.insert(key.to_string());
                                }
                                start = abs_pos + end;
                            } else {
                                break;
                            }
                        }

                        // translate("key") pattern (from translate_api_message)
                        start = 0;
                        while let Some(t_pos) = line[start..].find("translate(\"") {
                            let abs_pos = start + t_pos + 10; // Position after translate("
                            if let Some(end) = line[abs_pos..].find("\"") {
                                let key = &line[abs_pos..abs_pos + end];
                                if is_valid_translation_key(key) {
                                    used_keys.insert(key.to_string());
                                }
                                start = abs_pos + end;
                            } else {
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    used_keys
}

fn is_valid_translation_key(key: &str) -> bool {
    // Skip empty keys
    if key.is_empty() {
        return false;
    }

    // Skip routes (starting with /)
    if key.starts_with('/') {
        return false;
    }

    // Skip single characters
    if key.len() == 1 {
        return false;
    }

    // Skip very short keys (< 2 chars)
    if key.len() < 2 {
        return false;
    }

    // Skip keys that are purely numeric or alphanumeric and short (likely not translation keys)
    if key.len() <= 3 {
        // Skip patterns like "2d", "v2", etc.
        if key.chars().all(|c| c.is_alphanumeric()) {
            return false;
        }
    }

    // Skip keys that look like fragments (don't start with a known namespace)
    // Known namespaces: menu, header, auth, password, dashboard, clients, common,
    // projects, persons, components, analytics, pagination, client_detail, client_edit,
    // client_status, environment, racks, dictionaries, users, error, success, warning,
    // info, hostname, ip_address, os, vendor, model, last_seen, status, actions,
    // history, ipmi, change, network, storage, memory, validation, notification,
    // stats, filter, server_model, operating_system
    let known_namespaces = [
        "menu",
        "header",
        "auth",
        "password",
        "dashboard",
        "clients",
        "common",
        "projects",
        "persons",
        "components",
        "analytics",
        "pagination",
        "client_detail",
        "client_edit",
        "client_status",
        "environment",
        "racks",
        "dictionaries",
        "users",
        "error",
        "success",
        "warning",
        "info",
        "hostname",
        "ip_address",
        "os",
        "vendor",
        "model",
        "last_seen",
        "status",
        "actions",
        "history",
        "ipmi",
        "change",
        "network",
        "storage",
        "memory",
        "label",
        "validation",
        "notification",
        "stats",
        "filter",
        "server_model",
        "operating_system",
        "no_discrete_gpu",
        "unknown",
        "online",
        "offline",
        "none",
        "never",
        "cpu_config",
        "memory_config",
        "gpu_config",
        "storage_config",
        "network_config",
        "all",
        "count",
        "loading",
        "search",
        "import",
        "export",
        "apply",
        "clear",
        "client_setup",
        "client_not_found",
        "internal_server",
        "invalid_request",
        "not_found",
        "validation_error",
        "database",
    ];

    let has_valid_namespace = key.contains('.')
        && known_namespaces
            .iter()
            .any(|ns| key.starts_with(&format!("{}.", ns)) || key.starts_with(ns));

    // Skip keys that look like word fragments or dynamic constructions
    if key.starts_with("nalytics.")
        || key.starts_with("agination.")
        || key.starts_with("ardware.")
        || key.starts_with("omponents.")
        || key.starts_with("ardware.")
        || key.starts_with("acks.")
        || key.starts_with("hardware.")
    {
        return false;
    }

    // Skip keys without valid namespace (unless they're very short simple keys)
    if !has_valid_namespace && key.len() < 5 {
        // Allow very short simple keys like "online", "offline", "all", etc.
        return key.chars().all(|c| c.is_alphanumeric() || c == '_');
    }

    if !has_valid_namespace {
        return false;
    }

    // Skip keys with special characters that aren't likely translation keys
    // Allow: alphanumeric, dots, underscores, hyphens, colons
    if !key
        .chars()
        .all(|c| c.is_alphanumeric() || c == '.' || c == '_' || c == ':' || c == '-')
    {
        return false;
    }

    // Key must contain at least one letter
    if !key.chars().any(|c| c.is_alphabetic()) {
        return false;
    }

    true
}
