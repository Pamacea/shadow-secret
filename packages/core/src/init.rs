//! Project initialization module for shadow-secret.
//!
//! This module handles the `init-project` command, which automates the setup of
//! secret management infrastructure for a new project.

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Age key components extracted from key file
#[derive(Debug, Clone)]
pub struct AgeKeyPair {
    /// Public key (starts with "age1...")
    pub public_key: String,
    /// Private key (AGE-SECRET-KEY-1...)
    pub private_key: String,
}

/// Project initialization configuration
#[derive(Debug)]
pub struct InitConfig {
    /// Path to the age master key file
    pub master_key_path: PathBuf,
    /// Whether to create .enc.env with placeholder
    pub create_example: bool,
    /// Whether to prompt for global config addition
    pub prompt_global: bool,
}

impl Default for InitConfig {
    fn default() -> Self {
        Self {
            master_key_path: get_default_master_key_path(),
            create_example: true,
            prompt_global: true,
        }
    }
}

/// Get the default path for the master age key.
///
/// Priority:
/// 1. $SOPS_AGE_KEY_FILE environment variable
/// 2. V:\-recovery\keys.txt (Windows with VeraCrypt)
/// 3. ~/.shadow-secret/keys.txt (Linux/macOS)
pub fn get_default_master_key_path() -> PathBuf {
    // Check environment variable first
    if let Ok(path) = std::env::var("SOPS_AGE_KEY_FILE") {
        return PathBuf::from(path);
    }

    // Windows: Check for VeraCrypt volume
    #[cfg(target_os = "windows")]
    {
        let veracrypt_path = PathBuf::from(r"V:\-recovery\keys.txt");
        if veracrypt_path.exists() {
            return veracrypt_path;
        }
    }

    // Default: ~/.shadow-secret/keys.txt
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".shadow-secret").join("keys.txt")
}

/// Extract age keypair from a key file.
///
/// Age key file format:
/// ```text
/// # public key: age1ql3z7j3...
/// AGE-SECRET-KEY-1YPV883...
/// ```
pub fn extract_age_keypair(path: &Path) -> Result<AgeKeyPair> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read age key file: {:?}", path))?;

    let mut public_key = None;
    let mut private_key = None;

    for line in content.lines() {
        let line = line.trim();

        // Extract public key from comment
        if line.starts_with("# public key: age1") {
            public_key = Some(
                line.trim_start_matches("# public key:")
                    .trim()
                    .to_string(),
            );
        }

        // Extract private key (AGE-SECRET-KEY-1...)
        if line.starts_with("AGE-SECRET-KEY-1") {
            private_key = Some(line.to_string());
        }
    }

    let public_key = public_key.ok_or_else(|| {
        anyhow::anyhow!(
            "Public key not found in age key file. Expected format: '# public key: age1...'"
        )
    })?;

    let private_key = private_key.ok_or_else(|| {
        anyhow::anyhow!(
            "Private key not found in age key file. Expected format: 'AGE-SECRET-KEY-1...'"
        )
    })?;

    Ok(AgeKeyPair {
        public_key,
        private_key,
    })
}

/// Generate a new age keypair using age-keygen.
pub fn generate_age_keypair(output_path: &Path) -> Result<AgeKeyPair> {
    println!("🔐 Generating new age keypair...");

    // Check if age is installed
    let check = Command::new("age").arg("--version").output();

    match check {
        Ok(output) if output.status.success() => {
            // age is installed, continue
        }
        Ok(_) => {
            return Err(anyhow::anyhow!(
                "'age' is installed but --version command failed. Please verify age installation."
            ));
        }
        Err(e) => {
            return Err(anyhow::anyhow!(
                "'age' is not installed or not in PATH: {}. Please install age first: https://github.com/FiloSottile/age/releases",
                e
            ));
        }
    }

    // Create parent directory if it doesn't exist
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {:?}", parent))?;
    }

    // Run age-keygen
    let output = Command::new("age-keygen")
        .arg("-o")
        .arg(output_path)
        .output()
        .with_context(|| "Failed to execute age-keygen")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!(
            "age-keygen failed: {}",
            if stderr.is_empty() {
                "Unknown error"
            } else {
                &*stderr
            }
        ));
    }

    println!("✓ Keypair generated at: {:?}", output_path);

    // Extract the keypair from the generated file
    extract_age_keypair(output_path)
}

