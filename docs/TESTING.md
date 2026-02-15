# Testing Guide

Complete testing framework for Shadow Secret.

---

## Testing Strategy

We use a **multi-tier testing approach**:

1. **Unit Tests** - Test individual functions in isolation
2. **Integration Tests** - Test module interactions
3. **CLI Tests** - Test command-line interface

---

## Quality Gates

All code must pass these gates before merging:

- [ ] `cargo test` - All tests pass
- [ ] `cargo clippy` - No warnings
- [ ] `cargo fmt --check` - Code formatting
- [ ] Coverage > 80%

---

## Running Tests

### All Tests

```bash
cargo test
```

### Specific Test

```bash
cargo test test_parse_env
```

### Show Output

```bash
cargo test -- --nocapture
```

### Run Ignored Tests (Integration)

```bash
cargo test -- --ignored
```

---

## Test Structure

```
shadow-secret/
├── src/
│   ├── vault.rs          # Unit tests inline (#[cfg(test)])
│   ├── config.rs         # Unit tests inline
│   └── main.rs           # Integration tests inline
│
└── tests/
    ├── common/
    │   └── mod.rs        # Test utilities & mocks
    ├── vault_integration_test.rs
    └── cli_test.rs       # CLI integration tests
```

---

## Unit Tests

Located in `src/*.rs` files within `#[cfg(test)]` modules.

### Example: Vault Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_env_format() {
        let env_output = b"API_KEY=sk_test_123\n";
        let secrets = parse_env(env_output).unwrap();

        assert_eq!(secrets.get("API_KEY"), Some(&"sk_test_123".to_string()));
    }
}
```

---

## Integration Tests

Located in `tests/` directory.

### Test Utilities

```rust
use tests::common::{TestContext, MockSops};

#[test]
fn test_with_mock_sops() {
    let output = MockSops::env_output(&[("KEY", "value")]);
    // Test with mock output
}
```

### Temporary Files

```rust
#[test]
fn test_with_temp_files() -> anyhow::Result<()> {
    let ctx = TestContext::new()?;
    let file = ctx.create_file("test.txt", "content")?;

    // Test with temporary file
    Ok(())
}
```

---

## Mock SOPS

For testing without real SOPS installation:

```rust
// Mock ENV output
let env = MockSops::env_output(&[("API_KEY", "sk_test_123")]);

// Mock JSON output
let json = MockSops::json_output(&[("API_KEY", "sk_test_123")]);

// Mock YAML output
let yaml = MockSops::yaml_output(&[("API_KEY", "sk_test_123")]);

// Mock SOPS format (with metadata)
let sops_json = MockSops::sops_json_output(&[("API_KEY", "sk_test_123")]);
```

---

## CI/CD

### GitHub Actions Workflows

- **`.github/workflows/ci.yml`** - Main CI pipeline
  - Quality checks (fmt, clippy)
  - Unit tests
  - Build (Linux, macOS, Windows)

- **`.github/workflows/coverage.yml`** - Code coverage
  - Generates HTML report
  - Uploads to Codecov

---

## Test Coverage

### Generate Coverage Locally

```bash
cargo install cargo-llvm-cov
cargo llvm-cov --html
```

View report:
```bash
open target/llvm-cov/html/index.html
```

### Coverage Goals

- **Overall**: > 80%
- **Critical paths** (vault loading): > 95%
- **CLI commands**: > 90%

---

## Best Practices

### DO ✅

- Test both success and failure paths
- Use descriptive test names
- Use `tempfile` for temporary files
- Mock external dependencies (SOPS)
- Test error messages are helpful

### DON'T ❌

- Skip error case testing
- Use hardcoded paths
- Test with real secrets
- Ignore clippy warnings

---

## Common Patterns

### Testing Error Handling

```rust
#[test]
fn test_invalid_format_returns_error() {
    let invalid_output = b"not a valid format";
    let result = parse_env(invalid_output);

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("No secrets found"));
}
```

### Testing with Fixtures

```rust
#[test]
fn test_with_fixture() -> anyhow::Result<()> {
    let ctx = TestContext::new()?;

    // Create fixture files
    ctx.create_sops_env("test.env", &[("KEY", "value")])?;
    ctx.create_sops_json("test.json", &[("KEY", "value")])?;
    ctx.create_sops_yaml("test.yaml", &[("KEY", "value")])?;

    // Test with fixtures
    Ok(())
}
```

---

## Debugging Tests

### Show Test Output

```bash
cargo test -- --nocapture
```

### Run Single Test

```bash
cargo test test_name
```

### Run Tests in File

```bash
cargo test --lib vault
```

---

## Continuous Integration

### CI Checks

All PRs must pass:

1. **Format Check**
   ```bash
   cargo fmt -- --check
   ```

2. **Clippy**
   ```bash
   cargo clippy -- -D warnings
   ```

3. **Tests**
   ```bash
   cargo test
   ```

4. **Build**
   ```bash
   cargo build --release
   ```

---

## Security Testing

### Verify No Temp Files

```rust
#[test]
fn test_no_temp_files_created() {
    let before = std::fs::read_dir(".").unwrap().count();
    parse_env(b"KEY=value").unwrap();
    let after = std::fs::read_dir(".").unwrap().count();

    assert_eq!(before, after, "No files should be created");
}
```

### Test Secrets Never Logged

```rust
#[test]
fn test_secrets_not_in_logs() {
    // Ensure secrets are never logged or printed
}
```

---

## Performance Tests

```rust
#[test]
fn test_large_secrets_file_parsing() {
    let large_env = generate_large_env(1000); // 1000 secrets
    let start = std::time::Instant::now();

    let result = parse_env(&large_env);

    assert!(result.is_ok());
    assert!(start.elapsed() < std::time::Duration::from_millis(100));
}
```

---

## References

- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov)
- [assert_cmd](https://docs.rs/assert_cmd/)
- [tempfile](https://docs.rs/tempfile/)
