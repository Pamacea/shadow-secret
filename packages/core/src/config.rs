// Shadow Secret Configuration Module
//
// This module handles loading and parsing the configuration from shadow-secret.yaml

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Vault configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VaultConfig {
    /// Path to the encrypted secrets file
    pub source: String,

    /// Encryption engine (currently only "sops" is supported)
    pub engine: String,

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

    /// Load configuration from shadow-secret.yaml in the current directory
    pub fn from_current_dir() -> Result<Self> {
        Self::from_file("shadow-secret.yaml")
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
    pub fn vault_source_path(&self) -> Result<PathBuf> {
        let path = Path::new(&self.vault.source);
        if path.is_absolute() {
            Ok(path.to_path_buf())
        } else {
            // Expand ~ to home directory if present
            if self.vault.source.starts_with('~') {
                let home = dirs::home_dir()
                    .context("Failed to determine home directory")?;
                let expanded = self.vault.source.replacen('~', home.to_str().unwrap(), 1);
                Ok(PathBuf::from(expanded))
            } else {
                std::env::current_dir()
                    .map(|dir| dir.join(&self.vault.source))
                    .with_context(|| "Failed to get current directory")
            }
        }
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
                engine: "sops".to_string(),
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
                engine: "sops".to_string(),
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
                engine: "invalid".to_string(),
                require_mount: false,
            },
            targets: vec![],
        };

        assert!(config.validate().is_err());
    }
}
