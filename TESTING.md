# Testing Guide for rs-cmdb

This document describes the testing conventions, fixtures, and workflows used in the rs-cmdb project.

## Table of Contents

- [Running Tests](#running-tests)
- [Test Organization](#test-organization)
- [Test Fixtures](#test-fixtures)
- [Test Naming Conventions](#test-naming-conventions)
- [Writing Tests](#writing-tests)
- [Coverage](#coverage)

## Running Tests

### Run All Tests

```bash
# Run all tests in the workspace
cargo test --workspace

# Run tests with output
cargo test --workspace -- --nocapture

# Run tests in a specific package
cargo test -p server
cargo test -p client
cargo test -p common
```

### Run Specific Tests

```bash
# Run tests matching a pattern
cargo test test_password

# Run tests in a specific module
cargo test auth_service::tests

# Run a single test
cargo test test_password_hashing
```

## Test Organization

The project uses a hybrid testing approach:

### Unit Tests

Located alongside source code using `#[cfg(test)]` modules:

```
server/src/
├── service/
│   ├── auth_service.rs
│   └── auth_service.rs (contains #[cfg(test)] module)
```

### Integration Tests

Located in `tests/` directories:

```
server/
├── src/
│   └── ...
└── tests/
    ├── api/
    │   ├── auth_tests.rs
    │   └── client_tests.rs
    └── integration/
        └── end_to_end_tests.rs
```

### Test Modules

Common test utilities are organized in:

- `server/src/tests/` - Test fixtures for server tests
- `client/src/tests/` - Test fixtures for client tests

## Test Fixtures

### Server Fixtures

Located in `server/src/tests/fixtures/mod.rs`:

#### Database Setup

```rust
use crate::tests::fixtures::setup_test_db;

#[tokio::test]
async fn test_example() {
    let db = setup_test_db().unwrap();
    // Use db for testing...
}
```

#### Authentication Fixtures

```rust
use crate::tests::fixtures::{test_admin, test_user, generate_test_token, seed_test_user};

#[tokio::test]
async fn test_with_admin() {
    let db = setup_test_db().unwrap();
    let admin = test_admin();

    seed_test_user(&db, &admin).await.unwrap();

    let token = generate_test_token(&admin);
    // Use token for authenticated requests...
}
```

#### Available Test Users

- `test_admin()` - Admin role user
- `test_user()` - Regular user role
- `test_viewer()` - Viewer role user

#### Authentication Headers

```rust
use crate::tests::fixtures::{test_admin, generate_test_token, auth_headers};

let admin = test_admin();
let token = generate_test_token(&admin);
let (key, value) = auth_headers(&token);
// key = "authorization"
// value = "Bearer <token>"
```

### Client Fixtures

Located in `client/src/tests/mod.rs`:

#### Test Data Paths

```rust
use client::tests::test_data_path;

let cpuinfo_path = test_data_path("proc_cpuinfo.txt");
// Returns path to test data file
```

## Test Naming Conventions

### Function Naming

Use descriptive names following the pattern:

```rust
fn test_<unit>_when_<condition>_then_<outcome>()
```

Examples:

```rust
// Good
fn test_client_create_when_valid_data_then_succeeds() { }
fn test_auth_login_when_invalid_credentials_then_returns_401() { }
fn test_user_update_when_role_changes_then_persists() { }

// Acceptable
fn test_password_hashing() { }
fn test_jwt_token_generation() { }
```

### Test Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_async_operation() {
        // Setup
        let db = setup_test_db().unwrap();

        // Execute
        let result = db.get("key").await;

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_sync_operation() {
        // Setup
        let value = 42;

        // Execute & Assert
        assert_eq!(value, 42);
    }
}
```

## Writing Tests

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_returns_expected_value() {
        let input = "test";
        let result = function_to_test(input);
        assert_eq!(result, "expected");
    }

    #[tokio::test]
    async fn test_async_function() {
        let result = async_function().await.unwrap();
        assert!(!result.is_empty());
    }
}
```

### Integration Tests

```rust
use server::tests::fixtures::*;
use common::entity::user::Role;

#[tokio::test]
async fn test_api_endpoint_with_auth() {
    // Setup
    let db = setup_test_db().unwrap();
    let admin = test_admin();
    seed_test_user(&db, &admin).await.unwrap();
    let token = generate_test_token(&admin);

    // Execute
    let app = setup_test_app(db).await;
    let request = Request::builder()
        .uri("/api/clients")
        .header(auth_headers(&token).0, auth_headers(&token).1)
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), 200);
}
```

### Error Testing

```rust
#[tokio::test]
async fn test_repository_error_handling() {
    let db = setup_test_db().unwrap();

    // Test non-existent key
    let result: Option<Vec<u8>> = db.get("nonexistent").await.unwrap();
    assert_eq!(result, None);

    // Test serialization error
    let invalid_data = b"\xff\xff\xff\xff";
    db.set("test", invalid_data).await.unwrap();
    // ... handle error cases
}
```

## Coverage

### Install Coverage Tool

```bash
cargo install cargo-llvm-cov
```

### Generate Coverage Report

```bash
# Generate HTML coverage report
cargo llvm-cov --html --output-dir coverage

# Generate coverage in terminal
cargo llvm-cov --workspace

# Check coverage for specific package
cargo llvm-cov -p server
```

### Coverage Thresholds

Target coverage thresholds:

| Crate   | Minimum | Ideal |
|---------|---------|-------|
| server  | 60%     | 80%   |
| client  | 50%     | 70%   |
| common  | 70%     | 90%   |
| front   | 40%     | 60%   |

### CI Integration

Coverage is checked in CI. Tests must pass and coverage must meet minimum thresholds before merge.

## Best Practices

1. **Test Behavior, Not Implementation** - Focus on what the code does, not how it does it
2. **Use Fixtures** - Reuse common test fixtures to reduce duplication
3. **Keep Tests Fast** - Unit tests should be fast; integration tests can be slower but should still be reasonable
4. **Test Edge Cases** - Include tests for boundary conditions and error cases
5. **Avoid Brittle Tests** - Tests should be resilient to minor implementation changes
6. **Isolate Tests** - Each test should be independent and not rely on other tests
7. **Use Descriptive Names** - Test names should clearly indicate what is being tested

## Common Patterns

### Repository Tests

```rust
#[tokio::test]
async fn test_repository_create_success() {
    let db = setup_test_db().unwrap();
    let repo = ClientRepository::new(Arc::new(db));

    let client = Client {
        id: "test-123".to_string(),
        hostname: "test-server".to_string(),
        // ... other fields
    };

    let result = repo.create(&client).await;
    assert!(result.is_ok());
}
```

### Service Tests

```rust
#[tokio::test]
async fn test_service_validates_input() {
    let db = setup_test_db().unwrap();
    let repo = UserRepository::new(Arc::new(db));
    let service = AuthService::new(repo);

    let result = service.register("", "password").await;
    assert!(result.is_err());
}
```

### API Tests

```rust
#[tokio::test]
async fn test_api_returns_401_without_token() {
    let app = setup_test_app().await;

    let request = Request::builder()
        .uri("/api/clients")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), 401);
}
```

## Troubleshooting

### Tests Fail with "Database Already Open"

ReDB doesn't support true in-memory databases. Each test creates its own temporary database file. If tests run in parallel, ensure each uses a unique database instance.

### Tests Are Slow

- Run unit tests only: `cargo test --lib`
- Run specific test packages: `cargo test -p server`
- Use `--test-threads=1` if tests have conflicts

### Coverage Not Generated

Ensure `cargo-llvm-cov` is installed and LLVM tools are available:
```bash
cargo install cargo-llvm-cov
rustup component add llvm-tools-preview
```
