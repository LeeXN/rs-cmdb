#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = default_config();

        assert_eq!(config.server.url, "http://localhost:8080");
        assert!(config.server.verify_tls);
        assert!(config.report.service_mode);
        assert!(config.report.push_enabled);
        assert_eq!(config.report.push_interval, 300);
    }

    #[test]
    fn test_ensure_client_id_generates_uuid() {
        use crate::config::ensure_client_id;
        use uuid::Uuid;

        let client_id = ensure_client_id();

        let uuid = Uuid::parse_str(&client_id);
        assert!(uuid.is_ok());
    }

    #[test]
    fn test_get_config() {
        use crate::config::get_config;

        let config = get_config();

        assert!(config.client_id.is_some());
        assert_eq!(config.server.url, "http://localhost:8080");
    }

    #[test]
    fn test_display_options_creation() {
        use crate::display::DisplayOptions;
        use crate::display::HardwareType;
        use std::collections::HashSet;

        let mut types = HashSet::new();
        types.insert(HardwareType::CPU);
        types.insert(HardwareType::RAM);

        let options = DisplayOptions {
            hardware_types: types,
            show_detail: true,
        };

        assert!(options.show_detail);
        assert_eq!(options.hardware_types.len(), 2);
    }
}
