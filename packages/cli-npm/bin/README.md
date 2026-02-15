# Binary Directory

This directory contains the compiled Rust binaries for different platforms.

## CI/CD Injection

The binaries are **NOT** committed to this repository. They are injected during the CI/CD pipeline before publishing to NPM.

## Build Process

1. **Compile Rust binary:**
   ```bash
   cd packages/core
   cargo build --release
   ```

2. **Copy to bin directory:**
   - Windows: `target/release/shadow-secret.exe` → `packages/cli-npm/bin/shadow-secret.exe`
   - Unix: `target/release/shadow-secret` → `packages/cli-npm/bin/shadow-secret`

3. **Publish to NPM:**
   ```bash
   cd packages/cli-npm
   npm publish
   ```

## Development

For local development during active development:

```bash
# Build the binary
cd packages/core
cargo build --release

# Copy to bin directory (manual for development)
cp target/release/shadow-secret.exe ../cli-npm/bin/  # Windows
cp target/release/shadow-secret ../cli-npm/bin/       # Unix

# Test the NPM wrapper
cd ../cli-npm
npm link
shadow-secret --version
```

## Platform Support

- ✅ Windows (x64): `shadow-secret.exe`
- ✅ Linux (x64): `shadow-secret`
- ✅ macOS (x64/ARM64): `shadow-secret`
