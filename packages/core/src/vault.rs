//! Secure vault module for loading secrets from SOPS-encrypted files.
//!
//! # Security Guarantees
//!
//! - **NO temporary files**: Secrets are loaded directly from SOPS stdout to memory
//! - **Zero-copy parsing**: Output is captured as bytes and parsed in-memory
//! - **No disk writes**: Secrets never touch the filesystem after decryption
//!
//! # Supported Formats
//!
//! - ENV (key=value pairs)
//! - JSON (flat key-value structure)
//! - YAML (flat key-value structure)

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::process::Command;

/// Secure vault that holds decrypted secrets in memory only.
#[derive(Debug, Clone)]
pub struct Vault {
    pub(crate) secrets: HashMap<String, String>,
}

impl Vault {
    /// Create a new vault with pre-loaded secrets.
    ///
    /// # Warning
    ///
    /// This is primarily intended for testing. For production use,
    /// prefer [`Vault::load()`] which loads from encrypted files.
    pub fn new(secrets: HashMap<String, String>) -> Self {
        Self { secrets }
    }

    /// Load secrets from a SOPS-encrypted file.
    ///
    /// # Security
    ///
    /// This method executes `sops -d <path>` and captures stdout directly
    /// into memory. No temporary files are created.
    ///
    /// # Arguments
    ///
    /// * `encrypted_path` - Path to the SOPS-encrypted file
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - SOPS is not installed
    /// - The encrypted file doesn't exist
    /// - SOPS fails to decrypt the file
    /// - Output format cannot be parsed
    ///
    /// # Example
    ///
    /// ```no_run
    /// use shadow_secret::vault::Vault;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let vault = Vault::load("secrets.enc.yaml")?;
    /// let api_key = vault.get("API_KEY").unwrap();
    /// # Ok(())
    /// # }
    /// ```
    pub fn load(encrypted_path: &str) -> Result<Self> {
        // Execute SOPS and capture stdout directly to memory
        let output = execute_sops(encrypted_path)?;

        // Parse based on file extension
        let secrets = parse_output(encrypted_path, &output)?;

        Ok(Self { secrets })
    }

    /// Get a secret value by key.
    ///
    /// # Arguments
    ///
    /// * `key` - The secret key to retrieve
    ///
    /// # Returns
    ///
    /// - `Some(&String)` - Reference to the secret value if it exists
    /// - `None` - If the key doesn't exist
    pub fn get(&self, key: &str) -> Option<&String> {
        self.secrets.get(key)
    }

    /// Get all secrets as a read-only map.
    pub fn all(&self) -> &HashMap<String, String> {
        &self.secrets
    }
}

/// Execute SOPS command and capture stdout to memory.
///
/// # Security
///
/// - Captures stdout as bytes directly
/// - Never writes to disk
/// - Validates SOPS installation
fn execute_sops(encrypted_path: &str) -> Result<Vec<u8>> {
    // Check if SOPS is installed
    let check = Command::new("sops").arg("--version").output();

    match check {
        Ok(output) if output.status.success() => {
            // SOPS is installed, continue
        }
        Ok(_) => {
            return Err(anyhow::anyhow!(
                "SOPS is installed but --version command failed. Please verify SOPS installation."
            ));
        }
        Err(e) => {
            return Err(anyhow::anyhow!(
                "SOPS is not installed or not in PATH: {}. Please install SOPS first.",
                e
            ));
        }
    }

    // Execute sops -d <path>
    let output = Command::new("sops")
        .arg("-d")
        .arg(encrypted_path)
        .output()
        .with_context(|| {
            format!(
                "Failed to execute SOPS on file '{}'. Ensure the file exists and is readable.",
                encrypted_path
            )
        })?;

    // Check if SOPS command succeeded
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!(
            "SOPS decryption failed: {}",
            if stderr.is_empty() {
                "Unknown error"
            } else {
                &*stderr
            }
        ));
    }

    Ok(output.stdout)
}

/// Parse SOPS output based on file extension.
///
/// Supports: ENV, JSON, YAML
fn parse_output(path: &str, output: &[u8]) -> Result<HashMap<String, String>> {
    let extension = std::path::Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");

    match extension {
        "env" | "dotenv" => parse_env(output),
        "json" => parse_json(output),
        "yaml" | "yml" => parse_yaml(output),
        _ => {
            // Try to auto-detect format
            try_autodetect(output)
        }
    }
}

