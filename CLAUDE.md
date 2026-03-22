# Oalacea Shadow Secret - Project Context

> **Version:** 0.6.0 | **Type:** CLI Tool (Rust)
> **Status:** Active Development

---

## 🎯 Project Overview

**Oalacea Shadow Secret** is a secure, distributed secret management system that temporarily injects decrypted secrets (via SOPS) into configuration files, then automatically wipes them when the session ends.

### Core Philosophy

1. **Zero-Persistence**: Secrets are only in plain text on disk while working
2. **Transparency**: No need to modify existing tools (OpenClaw, Claude Code, etc.)
3. **Hygiene**: Automatic restoration of templates on exit or crash

---

## 🏗️ Architecture

### Project Structure

```
shadow-secret/
├── src/              # Source code
│   ├── main.rs       # CLI entry point
│   ├── config.rs     # Configuration parsing
│   ├── vault.rs      # Vault operations
│   ├── injector.rs   # Secret injection
│   ├── cleaner.rs    # Backup restoration
│   ├── init.rs       # Project initialization
│   └── cloud/        # Cloud integrations
├── tests/            # Integration tests
├── Cargo.toml        # Dependencies
└── docs/             # Technical documentation
```

---

## 🛠️ Tech Stack

**Language:** Rust 2021 edition

**Key Dependencies:**
- `clap` 4.5 - CLI argument parsing
- `tokio` 1.40 - Async runtime
- `serde` + `serde_yaml` - Configuration parsing
- `anyhow` - Error handling
- `age` - Encryption/decryption
- `dialoguer` - User prompts
- `ctrlc` - Signal handling
- `which` - Binary detection
- `dirs` - Home directory detection

**Build Profile (Release):**
```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
panic = "abort"
```

---

## 📋 Development Workflows

### Build

```bash
# Release build
cargo build --release

# Development build
cargo build
```

### Test

```bash
# Run tests
cargo test

# Run specific test
cargo test test_name
```

### Install Local

```bash
# Install from local path
cargo install --path .

# Force reinstall
cargo install --path . --force
```

### Publish to crates.io

```bash
# 1. Update version in Cargo.toml
# 2. Commit changes
git add .
git commit -m "Release vX.Y.Z"

# 3. Publish
cargo publish

# 4. Tag and push
git tag -a vX.Y.Z -m "Release vX.Y.Z"
git push --follow-tags
```

---

## 🎯 Commands

### `doctor`

Check prerequisites and system configuration.

**Verifies:**
- `sops` installation
- `age` installation
- `SOPS_AGE_KEY_FILE` environment variable
- Master key file existence
- Configuration accessibility

### `unlock`

Load secrets from vault and inject into target files.

**Workflow:**
1. Load and validate configuration
2. Decrypt secrets using SOPS
3. Inject into each target file
4. Register backups for restoration
5. Setup signal handlers (Ctrl+C)
6. Wait for user interruption
7. Restore all files on exit

### `init-project`

Bootstrap a new project with secret infrastructure.

**Creates:**
- `.sops.yaml` (SOPS configuration with public key)
- `.enc.env` (encrypted secrets file with placeholders)
- Prompts for global config addition

**Options:**
- `--master-key <path>` - Specify custom master key
- `--no-example` - Skip placeholder creation
- `--no-global` - Skip global config prompt

### `init-global`

Initialize global Shadow Secret configuration.

**Creates:**
- `~/.config/shadow-secret/global.yaml`
- `~/.config/shadow-secret/global.enc.env`
- `~/.config/shadow-secret/.sops.yaml`

### `push-cloud`

Push local secrets to Vercel environment variables.

**Features:**
- Auto-detect Vercel project ID
- Filter out `LOCAL_ONLY_*` prefixed secrets
- Dry-run mode for testing
- User confirmation before push

### `update`

Update to the latest version from crates.io.

**Features:**
- Check for updates without installing
- Automatic installation via cargo
- Force reinstall option

---

## 🔒 Security Principles

### RAM-Only Operations

Secrets are never written to temporary files:
- SOPS output flows directly into Rust HashMap
- No intermediate `.tmp` files
- Automatic memory cleanup on scope exit

### Zero Persistence

Secrets only exist in target files while process runs:
- Automatic restoration on Ctrl+C
- Signal handlers for SIGINT, SIGTERM
- Backup registration for all modified files

---

## 🧪 Testing

### Unit Tests

```bash
cargo test
```

### Integration Tests

Located in `tests/`:
- Test vault encryption/decryption
- Test file injection/restoration
- Test CLI commands

---

## 📝 Contributing

### Code Style

- **Rust**: Follow `cargo fmt` and `cargo clippy`
- **Comments**: Document public APIs
- **Errors**: Use `anyhow::Context` for error messages

### Commit Convention

Follow Git Flow Master conventions:

```
TYPE: Shadow Secret - vX.Y.Z

- Change description
```

Types:
- `RELEASE` - Breaking changes (MAJOR)
- `UPDATE` - New features (MINOR)
- `PATCH` - Bug fixes (PATCH)

---

## 🔗 Resources

- **Repository:** https://github.com/Pamacea/shadow-secret
- **crates.io:** https://crates.io/crates/oalacea-shadow-secret
- **SOPS:** https://github.com/getsops/sops
- **Age:** https://github.com/FiloSottile/age
- **Vercel CLI:** https://github.com/vercel/vercel

---

*Last Updated: 2025-03-23*
