//! Integration tests for push-cloud command.

use shadow_secret::cloud::detect_project_id;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Test helper to create a temporary .enc.env file
fn create_test_enc_env(temp_dir: &PathBuf, secrets: &HashMap<&str, &str>) -> PathBuf {
    let enc_env_path = temp_dir.join(".enc.env");

    let mut content = String::new();
    for (key, value) in secrets {
        content.push_str(&format!("{}={}\n", key, value));
    }

    fs::write(&enc_env_path, content).expect("Failed to write .enc.env");

    // Encrypt with SOPS
    let output = std::process::Command::new("sops")
        .arg("-e")
        .arg("--age")
        .arg("--kage")
        .arg("age1cypher56789yefg234567890abcdef1234567890abcdef1234567890abcd")
        .arg(&enc_env_path)
        .output();

    match output {
        Ok(out) if out.status.success() => {
            // Remove unencrypted file
            fs::remove_file(&enc_env_path).ok();
            enc_env_path.with_extension("enc.env")
        }
        _ => {
            // If SOPS fails, keep the unencrypted file for testing
            enc_env_path
        }
    }
}

/// Test helper to create global.yaml config
fn create_test_config(temp_dir: &PathBuf, vault_path: &str) -> PathBuf {
    let config_path = temp_dir.join("global.yaml");

    let content = format!(
        r#"vault:
  source: "{}"
  engine: "sops"
  require_mount: false

targets:
  - name: "test"
    path: "{}/.env"
    placeholders: ["$TEST_VAR"]
"#,
        vault_path, temp_dir.display()
    );

    fs::write(&config_path, content).expect("Failed to write config");

    config_path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Requires .vercel/project.json
    fn test_detect_project_id() {
        // This test requires .vercel/project.json to exist
        // In CI, we skip this test
        if PathBuf::from(".vercel/project.json").exists() {
            let result = detect_project_id();
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_vercel_cli_detection() {
        // Test that we can detect Vercel CLI
        let output = std::process::Command::new("vercel")
            .arg("--version")
            .output();

        // Test passes if Vercel CLI is installed, fails otherwise
        // This is expected behavior
        assert!(output.is_ok());
    }

    #[test]
    fn test_push_cloud_dry_run() {
        // Create temporary directory
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let temp_path = temp_dir.path();

        // Create test secrets
        let mut secrets = HashMap::new();
        secrets.insert("TEST_VAR", "test_value");
        secrets.insert("ANOTHER_VAR", "another_value");

        // Create .enc.env (unencrypted for testing)
        let enc_env_path = temp_path.join(".enc.env");
        let content = "TEST_VAR=test_value\nANOTHER_VAR=another_value\n";
        fs::write(&enc_env_path, content).expect("Failed to write .enc.env");

        // Create config
        let config_path = create_test_config(&temp_path.to_path_buf(), ".enc.env");

        // Run push-cloud in dry-run mode
        let output = std::process::Command::new("./target/release/shadow-secret.exe")
            .arg("push-cloud")
            .arg("--config")
            .arg(&config_path)
            .arg("--dry-run")
            .current_dir(temp_path)
            .output();

        // Verify command executed
        assert!(output.is_ok(), "push-cloud command should execute");

        let output = output.unwrap();

        // In dry-run mode, the command should not fail
        // It should show what would be pushed
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Dry run should mention "Dry run mode"
        assert!(
            stdout.contains("Dry run") || stderr.contains("Dry run"),
            "Dry run output should mention dry run mode"
        );
    }

    #[test]
    fn test_local_only_secrets_filtered() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let temp_path = temp_dir.path();

        // Create secrets with LOCAL_ONLY_ prefix
        let mut secrets = HashMap::new();
        secrets.insert("PUBLIC_VAR", "public_value");
        secrets.insert("LOCAL_ONLY_SECRET", "local_value");
        secrets.insert("ANOTHER_PUBLIC", "another_public");

        // Create .enc.env
        let enc_env_path = temp_path.join(".enc.env");
        let content = "PUBLIC_VAR=public_value\nLOCAL_ONLY_SECRET=local_value\nANOTHER_PUBLIC=another_public\n";
        fs::write(&enc_env_path, content).expect("Failed to write .enc.env");

        // Create config
        let config_path = create_test_config(&temp_path.to_path_buf(), ".enc.env");

        // Run push-cloud in dry-run mode
        let output = std::process::Command::new("./target/release/shadow-secret.exe")
            .arg("push-cloud")
            .arg("--config")
            .arg(&config_path)
            .arg("--dry-run")
            .current_dir(temp_path)
            .output();

        let output = output.expect("Command should execute");
        let stdout = String::from_utf8_lossy(&output.stdout);

        // LOCAL_ONLY_ secrets should be filtered out
        assert!(
            !stdout.contains("LOCAL_ONLY_SECRET"),
            "LOCAL_ONLY_SECRET should not appear in output"
        );

        // Public variables should appear
        assert!(
            stdout.contains("PUBLIC_VAR") || stdout.contains("ANOTHER_PUBLIC"),
            "Public variables should appear in output"
        );
    }
}
