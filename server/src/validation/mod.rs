//! Validation module for input sanitization and security
//!
//! Provides validation utilities to prevent injection attacks
//! and ensure data integrity.

pub mod input_validation;

pub use input_validation::{validate_ip_address, validate_ssh_argument};
