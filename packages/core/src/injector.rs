//! Secret injection module for replacing placeholders with actual secrets.
//!
//! # Security Guarantees
//!
//! - **NO new files created**: Only modifies existing files in-place
//! - **Atomic operations**: Creates backups before modification
//! - **Preserves permissions**: Maintains original file metadata
//! - **Format preservation**: Keeps structure and formatting intact
//!
//! # Supported Formats
//!
//! - JSON: Replaces string values while preserving structure
//! - YAML: Replaces string values while preserving structure
//! - ENV: Simple placeholder replacement
//!
//! # Placeholder Format
//!
//! Placeholders are formatted as: `$KEY_NAME` or `${KEY_NAME}`
//!
//! # Example
//!
//! ```no_run
//! use shadow_secret::injector::inject_secrets;
//! use std::collections::HashMap;
//!
//! # fn main() -> anyhow::Result<()> {
//! let mut secrets = HashMap::new();
//! secrets.insert("API_KEY".to_string(), "sk_live_12345".to_string());
//!
//! let placeholders = vec["$API_KEY".to_string()];
//!
//! let backup = inject_secrets(
//!     std::path::Path::new("config.json"),
//!     &secrets,
//!     &placeholders
//! )?;
//!
//! // If something goes wrong, restore the backup
//! // backup.restore()?;
//! # Ok(())
//! # }
//! ```

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// File backup containing original content for restoration.
#[derive(Debug, Clone)]
pub struct FileBackup {
    /// Original file content
    original_content: String,
    /// Path to the file
    file_path: PathBuf,
    /// Original file permissions (Unix-only)
    #[cfg(unix)]
    original_permissions: std::fs::Permissions,
}

impl FileBackup {
    /// Create a backup by reading the original file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to backup
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file doesn't exist
    /// - The file cannot be read
    /// - File metadata cannot be retrieved
    pub fn create(path: &Path) -> Result<Self> {
        // Read file content
        let original_content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read file for backup: {}", path.display()))?;

        // Get file permissions for restoration (Unix-only)
        #[cfg(unix)]
        let original_permissions = fs::metadata(path)
            .with_context(|| format!("Failed to get file metadata: {}", path.display()))?
            .permissions();

        Ok(Self {
            original_content,
            file_path: path.to_path_buf(),
            #[cfg(unix)]
            original_permissions,
        })
    }

    /// Restore the original file content.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file cannot be written
    /// - Permissions cannot be restored (Unix)
    pub fn restore(&self) -> Result<()> {
        // Write original content back to file
        let mut file = fs::File::create(&self.file_path).with_context(|| {
            format!(
                "Failed to create file for restore: {}",
                self.file_path.display()
            )
        })?;

        file.write_all(self.original_content.as_bytes()).with_context(|| {
            format!(
                "Failed to write restored content to: {}",
                self.file_path.display()
            )
        })?;

        // Restore original permissions (Unix-only)
        #[cfg(unix)]
        {
            fs::set_permissions(&self.file_path, self.original_permissions.clone()).with_context(|| {
                format!(
                    "Failed to restore permissions for: {}",
                    self.file_path.display()
                )
            })?;
        }

        Ok(())
    }

    /// Get the original file content.
    pub fn content(&self) -> &str {
        &self.original_content
    }

    /// Get the file path.
    pub fn path(&self) -> &Path {
        &self.file_path
    }
}

