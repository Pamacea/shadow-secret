// Shadow Secret Configuration Module
//
// This module handles loading and parsing the configuration from project.yaml or global.yaml

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Vault configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VaultConfig {
    /// Path to the encrypted secrets file
    pub source: String,

    /// Optional: Explicit vault path (overrides source-based resolution)
    /// If specified, this path is used directly (absolute or ~/relative to home)
    /// Useful for encrypted drives or custom vault locations
    #[serde(default)]
    pub vault_path: Option<String>,

    /// Encryption engine (currently only "sops" is supported)
    pub engine: String,

    /// Path to age private key for SOPS encryption/decryption
    #[serde(default)]
    pub age_key_path: Option<String>,

    /// Whether to require the vault to be mounted (for VeraCrypt volumes)
    #[serde(default = "default_require_mount")]
    pub require_mount: bool,
}

fn default_require_mount() -> bool {
    false
}

/// Target configuration - where secrets are injected
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TargetConfig {
    /// Name of the target (e.g., "openclaw", "claude")
    pub name: String,

    /// Path to the target file
    pub path: String,

    /// List of placeholders to replace (e.g., ["$WEB_API_KEY", "$HOOK_TOKEN"])
    pub placeholders: Vec<String>,
}

/// Main configuration structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// Vault configuration
    pub vault: VaultConfig,

    /// List of targets
    pub targets: Vec<TargetConfig>,
}

impl Config {
    /// Load configuration from a YAML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())
            .with_context(|| format!("Failed to read config file: {:?}", path.as_ref()))?;

        let config: Config = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {:?}", path.as_ref()))?;

