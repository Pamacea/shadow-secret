# Shadow Secret

A secure, distributed secret management system for modern development workflows.

## Quick Start

```bash
# Install via NPM
npm install -g @oalacea/shadow-secret

# Initialize project
shadow-secret init-project

# Unlock secrets
shadow-secret unlock

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

### `unlock`

Load secrets from vault and inject into target files.

```bash
shadow-secret unlock
```

### `init-project`

Bootstrap a new project with secret infrastructure.

```bash
shadow-secret init-project
```

Creates `.sops.yaml` and `.enc.env` with your age public key.

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

Create `shadow-secret.yaml` in your project root:

```yaml
vault:
  source: "path/to/.enc.env"  # Encrypted secrets file
  engine: "sops"

targets:
  - name: "app"
    path: ".env"
    placeholders: ["API_KEY", "DATABASE_URL"]
```

## Security

- Secrets are **never written to disk** in plain text (except target files while active)
- All secret operations happen **in RAM only**
- Automatic file restoration on process exit
- No temporary files or swap exposure

## License

MIT - see [LICENSE](LICENSE) file

## Repository

https://github.com/Pamacea/shadow-secret