/// Inject secrets into a file by replacing placeholders.
///
/// # Security
///
/// - Creates a backup before modification
/// - Modifies file in-place (never creates new files)
/// - Preserves file permissions
///
/// # Arguments
///
/// * `file_path` - Path to the file to modify
/// * `secrets` - Map of secret keys to values
/// * `placeholders` - List of placeholders to replace (e.g., "$API_KEY")
///
/// # Errors
///
/// Returns an error if:
/// - The file doesn't exist
/// - The file cannot be read
/// - A placeholder cannot be matched with a secret
/// - The file cannot be written
///
/// # Example
///
/// ```no_run
/// use shadow_secret::injector::inject_secrets;
/// use std::collections::HashMap;
///
/// # fn main() -> anyhow::Result<()> {
/// let mut secrets = HashMap::new();
/// secrets.insert("API_KEY".to_string(), "sk_live_12345".to_string());
///
/// let placeholders = vec["$API_KEY".to_string()];
///
/// let backup = inject_secrets(
///     std::path::Path::new("config.json"),
///     &secrets,
///     &placeholders
/// )?;
/// # Ok(())
/// # }
/// ```
pub fn inject_secrets(
    file_path: &Path,
    secrets: &HashMap<String, String>,
    placeholders: &[String],
) -> Result<FileBackup> {
    eprintln!("🔍 [DEBUG] Starting injection for: {}", file_path.display());
    eprintln!("🔍 [DEBUG] Placeholders: {:?}", placeholders);
    eprintln!("🔍 [DEBUG] Secrets keys: {:?}", secrets.keys().collect::<Vec<_>>());

    // Create backup
    let backup = match FileBackup::create(file_path) {
        Ok(b) => {
            eprintln!("✓ [DEBUG] Backup created successfully");
            b
        }
        Err(e) => {
            eprintln!("❌ [DEBUG] Failed to create backup: {:#?}", e);
            return Err(e.into());
        }
    };

    // Read file content
    let content = match fs::read_to_string(file_path) {
        Ok(c) => {
            eprintln!("✓ [DEBUG] File read successfully ({} bytes)", c.len());
            c
        }
        Err(e) => {
            eprintln!("❌ [DEBUG] Failed to read file: {:#?}", e);
            return Err(e.into());
        }
    };

    // Detect file format and replace placeholders
    let extension = file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");

    eprintln!("🔍 [DEBUG] File extension: '{}'", extension);

    let modified_content = match extension {
        "json" => {
            eprintln!("🔍 [DEBUG] Processing as JSON...");
            match replace_placeholders_json(&content, secrets, placeholders) {
                Ok(c) => {
                    eprintln!("✓ [DEBUG] JSON replacement successful");
                    c
                }
                Err(e) => {
                    eprintln!("❌ [DEBUG] JSON replacement failed: {:#?}", e);
                    return Err(e);
                }
            }
        }
        "yaml" | "yml" => replace_placeholders_yaml(&content, secrets, placeholders)
            .with_context(|| format!("Failed to replace placeholders in YAML file: {}", file_path.display()))?,
        "env" | "dotenv" => replace_placeholders(&content, secrets, placeholders),
        _ => {
            // Try to auto-detect format
            if content.trim_start().starts_with('{') {
                // JSON-like
                replace_placeholders_json(&content, secrets, placeholders)
                    .with_context(|| "Failed to replace placeholders in auto-detected JSON")?
            } else {
                // Default to simple replacement
                replace_placeholders(&content, secrets, placeholders)
            }
        }
    };

    // Write modified content back to file
    eprintln!("🔍 [DEBUG] Writing modified content back to file...");
    let mut file = match fs::File::create(file_path) {
        Ok(f) => {
            eprintln!("✓ [DEBUG] File opened for writing");
            f
        }
        Err(e) => {
            eprintln!("❌ [DEBUG] Failed to open file for writing: {:#?}", e);
            return Err(e.into());
        }
    };

    match file.write_all(modified_content.as_bytes()) {
        Ok(_) => eprintln!("✓ [DEBUG] Content written successfully"),
        Err(e) => {
            eprintln!("❌ [DEBUG] Failed to write content: {:#?}", e);
            return Err(e.into());
        }
    }

    eprintln!("✓ [DEBUG] Injection completed successfully");
    Ok(backup)
}

