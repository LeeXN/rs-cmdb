//! Input validation module for security
//!
//! Provides input sanitization and validation to prevent injection attacks
//! and ensure data integrity.

use common::error::{CmdbError, CmdbResult};
use std::net::IpAddr;
use tracing::{debug, warn};

/// Maximum length for hostname to prevent DoS
const MAX_HOSTNAME_LENGTH: usize = 253;

/// Maximum length for IP address string
const MAX_IP_LENGTH: usize = 45; // IPv6 max length

/// Validate and sanitize an IP address
///
/// # Arguments
/// * `ip` - The IP address string to validate
///
/// # Returns
/// * `Ok(String)` - The validated IP address
/// * `Err(CmdbError)` - If the IP address is invalid
pub fn validate_ip_address(ip: &str) -> CmdbResult<String> {
    // Check length first to prevent potential DoS
    if ip.len() > MAX_IP_LENGTH {
        return Err(CmdbError::Validation(format!(
            "IP address exceeds maximum length of {} characters",
            MAX_IP_LENGTH
        )));
    }

    // Trim whitespace
    let ip = ip.trim();

    // Check for empty string
    if ip.is_empty() {
        return Err(CmdbError::Validation(
            "IP address cannot be empty".to_string(),
        ));
    }

    // Parse as IP address to validate format
    match ip.parse::<IpAddr>() {
        Ok(_) => {
            debug!("Valid IP address: {}", ip);
            Ok(ip.to_string())
        }
        Err(e) => {
            warn!("Invalid IP address '{}': {}", ip, e);
            Err(CmdbError::Validation(format!(
                "Invalid IP address format: '{}'",
                ip
            )))
        }
    }
}

/// Validate IPv4 address specifically
pub fn validate_ipv4(ip: &str) -> CmdbResult<String> {
    let validated = validate_ip_address(ip)?;
    match validated.parse::<IpAddr>() {
        Ok(IpAddr::V4(_)) => Ok(validated),
        Ok(IpAddr::V6(_)) => Err(CmdbError::Validation(
            "IPv6 address not allowed, expected IPv4".to_string(),
        )),
        Err(_) => Err(CmdbError::Validation(
            "Invalid IPv4 address format".to_string(),
        )),
    }
}

/// Validate IPv6 address specifically
pub fn validate_ipv6(ip: &str) -> CmdbResult<String> {
    let validated = validate_ip_address(ip)?;
    match validated.parse::<IpAddr>() {
        Ok(IpAddr::V6(_)) => Ok(validated),
        Ok(IpAddr::V4(_)) => Err(CmdbError::Validation(
            "IPv4 address not allowed, expected IPv6".to_string(),
        )),
        Err(_) => Err(CmdbError::Validation(
            "Invalid IPv6 address format".to_string(),
        )),
    }
}

/// Validate hostname
///
/// Hostnames must:
/// - Contain only alphanumeric characters, hyphens, and dots
/// - Not start or end with a hyphen
/// - Not have consecutive hyphens
/// - Be between 1 and 253 characters
/// - Each label between dots must be 1-63 characters
pub fn validate_hostname(hostname: &str) -> CmdbResult<String> {
    let hostname = hostname.trim();

    if hostname.is_empty() {
        return Err(CmdbError::Validation(
            "Hostname cannot be empty".to_string(),
        ));
    }

    if hostname.len() > MAX_HOSTNAME_LENGTH {
        return Err(CmdbError::Validation(format!(
            "Hostname exceeds maximum length of {} characters",
            MAX_HOSTNAME_LENGTH
        )));
    }

    // Check for invalid characters
    let valid_chars = hostname
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '.');

    if !valid_chars {
        return Err(CmdbError::Validation(
            "Hostname can only contain alphanumeric characters, hyphens, and dots".to_string(),
        ));
    }

    // Cannot start or end with hyphen or dot
    if hostname.starts_with('-')
        || hostname.ends_with('-')
        || hostname.starts_with('.')
        || hostname.ends_with('.')
    {
        return Err(CmdbError::Validation(
            "Hostname cannot start or end with a hyphen or dot".to_string(),
        ));
    }

    // Check each label
    for label in hostname.split('.') {
        if label.is_empty() {
            return Err(CmdbError::Validation(
                "Hostname cannot contain consecutive dots".to_string(),
            ));
        }
        if label.len() > 63 {
            return Err(CmdbError::Validation(
                "Each label in hostname must be 63 characters or less".to_string(),
            ));
        }
        if label.starts_with('-') || label.ends_with('-') {
            return Err(CmdbError::Validation(
                "Hostname labels cannot start or end with a hyphen".to_string(),
            ));
        }
    }

    Ok(hostname.to_string())
}

