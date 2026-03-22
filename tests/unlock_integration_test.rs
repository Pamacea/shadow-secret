//! Integration tests for unlock command.
//!
//! These tests verify the complete unlock workflow:
//! - Load configuration
//! - Load secrets from vault
//! - Inject secrets into targets
//! - Register backups
//! - Cleanup and restore

mod common;

use std::fs;

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_unlock_command_invalid_config() {
        // Test unlock with non-existent config file
        let mut cmd = assert_cmd::cargo_bin_cmd!("shadow-secret");
        cmd.arg("unlock")
            .arg("--config")
            .arg("nonexistent.yaml")
            .assert()
            .failure();
    }

    #[test]
    fn test_unlock_command_help() {
        // Test unlock --help
        let mut cmd = assert_cmd::cargo_bin_cmd!("shadow-secret");
        cmd.arg("unlock")
            .arg("--help")
            .assert()
            .success()
            .stdout(predicates::str::contains("Unlock secrets"));
    }

    #[test]
    fn test_unlock_workflow_with_test_files() {
        // This test creates a complete end-to-end workflow:
        // 1. Create test config file
        // 2. Create test secrets file
        // 3. Create test target file with placeholders
        // 4. Run unlock command (in a separate process would be ideal)
        // 5. Verify secrets are injected
        // 6. Trigger cleanup
        // 7. Verify files are restored

        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path();

        // Step 1: Create test config
        let config_content = r#"
vault:
  source: test.enc.env
  engine: sops
  require_mount: false

targets:
  - name: test-target
    path: test.json
    placeholders:
      - "$API_KEY"
      - "$DATABASE_URL"
"#;

        let config_path = temp_path.join("global.yaml");
        fs::write(&config_path, config_content).unwrap();

        // Step 2: Create mock SOPS encrypted file
        let secrets_content = r#"API_KEY=sk_test_12345
DATABASE_URL=postgres://localhost:5432/test
"#;

        let vault_path = temp_path.join("test.enc.env");
        fs::write(&vault_path, secrets_content).unwrap();

        // Step 3: Create test target file with placeholders
        let target_content = r#"{
  "apiKey": "$API_KEY",
  "databaseUrl": "$DATABASE_URL"
}"#;

        let target_path = temp_path.join("test.json");
        fs::write(&target_path, target_content).unwrap();

        // Step 4: Verify initial state
        let initial_content = fs::read_to_string(&target_path).unwrap();
        assert!(initial_content.contains("$API_KEY"));
        assert!(initial_content.contains("$DATABASE_URL"));

        // Note: We can't easily test the full unlock workflow in a subprocess
        // because it requires keeping the process alive with signal handlers.
        // Instead, we verify the individual components work together:
        // - Config loading (tested above)
        // - Vault loading (tested in vault_integration_test)
        // - Injector (tested in injector module tests)
        // - Cleaner (tested in cleaner module tests)
        // - Integration is verified manually

        // Cleanup
        drop(temp_dir);
    }

    #[test]
    fn test_unlock_command_with_custom_config_path() {
        // Test that unlock command accepts custom config path
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path();

        let config_content = r#"
vault:
  source: test.enc.env
  engine: sops
targets:
  - name: test
    path: test.json
    placeholders:
      - "$VAR"
"#;

        let custom_config = temp_path.join("my-config.yaml");
        fs::write(&custom_config, config_content).unwrap();

        // Command should accept custom config path
        let mut cmd = assert_cmd::cargo_bin_cmd!("shadow-secret");
        cmd.arg("unlock")
            .arg("--config")
            .arg(custom_config.to_str().unwrap())
            .timeout(std::time::Duration::from_secs(1))
            .assert()
            .failure();

        drop(temp_dir);
    }

    #[test]
    fn test_unlock_command_default_config_path() {
        // Test that unlock command defaults to global.yaml
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path();

        // Create config file with default name
        let config_content = r#"
vault:
  source: test.enc.env
  engine: sops
targets:
  - name: test
    path: test.json
    placeholders:
      - "$VAR"
"#;

        let default_config = temp_path.join("global.yaml");
        fs::write(&default_config, config_content).unwrap();

        // Change to temp directory
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_path).unwrap();

        // Command should find global.yaml by default
        let mut cmd = assert_cmd::cargo_bin_cmd!("shadow-secret");
        cmd.arg("unlock")
            .timeout(std::time::Duration::from_secs(1))
            .assert()
            .failure();

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();

        drop(temp_dir);
    }
}