/// Replace placeholders in any text content.
///
/// This is a simple string replacement function that preserves formatting.
/// It handles both `$KEY` and `${KEY}` placeholder formats.
///
/// # Arguments
///
/// * `content` - Original content
/// * `secrets` - Map of secret keys to values
/// * `placeholders` - List of placeholders to replace
///
/// # Returns
///
/// Modified content with placeholders replaced by secret values.
pub fn replace_placeholders(
    content: &str,
    secrets: &HashMap<String, String>,
    placeholders: &[String],
) -> String {
    let mut result = content.to_string();

    for placeholder in placeholders {
        // Extract key name from placeholder
        // Supports both $KEY and ${KEY} formats
        let key = if placeholder.starts_with("${") && placeholder.ends_with('}') {
            // ${KEY} format
            &placeholder[2..placeholder.len() - 1]
        } else if placeholder.starts_with('$') {
            // $KEY format
            &placeholder[1..]
        } else {
            // No prefix, treat entire string as key
            placeholder.as_str()
        };

        // Look up secret value
        if let Some(secret_value) = secrets.get(key) {
            // Replace all occurrences
            result = result.replace(placeholder, secret_value);
        }
    }

    result
}

/// Replace placeholders in JSON content while preserving structure.
///
/// # Arguments
///
/// * `content` - JSON content as string
/// * `secrets` - Map of secret keys to values
/// * `placeholders` - List of placeholders to replace
///
/// # Returns
///
/// Modified JSON content with string values replaced.
fn replace_placeholders_json(
    content: &str,
    secrets: &HashMap<String, String>,
    placeholders: &[String],
) -> Result<String> {
    // Strip UTF-8 BOM if present (EF BB BF)
    let content = content.strip_prefix('\u{FEFF}').unwrap_or(content);

    // Parse JSON to preserve structure
    let mut json: serde_json::Value =
        serde_json::from_str(content).context("Failed to parse JSON content")?;

    // Recursively replace placeholders in string values
    replace_in_json_value(&mut json, secrets, placeholders)?;

    // Serialize back to JSON with pretty printing (4 spaces)
    let modified_content = serde_json::to_string_pretty(&json)
        .context("Failed to serialize modified JSON content")?;

    Ok(modified_content)
}

/// Recursively replace placeholders in JSON values.
fn replace_in_json_value(
    value: &mut serde_json::Value,
    secrets: &HashMap<String, String>,
    placeholders: &[String],
) -> Result<()> {
    match value {
        serde_json::Value::String(s) => {
            // Replace placeholders in string values
            *s = replace_placeholders(s, secrets, placeholders);
            Ok(())
        }
        serde_json::Value::Array(arr) => {
            // Recursively process array elements
            for item in arr.iter_mut() {
                replace_in_json_value(item, secrets, placeholders)?;
            }
            Ok(())
        }
        serde_json::Value::Object(obj) => {
            // Recursively process object values
            for (_key, value) in obj.iter_mut() {
                replace_in_json_value(value, secrets, placeholders)?;
            }
            Ok(())
        }
        _ => Ok(()), // Numbers, booleans, null remain unchanged
    }
}

/// Replace placeholders in YAML content while preserving structure.
///
/// # Arguments
///
/// * `content` - YAML content as string
/// * `secrets` - Map of secret keys to values
/// * `placeholders` - List of placeholders to replace
///
/// # Returns
///
/// Modified YAML content with string values replaced.
fn replace_placeholders_yaml(
    content: &str,
    secrets: &HashMap<String, String>,
    placeholders: &[String],
) -> Result<String> {
    // Strip UTF-8 BOM if present (EF BB BF)
    let content = content.strip_prefix('\u{FEFF}').unwrap_or(content);

    // Parse YAML to preserve structure
    let yaml: serde_yaml::Value =
        serde_yaml::from_str(content).context("Failed to parse YAML content")?;

    // Recursively replace placeholders in string values
    let modified_yaml = replace_in_yaml_value(yaml, secrets, placeholders)?;

    // Serialize back to YAML
    let modified_content = serde_yaml::to_string(&modified_yaml)
        .context("Failed to serialize modified YAML content")?;

    Ok(modified_content)
}