/// Create .sops.yaml configuration file.
pub fn create_sops_config(project_dir: &Path, public_key: &str) -> Result<PathBuf> {
    let config_path = project_dir.join(".sops.yaml");

    let config_content = format!(
        r#"# SOPS configuration for shadow-secret
# This file was auto-generated by: shadow-secret init-project

creation_rules:
  - path_regex: .*\.enc\.env$
    age: "{}" # Age public key for encryption

# For more information, see: https://github.com/getsops/sops
"#,
        public_key
    );

    fs::write(&config_path, config_content)
        .with_context(|| format!("Failed to write .sops.yaml to: {:?}", config_path))?;

    Ok(config_path)
}

/// Create initial .enc.env file (plaintext before encryption).
pub fn create_enc_env(project_dir: &Path, with_example: bool) -> Result<PathBuf> {
    let enc_env_path = project_dir.join(".enc.env");

    let content = if with_example {
        r#"# Example secrets file (will be encrypted)
# Replace placeholders with actual values after encryption

API_KEY=PLACEHOLDER
DATABASE_URL=PLACEHOLDER
"#
    } else {
        "# Encrypted secrets file (empty for now)\n"
    };

    fs::write(&enc_env_path, content)
        .with_context(|| format!("Failed to write .enc.env to: {:?}", enc_env_path))?;

    Ok(enc_env_path)
}

/// Encrypt .enc.env file using SOPS.
pub fn encrypt_enc_env(enc_env_path: &Path) -> Result<()> {
    println!("🔒 Encrypting .enc.env with SOPS...");

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
                "SOPS is not installed or not in PATH: {}. Please install SOPS first: https://github.com/getsops/sops/releases",
                e
            ));
        }
    }

    // Run sops --encrypt from the directory containing the file
    // This ensures SOPS can find .sops.yaml in the same directory
    let enc_dir = if let Some(parent) = enc_env_path.parent() {
        parent
    } else {
        Path::new(".")
    };

    let output = Command::new("sops")
        .arg("--encrypt")
        .arg("--output")
        .arg(enc_env_path)  // Output to same file for in-place encryption
        .arg(enc_env_path)  // Input file
        .current_dir(enc_dir)
        .output()
        .with_context(|| "Failed to execute SOPS encryption")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(anyhow::anyhow!(
            "SOPS encryption failed:\nstderr: {}\nstdout: {}",
            if stderr.is_empty() {
                "(empty)"
            } else {
                &*stderr
            },
            if stdout.is_empty() {
                "(empty)"
            } else {
                &*stdout
            }
        ));
    }

    println!("✓ .enc.env encrypted successfully");
    Ok(())
}

/// Add project to global shadow-secret.yaml configuration.
pub fn add_to_global_config(project_dir: &Path) -> Result<()> {
    let home = dirs::home_dir()
        .context("Failed to determine home directory")?;

    let global_config_path = home.join(".shadow-secret.yaml");

    // Check if global config exists
    if !global_config_path.exists() {
        println!("⚠️  Global config not found at: {:?}", global_config_path);
        println!("💡 Run 'shadow-secret init' first to create global config");
        return Ok(());
    }

    // Read existing config
    let content = fs::read_to_string(&global_config_path)
        .with_context(|| format!("Failed to read global config: {:?}", global_config_path))?;

    // Parse YAML
    let mut config: serde_yaml::Value = serde_yaml::from_str(&content)
        .with_context(|| "Failed to parse global config YAML")?;

    // Add project to targets
    let project_path = project_dir.to_string_lossy().to_string();

    if let Some(targets) = config["targets"].as_sequence_mut() {
        // Check if already exists
        for target in targets.iter() {
            if target["path"].as_str() == Some(&project_path) {
                println!("ℹ️  Project already in global config");
                return Ok(());
            }
        }

        // Add new target
        let new_target = serde_yaml::Mapping::from_iter([
            (serde_yaml::Value::String("name".to_string()), serde_yaml::Value::String(
                project_dir.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
            )),
            (serde_yaml::Value::String("path".to_string()), serde_yaml::Value::String(project_path.clone())),
            (
                serde_yaml::Value::String("placeholders".to_string()),
                serde_yaml::Value::Sequence(vec![
                    serde_yaml::Value::String("$ALL".to_string()),
                ]),
            ),
        ]);

        targets.push(serde_yaml::Value::Mapping(new_target));

        println!("✓ Added project to global config");
    } else {
        // Create targets array if it doesn't exist
        let targets = serde_yaml::Value::Sequence(vec![
            serde_yaml::Value::Mapping(serde_yaml::Mapping::from_iter([
                (serde_yaml::Value::String("name".to_string()), serde_yaml::Value::String(
                    project_dir
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                )),
                (serde_yaml::Value::String("path".to_string()), serde_yaml::Value::String(project_path.clone())),
                (
                    serde_yaml::Value::String("placeholders".to_string()),
                    serde_yaml::Value::Sequence(vec![serde_yaml::Value::String("$ALL".to_string())]),
                ),
            ])),
        ]);

        config["targets"] = targets;
    }

    // Write back
    let yaml_content = serde_yaml::to_string(&config)
        .with_context(|| "Failed to serialize global config")?;

    fs::write(&global_config_path, yaml_content)
        .with_context(|| format!("Failed to write global config: {:?}", global_config_path))?;

    Ok(())
}