/// Validate SSH command argument
///
/// Ensures that command arguments don't contain potentially dangerous characters
/// that could be used for injection if the code is ever modified to use shell execution.
pub fn validate_ssh_argument(arg: &str) -> CmdbResult<String> {
    // Whitelist of allowed characters for SSH arguments
    // Alphanumeric, spaces, tabs, and common safe characters
    let allowed_chars = |c: char| -> bool {
        c.is_alphanumeric()
            || c.is_whitespace()
            || matches!(c, '-' | '_' | '/' | '.' | '=' | ':' | '@')
    };

    if !arg.chars().all(allowed_chars) {
        return Err(CmdbError::Validation(format!(
            "SSH argument contains invalid characters: {}",
            arg
        )));
    }

    // Check for dangerous patterns
    let dangerous = [
        ";", "&", "|", "$", "`", "(", ")", "\n", "\r", "&&", "||", ">",
    ];

    for pattern in &dangerous {
        if arg.contains(pattern) {
            return Err(CmdbError::Validation(format!(
                "SSH argument contains potentially dangerous pattern: '{}'",
                pattern
            )));
        }
    }

    Ok(arg.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_valid_ipv4() {
        assert_eq!(validate_ip_address("192.168.1.1").unwrap(), "192.168.1.1");
        assert_eq!(validate_ip_address("10.0.0.1").unwrap(), "10.0.0.1");
        assert_eq!(validate_ip_address("127.0.0.1").unwrap(), "127.0.0.1");
    }

    #[test]
    fn test_validate_valid_ipv6() {
        assert_eq!(validate_ip_address("::1").unwrap(), "::1");
        assert_eq!(validate_ip_address("2001:db8::1").unwrap(), "2001:db8::1");
    }

    #[test]
    fn test_validate_invalid_ip() {
        assert!(validate_ip_address("256.256.256.256").is_err());
        assert!(validate_ip_address("not.an.ip").is_err());
        assert!(validate_ip_address("").is_err());
        assert!(validate_ip_address("192.168.1.1; rm -rf /").is_err());
    }

    #[test]
    fn test_validate_ipv4_specific() {
        assert!(validate_ipv4("192.168.1.1").is_ok());
        assert!(validate_ipv4("::1").is_err());
    }

    #[test]
    fn test_validate_ipv6_specific() {
        assert!(validate_ipv6("::1").is_ok());
        assert!(validate_ipv6("192.168.1.1").is_err());
    }

    #[test]
    fn test_validate_hostname() {
        assert!(validate_hostname("example.com").is_ok());
        assert!(validate_hostname("my-server").is_ok());
        assert!(validate_hostname("server.local").is_ok());
        assert!(validate_hostname("-invalid").is_err());
        assert!(validate_hostname("invalid.").is_err());
        assert!(validate_hostname("in..valid").is_err());
    }

    #[test]
    fn test_ssh_argument_validation() {
        assert!(validate_ssh_argument("systemctl status").is_ok());
        assert!(validate_ssh_argument("ls -la /tmp").is_ok());
        assert!(validate_ssh_argument("systemctl stop rs-cmdb-client").is_ok());

        // Test shell injection attempts - these should fail
        assert!(validate_ssh_argument("systemctl status; rm -rf /").is_err()); // Contains ;
        assert!(validate_ssh_argument("cat /etc/passwd | grep root").is_err()); // Contains |
        assert!(validate_ssh_argument("ls && evil").is_err()); // Contains &&
        assert!(validate_ssh_argument("ls || evil").is_err()); // Contains ||
        assert!(validate_ssh_argument("echo $(whoami)").is_err()); // Contains $()
        assert!(validate_ssh_argument("echo `whoami`").is_err()); // Contains backticks
        assert!(validate_ssh_argument("ls\nmalicious").is_err()); // Contains newline
    }
}
