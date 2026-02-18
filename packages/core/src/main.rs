// Shadow Secret - A secure, distributed secret management system
//
// This is the main entry point for the application.

mod cleaner;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use shadow_secret::cloud::vercel::{detect_project_id, push_secrets_to_vercel};
use shadow_secret::config::Config;
use shadow_secret::vault::Vault;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Shadow Secret - A secure, distributed secret management system
#[derive(Parser, Debug)]
#[command(name = "shadow-secret")]
#[command(author = "Yanis <oalacea@proton.me>")]
#[command(version = "0.5.6")]
#[command(about = "A secure, distributed secret management system", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Check prerequisites and system configuration
    Doctor,

    /// Unlock secrets for current project (project-specific config only)
    Unlock {
        /// Path to the configuration file (default: project.yaml)
        #[arg(short, long, default_value = "project.yaml")]
        config: String,
    },

    /// Unlock global secrets (global config only)
    UnlockGlobal,

    /// Initialize a new project with secret management infrastructure
    InitProject {
        /// Path to the age master key file (default: auto-detected)
        #[arg(short, long)]
        master_key: Option<String>,

        /// Don't create example secrets in .enc.env
        #[arg(long, default_value = "false")]
        no_example: bool,

        /// Don't prompt to add to global config
        #[arg(long, default_value = "false")]
        no_global: bool,
    },

    /// Initialize global Shadow Secret configuration
    InitGlobal,

    /// Push secrets from local .enc.env to Vercel cloud
    PushCloud {
        /// Path to the configuration file (default: project.yaml)
        #[arg(short, long, default_value = "project.yaml")]
        config: String,

        /// Override Vercel project ID (auto-detected if not specified)
        #[arg(short, long)]
        project: Option<String>,

        /// Dry run - show what would be pushed without actually pushing
        #[arg(long, default_value = "false")]
        dry_run: bool,
    },

    /// Update Shadow Secret to latest version from NPM
    Update {
        /// Check for updates without installing
        #[arg(long, default_value = "false")]
        check_only: bool,
    },
}