/// Global configuration directory path
pub fn get_global_config_dir() -> Result<PathBuf> {
    let home = dirs::home_dir()
        .context("Failed to determine home directory")?;
    Ok(home.join(".config").join("shadow-secret"))
}

/// Initialize global Shadow Secret configuration.
///
/// This creates:
/// - ~/.config/shadow-secret/ directory
/// - global.yaml (configuration file)
/// - global.enc.env (encrypted secrets, created as empty file first)
///
/// The user can then move this directory to an encrypted drive for security.
pub fn init_global() -> Result<()> {
    println!("🌍 Shadow Secret Global Configuration Initialization");
    println!();

    // Step 1: Create global config directory
    println!("📁 Step 1: Creating global configuration directory");
    let global_dir = get_global_config_dir()?;

    if global_dir.exists() {
        println!("   ⚠️  Directory already exists: {:?}", global_dir);
        print!("   Continue? [Y/n]: ");
        use std::io::Write;
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if input.trim().to_lowercase() == "n" {
            return Ok(());
        }
    } else {
        fs::create_dir_all(&global_dir)
            .with_context(|| format!("Failed to create directory: {:?}", global_dir))?;
        println!("   ✓ Created: {:?}", global_dir);
    }
    println!();

    // Step 2: Check for or generate age keypair
    println!("📝 Step 2: Age Encryption Key");
    let default_key_path = get_default_master_key_path();

    let keypair = if default_key_path.exists() {
        println!("   ✓ Existing key found: {:?}", default_key_path);
        extract_age_keypair(&default_key_path)?
    } else {
        println!("   ✗ No age key found");
        println!("   💡 Generating new age keypair...");

        generate_age_keypair(&default_key_path)?
    };

    println!("   Public key: age1{}...", &keypair.public_key[..16]);
    println!();

    // Step 3: Create .sops.yaml in global directory
    println!("📝 Step 3: SOPS Configuration");
    let sops_config_path = global_dir.join(".sops.yaml");
    let sops_config_content = format!(
        r#"# SOPS configuration for Shadow Secret (global)
# This file was auto-generated by: shadow-secret init-global

creation_rules:
  - path_regex: .*\.enc\.env$
    age: "{}" # Age public key for encryption

# For more information, see: https://github.com/getsops/sops
"#,
        keypair.public_key
    );

    fs::write(&sops_config_path, sops_config_content)
        .with_context(|| format!("Failed to write .sops.yaml to: {:?}", sops_config_path))?;
    println!("   ✓ Created: {:?}", sops_config_path);
    println!();

    // Step 4: Create global.enc.env with placeholder and encrypt it
    println!("📝 Step 4: Global Secrets File");
    let global_enc_env = global_dir.join("global.enc.env");

    if global_enc_env.exists() {
        println!("   ℹ️  File already exists: {:?}", global_enc_env);
    } else {
        // Create the .enc.env file directly with placeholder secret
        // SOPS will encrypt it in place
        let enc_env_content = r#"# Global encrypted secrets file
# This file is encrypted with SOPS using your age key
#
# To edit secrets:
#   sops global.enc.env
#   OR
#   sops --decrypt global.enc.env > global.env.tmp
#   # Edit global.env.tmp with your secrets
#   sops --encrypt global.env.tmp > global.enc.env
#   rm global.env.tmp

# Example secrets (add more as needed):
EXAMPLE_SECRET=placeholder_value
"#;

        fs::write(&global_enc_env, enc_env_content)
            .with_context(|| format!("Failed to write global.enc.env: {:?}", global_enc_env))?;

        // Encrypt with SOPS (encrypts in place)
        println!("   🔒 Encrypting with SOPS...");
        encrypt_enc_env(&global_enc_env)?;

        println!("   ✓ Created and encrypted: {:?}", global_enc_env);
    }
    println!();

    // Step 5: Create global.yaml configuration
    println!("📝 Step 5: Global Configuration File");
    let global_yaml = global_dir.join("global.yaml");

    let global_yaml_content = format!(
        r#"# Shadow Secret Global Configuration
# This file was auto-generated by: shadow-secret init-global

vault:
  # Path to encrypted secrets file (relative to this config)
  source: "global.enc.env"

  # Optional: Explicit vault path (overrides source computation)
  # Uncomment to specify custom location (useful for encrypted drives)
  # Supports:
  #   - Absolute paths: "C:/encrypted-drive/global.enc.env" (Windows)
  #   - Absolute paths: "/Volumes/encrypted/global.enc.env" (macOS)
  #   - Absolute paths: "/mnt/encrypted/global.enc.env" (Linux)
  #   - Home-relative: "~/encrypted-drive/global.enc.env"
  # vault_path: "/path/to/your/encrypted/drive/global.enc.env"

  # Encryption engine (sops with age)
  engine: "sops"

  # Path to age private key for SOPS encryption/decryption
  age_key_path: "{}"

  # Whether to require vault mount (for VeraCrypt volumes)
  require_mount: false

# Example targets - modify as needed
targets:
  - name: "example-target"
    # Example: inject all secrets into a JSON file
    path: "example-config.json"
    # Use $ALL to inject all secrets, or specify individual ones
    placeholders:
      - "$ALL"

# ================================
# IMPORTANT: Configuration Required
# ================================

# For SOPS to encrypt/decrypt:
#   The age_key_path above is automatically used by shadow-secret
#   No manual environment variable configuration needed!
#
# To edit secrets (opens in your $EDITOR):
#    cd ~/.config/shadow-secret
#    sops global.enc.env
#
#    OR manually decrypt/edit/encrypt:
#    sops --decrypt global.enc.env > global.env.tmp
#    # Edit global.env.tmp
#    sops --encrypt global.env.tmp > global.enc.env
#    rm global.env.tmp
#
# ENCRYPTED DRIVE SUPPORT:
#   If your ~/.config/shadow-secret/ is on an encrypted drive:
#   1. Uncomment the vault_path field above
#   2. Set it to the absolute path where global.enc.env is located
#   3. Example: vault_path: "E:/shadow-secret/global.enc.env" (Windows)
#   4. Example: vault_path: "/Volumes/ShadowSecret/global.enc.env" (macOS)
#   5. Example: vault_path: "~/encrypted-drive/global.enc.env" (Linux)
#
# For project-specific usage:
#    - Create shadow-secret.yaml in your project
#    - Set vault.source to point to this global.enc.env
#    - Define your project-specific targets
"#,
        default_key_path.display().to_string()
    );

    fs::write(&global_yaml, global_yaml_content)
        .with_context(|| format!("Failed to write global.yaml to: {:?}", global_yaml))?;
    println!("   ✓ Created: {:?}", global_yaml);
    println!();

    // Step 6: Final instructions
    println!("✅ Global configuration initialized successfully!");
    println!();
    println!("📁 Configuration directory: {:?}", global_dir);
    println!();
    println!("🔐 Security Note:");
    println!("   You can now move the entire ~/.config/shadow-secret/ directory");
    println!("   to an encrypted drive (e.g., VeraCrypt volume) for enhanced security.");
    println!("   Just update the path in your project configurations accordingly.");
    println!();
    println!("📝 Next steps:");
    println!("   1. Add secrets to global.enc.env:");
    println!("      sops --encrypt {:?} < {:?}.tmp", global_enc_env, global_enc_env);
    println!("   2. Use in any project:");
    println!("      - Create shadow-secret.yaml in your project");
    println!("      - Set vault.source to point to this global.enc.env");
    println!("      - Define your project targets");
    println!("   3. Run: shadow-secret unlock");
    println!();

    Ok(())
}

