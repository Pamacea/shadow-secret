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

/// Shadow Secret - A secure, distributed secret management system
#[derive(Parser, Debug)]
#[command(name = "shadow-secret")]
#[command(author = "Yanis <yanis@example.com>")]
#[command(version = "0.1.0")]
#[command(about = "A secure, distributed secret management system", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Check prerequisites and system configuration
    Doctor,

    /// Unlock secrets and inject them into target files
    Unlock {
        /// Path to the configuration file (default: shadow-secret.yaml)
        #[arg(short, long, default_value = "shadow-secret.yaml")]
        config: String,
    },

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

    /// Push secrets from local .enc.env to Vercel cloud
    PushCloud {
        /// Path to the configuration file (default: shadow-secret.yaml)
        #[arg(short, long, default_value = "shadow-secret.yaml")]
        config: String,

        /// Override Vercel project ID (auto-detected if not specified)
        #[arg(short, long)]
        project: Option<String>,

        /// Dry run - show what would be pushed without actually pushing
        #[arg(long, default_value = "false")]
        dry_run: bool,
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
    match check_env_var("SOPS_AGE_KEY_FILE") {
        Ok(true) => println!("✓"),
        Ok(false) => {
            println!("✗");
            println!("   ❌ $SOPS_AGE_KEY_FILE is not set");
            println!("   💡 Set it with: export SOPS_AGE_KEY_FILE=/path/to/key.txt");
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

    // Check 5: Vault source path accessibility
    print!("5. Checking vault source path accessibility... ");
    // For now, we'll check if we can read the config file
    match check_file_exists("shadow-secret.yaml") {
        Ok(true) => println!("✓"),
        Ok(false) => {
            println!("✗");
            println!("   ❌ shadow-secret.yaml not found in current directory");
            println!("   💡 Create a configuration file first");
            all_checks_passed = false;
        }
        Err(e) => {
            println!("✗");
            println!("   ❌ Error checking vault path: {}", e);
            all_checks_passed = false;
        }
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
    println!("🔓 Shadow Secret Unlock");
    println!("Loading configuration from: {}\n", config_path);

    // Step 1: Load and validate configuration
    let config = Config::from_file(config_path)
        .with_context(|| format!("Failed to load config from: {}", config_path))?;

    config.validate()
        .with_context(|| "Configuration validation failed")?;

    println!("✓ Configuration loaded and validated");

    // Step 2: Load secrets from vault
    let vault_path = config.vault_source_path()?;
    let vault_path_str = vault_path.to_str()
        .ok_or_else(|| anyhow::anyhow!("Vault path contains invalid UTF-8"))?;

    println!("📖 Loading secrets from: {}", vault_path_str);

    let vault = Vault::load(vault_path_str)
        .with_context(|| format!("Failed to load vault from: {}", vault_path_str))?;

    let secrets = vault.all();
    println!("✓ Loaded {} secret(s)", secrets.len());

    // Step 3: Inject secrets into each target
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

    println!("\n✓ All secrets injected successfully");

    // Step 4: Setup signal handlers for cleanup
    cleaner::setup_signal_handlers();

    println!("\n🎉 Secrets are now unlocked and injected!");
    println!("Press Ctrl+C to lock secrets and restore original files.");
    println!("\nWaiting... (Press Ctrl+C to exit)\n");

    // Step 5: Keep the process alive until Ctrl+C
    // We use a simple loop that sleeps to keep the process running
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
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

fn run_push_cloud(config_path: &str, project_id: Option<String>, dry_run: bool) -> Result<()> {
    println!("🚀 Shadow Secret Push-Cloud");
    println!("Loading configuration from: {}\n", config_path);

    // Step 1: Load and validate configuration
    let config = Config::from_file(config_path)
        .with_context(|| format!("Failed to load config from: {}", config_path))?;

    config.validate()
        .with_context(|| "Configuration validation failed")?;

    println!("✓ Configuration loaded and validated");

    // Step 2: Load secrets from vault
    let vault_path = config.vault_source_path()?;
    let vault_path_str = vault_path.to_str()
        .ok_or_else(|| anyhow::anyhow!("Vault path contains invalid UTF-8"))?;

    println!("📖 Loading secrets from: {}", vault_path_str);

    let vault = Vault::load(vault_path_str)
        .with_context(|| format!("Failed to load vault from: {}", vault_path_str))?;

    let secrets: HashMap<String, String> = vault.all().clone();
    println!("✓ Loaded {} secret(s)", secrets.len());

    // Step 3: Detect or use provided project ID
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

    // Step 4: Push secrets to Vercel
    println!("\n🎯 Pushing secrets to Vercel...\n");

    // Push secrets using Vercel CLI
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async {
            push_secrets_to_vercel(&secrets, project_id, dry_run).await
        })?;

    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Doctor => {
            if let Err(e) = run_doctor() {
                eprintln!("\nError: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Unlock { config } => {
            if let Err(e) = run_unlock(&config) {
                eprintln!("\nError: {}", e);
                eprintln!("\n⚠️  Secrets may not be properly injected.");
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
    }

    Ok(())
}
