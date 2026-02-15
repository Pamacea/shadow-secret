//! Integration tests for shadow-secret init-project command
//!
//! These tests verify the complete init-project workflow including:
//! - Age key generation/detection
//! - .sops.yaml creation
//! - .enc.env creation and encryption
//! - Global config updates

use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Helper struct to manage test environment
struct TestEnv {
    temp_dir: TempDir,
    original_dir: PathBuf,
}

impl TestEnv {
    fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();

        Self {
            temp_dir,
            original_dir,
        }
    }

    fn project_dir(&self) -> &Path {
        self.temp_dir.path()
    }

    /// Change to test directory
    fn enter(&self) {
        std::env::set_current_dir(self.temp_dir.path()).unwrap();
    }

    /// Restore original directory
    fn leave(&self) {
        std::env::set_current_dir(&self.original_dir).unwrap();
    }

    /// Create a fake age key file for testing
    fn create_age_key(&self, content: &str) -> PathBuf {
        let key_path = self.temp_dir.path().join("test_age_key.txt");
        fs::write(&key_path, content).unwrap();
        key_path
    }

    /// Create global config file
    fn create_global_config(&self) -> PathBuf {
        let home = self.temp_dir.path();
        let config_path = home.join(".shadow-secret.yaml");
        let config_content = r#"vault:
  source: "test.enc.env"
  engine: "sops"
targets: []
"#;
        fs::write(&config_path, config_content).unwrap();
        config_path
    }
}

impl Drop for TestEnv {
    fn drop(&mut self) {
        self.leave();
    }
}

#[test]
fn test_init_project_creates_sops_config() {
    let env = TestEnv::new();
    env.enter();

    // Create test age key
    let age_key_content = r#"# public key: age1test_public_key_for_testing
AGE-SECRET-KEY-1TESTPRIVATEKEYFORTESTING
"#;
    let key_path = env.create_age_key(age_key_content);

    // Run init-project
    let config = shadow_secret::init::InitConfig {
        master_key_path: key_path.clone(),
        create_example: false,
        prompt_global: false,
    };

    shadow_secret::init::init_project(config).unwrap();

    // Verify .sops.yaml was created
    let sops_config_path = env.project_dir().join(".sops.yaml");
    assert!(sops_config_path.exists(), ".sops.yaml should be created");

    let sops_content = fs::read_to_string(&sops_config_path).unwrap();
    assert!(sops_content.contains("age1test_public_key_for_testing"));
    assert!(sops_content.contains("creation_rules:"));
    assert!(sops_content.contains("path_regex: \\.enc\\.env$"));
}

#[test]
fn test_init_project_creates_enc_env_with_example() {
    let env = TestEnv::new();
    env.enter();

    // Create test age key
    let age_key_content = r#"# public key: age1test_public_key_123
AGE-SECRET-KEY-1TESTPRIVATEKEYABC
"#;
    let key_path = env.create_age_key(age_key_content);

    // Run init-project with example
    let config = shadow_secret::init::InitConfig {
        master_key_path: key_path,
        create_example: true,
        prompt_global: false,
    };

    shadow_secret::init::init_project(config).unwrap();

    // Verify .enc.env was created with examples
    let enc_env_path = env.project_dir().join(".enc.env");
    assert!(enc_env_path.exists(), ".enc.env should be created");

    let enc_env_content = fs::read_to_string(&enc_env_path).unwrap();
    assert!(enc_env_content.contains("API_KEY=PLACEHOLDER"));
    assert!(enc_env_content.contains("DATABASE_URL=PLACEHOLDER"));
}

#[test]
fn test_init_project_creates_empty_enc_env() {
    let env = TestEnv::new();
    env.enter();

    // Create test age key
    let age_key_content = r#"# public key: age1test_public_key_456
AGE-SECRET-KEY-1TESTPRIVATEKEYXYZ
"#;
    let key_path = env.create_age_key(age_key_content);

    // Run init-project without example
    let config = shadow_secret::init::InitConfig {
        master_key_path: key_path,
        create_example: false,
        prompt_global: false,
    };

    shadow_secret::init::init_project(config).unwrap();

    // Verify .enc.env was created empty
    let enc_env_path = env.project_dir().join(".enc.env");
    assert!(enc_env_path.exists(), ".enc.env should be created");

    let enc_env_content = fs::read_to_string(&enc_env_path).unwrap();
    assert!(enc_env_content.contains("# Encrypted secrets file"));
    assert!(!enc_env_content.contains("API_KEY"));
}

