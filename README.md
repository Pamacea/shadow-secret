# Shadow Secret

A secure, distributed secret management system for modern development workflows.

## Quick Start

```bash
# Install via NPM
npm install -g @oalacea/shadow-secret

# Option 1: Initialize global configuration (recommended for first-time users)
shadow-secret init-global

# Option 2: Initialize project-specific configuration
shadow-secret init-project

# Unlock secrets (project-specific)
shadow-secret unlock

# Or unlock global secrets
shadow-secret unlock-global

# Push to Vercel
shadow-secret push-cloud
```

## What is Shadow Secret?

Shadow Secret is a CLI tool that temporarily injects decrypted secrets (encrypted via SOPS) into your configuration files, then automatically wipes them when you're done.

**Key features:**
- Zero persistence: Secrets only exist in memory while active
- Automatic cleanup: Restores original files on exit or crash
- Platform agnostic: Works with any tool (OpenClaw, Claude Code, etc.)
- Vercel integration: Sync secrets to cloud deployments

## Architecture

This project uses a **monorepo structure**:

```
packages/
├── core/      # Rust engine (secret management logic)
└── cli-npm/   # NPM wrapper (cross-platform distribution)
```

The Rust binary is platform-agnostic and wrapped in an NPM package for easy distribution.

## Commands

### `doctor`

Check system prerequisites (sops, age, environment variables).

```bash
shadow-secret doctor
```

### `init-global`

Initialize global Shadow Secret configuration (recommended for first-time users).

```bash
shadow-secret init-global
```

Creates `~/.config/shadow-secret/` with:
- `global.yaml` - Configuration file (with `age_key_path` for automatic SOPS key detection)
- `global.enc.env` - Encrypted secrets
- `.sops.yaml` - SOPS encryption rules

**Benefits:**
- Centralized secret management for all projects
- Can be moved to encrypted drive (e.g., VeraCrypt volume)
- Shared across multiple projects

### `init-project`

Bootstrap a new project with secret infrastructure.

```bash
shadow-secret init-project
```

Creates `.sops.yaml` and `.enc.env` with your age public key.

### `unlock`

Load secrets from project-specific vault and inject into target files.

```bash
shadow-secret unlock
```

**Loads:** `shadow-secret.yaml` from current directory (project-specific config only)

**Does NOT fall back to global config.** Use `unlock-global` for global secrets.

### `unlock-global`

Load secrets from global vault and inject into target files.

```bash
shadow-secret unlock-global
```

**Loads:** `~/.config/shadow-secret/global.yaml`

**Best for:**
- Centralized secret management across multiple projects
- Encrypted drive setups (see [Encrypted Drive Setup](#encrypted-drive-setup))
- Shared development team secrets

### Encrypted Drive Setup

Starting with v0.3.8, you can store your encrypted vault on a separate drive (VeraCrypt, FileVault, LUKS) for enhanced security.

1. **Initialize global config:**
   ```bash
   shadow-secret init-global
   ```

2. **Edit `~/.config/shadow-secret/global.yaml` to add `vault_path`:**

   ```yaml
   vault:
     source: "global.enc.env"
     # Windows (VeraCrypt)
     vault_path: "E:/encrypted-drive/global.enc.env"
     # macOS (FileVault)
     # vault_path: "/Volumes/ShadowSecret/global.enc.env"
     # Linux (LUKS)
     # vault_path: "/mnt/encrypted/global.enc.env"
     engine: "sops"
     age_key_path: "~/.config/shadow-secret/keys.txt"
   ```

3. **Move vault to encrypted drive:**
   ```bash
   # Example for macOS
   mv ~/.config/shadow-secret/global.enc.env /Volumes/ShadowSecret/
   ```

4. **Unlock (mount encrypted drive first):**
   ```bash
   shadow-secret unlock-global
   ```

For detailed encrypted drive setup instructions, see [docs/GLOBAL_SETUP.md](docs/GLOBAL_SETUP.md).

### `push-cloud`

Push secrets to Vercel environment variables.

```bash
shadow-secret push-cloud
shadow-secret push-cloud --dry-run  # Preview changes
```

## Development

### Build Rust Core

```bash
cd packages/core
cargo build --release
```

### Test NPM Wrapper

```bash
cd packages/cli-npm
npm pack
```

### Run Tests

```bash
cd packages/core
cargo test
```

## Prerequisites

- **Rust** 2021 edition (for development)
- **sops** - Secret encryption
- **age** - Age encryption tool
- **SOPS_AGE_KEY_FILE** - Environment variable pointing to master key

Install sops and age:
```bash
# macOS
brew install sops age

# Linux
wget https://github.com/getsops/sops/releases/download/v3.8.1/sops-v3.8.1.linux.amd64
chmod +x sops-v3.8.1.linux.amd64
sudo mv sops-v3.8.1.linux.amd64 /usr/local/bin/sops
```

## Configuration

### Global Configuration (Recommended for Multiple Projects)

Create a global configuration once:

```bash
shadow-secret init-global
```

This creates `~/.config/shadow-secret/global.yaml`:

```yaml
vault:
  source: "global.enc.env"
  engine: "sops"
  require_mount: false

targets:
  - name: "example-target"
    path: "config.json"
    placeholders: ["$ALL"]  # Inject all secrets
```

**Using Global Config in Projects:**

Use `shadow-secret unlock-global` to explicitly load global secrets. Alternatively, create a `shadow-secret.yaml` in your project directory to use project-specific configuration.

### Project-Specific Configuration

Create `shadow-secret.yaml` in your project root:

```yaml
vault:
  source: ".enc.env"  # Or point to global: "~/.config/shadow-secret/global.enc.env"
  engine: "sops"

targets:
  - name: "app"
    path: ".env"
    placeholders: ["API_KEY", "DATABASE_URL"]
```

**Placeholders:**
- `$ALL` - Inject all secrets
- `SECRET_NAME` - Inject specific secret (e.g., `API_KEY`)
- Mix and match as needed

## Security

- Secrets are **never written to disk** in plain text (except target files while active)
- All secret operations happen **in RAM only**
- Automatic file restoration on process exit
- No temporary files or swap exposure

## License

MIT - see [LICENSE](LICENSE) file

## Repository

https://github.com/Pamacea/shadow-secret
