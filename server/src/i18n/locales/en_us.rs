use std::collections::HashMap;

pub fn get_translations() -> HashMap<String, String> {
    let mut translations = HashMap::new();

    // Common validation messages
    translations.insert("validation.required".to_string(), "This field is required".to_string());
    translations.insert("validation.invalid_email".to_string(), "Invalid email format".to_string());
    translations.insert("validation.invalid_url".to_string(), "Invalid URL format".to_string());
    translations.insert("validation.too_short".to_string(), "Must be at least {min} characters".to_string());
    translations.insert("validation.too_long".to_string(), "Must be at most {max} characters".to_string());
    translations.insert("validation.invalid_format".to_string(), "Invalid format".to_string());
    translations.insert("validation.out_of_range".to_string(), "Value must be between {min} and {max}".to_string());

    // Common API success messages
    translations.insert("success.created".to_string(), "Created successfully".to_string());
    translations.insert("success.updated".to_string(), "Updated successfully".to_string());
    translations.insert("success.deleted".to_string(), "Deleted successfully".to_string());
    translations.insert("success.operation_complete".to_string(), "Operation completed successfully".to_string());

    // Common API error messages
    translations.insert("error.unauthorized".to_string(), "Unauthorized".to_string());
    translations.insert("error.forbidden".to_string(), "Forbidden".to_string());
    translations.insert("error.not_found".to_string(), "Resource not found".to_string());
    translations.insert("error.conflict".to_string(), "Resource already exists".to_string());
    translations.insert("error.internal_server".to_string(), "Internal server error".to_string());
    translations.insert("error.bad_request".to_string(), "Invalid request".to_string());
    translations.insert("error.method_not_allowed".to_string(), "Method not allowed".to_string());
    translations.insert("error.request_timeout".to_string(), "Request timeout".to_string());
    translations.insert("error.too_many_requests".to_string(), "Too many requests".to_string());

    // Client-specific messages
    translations.insert("client.not_found".to_string(), "Client not found".to_string());
    translations.insert("client.created".to_string(), "Client registered successfully".to_string());
    translations.insert("client.updated".to_string(), "Client updated successfully".to_string());
    translations.insert("client.deleted".to_string(), "Client deleted successfully".to_string());
    translations.insert("client.invalid_id".to_string(), "Invalid client ID".to_string());
    translations.insert("client.duplicate_hostname".to_string(), "Client with this hostname already exists".to_string());

    // Component-specific messages
    translations.insert("component.not_found".to_string(), "Component not found".to_string());
    translations.insert("component.created".to_string(), "Component created successfully".to_string());
    translations.insert("component.updated".to_string(), "Component updated successfully".to_string());
    translations.insert("component.deleted".to_string(), "Component deleted successfully".to_string());
    translations.insert("component.invalid_type".to_string(), "Invalid component type".to_string());

    // Rack-specific messages
    translations.insert("rack.not_found".to_string(), "Rack not found".to_string());
    translations.insert("rack.created".to_string(), "Rack created successfully".to_string());
    translations.insert("rack.updated".to_string(), "Rack updated successfully".to_string());
    translations.insert("rack.deleted".to_string(), "Rack deleted successfully".to_string());
    translations.insert("rack.capacity_exceeded".to_string(), "Rack capacity exceeded".to_string());

    // Project-specific messages
    translations.insert("project.not_found".to_string(), "Project not found".to_string());
    translations.insert("project.created".to_string(), "Project created successfully".to_string());
    translations.insert("project.updated".to_string(), "Project updated successfully".to_string());
    translations.insert("project.deleted".to_string(), "Project deleted successfully".to_string());
    translations.insert("project.invalid_manager".to_string(), "Invalid manager specified".to_string());

    // User-specific messages
    translations.insert("user.not_found".to_string(), "User not found".to_string());
    translations.insert("user.created".to_string(), "User created successfully".to_string());
    translations.insert("user.updated".to_string(), "User updated successfully".to_string());
    translations.insert("user.deleted".to_string(), "User deleted successfully".to_string());
    translations.insert("user.invalid_credentials".to_string(), "Invalid username or password".to_string());
    translations.insert("user.username_exists".to_string(), "Username already exists".to_string());

    // Authentication messages
    translations.insert("auth.login_success".to_string(), "Login successful".to_string());
    translations.insert("auth.login_failed".to_string(), "Login failed".to_string());
    translations.insert("auth.logout_success".to_string(), "Logout successful".to_string());
    translations.insert("auth.token_invalid".to_string(), "Invalid or expired token".to_string());
    translations.insert("auth.permission_denied".to_string(), "Permission denied".to_string());

    // Database messages
    translations.insert("db.connection_error".to_string(), "Database connection error".to_string());
    translations.insert("db.query_error".to_string(), "Database query error".to_string());
    translations.insert("db.transaction_error".to_string(), "Database transaction error".to_string());

    translations
}