/// Initialize a new project with shadow-secret infrastructure.
///
/// This is the main entry point for the `init-project` command.
pub fn init_project(config: InitConfig) -> Result<()> {
    println!("🚀 Shadow Secret Project Initialization");
    println!("Current directory: {:?}\n", std::env::current_dir());

    // Step 1: Check for or generate age master key
    println!("📝 Step 1: Age Master Key");
    println!("   Checking: {:?}", config.master_key_path);

    let keypair = if config.master_key_path.exists() {
        println!("   ✓ Existing key found");
        extract_age_keypair(&config.master_key_path)?
    } else {
        println!("   ✗ No key found");
        println!("   💡 To generate manually: age-keygen -o {:?}", config.master_key_path);

        // Prompt user
        print!("   Generate new keypair now? [Y/n]: ");
        use std::io::Write;
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if input.trim().to_lowercase() == "n" {
            return Err(anyhow::anyhow!(
                "Age key required. Please generate one first."
            ));
        }

        generate_age_keypair(&config.master_key_path)?
    };

    println!("   Public key: age1{}...\n", &keypair.public_key[..16]);

    // Step 2: Create .sops.yaml
    println!("📝 Step 2: SOPS Configuration");
    let project_dir = std::env::current_dir()?;
    let sops_config_path = create_sops_config(&project_dir, &keypair.public_key)?;
    println!("   ✓ Created: {:?}\n", sops_config_path);

    // Step 3: Create .enc.env
    println!("📝 Step 3: Encrypted Secrets File");
    let enc_env_path = create_enc_env(&project_dir, config.create_example)?;
    println!("   ✓ Created: {:?}\n", enc_env_path);

    // Step 4: Encrypt .enc.env
    println!("📝 Step 4: Encryption");
    encrypt_enc_env(&enc_env_path)?;
    println!();

    // Step 5: Optional global config
    if config.prompt_global {
        println!("📝 Step 5: Global Configuration");
        print!("   Add this project to global shadow-secret.yaml? [Y/n]: ");
        use std::io::Write;
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if input.trim().to_lowercase() != "n" {
            add_to_global_config(&project_dir)?;
        } else {
            println!("   ⊘ Skipped");
        }
        println!();
    }

    // Summary
    println!("✅ Project initialized successfully!");
    println!();
    println!("Next steps:");
    println!("  1. Edit .enc.env: sops --decrypt .enc.env > .env.tmp");
    println!("  2. Add secrets, then encrypt: sops --encrypt .env.tmp > .enc.env");
    println!("  3. Run: shadow-secret unlock");
    println!();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_extract_age_keypair_valid() {
        let temp_dir = TempDir::new().unwrap();
        let key_file = temp_dir.path().join("test_key.txt");

        let content = r#"# public key: age1test_public_key_123456789
AGE-SECRET-KEY-1TESTPRIVATEKEYABCDEFGHIJKLMNOPQRSTUVWXYZ
"#;

        fs::write(&key_file, content).unwrap();

        let keypair = extract_age_keypair(&key_file).unwrap();
        assert_eq!(keypair.public_key, "age1test_public_key_123456789");
        assert_eq!(keypair.private_key, "AGE-SECRET-KEY-1TESTPRIVATEKEYABCDEFGHIJKLMNOPQRSTUVWXYZ");
    }

    #[test]
    fn test_extract_age_keypair_missing_public() {
        let temp_dir = TempDir::new().unwrap();
        let key_file = temp_dir.path().join("test_key.txt");

        let content = r#"AGE-SECRET-KEY-1TESTPRIVATEKEYABCDEFGHIJKLMNOPQRSTUVWXYZ
"#;

        fs::write(&key_file, content).unwrap();

        let result = extract_age_keypair(&key_file);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Public key not found"));
    }

    #[test]
    fn test_extract_age_keypair_missing_private() {
        let temp_dir = TempDir::new().unwrap();
        let key_file = temp_dir.path().join("test_key.txt");

        let content = r#"# public key: age1test_public_key_123456789
"#;

        fs::write(&key_file, content).unwrap();

        let result = extract_age_keypair(&key_file);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Private key not found"));
    }

    #[test]
    fn test_create_sops_config() {
        let temp_dir = TempDir::new().unwrap();
        let public_key = "age1test_public_key";

        let config_path = create_sops_config(temp_dir.path(), public_key).unwrap();

        assert!(config_path.exists());
        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("age: \"age1test_public_key\""));
        assert!(content.contains("creation_rules:"));
        assert!(content.contains("path_regex: \\.enc\\.env$") || content.contains(r"path_regex: \.enc\.env$"));
    }

    #[test]
    fn test_create_enc_env_with_example() {
        let temp_dir = TempDir::new().unwrap();

        let enc_env_path = create_enc_env(temp_dir.path(), true).unwrap();

        assert!(enc_env_path.exists());
        let content = fs::read_to_string(&enc_env_path).unwrap();
        assert!(content.contains("API_KEY=PLACEHOLDER"));
        assert!(content.contains("DATABASE_URL=PLACEHOLDER"));
    }

    #[test]
    fn test_create_enc_env_empty() {
        let temp_dir = TempDir::new().unwrap();

        let enc_env_path = create_enc_env(temp_dir.path(), false).unwrap();

        assert!(enc_env_path.exists());
        let content = fs::read_to_string(&enc_env_path).unwrap();
        assert!(content.contains("# Encrypted secrets file"));
        assert!(!content.contains("API_KEY"));
    }
}