        Ok(config)
    }

    /// Load configuration from project.yaml in the current directory
    /// Falls back to global config if not found
    pub fn from_current_dir() -> Result<Self> {
        // Try project-specific config first
        let project_config = PathBuf::from("project.yaml");
        if project_config.exists() {
            return Self::from_file(&project_config);
        }

        // Fall back to global config
        let global_config = dirs::home_dir()
            .map(|home| home.join(".config/shadow-secret/global.yaml"))
            .context("Failed to determine global config path")?;

        if global_config.exists() {
            println!("🔑 Using global Shadow Secret configuration from ~/.config/shadow-secret/global.yaml");
            return Self::from_file(&global_config);
        }

        anyhow::bail!(
            "No Shadow Secret configuration found.\n\
            Create one of:\n\
            1. Project-specific: project.yaml (in current directory) - run 'shadow-secret init-project'\n\
            2. Global: ~/.config/shadow-secret/global.yaml - run 'shadow-secret init-global'\n\
            \n\
            "
        )
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        // Check vault source
        if self.vault.source.is_empty() {
            anyhow::bail!("Vault source cannot be empty");
        }

        // Check vault engine
        if self.vault.engine != "sops" {
            anyhow::bail!("Unsupported vault engine: '{}'. Only 'sops' is supported.", self.vault.engine);
        }

        // Check targets
        if self.targets.is_empty() {
            anyhow::bail!("At least one target must be configured");
        }

        // Validate each target
        for target in &self.targets {
            if target.name.is_empty() {
                anyhow::bail!("Target name cannot be empty");
            }
            if target.path.is_empty() {
                anyhow::bail!("Target path cannot be empty for target '{}'", target.name);
            }
            if target.placeholders.is_empty() {
                anyhow::bail!("Placeholders cannot be empty for target '{}'", target.name);
            }
        }

        Ok(())
    }

    /// Get the absolute path for the vault source
    ///
    /// # Arguments
    /// * `config_dir` - Directory containing the config file (for relative paths)
    ///
    /// # Resolution Order
    /// 1. If `vault_path` is specified, use it directly
    /// 2. If `source` is absolute, use it
    /// 3. If `source` starts with `~`, expand to home
    /// 4. Otherwise, relative to `config_dir` (not CWD)
    pub fn vault_source_path(&self, config_dir: &Path) -> Result<PathBuf> {
        // 1. Check explicit vault_path first (overrides source)
        if let Some(ref vault_path) = self.vault.vault_path {
            return Self::resolve_path(vault_path, config_dir);
        }

        // 2. Fall back to source field
        Self::resolve_path(&self.vault.source, config_dir)
    }

    /// Helper to resolve a path (absolute, ~, or relative to config_dir)
    fn resolve_path(path_str: &str, config_dir: &Path) -> Result<PathBuf> {
        let path = Path::new(path_str);

        // Absolute path
        if path.is_absolute() {
            return Ok(path.to_path_buf());
        }

        // ~ expansion (home directory)
        if path_str.starts_with('~') {
            let home = dirs::home_dir()
                .context("Failed to determine home directory")?;
            let expanded = path_str.replacen('~', home.to_str().unwrap(), 1);
            return Ok(PathBuf::from(expanded));
        }

        // Relative to config directory (CHANGED: was CWD)
        Ok(config_dir.join(path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validation() {
        let config = Config {
            vault: VaultConfig {
                source: "test.enc.env".to_string(),
                vault_path: None,
                engine: "sops".to_string(),
                age_key_path: None,
                require_mount: false,
            },
            targets: vec![
                TargetConfig {
                    name: "test".to_string(),
                    path: "/tmp/test.json".to_string(),
                    placeholders: vec!["$VAR".to_string()],
                },
            ],
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation_empty_source() {
        let config = Config {
            vault: VaultConfig {
                source: "".to_string(),
                vault_path: None,
                engine: "sops".to_string(),
                age_key_path: None,
                require_mount: false,
            },
            targets: vec![],
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_unsupported_engine() {
        let config = Config {
            vault: VaultConfig {
                source: "test.enc.env".to_string(),
                vault_path: None,
                engine: "invalid".to_string(),
                age_key_path: None,
                require_mount: false,
            },
            targets: vec![],
        };

        assert!(config.validate().is_err());
    }

    // NEW TESTS for vault_path functionality

    #[test]
    fn test_vault_path_explicit_overrides_source() {
        let config = Config {
            vault: VaultConfig {
                source: "ignored.enc.env".to_string(),
                vault_path: Some("/absolute/path/vault.enc.env".to_string()),
                engine: "sops".to_string(),
                age_key_path: None,
                require_mount: false,
            },
            targets: vec![],
        };

        let config_dir = Path::new("/any/dir");
        let result = config.vault_source_path(config_dir).unwrap();

        assert_eq!(result, PathBuf::from("/absolute/path/vault.enc.env"));
    }

    #[test]
    fn test_vault_path_relative_to_config_dir() {
        let config = Config {
            vault: VaultConfig {
                source: "vault.enc.env".to_string(),
                vault_path: None,
                engine: "sops".to_string(),
                age_key_path: None,
                require_mount: false,
            },
            targets: vec![],
        };

        let config_dir = Path::new("/home/user/.config/shadow-secret");
        let result = config.vault_source_path(config_dir).unwrap();

        assert_eq!(
            result,
            PathBuf::from("/home/user/.config/shadow-secret/vault.enc.env")
        );
    }

    #[test]
    fn test_vault_path_absolute() {
        let config = Config {
            vault: VaultConfig {
                source: "/absolute/vault.enc.env".to_string(),
                vault_path: None,
                engine: "sops".to_string(),
                age_key_path: None,
                require_mount: false,
            },
            targets: vec![],
        };

        let config_dir = Path::new("/any/dir");
        let result = config.vault_source_path(config_dir).unwrap();

        assert_eq!(result, PathBuf::from("/absolute/vault.enc.env"));
    }

    #[test]
    fn test_vault_path_tilde_expansion() {
        let config = Config {
            vault: VaultConfig {
                source: "~/vault.enc.env".to_string(),
                vault_path: None,
                engine: "sops".to_string(),
                age_key_path: None,
                require_mount: false,
            },
            targets: vec![],
        };

        let config_dir = Path::new("/any/dir");
        let result = config.vault_source_path(config_dir).unwrap();

        // Should expand ~ to home directory
        assert!(result.starts_with(dirs::home_dir().unwrap()));
        assert!(result.ends_with("vault.enc.env"));
    }

    #[test]
    fn test_vault_path_with_tilde_in_explicit_field() {
        let config = Config {
            vault: VaultConfig {
                source: "ignored.enc.env".to_string(),
                vault_path: Some("~/custom-drive/vault.enc.env".to_string()),
                engine: "sops".to_string(),
                age_key_path: None,
                require_mount: false,
            },
            targets: vec![],
        };

        let config_dir = Path::new("/any/dir");
        let result = config.vault_source_path(config_dir).unwrap();

        // Should expand ~ in vault_path field
        assert!(result.starts_with(dirs::home_dir().unwrap()));
        assert!(result.ends_with("custom-drive/vault.enc.env"));
    }
}
