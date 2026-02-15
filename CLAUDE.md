# Shadow Secret - Project Context

> **Version:** 0.3.0 (Unreleased)
> **Type:** CLI Tool (Rust + NPM wrapper)
> **Status:** Active Development

---

## 🎯 Project Overview

**Shadow Secret** is a secure, distributed secret management system that temporarily injects decrypted secrets (via SOPS) into configuration files, then automatically wipes them when the session ends.

### Core Philosophy

1. **Zero-Persistence**: Secrets are only in plain text on disk while working
2. **Transparency**: No need to modify existing tools (OpenClaw, Claude Code, etc.)
3. **Hygiene**: Automatic restoration of templates on exit or crash

---

## 🏗️ Architecture

### Monorepo Structure

```
shadow-secret/
├── packages/
│   ├── core/              # Rust project (the engine)
│   │   ├── src/           # Source code
│   │   ├── tests/         # Integration tests
│   │   ├── Cargo.toml     # Rust dependencies
│   │   └── shadow-secret.yaml  # Config template
│   └── cli-npm/           # NPM wrapper (distribution)
│       ├── bin/           # Compiled binaries (gitignored)
│       ├── lib/           # bridge.js (OS detection + spawning)
│       └── package.json   # NPM manifest
├── docs/                  # Technical documentation
├── .github/               # CI/CD workflows
├── .gitignore
├── README.md
├── CHANGELOG.md
├── CLAUDE.md              # This file
└── LICENSE
```

### Component Responsibilities

**Core (Rust):**
- Decrypt secrets using SOPS
- Inject into target files
- Restore on exit
- Platform-agnostic (no NPM awareness)
- Extensions: init-project, push-cloud

**CLI-NPM (Node.js):**
- Detect user's OS (Windows/Unix)
- Spawn correct binary
- Forward all arguments
- Terminal inheritance (stdio: 'inherit')

---

## 🛠️ Tech Stack

### Core (Rust)

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

### Distribution (Node.js)

**Runtime:** Node.js (any version, used only for spawn)

**Package:** NPM registry distribution

**Key Files:**
- `lib/bridge.js` - OS detection and binary spawning
- `package.json` - NPM manifest with bin entry point

---

## 📋 Development Workflows

### Build Rust Binary

```bash
cd packages/core

# Development build (unoptimized)
cargo build

# Release build (optimized, stripped)
cargo build --release

# Run tests
cargo test

# Check compilation
cargo check
```

### Test NPM Wrapper (Development)

```bash
# 1. Build the Rust binary
cd packages/core
cargo build --release

# 2. Copy to bin directory (manual for dev)
cp target/release/shadow-secret.exe ../cli-npm/bin/  # Windows
cp target/release/shadow-secret ../cli-npm/bin/       # Unix

# 3. Test the wrapper
cd ../cli-npm
node lib/bridge.js --version
# Or: npm link && shadow-secret --version
```

### Full Release Workflow

```bash
# 1. Build release binaries
cd packages/core
cargo build --release

# 2. Copy binaries to cli-npm/bin
# Windows
cp target/release/shadow-secret.exe ../cli-npm/bin/
# macOS/Linux
cp target/release/shadow-secret ../cli-npm/bin/

# 3. Publish to NPM
cd ../cli-npm
npm publish

# 4. Tag and push to Git (from repo root)
cd ../..
git tag -a v0.1.0 -m "Release v0.1.0"
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
- `shadow-secret.yaml` accessibility

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

### `push-cloud`

Push local secrets to Vercel environment variables.

**Features:**
- Auto-detect Vercel project ID
- Filter out `LOCAL_ONLY_*` prefixed secrets
- Dry-run mode for testing
- User confirmation before push

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

### Distribution Safety

- Compiled binaries gitignored
- Injected during CI/CD only
- No secrets in repository
- `.enc` files never committed

---

## 🧪 Testing

### Unit Tests

```bash
cd packages/core
cargo test
```

### Integration Tests

Located in `packages/core/tests/`:
- Test vault encryption/decryption
- Test file injection/restoration
- Test CLI commands

### Manual Testing

```bash
# 1. Build and setup
cd packages/core && cargo build --release
cp target/release/shadow-secret ../cli-npm/bin/

