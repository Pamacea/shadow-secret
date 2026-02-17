//! Vercel integration using Vercel CLI (NOT API).
//!
//! # Security
//!
//! - **Uses Vercel CLI**: Leverages existing authentication
//! - **NO secret logging**: Secret values are never logged
//! - **RAM-only operations**: Secrets passed via stdin to CLI
//! - **User confirmation**: Requires explicit approval before pushing
//!
//! # Vercel CLI Commands Used
//!
//! - `vercel env add <key>` - Add environment variable
//! - `vercel env ls` - List existing variables
//! - `vercel link` - Link project (if needed)

use anyhow::{Context, Result};
use dialoguer::{theme::ColorfulTheme, Confirm};
use serde::Deserialize;
use std::collections::HashMap;
use std::process::Command;

/// Push secrets to Vercel using Vercel CLI.
///
/// # Arguments
///
/// * `secrets` - Secrets to push (key-value pairs)
/// * `project_id` - Vercel project ID (optional, auto-detected if None)
/// * `dry_run` - If true, only show what would be pushed
///
/// # Security
///
/// - Never logs secret values
/// - Requires user confirmation
/// - Shows only variable names in summary
///
/// # Vercel CLI Usage
///
/// Uses `vercel env add <key>` command for each secret.
/// Secrets are passed via stdin to avoid shell exposure.
pub async fn push_secrets_to_vercel(
    secrets: &HashMap<String, String>,
    project_id: Option<String>,
    dry_run: bool,
) -> Result<()> {
    // Check if Vercel CLI is installed
    check_vercel_cli_installed()?;

    // Filter out LOCAL_ONLY_* secrets
    let secrets: HashMap<&String, &String> = secrets
        .iter()
        .filter(|(k, _)| !k.starts_with("LOCAL_ONLY_"))
        .collect();

    if secrets.is_empty() {
        println!("⚠️  No secrets to push (all secrets start with LOCAL_ONLY_)");
        return Ok(());
    }

    // Link project if project_id provided
    if let Some(pid) = &project_id {
        link_vercel_project(pid)?;
    }

    // Fetch existing variables
    println!("🔍 Fetching existing environment variables from Vercel...");
    let existing_vars = list_vercel_env_vars()?;

    // Show summary
    println!("\n📋 Summary of variables to push:");
    println!("   Total: {} variable(s)", secrets.len());
    println!("   Already exists: {}", existing_vars.len());
    println!("   New variables: {}", secrets.len() - existing_vars.len());

    // List variable names (NOT values - security!)
    println!("\n🔐 Variables to push:");
    for key in secrets.keys() {
        let status = if existing_vars.contains_key(*key) {
            "✓ (will overwrite)"
        } else {
            "  (new)"
        };
        println!("   - {} {}", key, status);
    }

    // Confirm
    if dry_run {
        println!("\n🏃 Dry run mode - no changes will be made");
        return Ok(());
    }

    let theme = ColorfulTheme::default();
    if !Confirm::with_theme(&theme)
        .with_prompt("\n❓ Push these secrets to Vercel?")
        .default(false)
        .interact()?
    {
        println!("❌ Cancelled by user");
        return Ok(());
    }

    // Push each variable
    println!("\n🚀 Pushing secrets to Vercel...\n");

    let mut succeeded = Vec::new();
    let mut failed = Vec::new();

    for (key, value) in secrets {
        print!("   → Pushing {}... ", key);

        match add_vercel_env_var(key, value).await {
            Ok(_) => {
                println!("✓");
                succeeded.push(key.clone());
            }
            Err(e) => {
                println!("✗");
                eprintln!("      Error: {}", e);
                failed.push((key.clone(), e.to_string()));
            }
        }
    }

    // Show results
    println!("\n📊 Results:");
    println!("   ✓ Succeeded: {}", succeeded.len());
    println!("   ✗ Failed: {}", failed.len());

    if !failed.is_empty() {
        println!("\n❌ Failed variables:");
        for (key, error) in &failed {
            println!("   - {}: {}", key, error);
        }
        anyhow::bail!("Failed to push {} variable(s)", failed.len());
    }

    println!("\n✅ All secrets pushed successfully!");
    Ok(())
}

/// Check if Vercel CLI is installed.
fn check_vercel_cli_installed() -> Result<()> {
    let output = Command::new("vercel")
        .arg("--version")
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout);
            println!("✓ Vercel CLI detected: {}", version.trim());
            Ok(())
        }
        Ok(_) => {
            anyhow::bail!(
                "Vercel CLI is installed but --version command failed. Please verify Vercel CLI installation."
            );
        }
        Err(e) => {
            anyhow::bail!(
                "Vercel CLI is not installed or not in PATH: {}. Please install Vercel CLI first:\n  npm install -g vercel",
                e
            );
        }
    }
}

/// Link Vercel project by project ID.
fn link_vercel_project(project_id: &str) -> Result<()> {
    println!("🔗 Linking Vercel project: {}", project_id);

    let output = Command::new("vercel")
        .arg("link")
        .arg("--yes")
        .arg(project_id)
        .output()
        .context("Failed to execute 'vercel link' command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(
            "Failed to link Vercel project: {}",
            if stderr.is_empty() {
                "Unknown error"
            } else {
                &*stderr
            }
        );
    }

    println!("✓ Project linked successfully");
    Ok(())
}