#[test]
fn test_extract_age_keypair_valid() {
    let env = TestEnv::new();

    let age_key_content = r#"# public key: age1test_public_key_789
AGE-SECRET-KEY-1TESTPRIVATEKEYDEF
"#;
    let key_path = env.create_age_key(age_key_content);

    let keypair = shadow_secret::init::extract_age_keypair(&key_path).unwrap();

    assert_eq!(keypair.public_key, "age1test_public_key_789");
    assert_eq!(keypair.private_key, "AGE-SECRET-KEY-1TESTPRIVATEKEYDEF");
}

#[test]
fn test_extract_age_keypair_missing_public_key() {
    let env = TestEnv::new();

    let age_key_content = r#"AGE-SECRET-KEY-1TESTPRIVATEKEYDEF
"#;
    let key_path = env.create_age_key(age_key_content);

    let result = shadow_secret::init::extract_age_keypair(&key_path);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Public key not found"));
}

#[test]
fn test_extract_age_keypair_missing_private_key() {
    let env = TestEnv::new();

    let age_key_content = r#"# public key: age1test_public_key_789
"#;
    let key_path = env.create_age_key(age_key_content);

    let result = shadow_secret::init::extract_age_keypair(&key_path);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Private key not found"));
}

#[test]
fn test_create_sops_config_with_public_key() {
    let env = TestEnv::new();
    let public_key = "age1test_public_key";

    let config_path = shadow_secret::init::create_sops_config(env.project_dir(), public_key).unwrap();

    assert!(config_path.exists());
    let content = fs::read_to_string(&config_path).unwrap();

    assert!(content.contains(&format!("age: \"{}\"", public_key)));
    assert!(content.contains("creation_rules:"));
    assert!(content.contains("path_regex: \\.enc\\.env$") || content.contains(r"path_regex: \.enc\.env$"));
}

#[test]
fn test_add_to_global_config_creates_targets() {
    let env = TestEnv::new();
    env.enter();

    // Set HOME to temp directory for testing
    std::env::set_var("HOME", env.temp_dir.path());

    // Create global config
    env.create_global_config();

    // Add project to global config
    shadow_secret::init::add_to_global_config(env.project_dir()).unwrap();

    // Verify project was added
    let global_config_path = env.temp_dir.path().join(".shadow-secret.yaml");
    let content = fs::read_to_string(&global_config_path).unwrap();

    assert!(content.contains("targets:"));
    assert!(content.contains(&env.temp_dir.path().to_string_lossy().to_string()));
}

#[test]
fn test_add_to_global_config_handles_missing_config() {
    let env = TestEnv::new();
    env.enter();

    // Set HOME to temp directory for testing
    std::env::set_var("HOME", env.temp_dir.path());

    // Don't create global config - should not error
    let result = shadow_secret::init::add_to_global_config(env.project_dir());
    assert!(result.is_ok());
}

#[test]
fn test_default_master_key_path_from_env() {
    // Set environment variable
    std::env::set_var("SOPS_AGE_KEY_FILE", "/test/custom/path.txt");
    let path = shadow_secret::init::get_default_master_key_path();
    assert_eq!(path, PathBuf::from("/test/custom/path.txt"));
    std::env::remove_var("SOPS_AGE_KEY_FILE");
}

#[test]
fn test_default_master_key_path_fallback() {
    // Remove env variable if set
    std::env::remove_var("SOPS_AGE_KEY_FILE");

    let path = shadow_secret::init::get_default_master_key_path();

    // Should contain .shadow-secret/keys.txt as fallback
    assert!(path.to_string_lossy().contains(".shadow-secret"));
    assert!(path.to_string_lossy().contains("keys.txt"));
}

#[test]
fn test_init_project_error_when_key_missing_and_no_generate() {
    let env = TestEnv::new();
    env.enter();

    // Don't create age key - should error
    let config = shadow_secret::init::InitConfig {
        master_key_path: env.project_dir().join("nonexistent_key.txt"),
        create_example: false,
        prompt_global: false,
    };

    let result = shadow_secret::init::init_project(config);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Failed to read age key file"));
}

// Note: We skip testing age-keygen generation in automated tests
// as it requires the age binary to be installed and creates actual keys
// These scenarios are covered by manual integration testing