fn check_binary(name: &str) -> Result<bool> {
    match which::which(name) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

fn check_env_var(var: &str) -> Result<bool> {
    Ok(std::env::var(var).is_ok())
}

fn check_file_exists(path: &str) -> Result<bool> {
    Ok(Path::new(path).exists())
}

/// Run basic prerequisite checks (sops, age, SOPS_AGE_KEY_FILE)
/// Used when checking system regardless of config mode
fn run_basic_checks() -> Result<()> {
    let mut all_checks_passed = true;

    // Check 1: sops installation
    print!("1. Checking if 'sops' is installed... ");
    match check_binary("sops") {
        Ok(true) => println!("✓"),
        Ok(false) => {
            println!("✗");
            println!("   ❌ 'sops' is not installed or not in PATH");
            println!("   📦 Install from: https://github.com/getsops/sops/releases");
            all_checks_passed = false;
        }
        Err(e) => {
            println!("✗");
            println!("   ❌ Error checking for 'sops': {}", e);
            all_checks_passed = false;
        }
    }

    // Check 2: age installation
    print!("2. Checking if 'age' is installed... ");
    match check_binary("age") {
        Ok(true) => println!("✓"),
        Ok(false) => {
            println!("✗");
            println!("   ❌ 'age' is not installed or not in PATH");
            println!("   📦 Install from: https://github.com/FiloSottile/age/releases");
            all_checks_passed = false;
        }
        Err(e) => {
            println!("✗");
            println!("   ❌ Error checking for 'age': {}", e);
            all_checks_passed = false;
        }
    }

    // Check 3: SOPS_AGE_KEY_FILE environment variable
    print!("3. Checking $SOPS_AGE_KEY_FILE environment variable... ");
    match check_env_var("SOPS_AGE_KEY_FILE") {
        Ok(true) => println!("✓"),
        Ok(false) => {
            println!("✗");
            println!("   ❌ $SOPS_AGE_KEY_FILE is not set");
            println!("   💡 Set it with: export SOPS_AGE_KEY_FILE=/path/to/key.txt");
            println!("   💡 Or specify 'age_key_path' in global.yaml");
            all_checks_passed = false;
        }
        Err(e) => {
            println!("✗");
            println!("   ❌ Error checking environment variable: {}", e);
            all_checks_passed = false;
        }
    }

    // Check 4: SOPS_AGE_KEY_FILE file existence
    print!("4. Checking if $SOPS_AGE_KEY_FILE file exists... ");
    if let Ok(key_file) = std::env::var("SOPS_AGE_KEY_FILE") {
        match check_file_exists(&key_file) {
            Ok(true) => println!("✓"),
            Ok(false) => {
                println!("✗");
                println!("   ❌ File not found: {}", key_file);
                println!("   💡 Verify the path is correct");
                all_checks_passed = false;
            }
            Err(e) => {
                println!("✗");
                println!("   ❌ Error checking file: {}", e);
                all_checks_passed = false;
            }
        }
    } else {
        println!("⊘");
        println!("   ⚠️  Skipped (environment variable not set)");
    }

    println!();
    if all_checks_passed {
        println!("✅ All basic checks passed! Your system is ready.");
        Ok(())
    } else {
        println!("❌ Some checks failed. Please fix the issues above.");
        Err(anyhow::anyhow!("Basic checks failed"))
    }
}

fn run_doctor() -> Result<()> {
    println!("🔍 Shadow Secret Doctor");
    println!("Checking prerequisites...\n");

    let mut all_checks_passed = true;

    // Check 1: sops installation
    print!("1. Checking if 'sops' is installed... ");
    match check_binary("sops") {
        Ok(true) => println!("✓"),
        Ok(false) => {
            println!("✗");
            println!("   ❌ 'sops' is not installed or not in PATH");
            println!("   📦 Install from: https://github.com/getsops/sops/releases");
            all_checks_passed = false;
        }
        Err(e) => {
            println!("✗");
            println!("   ❌ Error checking for 'sops': {}", e);
            all_checks_passed = false;
        }
    }

    // Check 2: age installation
    print!("2. Checking if 'age' is installed... ");
    match check_binary("age") {
        Ok(true) => println!("✓"),
        Ok(false) => {
            println!("✗");
            println!("   ❌ 'age' is not installed or not in PATH");
            println!("   📦 Install from: https://github.com/FiloSottile/age/releases");
            all_checks_passed = false;
        }
        Err(e) => {
            println!("✗");
            println!("   ❌ Error checking for 'age': {}", e);
            all_checks_passed = false;
        }
    }

    // Check 3: SOPS_AGE_KEY_FILE environment variable
    print!("3. Checking $SOPS_AGE_KEY_FILE environment variable... ");
    let env_var_set = match check_env_var("SOPS_AGE_KEY_FILE") {
        Ok(true) => {
            println!("✓");
            true
        }
        Ok(false) => {
            println!("⊘");
            println!("   ⚠️  $SOPS_AGE_KEY_FILE is not set");
            println!("   💡 You can either:");
            println!("      1. Set it: export SOPS_AGE_KEY_FILE=/path/to/key.txt");
            println!("      2. Add 'age_key_path' field to your vault config");
            false
        }
        Err(e) => {
            println!("✗");
            println!("   ❌ Error checking environment variable: {}", e);
            all_checks_passed = false;
            false
        }
    };

    // Additional check: if env var not set, check if age_key_path is in config
    if !env_var_set {
        // Check if project.yaml or global config has age_key_path
        let config_path = if Path::new("project.yaml").exists() {
            "project.yaml"
        } else {
            "~/.config/shadow-secret/global.yaml"
        };

        print!("   Checking if 'age_key_path' is in config... ");
        match check_file_exists(config_path) {
            Ok(true) => {
                // Try to read and parse config to check for age_key_path field
                if let Ok(content) = std::fs::read_to_string(config_path) {
                    if content.contains("age_key_path:") {
                        println!("✓");
                        println!("   ℹ️  Config has 'age_key_path' field");
                    } else {
                        println!("⊘");
                        println!("   ⚠️  Config does not have 'age_key_path' field");
                        println!("   💡 Add it to your vault config:");
                        println!("      vault:");
                        println!("        age_key_path: \"/path/to/your/keys.txt\"");
                    }
                } else {
                    println!("⊘");
                    println!("   ⚠️  Could not read config file");
                }
            }
            Ok(false) => {
                println!("⊘");
                println!("   ℹ️  No config file found to check");
            }
            Err(e) => {
                println!("⊘");
                println!("   ⚠️  Could not check config file: {}", e);
            }
        }
    }

    // Check 4: SOPS_AGE_KEY_FILE file existence
    print!("4. Checking if $SOPS_AGE_KEY_FILE file exists... ");
    if let Ok(key_file) = std::env::var("SOPS_AGE_KEY_FILE") {
        match check_file_exists(&key_file) {
            Ok(true) => println!("✓"),
            Ok(false) => {
                println!("✗");
                println!("   ❌ File not found: {}", key_file);
                println!("   💡 Verify the path is correct");
                all_checks_passed = false;
            }
            Err(e) => {
                println!("✗");
                println!("   ❌ Error checking file: {}", e);
                all_checks_passed = false;
            }
        }
    } else {
        println!("⊘");
        println!("   ⚠️  Skipped (environment variable not set)");
    }

    // Check 5: Vault source path accessibility
    print!("5. Checking vault source path accessibility... ");

    // Check if we're in global mode or project mode
    let project_config_exists = check_file_exists("project.yaml")?;

    let global_config_path = dirs::home_dir()
        .map(|home| home.join(".config/shadow-secret/global.yaml"));

    let global_config_exists = if let Some(ref path) = global_config_path {
        check_file_exists(path.to_str().unwrap_or(""))?
    } else {
        false
    };

    if project_config_exists {
        println!("✓");
        println!("   ℹ️  Project config found: project.yaml");
    } else if global_config_exists {
        println!("✓");
        println!("   ℹ️  Global config found: ~/.config/shadow-secret/global.yaml");
        println!("   💡 Use 'shadow-secret unlock-global' for global secrets");
    } else {
        println!("✗");
        println!("   ❌ No configuration found");
        println!("   💡 Create one of:");
        println!("      1. Project: project.yaml in current directory (run 'shadow-secret init-project')");
        println!("      2. Global: ~/.config/shadow-secret/global.yaml (run 'shadow-secret init-global')");
        println!("   💡 Run 'shadow-secret init-global' to create global config");
        all_checks_passed = false;
    }

    println!();
    if all_checks_passed {
        println!("✅ All checks passed! Your system is ready.");
        Ok(())
    } else {
        println!("❌ Some checks failed. Please fix the issues above.");
        Err(anyhow::anyhow!("Doctor checks failed"))
    }
}

fn run_unlock(config_path: &str) -> Result<()> {
    println!("🔓 Shadow Secret Unlock (Project)");
    println!("Loading configuration from: {}\n", config_path);

    // Step 1: Load and validate configuration (project-specific only, no global fallback)
    let config = Config::from_file(config_path)
        .with_context(|| format!("Failed to load config from: {}", config_path))?;

    config.validate()
        .with_context(|| "Configuration validation failed")?;

    println!("✓ Configuration loaded and validated");

    // Step 2: Get config directory for path resolution
    let config_abs_path = PathBuf::from(config_path)
        .canonicalize()
        .with_context(|| format!("Failed to resolve config file path: {}", config_path))?;

    let config_dir = config_abs_path
        .parent()
        .context("Config file has no parent directory")?;

    // Step 3: Load secrets from vault
    let vault_path = config.vault_source_path(config_dir)?;
    let vault_path_str = vault_path.to_str()
        .ok_or_else(|| anyhow::anyhow!("Vault path contains invalid UTF-8"))?;

    println!("📖 Loading secrets from: {}", vault_path_str);

    // Extract age_key_path from config if available
    let age_key_path = config.vault.age_key_path.as_deref();

    let vault = Vault::load(vault_path_str, age_key_path)
        .with_context(|| format!("Failed to load vault from: {}", vault_path_str))?;

    let secrets = vault.all();
    println!("✓ Loaded {} secret(s)", secrets.len());

    // Step 4: Inject secrets into each target
    println!("\n🎯 Injecting secrets into targets...");

    for target in &config.targets {
        println!("  → Target: {}", target.name);
        println!("    File: {}", target.path);

        // Create a copy of placeholders for the injector
        let placeholders: Vec<String> = target.placeholders.iter().cloned().collect();

        // Inject secrets
        let backup = shadow_secret::injector::inject_secrets(
            Path::new(&target.path),
            secrets,
            &placeholders,
        ).with_context(|| format!("Failed to inject secrets into: {}", target.path))?;

        // Register backup for cleanup
        cleaner::register_backup(&target.path, backup.content());

        println!("    ✓ Injected {} placeholder(s)", placeholders.len());
    }

    println!("\n✓ All secrets injected successfully!");
    println!("\n🎉 Secrets are now unlocked and injected!");
    println!("👉 Press Enter to lock secrets and restore templates...");

    // Wait for user input
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    println!("\n🔄 Restoring templates...");

    // Restore all backups
    cleaner::cleanup_and_restore();

    println!("✓ Templates restored!");
    println!("👋 See you next time!");

    Ok(())
}

fn run_unlock_global() -> Result<()> {
    println!("🔓 Shadow Secret Unlock (Global)");
    println!("Loading global configuration from ~/.config/shadow-secret/global.yaml\n");

    // Step 1: Load global config explicitly
    let global_config_path = dirs::home_dir()
        .map(|home| home.join(".config/shadow-secret/global.yaml"))
        .context("Failed to determine global config path")?;

    let config = Config::from_file(&global_config_path)
        .with_context(|| "Failed to load global config")?;

    config.validate()
        .with_context(|| "Global configuration validation failed")?;

    println!("✓ Global configuration loaded and validated");

    // Step 2: Get config directory for path resolution
    let config_dir = global_config_path
        .parent()
        .context("Global config has no parent directory")?;

    // Step 3: Load secrets from vault
    let vault_path = config.vault_source_path(config_dir)?;
    let vault_path_str = vault_path.to_str()
        .ok_or_else(|| anyhow::anyhow!("Vault path contains invalid UTF-8"))?;

    println!("📖 Loading secrets from: {}", vault_path_str);

    // Extract age_key_path from config if available
    let age_key_path = config.vault.age_key_path.as_deref();

    let vault = Vault::load(vault_path_str, age_key_path)
        .with_context(|| format!("Failed to load vault from: {}", vault_path_str))?;

    let secrets = vault.all();
    println!("✓ Loaded {} secret(s)", secrets.len());

    // Step 4: Inject secrets into each target
    println!("\n🎯 Injecting secrets into targets...");

    for target in &config.targets {
        println!("  → Target: {}", target.name);
        println!("    File: {}", target.path);

        let placeholders: Vec<String> = target.placeholders.iter().cloned().collect();

        let backup = shadow_secret::injector::inject_secrets(
            Path::new(&target.path),
            secrets,
            &placeholders,
        ).with_context(|| format!("Failed to inject secrets into: {}", target.path))?;

        cleaner::register_backup(&target.path, backup.content());

        println!("    ✓ Injected {} placeholder(s)", placeholders.len());
    }

    println!("\n✓ All secrets injected successfully!");
    println!("\n🎉 Global secrets are now unlocked and injected!");
    println!("👉 Press Enter to lock secrets and restore templates...");

    // Wait for user input
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    println!("\n🔄 Restoring templates...");

    // Restore all backups
    cleaner::cleanup_and_restore();

    println!("✓ Templates restored!");
    println!("👋 See you next time!");

    Ok(())
}

fn run_init_project(
    master_key: Option<String>,
    no_example: bool,
    no_global: bool,
) -> Result<()> {
    use shadow_secret::init::init_project;

    let config = shadow_secret::init::InitConfig {
        master_key_path: if let Some(path) = master_key {
            PathBuf::from(path)
        } else {
            shadow_secret::init::get_default_master_key_path()
        },
        create_example: !no_example,
        prompt_global: !no_global,
    };

    init_project(config)
}

fn run_init_global() -> Result<()> {
    use shadow_secret::init::init_global;

    init_global()
}

fn run_push_cloud(config_path: &str, project_id: Option<String>, dry_run: bool) -> Result<()> {
    println!("🚀 Shadow Secret Push-Cloud");
    println!("Loading configuration from: {}\n", config_path);

    // Step 1: Load and validate configuration
    let config = Config::from_file(config_path)
        .with_context(|| format!("Failed to load config from: {}", config_path))?;

    config.validate()
        .with_context(|| "Configuration validation failed")?;

    println!("✓ Configuration loaded and validated");

    // Step 2: Get config directory for path resolution
    let config_abs_path = PathBuf::from(config_path)
        .canonicalize()
        .with_context(|| format!("Failed to resolve config file path: {}", config_path))?;

    let config_dir = config_abs_path
        .parent()
        .context("Config file has no parent directory")?;

    // Step 3: Load secrets from vault
    let vault_path = config.vault_source_path(config_dir)?;
    let vault_path_str = vault_path.to_str()
        .ok_or_else(|| anyhow::anyhow!("Vault path contains invalid UTF-8"))?;

    println!("📖 Loading secrets from: {}", vault_path_str);

    // Extract age_key_path from config if available
    let age_key_path = config.vault.age_key_path.as_deref();

    let vault = Vault::load(vault_path_str, age_key_path)
        .with_context(|| format!("Failed to load vault from: {}", vault_path_str))?;

    let secrets: HashMap<String, String> = vault.all().clone();
    println!("✓ Loaded {} secret(s)", secrets.len());

    // Step 4: Detect or use provided project ID
    let project_id = if let Some(pid) = project_id {
        println!("🔗 Using provided project ID: {}", pid);
        Some(pid)
    } else {
        println!("🔍 Detecting Vercel project ID...");
        match detect_project_id()? {
            Some(id) => {
                println!("✓ Detected project ID: {}", id);
                Some(id)
            }
            None => {
                println!("⚠️  No project ID found. Using current Vercel CLI context.");
                None
            }
        }
    };

    // Step 5: Push secrets to Vercel
    println!("\n🎯 Pushing secrets to Vercel...\n");

    // Push secrets using Vercel CLI
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async {
            push_secrets_to_vercel(&secrets, project_id, dry_run).await
        })?;

    Ok(())
}