# 2. Test doctor command
shadow-secret doctor

# 3. Test init-project
shadow-secret init-project

# 4. Test unlock
shadow-secret unlock

# 5. Test push-cloud (dry-run)
shadow-secret push-cloud --dry-run
```

---

## 🚀 CI/CD Pipeline

### GitHub Actions Workflow

**Steps:**
1. Checkout code
2. Install Rust toolchain
3. Build release binary (`cargo build --release`)
4. Copy binary to `packages/cli-npm/bin/`
5. Publish to NPM (on tag only)
6. Create GitHub Release

**Triggers:**
- Push to main
- Git tags (for releases)

---

## 📝 Contributing

### Code Style

- **Rust**: Follow `cargo fmt` and `cargo clippy`
- **Comments**: Document public APIs
- **Errors**: Use `anyhow::Context` for error messages

### Commit Convention

Follow [Git Flow Master](https://github.com/Pamacea/git-flow-master) conventions:

```
TYPE: Shadow Secret - vX.Y.Z

- Change description
```

Types:
- `RELEASE` - Breaking changes (MAJOR)
- `UPDATE` - New features (MINOR)
- `PATCH` - Bug fixes (PATCH)

---

## 🌍 Multi-Platform Release Strategy (v0.3.0)

**Documentation:** See [docs/MULTI_PLATFORM_STRATEGY.md](../docs/MULTI_PLATFORM_STRATEGY.md)

### Overview

Starting with v0.3.0, Shadow Secret will use **GitHub Actions** to:

1. **Automatically compile binaries** for all platforms on release
2. **Package all binaries** in single NPM package
3. **Publish to NPM** automatically on tag push

### Supported Platforms

| Platform | Architecture | Binary Name |
|----------|-------------|-------------|
| Windows | x64 | `shadow-secret.exe` |
| Linux | x64 | `shadow-secret` |
| macOS | x64 | `snapshot-secret` |
| macOS | ARM64 (Apple Silicon) | `shadow-secret` |

### Release Workflow

```bash
# 1. Update versions
# Edit packages/core/Cargo.toml
# Edit packages/cli-npm/package.json

# 2. Tag and push (triggers CI/CD)
git tag -a v0.3.0 -m "Release v0.3.0 - Multi-Platform Support"
git push origin main
git push origin v0.3.0

# 3. GitHub Actions automatically:
#    - Builds for all 4 platforms
#    - Packages all binaries in NPM package
#    - Publishes to @oalacea/shadow-secret
```

### Development Workflow

For testing before release:

```bash
# Build locally (your platform only)
cd packages/core
cargo build --release
cp target/release/shadow-secret.exe ../cli-npm/bin/  # Windows
cp target/release/shadow-secret ../cli-npm/bin/       # Unix

# Test locally
cd ../cli-npm
npm pack
npm install -g ./oalacea-shadow-secret-0.3.0.tgz
shadow-secret --version
```

### GitHub Actions Workflow

**File:** `.github/workflows/publish.yml`

**Triggers:** Push to `main` with tag pattern `v*`

**Matrix Build:**
- Ubuntu Linux (x86_64-unknown-linux-gnu)
- macOS x64 (x86_64-apple-darwin)
- macOS ARM64 (aarch64-apple-darwin)
- Windows (x86_64-pc-windows-msvc)

**Result:** All 4 binaries compiled and packaged automatically

---

## 🔗 Resources

- **Repository:** https://github.com/Pamacea/shadow-secret
- **SOPS:** https://github.com/getsops/sops
- **Age:** https://github.com/FiloSottile/age
- **Vercel CLI:** https://github.com/vercel/vercel

---

*Last Updated: 2025-02-15*