/// Recursively replace placeholders in YAML values.
fn replace_in_yaml_value(
    value: serde_yaml::Value,
    secrets: &HashMap<String, String>,
    placeholders: &[String],
) -> Result<serde_yaml::Value> {
    match value {
        serde_yaml::Value::String(s) => {
            // Replace placeholders in string values
            Ok(serde_yaml::Value::String(replace_placeholders(
                &s, secrets, placeholders,
            )))
        }
        serde_yaml::Value::Sequence(arr) => {
            // Recursively process sequence elements
            let mut modified_arr = Vec::new();
            for item in arr.into_iter() {
                modified_arr.push(replace_in_yaml_value(item, secrets, placeholders)?);
            }
            Ok(serde_yaml::Value::Sequence(modified_arr))
        }
        serde_yaml::Value::Mapping(map) => {
            // Recursively process mapping values
            let mut modified_map = serde_yaml::Mapping::new();
            for (key, value) in map.into_iter() {
                let modified_key = replace_in_yaml_value(key, secrets, placeholders)?;
                let modified_value = replace_in_yaml_value(value, secrets, placeholders)?;
                modified_map.insert(modified_key, modified_value);
            }
            Ok(serde_yaml::Value::Mapping(modified_map))
        }
        _ => Ok(value), // Numbers, booleans, null remain unchanged
    }
}