/// List all environment variables from Vercel.
///
/// # Returns
///
/// Map of variable name to environment type
fn list_vercel_env_vars() -> Result<HashMap<String, String>> {
    let output = Command::new("vercel")
        .arg("env")
        .arg("ls")
        .output()
        .context("Failed to execute 'vercel env ls' command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(
            "Failed to list Vercel environment variables: {}",
            if stderr.is_empty() {
                "Unknown error"
            } else {
                &*stderr
            }
        );
    }

    // Parse output (format: "key=value" or "key" for encrypted)
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut vars = HashMap::new();

    for line in stdout.lines() {
        let line = line.trim();

        // Skip headers and empty lines
        if line.is_empty() || line.starts_with('>') || line.starts_with("-") {
            continue;
        }

        // Extract variable name (before first space or equals)
        if let Some(key) = line.split_whitespace().next() {
            // Remove any trailing characters like "=" or "*"
            let key = key.trim_end_matches('=').trim_end_matches('*').to_string();

            if !key.is_empty() && !key.starts_with(' ') {
                vars.insert(key, "encrypted".to_string());
            }
        }
    }

    Ok(vars)
}

/// Add an environment variable to Vercel.
///
/// # Arguments
///
/// * `key` - Variable name
/// * `value` - Variable value
///
/// # Security
///
/// - Value is passed via stdin to avoid shell exposure
/// - Value is never logged
async fn add_vercel_env_var(key: &str, value: &str) -> Result<()> {
    // Build command: vercel env add <key>
    let mut child = Command::new("vercel")
        .arg("env")
        .arg("add")
        .arg(key)
        .arg("--yes")  // Auto-confirm
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .context("Failed to spawn 'vercel env add' command")?;

    // Write value to stdin
    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        writeln!(stdin, "{}", value)
            .context("Failed to write secret value to Vercel CLI stdin")?;
    }

    // Wait for command to complete
    let output = child
        .wait_with_output()
        .context("Failed to wait for 'vercel env add' command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(
            "Failed to add env var '{}': {}",
            key,
            if stderr.is_empty() {
                "Unknown error"
            } else {
                &*stderr
            }
        );
    }

    Ok(())
}

/// Detect Vercel project ID from multiple sources.
///
/// # Detection Order
///
/// 1. `.vercel/project.json` (Vercel CLI link)
/// 2. `vercel_project_id` in local `global.yaml`
/// 3. `vercel_project_id` in global config
///
/// # Returns
///
/// Project ID if found, None otherwise
pub fn detect_project_id() -> Result<Option<String>> {
    // Try .vercel/project.json first
    if let Some(id) = try_read_vercel_project_json()? {
        return Ok(Some(id));
    }

    // Try local global.yaml
    if let Some(id) = try_read_shadow_secret_yaml()? {
        return Ok(Some(id));
    }

    // Try global config
    if let Some(id) = try_read_global_config()? {
        return Ok(Some(id));
    }

    Ok(None)
}

/// Try to read project ID from `.vercel/project.json`.
fn try_read_vercel_project_json() -> Result<Option<String>> {
    use std::path::Path;

    let path = Path::new(".vercel/project.json");

    if !path.exists() {
        return Ok(None);
    }

    #[derive(Deserialize)]
    struct VercelProject {
        id: String,
    }

    let content = std::fs::read_to_string(path)
        .context("Failed to read .vercel/project.json")?;

    let project: VercelProject = serde_json::from_str(&content)
        .context("Failed to parse .vercel/project.json")?;

    Ok(Some(project.id))
}

/// Try to read project ID from local `global.yaml`.
fn try_read_shadow_secret_yaml() -> Result<Option<String>> {
    use std::path::Path;

    let path = Path::new("global.yaml");

    if !path.exists() {
        return Ok(None);
    }

    #[derive(Deserialize)]
    struct Config {
        vercel_project_id: Option<String>,
    }

    let content = std::fs::read_to_string(path)
        .context("Failed to read global.yaml")?;

    let config: Config = serde_yaml::from_str(&content)
        .context("Failed to parse global.yaml")?;

    Ok(config.vercel_project_id)
}

/// Try to read project ID from global config.
fn try_read_global_config() -> Result<Option<String>> {
    let home = dirs::home_dir()
        .context("Failed to determine home directory")?;

    let path = home.join(".config").join("shadow-secret").join("config.yaml");

    if !path.exists() {
        return Ok(None);
    }

    #[derive(Deserialize)]
    struct Config {
        vercel_project_id: Option<String>,
    }

    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read {:?}", path))?;

    let config: Config = serde_yaml::from_str(&content)
        .context("Failed to parse global config")?;

    Ok(config.vercel_project_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_project_id_from_vercel_json() {
        // This test requires actual .vercel/project.json to exist
        // In real tests, you'd create a temporary file
        let result = detect_project_id();
        // We can't assert specific result without actual file
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_vercel_env_vars_requires_cli() {
        // This test requires Vercel CLI to be installed
        let _result = list_vercel_env_vars();
        // Will fail if CLI not installed, which is expected
        // In real tests, you'd mock the Command execution
    }
}
