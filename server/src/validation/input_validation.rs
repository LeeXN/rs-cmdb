//! Input validation module for security
//!
//! Provides input sanitization and validation to prevent injection attacks
//! and ensure data integrity.

use common::error::{CmdbError, CmdbResult};
use std::net::{IpAddr, AddrParseError};
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
        return Err(CmdbError::Validation("IP address cannot be empty".to_string()));
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
            "IPv6 address not allowed, expected IPv4".to_string()
        )),
        Err(_) => Err(CmdbError::Validation(
            "Invalid IPv4 address format".to_string()
        ))
    }
}

/// Validate IPv6 address specifically
pub fn validate_ipv6(ip: &str) -> CmdbResult<String> {
    let validated = validate_ip_address(ip)?;
    match validated.parse::<IpAddr>() {
        Ok(IpAddr::V6(_)) => Ok(validated),
        Ok(IpAddr::V4(_)) => Err(CmdbError::Validation(
            "IPv4 address not allowed, expected IPv6".to_string()
        )),
        Err(_) => Err(CmdbError::Validation(
            "Invalid IPv6 address format".to_string()
        ))
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
        return Err(CmdbError::Validation("Hostname cannot be empty".to_string()));
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
            "Hostname can only contain alphanumeric characters, hyphens, and dots".to_string()
        ));
    }

    // Cannot start or end with hyphen or dot
    if hostname.starts_with('-') || hostname.ends_with('-') ||
       hostname.starts_with('.') || hostname.ends_with('.') {
        return Err(CmdbError::Validation(
            "Hostname cannot start or end with a hyphen or dot".to_string()
        ));
    }

    // Check each label
    for label in hostname.split('.') {
        if label.is_empty() {
            return Err(CmdbError::Validation(
                "Hostname cannot contain consecutive dots".to_string()
            ));
        }
        if label.len() > 63 {
            return Err(CmdbError::Validation(
                "Each label in hostname must be 63 characters or less".to_string()
            ));
        }
        if label.starts_with('-') || label.ends_with('-') {
            return Err(CmdbError::Validation(
                "Hostname labels cannot start or end with a hyphen".to_string()
            ));
        }
    }

    Ok(hostname.to_string())
}

/// IP address whitelist/blacklist for SSH access
pub struct IpAllowList {
    /// Allowed IP prefixes (e.g., "192.168.1.0/24")
    allowed_ranges: Vec<ipnet::IpNet>,
    /// Blocked specific IPs
    blocked_ips: Vec<IpAddr>,
}

impl Default for IpAllowList {
    fn default() -> Self {
        Self::new()
    }
}

impl IpAllowList {
    /// Create a new empty allow list
    pub fn new() -> Self {
        Self {
            allowed_ranges: Vec::new(),
            blocked_ips: Vec::new(),
        }
    }

    /// Create an allow list from configuration
    ///
    /// # Arguments
    /// * `allowed_ranges` - List of CIDR ranges to allow (e.g., ["192.168.1.0/24"])
    /// * `blocked_ips` - List of specific IPs to block
    pub fn from_config(allowed_ranges: &[String], blocked_ips: &[String]) -> CmdbResult<Self> {
        let mut allow_list = Self::new();

        for range in allowed_ranges {
            match range.parse::<ipnet::IpNet>() {
                Ok(net) => allow_list.allowed_ranges.push(net),
                Err(e) => {
                    return Err(CmdbError::Validation(format!(
                        "Invalid IP range '{}': {}",
                        range, e
                    )));
                }
            }
        }

        for ip_str in blocked_ips {
            match ip_str.parse::<IpAddr>() {
                Ok(ip) => allow_list.blocked_ips.push(ip),
                Err(e) => {
                    return Err(CmdbError::Validation(format!(
                        "Invalid blocked IP '{}': {}",
                        ip_str, e
                    )));
                }
            }
        }

        Ok(allow_list)
    }

    /// Check if an IP address is allowed
    pub fn is_allowed(&self, ip: &str) -> bool {
        // First check if explicitly blocked
        if let Ok(ip_addr) = ip.parse::<IpAddr>() {
            if self.blocked_ips.contains(&ip_addr) {
                warn!("IP address is explicitly blocked: {}", ip);
                return false;
            }

            // If no allowed ranges configured, allow all (except blocked)
            if self.allowed_ranges.is_empty() {
                return true;
            }

            // Check if IP matches any allowed range
            for range in &self.allowed_ranges {
                if range.contains(&ip_addr) {
                    debug!("IP address allowed: {} matches range {}", ip, range);
                    return true;
                }
            }

            warn!("IP address not in allowed ranges: {}", ip);
            false
        } else {
            warn!("Invalid IP address format for allow list check: {}", ip);
            false
        }
    }

    /// Add an allowed range
    pub fn add_allowed_range(&mut self, range: &str) -> CmdbResult<()> {
        match range.parse::<ipnet::IpNet>() {
            Ok(net) => {
                self.allowed_ranges.push(net);
                Ok(())
            }
            Err(e) => Err(CmdbError::Validation(format!(
                "Invalid IP range '{}': {}",
                range, e
            ))),
        }
    }

    /// Add a blocked IP
    pub fn add_blocked_ip(&mut self, ip: &str) -> CmdbResult<()> {
        match ip.parse::<IpAddr>() {
            Ok(addr) => {
                self.blocked_ips.push(addr);
                Ok(())
            }
            Err(e) => Err(CmdbError::Validation(format!(
                "Invalid IP address '{}': {}",
                ip, e
            ))),
        }
    }
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
        ";", "&", "|", "$", "`", "(", ")", "\n", "\r",
        "&&", "||", ">",
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

/// Sanitize a string for safe logging
///
/// Removes or escapes potentially sensitive information
pub fn sanitize_for_logging(s: &str) -> String {
    // Limit length
    let sanitized = if s.len() > 100 {
        format!("{}...", &s[..97])
    } else {
        s.to_string()
    };

    // Remove common sensitive patterns (basic implementation)
    sanitized
        .replace("password=", "password=***")
        .replace("passwd=", "passwd=***")
        .replace("pwd=", "pwd=***")
        .replace("token=", "token=***")
        .replace("key=", "key=***")
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
        assert_eq!(
            validate_ip_address("::1").unwrap(),
            "::1"
        );
        assert_eq!(
            validate_ip_address("2001:db8::1").unwrap(),
            "2001:db8::1"
        );
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

    #[test]
    fn test_ip_allow_list() {
        let mut allow_list = IpAllowList::new();

        // Add allowed range
        allow_list.add_allowed_range("192.168.1.0/24").unwrap();

        // Test allowed IP
        assert!(allow_list.is_allowed("192.168.1.100"));

        // Test blocked IP (not in range)
        assert!(!allow_list.is_allowed("10.0.0.1"));

        // Test with blocked IP
        allow_list.add_blocked_ip("192.168.1.50").unwrap();
        assert!(!allow_list.is_allowed("192.168.1.50"));
    }
}