/// Extract key name from placeholder.
///
/// Supports:
/// - `$KEY` -> "KEY"
/// - `${KEY}` -> "KEY"
/// - `KEY` -> "KEY"
pub fn extract_key_name(placeholder: &str) -> &str {
    if placeholder.starts_with("${") && placeholder.ends_with('}') {
        &placeholder[2..placeholder.len() - 1]
    } else if placeholder.starts_with('$') {
        &placeholder[1..]
    } else {
        placeholder
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    /// Helper to create a temporary file with content
    fn create_temp_file(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file.flush().unwrap();
        file
    }

    #[test]
    fn test_extract_key_name_dollar_format() {
        assert_eq!(extract_key_name("$API_KEY"), "API_KEY");
        assert_eq!(extract_key_name("$DATABASE_URL"), "DATABASE_URL");
    }

    #[test]
    fn test_extract_key_name_braced_format() {
        assert_eq!(extract_key_name("${API_KEY}"), "API_KEY");
        assert_eq!(extract_key_name("${DATABASE_URL}"), "DATABASE_URL");
    }

    #[test]
    fn test_extract_key_name_no_prefix() {
        assert_eq!(extract_key_name("API_KEY"), "API_KEY");
    }

    #[test]
    fn test_replace_placeholders_simple() {
        let content = "API_KEY=$API_KEY\nDATABASE_URL=$DATABASE_URL";
        let mut secrets = HashMap::new();
        secrets.insert("API_KEY".to_string(), "sk_live_12345".to_string());
        secrets.insert("DATABASE_URL".to_string(), "postgres://localhost".to_string());

        let placeholders = vec!["$API_KEY".to_string(), "$DATABASE_URL".to_string()];
        let result = replace_placeholders(content, &secrets, &placeholders);

        assert!(result.contains("sk_live_12345"));
        assert!(result.contains("postgres://localhost"));
        assert!(!result.contains("$API_KEY"));
        assert!(!result.contains("$DATABASE_URL"));
    }

    #[test]
    fn test_replace_placeholders_braced_format() {
        let content = "API_KEY=${API_KEY}\nDATABASE_URL=${DATABASE_URL}";
        let mut secrets = HashMap::new();
        secrets.insert("API_KEY".to_string(), "sk_live_12345".to_string());
        secrets.insert("DATABASE_URL".to_string(), "postgres://localhost".to_string());

        let placeholders = vec![
            "${API_KEY}".to_string(),
            "${DATABASE_URL}".to_string(),
        ];
        let result = replace_placeholders(content, &secrets, &placeholders);

        assert!(result.contains("sk_live_12345"));
        assert!(result.contains("postgres://localhost"));
        assert!(!result.contains("${API_KEY}"));
        assert!(!result.contains("${DATABASE_URL}"));
    }

    #[test]
    fn test_replace_placeholders_missing_secret() {
        let content = "API_KEY=$API_KEY\nSECRET=$MISSING";
        let mut secrets = HashMap::new();
        secrets.insert("API_KEY".to_string(), "sk_live_12345".to_string());

        let placeholders = vec!["$API_KEY".to_string(), "$MISSING".to_string()];
        let result = replace_placeholders(content, &secrets, &placeholders);

        // API_KEY should be replaced
        assert!(result.contains("sk_live_12345"));
        // MISSING should remain unchanged (secret not found)
        assert!(result.contains("$MISSING"));
    }

    #[test]
    fn test_replace_placeholders_json_simple() {
        let content = r#"{"api_key": "$API_KEY", "database": "$DATABASE_URL"}"#;
        let mut secrets = HashMap::new();
        secrets.insert("API_KEY".to_string(), "sk_live_12345".to_string());
        secrets.insert("DATABASE_URL".to_string(), "postgres://localhost".to_string());

        let placeholders = vec!["$API_KEY".to_string(), "$DATABASE_URL".to_string()];
        let result = replace_placeholders_json(content, &secrets, &placeholders).unwrap();

        assert!(result.contains("sk_live_12345"));
        assert!(result.contains("postgres://localhost"));
        assert!(!result.contains("$API_KEY"));
        assert!(!result.contains("$DATABASE_URL"));

        // Verify it's valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["api_key"], "sk_live_12345");
        assert_eq!(parsed["database"], "postgres://localhost");
    }

    #[test]
    fn test_replace_placeholders_json_nested() {
        let content = r#"{
  "service": {
    "api_key": "$API_KEY",
    "endpoints": {
      "production": "$PROD_URL"
    }
  }
}"#;
        let mut secrets = HashMap::new();
        secrets.insert("API_KEY".to_string(), "sk_live_12345".to_string());
        secrets.insert("PROD_URL".to_string(), "https://api.example.com".to_string());

        let placeholders = vec!["$API_KEY".to_string(), "$PROD_URL".to_string()];
        let result = replace_placeholders_json(content, &secrets, &placeholders).unwrap();

        assert!(result.contains("sk_live_12345"));
        assert!(result.contains("https://api.example.com"));

        // Verify it's valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["service"]["api_key"], "sk_live_12345");
        assert_eq!(
            parsed["service"]["endpoints"]["production"],
            "https://api.example.com"
        );
    }

    #[test]
    fn test_replace_placeholders_json_preserves_numbers() {
        let content = r#"{"port": 8080, "timeout": 30.5, "enabled": true}"#;
        let secrets = HashMap::new();
        let placeholders = vec!["$NONEXISTENT".to_string()];

        let result = replace_placeholders_json(content, &secrets, &placeholders).unwrap();

        // Verify numbers and booleans are preserved
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["port"], 8080);
        assert_eq!(parsed["timeout"], 30.5);
        assert_eq!(parsed["enabled"], true);
    }

    #[test]
    fn test_replace_placeholders_yaml_simple() {
        let content = "api_key: $API_KEY\ndatabase_url: $DATABASE_URL";
        let mut secrets = HashMap::new();
        secrets.insert("API_KEY".to_string(), "sk_live_12345".to_string());
        secrets.insert("DATABASE_URL".to_string(), "postgres://localhost".to_string());

        let placeholders = vec!["$API_KEY".to_string(), "$DATABASE_URL".to_string()];
        let result = replace_placeholders_yaml(content, &secrets, &placeholders).unwrap();

        assert!(result.contains("sk_live_12345"));
        assert!(result.contains("postgres://localhost"));
        assert!(!result.contains("$API_KEY"));
        assert!(!result.contains("$DATABASE_URL"));

        // Verify it's valid YAML
        let parsed: serde_yaml::Value = serde_yaml::from_str(&result).unwrap();
        assert_eq!(parsed["api_key"], "sk_live_12345");
        assert_eq!(parsed["database_url"], "postgres://localhost");
    }

    #[test]
    fn test_replace_placeholders_yaml_nested() {
        let content = r#"service:
  api_key: $API_KEY
  endpoints:
    production: $PROD_URL"#;
        let mut secrets = HashMap::new();
        secrets.insert("API_KEY".to_string(), "sk_live_12345".to_string());
        secrets.insert("PROD_URL".to_string(), "https://api.example.com".to_string());

        let placeholders = vec!["$API_KEY".to_string(), "$PROD_URL".to_string()];
        let result = replace_placeholders_yaml(content, &secrets, &placeholders).unwrap();

        assert!(result.contains("sk_live_12345"));
        assert!(result.contains("https://api.example.com"));

        // Verify it's valid YAML
        let parsed: serde_yaml::Value = serde_yaml::from_str(&result).unwrap();
        assert_eq!(parsed["service"]["api_key"], "sk_live_12345");
        assert_eq!(
            parsed["service"]["endpoints"]["production"],
            "https://api.example.com"
        );
    }

    #[test]
    fn test_file_backup_create() {
        let content = "API_KEY=$API_KEY\nSECRET=value";
        let temp_file = create_temp_file(content);

        let backup = FileBackup::create(temp_file.path()).unwrap();

        assert_eq!(backup.content(), content);
        assert_eq!(backup.path(), temp_file.path());
    }

    #[test]
    fn test_file_backup_restore() {
        let original_content = "API_KEY=$API_KEY\nSECRET=value";
        let temp_file = create_temp_file(original_content);

        // Create backup
        let backup = FileBackup::create(temp_file.path()).unwrap();

        // Modify file
        let mut file = fs::File::create(temp_file.path()).unwrap();
        file.write_all(b"MODIFIED CONTENT").unwrap();
        file.flush().unwrap();

        // Verify file was modified
        let current_content = fs::read_to_string(temp_file.path()).unwrap();
        assert_eq!(current_content, "MODIFIED CONTENT");

        // Restore backup
        backup.restore().unwrap();

        // Verify original content restored
        let restored_content = fs::read_to_string(temp_file.path()).unwrap();
        assert_eq!(restored_content, original_content);
    }

    #[test]
    fn test_inject_secrets_json_file() {
        let content = r#"{"api_key": "$API_KEY", "database": "$DATABASE_URL"}"#;
        let temp_file = create_temp_file(content);

        let mut secrets = HashMap::new();
        secrets.insert("API_KEY".to_string(), "sk_live_12345".to_string());
        secrets.insert("DATABASE_URL".to_string(), "postgres://localhost".to_string());

        let placeholders = vec!["$API_KEY".to_string(), "$DATABASE_URL".to_string()];

        let backup = inject_secrets(temp_file.path(), &secrets, &placeholders).unwrap();

        // Verify file was modified
        let modified_content = fs::read_to_string(temp_file.path()).unwrap();
        assert!(modified_content.contains("sk_live_12345"));
        assert!(modified_content.contains("postgres://localhost"));

        // Verify backup contains original content
        assert_eq!(backup.content(), content);

        // Restore backup to clean up
        backup.restore().unwrap();
    }

    #[test]
    fn test_inject_secrets_env_file() {
        let content = "API_KEY=$API_KEY\nDATABASE_URL=$DATABASE_URL";
        let temp_file = create_temp_file(content);

        // Rename to .env for format detection
        let env_path = temp_file.path().with_extension("env");
        fs::rename(temp_file.path(), &env_path).unwrap();

        let mut secrets = HashMap::new();
        secrets.insert("API_KEY".to_string(), "sk_live_12345".to_string());
        secrets.insert("DATABASE_URL".to_string(), "postgres://localhost".to_string());

        let placeholders = vec!["$API_KEY".to_string(), "$DATABASE_URL".to_string()];

        let backup = inject_secrets(&env_path, &secrets, &placeholders).unwrap();

        // Verify file was modified
        let modified_content = fs::read_to_string(&env_path).unwrap();
        assert!(modified_content.contains("sk_live_12345"));
        assert!(modified_content.contains("postgres://localhost"));

        // Clean up
        backup.restore().unwrap();
        fs::remove_file(&env_path).unwrap();
    }

    #[test]
    fn test_inject_secrets_yaml_file() {
        let content = "api_key: $API_KEY\ndatabase_url: $DATABASE_URL";
        let temp_file = create_temp_file(content);

        // Rename to .yaml for format detection
        let yaml_path = temp_file.path().with_extension("yaml");
        fs::rename(temp_file.path(), &yaml_path).unwrap();

        let mut secrets = HashMap::new();
        secrets.insert("API_KEY".to_string(), "sk_live_12345".to_string());
        secrets.insert("DATABASE_URL".to_string(), "postgres://localhost".to_string());

        let placeholders = vec!["$API_KEY".to_string(), "$DATABASE_URL".to_string()];

        let backup = inject_secrets(&yaml_path, &secrets, &placeholders).unwrap();

        // Verify file was modified
        let modified_content = fs::read_to_string(&yaml_path).unwrap();
        assert!(modified_content.contains("sk_live_12345"));
        assert!(modified_content.contains("postgres://localhost"));

        // Clean up
        backup.restore().unwrap();
        fs::remove_file(&yaml_path).unwrap();
    }

    #[test]
    fn test_replace_placeholders_preserves_formatting() {
        let content = r#"{
  "api_key": "$API_KEY",
  "database": "$DATABASE_URL"
}"#;
        let mut secrets = HashMap::new();
        secrets.insert("API_KEY".to_string(), "sk_live_12345".to_string());
        secrets.insert("DATABASE_URL".to_string(), "postgres://localhost".to_string());

        let placeholders = vec!["$API_KEY".to_string(), "$DATABASE_URL".to_string()];
        let result = replace_placeholders_json(content, &secrets, &placeholders).unwrap();

        // Verify formatting is preserved (pretty printed with 2 spaces)
        assert!(result.contains("\n  "));
        assert!(result.contains("sk_live_12345"));
        assert!(result.contains("postgres://localhost"));
    }

    #[test]
    fn test_replace_placeholders_multiple_occurrences() {
        let content = "API_KEY=$API_KEY\nBACKUP_API_KEY=$API_KEY";
        let mut secrets = HashMap::new();
        secrets.insert("API_KEY".to_string(), "sk_live_12345".to_string());

        let placeholders = vec!["$API_KEY".to_string()];
        let result = replace_placeholders(content, &secrets, &placeholders);

        // Both occurrences should be replaced
        let parts: Vec<&str> = result.split('\n').collect();
        assert_eq!(parts[0], "API_KEY=sk_live_12345");
        assert_eq!(parts[1], "BACKUP_API_KEY=sk_live_12345");
    }

    #[test]
    fn test_inject_secrets_nonexistent_file() {
        let nonexistent_path = Path::new("/nonexistent/path/config.json");
        let secrets = HashMap::new();
        let placeholders = vec!["$API_KEY".to_string()];

        let result = inject_secrets(nonexistent_path, &secrets, &placeholders);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to read file"));
    }

    #[test]
    fn test_replace_placeholders_json_array() {
        let content = r#"{"keys": ["$API_KEY", "$SECRET_KEY"]}"#;
        let mut secrets = HashMap::new();
        secrets.insert("API_KEY".to_string(), "key1".to_string());
        secrets.insert("SECRET_KEY".to_string(), "key2".to_string());

        let placeholders = vec!["$API_KEY".to_string(), "$SECRET_KEY".to_string()];
        let result = replace_placeholders_json(content, &secrets, &placeholders).unwrap();

        // Verify it's valid JSON and values were replaced
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        let keys = parsed["keys"].as_array().unwrap();
        assert_eq!(keys[0], "key1");
        assert_eq!(keys[1], "key2");
    }

    #[test]
    fn test_replace_placeholders_yaml_sequence() {
        let content = r#"keys:
  - $API_KEY
  - $SECRET_KEY"#;
        let mut secrets = HashMap::new();
        secrets.insert("API_KEY".to_string(), "key1".to_string());
        secrets.insert("SECRET_KEY".to_string(), "key2".to_string());

        let placeholders = vec!["$API_KEY".to_string(), "$SECRET_KEY".to_string()];
        let result = replace_placeholders_yaml(content, &secrets, &placeholders).unwrap();

        // Verify it's valid YAML and values were replaced
        let parsed: serde_yaml::Value = serde_yaml::from_str(&result).unwrap();
        let keys = parsed["keys"].as_sequence().unwrap();
        assert_eq!(keys[0], "key1");
        assert_eq!(keys[1], "key2");
    }
}