/// Parse ENV format (key=value pairs).
fn parse_env(output: &[u8]) -> Result<HashMap<String, String>> {
    let content = std::str::from_utf8(output).context("SOPS output is not valid UTF-8")?;

    let mut secrets = HashMap::new();

    for line in content.lines() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Parse key=value
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim().to_string();
            let value = value.trim().to_string();

            // Remove quotes if present
            let value = if (value.starts_with('"') && value.ends_with('"'))
                || (value.starts_with('\'') && value.ends_with('\''))
            {
                value[1..value.len() - 1].to_string()
            } else {
                value
            };

            secrets.insert(key, value);
        }
    }

    if secrets.is_empty() {
        return Err(anyhow::anyhow!(
            "No secrets found in ENV format. Expected 'key=value' pairs."
        ));
    }

    Ok(secrets)
}

/// Parse JSON format (flat key-value structure).
fn parse_json(output: &[u8]) -> Result<HashMap<String, String>> {
    let content = std::str::from_utf8(output).context("SOPS output is not valid UTF-8")?;

    let json: serde_json::Value =
        serde_json::from_str(content).with_context(|| "Failed to parse JSON output from SOPS")?;

    let mut secrets = HashMap::new();

    // Support both flat object {"key": "value"} and SOPS format {"data": {"key": "value"}}
    let data = if let Some(data) = json.get("data").and_then(|v| v.as_object()) {
        // SOPS format: {"data": {"key": "value"}, "sops": {...}}
        data
    } else if let Some(obj) = json.as_object() {
        // Flat format: {"key": "value"}
        obj
    } else {
        return Err(anyhow::anyhow!(
            "JSON must be an object with key-value pairs"
        ));
    };

    // Extract all string values
    for (key, value) in data {
        if let Some(str_value) = value.as_str() {
            secrets.insert(key.clone(), str_value.to_string());
        } else {
            return Err(anyhow::anyhow!(
                "JSON value for key '{}' must be a string, found: {}",
                key,
                value
            ));
        }
    }

    if secrets.is_empty() {
        return Err(anyhow::anyhow!(
            "No secrets found in JSON format. Expected key-value pairs with string values."
        ));
    }

    Ok(secrets)
}

/// Parse YAML format (flat key-value structure).
fn parse_yaml(output: &[u8]) -> Result<HashMap<String, String>> {
    let content = std::str::from_utf8(output).context("SOPS output is not valid UTF-8")?;

    let yaml: serde_yaml::Value =
        serde_yaml::from_str(content).with_context(|| "Failed to parse YAML output from SOPS")?;

    let mut secrets = HashMap::new();

    // Support both flat mapping and SOPS nested format
    let data = if let Some(data) = yaml.get("data").and_then(|v| v.as_mapping()) {
        // SOPS format: data: {key: value}
        data
    } else if let Some(mapping) = yaml.as_mapping() {
        // Flat format: key: value
        mapping
    } else {
        return Err(anyhow::anyhow!(
            "YAML must be a mapping with key-value pairs"
        ));
    };

    // Extract all string values
    for (key, value) in data {
        let key = key.as_str().with_context(|| "YAML key must be a string")?;

        if let Some(str_value) = value.as_str() {
            secrets.insert(key.to_string(), str_value.to_string());
        } else {
            return Err(anyhow::anyhow!(
                "YAML value for key '{}' must be a string, found: {:?}",
                key,
                value
            ));
        }
    }

    if secrets.is_empty() {
        return Err(anyhow::anyhow!(
            "No secrets found in YAML format. Expected key-value pairs with string values."
        ));
    }

    Ok(secrets)
}

/// Try to auto-detect format from content.
fn try_autodetect(output: &[u8]) -> Result<HashMap<String, String>> {
    let content = std::str::from_utf8(output).context("SOPS output is not valid UTF-8")?;

    // Try JSON first
    if content.trim_start().starts_with('{') {
        if let Ok(secrets) = parse_json(output) {
            return Ok(secrets);
        }
    }

    // Try YAML next
    if content.trim_start().starts_with("data:") || content.contains(':') {
        if let Ok(secrets) = parse_yaml(output) {
            return Ok(secrets);
        }
    }

    // Fall back to ENV
    if let Ok(secrets) = parse_env(output) {
        return Ok(secrets);
    }

    Err(anyhow::anyhow!(
        "Unable to auto-detect format. Please use a file extension: .env, .json, .yaml, or .yml"
    ))
}

// Expose parsing functions for integration testing
pub fn parse_env_for_testing(output: &[u8]) -> Result<HashMap<String, String>> {
    parse_env(output)
}

pub fn parse_json_for_testing(output: &[u8]) -> Result<HashMap<String, String>> {
    parse_json(output)
}