fn get_current_version() -> Result<String> {
    // Version from Cargo.toml
    Ok(env!("CARGO_PKG_VERSION").to_string())
}

fn get_latest_version() -> Result<String> {
    println!("🔍 Checking for updates on NPM...\n");

    // On Windows, npm is npm.cmd; on Unix, it's npm
    // Use which to find the actual npm executable
    let npm_exe = which::which("npm")
        .context("Failed to find 'npm'. Is NPM installed and in PATH?")?;

    let output = Command::new(&npm_exe)
        .args(["view", "@oalacea/shadow-secret", "version"])
        .output()
        .context("Failed to execute 'npm view'. Is NPM installed?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("npm view failed: {}", stderr));
    }

    let version = String::from_utf8_lossy(&output.stdout)
        .trim()
        .to_string();

    Ok(version)
}

fn run_update(check_only: bool) -> Result<()> {
    println!("🔄 Shadow Secret Update");
    println!();

    let current = get_current_version()?;
    let latest = get_latest_version()?;

    println!("📦 Current version: {}", current);
    println!("📦 Latest version:  {}", latest);
    println!();

    if current == latest {
        println!("✅ You're already on the latest version!");
        return Ok(());
    }

    println!("🆕 A new version is available!");
    println!();

    if check_only {
        println!("ℹ️  Run 'shadow-secret update' to install the latest version.");
        return Ok(());
    }

    println!("📥 Installing @oalacea/shadow-secret@{}...\n", latest);

    // On Windows, npm is npm.cmd; on Unix, it's npm
    // Use which to find the actual npm executable
    let npm_exe = which::which("npm")
        .context("Failed to find 'npm'. Is NPM installed and in PATH?")?;

    let output = Command::new(&npm_exe)
        .args(["install", "-g", "@oalacea/shadow-secret@latest"])
        .status()
        .context("Failed to execute 'npm install'. Is NPM installed?")?;

    if !output.success() {
        return Err(anyhow::anyhow!("npm install failed with exit code: {:?}", output));
    }

    println!();
    println!("✅ Successfully updated to version {}!", latest);
    println!();
    println!("🎉 Shadow Secret has been updated!");
    println!("💡 Run 'shadow-secret --version' to verify the update.");

    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Doctor => {
            // Smart doctor: auto-detect if we should check global config
            let project_config_exists = Path::new("project.yaml").exists();

            let global_config_path = dirs::home_dir()
                .map(|home| home.join(".config/shadow-secret/global.yaml"));

            let global_config_exists = if let Some(ref path) = global_config_path {
                path.exists()
            } else {
                false
            };

            // If only global config exists, provide helpful hint
            if !project_config_exists && global_config_exists {
                println!("🔍 Shadow Secret Doctor");
                println!("Checking prerequisites...\n");
                println!("ℹ️  No project config found (project.yaml)");
                println!("ℹ️  Global config detected: ~/.config/shadow-secret/global.yaml");
                println!("\n💡 Use 'shadow-secret unlock-global' for global secrets");
                println!("💡 Or create a project config with 'shadow-secret init-project'");

                // Run basic checks (sops, age, SOPS_AGE_KEY_FILE)
                run_basic_checks()?;
            } else {
                // Normal doctor for project mode
                if let Err(e) = run_doctor() {
                    eprintln!("\nError: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Unlock { config } => {
            if let Err(e) = run_unlock(&config) {
                eprintln!("\nError: {}", e);
                eprintln!("\n⚠️  Project secrets may not be properly injected.");
                eprintln!("💡 Run 'shadow-secret doctor' to check your configuration.");
                eprintln!("💡 Use 'shadow-secret unlock-global' for global secrets.");
                std::process::exit(1);
            }
        }
        Commands::UnlockGlobal => {
            if let Err(e) = run_unlock_global() {
                eprintln!("\nError: {}", e);
                eprintln!("\n⚠️  Global secrets may not be properly injected.");
                eprintln!("💡 Run 'shadow-secret doctor' to check your configuration.");
                std::process::exit(1);
            }
        }
        Commands::InitProject {
            master_key,
            no_example,
            no_global,
        } => {
            if let Err(e) = run_init_project(master_key, no_example, no_global) {
                eprintln!("\nError: {}", e);
                eprintln!("\n⚠️  Project initialization failed.");
                eprintln!("💡 Run 'shadow-secret doctor' to check your configuration.");
                std::process::exit(1);
            }
        }
        Commands::InitGlobal => {
            if let Err(e) = run_init_global() {
                eprintln!("\nError: {}", e);
                eprintln!("\n⚠️  Global initialization failed.");
                eprintln!("💡 Run 'shadow-secret doctor' to check your configuration.");
                std::process::exit(1);
            }
        }
        Commands::PushCloud {
            config,
            project,
            dry_run,
        } => {
            if let Err(e) = run_push_cloud(&config, project, dry_run) {
                eprintln!("\nError: {}", e);
                eprintln!("\n⚠️  Failed to push secrets to Vercel.");
                eprintln!("💡 Run 'shadow-secret doctor' to check your configuration.");
                eprintln!("💡 Make sure Vercel CLI is installed: npm install -g vercel");
                std::process::exit(1);
            }
        }
        Commands::Update { check_only } => {
            if let Err(e) = run_update(check_only) {
                eprintln!("\nError: {}", e);
                eprintln!("\n⚠️  Update failed.");
                eprintln!("💡 You can manually update with: npm install -g @oalacea/shadow-secret@latest");
                std::process::exit(1);
            }
        }
    }

    Ok(())
}