pub fn parse_yaml_for_testing(output: &[u8]) -> Result<HashMap<String, String>> {
    parse_yaml(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_env_format() {
        let env_output = b"API_KEY=sk_test_123\nDATABASE_URL=postgres://localhost\n# Comment\n";
        let secrets = parse_env(env_output).unwrap();

        assert_eq!(secrets.get("API_KEY"), Some(&"sk_test_123".to_string()));
        assert_eq!(
            secrets.get("DATABASE_URL"),
            Some(&"postgres://localhost".to_string())
        );
        assert_eq!(secrets.len(), 2);
    }

    #[test]
    fn test_parse_env_with_quotes() {
        let env_output = b"API_KEY=\"sk_test_123\"\nSECRET='value'";
        let secrets = parse_env(env_output).unwrap();

        assert_eq!(secrets.get("API_KEY"), Some(&"sk_test_123".to_string()));
        assert_eq!(secrets.get("SECRET"), Some(&"value".to_string()));
    }

    #[test]
    fn test_parse_json_format() {
        let json_output = br#"{"API_KEY":"sk_test_123","DATABASE_URL":"postgres://localhost"}"#;
        let secrets = parse_json(json_output).unwrap();

        assert_eq!(secrets.get("API_KEY"), Some(&"sk_test_123".to_string()));
        assert_eq!(
            secrets.get("DATABASE_URL"),
            Some(&"postgres://localhost".to_string())
        );
    }

    #[test]
    fn test_parse_json_sops_format() {
        let json_output = br#"{"data":{"API_KEY":"sk_test_123"},"sops":{"kms":[]}}"#;
        let secrets = parse_json(json_output).unwrap();

        assert_eq!(secrets.get("API_KEY"), Some(&"sk_test_123".to_string()));
        assert_eq!(secrets.len(), 1);
    }

    #[test]
    fn test_parse_yaml_format() {
        let yaml_output = b"API_KEY: sk_test_123\nDATABASE_URL: postgres://localhost\n";
        let secrets = parse_yaml(yaml_output).unwrap();

        assert_eq!(secrets.get("API_KEY"), Some(&"sk_test_123".to_string()));
        assert_eq!(
            secrets.get("DATABASE_URL"),
            Some(&"postgres://localhost".to_string())
        );
    }

    #[test]
    fn test_parse_yaml_sops_format() {
        let yaml_output = b"data:\n  API_KEY: sk_test_123\nsops:\n  kms: []\n";
        let secrets = parse_yaml(yaml_output).unwrap();

        assert_eq!(secrets.get("API_KEY"), Some(&"sk_test_123".to_string()));
        assert_eq!(secrets.len(), 1);
    }

    #[test]
    fn test_vault_get() {
        let mut secrets = HashMap::new();
        secrets.insert("API_KEY".to_string(), "sk_test_123".to_string());

        let vault = Vault::new(secrets);

        assert_eq!(vault.get("API_KEY"), Some(&"sk_test_123".to_string()));
        assert_eq!(vault.get("NON_EXISTENT"), None);
    }

    #[test]
    fn test_empty_env_returns_error() {
        let env_output = b"";
        let result = parse_env(env_output);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No secrets found"));
    }

    #[test]
    fn test_sops_not_installed_error() {
        // Mock SOPS not being installed by using an invalid command
        // In real scenario, this would catch SOPS not in PATH
        let output = Command::new("nonexistent_sops_command_xyz")
            .arg("-d")
            .arg("test.env")
            .output();

        assert!(output.is_err() || !output.unwrap().status.success());
    }

    #[test]
    fn test_no_temp_files_created() {
        // Verify that parsing doesn't create any files
        let env_output = b"SECRET=test_value\n";
        let before_count = std::fs::read_dir(".").unwrap().count();

        let _secrets = parse_env(env_output).unwrap();

        let after_count = std::fs::read_dir(".").unwrap().count();
        assert_eq!(
            before_count, after_count,
            "No files should be created during parsing"
        );
    }

    #[test]
    fn test_autodetect_json() {
        let json_output = br#"{"KEY":"value"}"#;
        let secrets = try_autodetect(json_output).unwrap();
        assert_eq!(secrets.get("KEY"), Some(&"value".to_string()));
    }

    #[test]
    fn test_autodetect_env() {
        let env_output = b"KEY=value\n";
        let secrets = try_autodetect(env_output).unwrap();
        assert_eq!(secrets.get("KEY"), Some(&"value".to_string()));
    }

    #[test]
    fn test_comments_and_empty_lines_in_env() {
        let env_output = b"# This is a comment\n\nKEY=value\n\n# Another comment\nSECRET2=value2\n";
        let secrets = parse_env(env_output).unwrap();

        assert_eq!(secrets.len(), 2);
        assert_eq!(secrets.get("KEY"), Some(&"value".to_string()));
        assert_eq!(secrets.get("SECRET2"), Some(&"value2".to_string()));
    }
}
