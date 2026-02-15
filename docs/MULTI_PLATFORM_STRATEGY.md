# Multi-Platform Release Strategy for v0.3.0

> **Version Target:** v0.3.0
> **Objective:** Full cross-platform support (Windows, Linux, macOS)

---

## 🎯 Goal

Automate multi-platform binary compilation and NPM publication via GitHub Actions when a new version tag is pushed.

---

## 📋 Current State (v0.2.1)

**Supported:**
- ✅ Windows x64 (manually compiled, included in package)
- ❌ Linux x64 (not compiled yet)
- ❌ macOS x64/ARM64 (not compiled yet)

**Limitation:**
- Binaires must be manually compiled and copied
- No automated multi-platform testing
- Each platform requires native compilation or cross-compilation setup

---

## 🚀 Strategy for v0.3.0

### 1. GitHub Actions Workflow (Primary)

**File:** `.github/workflows/publish.yml`

**Trigger:** Git tag push (e.g., `git tag v0.3.0 && git push origin v0.3.0`)

**Workflow:**

```yaml
build-and-publish:
  strategy:
    matrix:
      include:
        - os: ubuntu-latest
          target: x86_64-unknown-linux-gnu
        - os: macos-latest
          target: x86_64-apple-darwin
        - os: macos-latest
          target: aarch64-apple-darwin  # Apple Silicon
        - os: windows-latest
          target: x86_64-pc-windows-msvc
```

**Steps:**
1. Checkout code
2. Install Rust toolchain for target
3. Build release binary
4. Rename binary to `shadow-secret` (standardize naming)
5. Download all binaries from other matrix jobs
6. Copy all binaries to `packages/cli-npm/bin/`
7. Update `bridge.js` to remove "TODO" comments
8. Commit binaries to git (not pushed, just for packaging)
9. Publish to NPM registry

**Result:** All 4 platform binaries included in single NPM package

### 2. Platform Detection in bridge.js

**File:** `packages/cli-npm/lib/bridge.js`

**Logic:**
```javascript
const platform = process.platform;
const isWindows = platform === 'win32';
const binaryName = isWindows ? 'shadow-secret.exe' : 'shadow-secret';
```

**Supported Platforms:**
- Windows → `shadow-secret.exe`
- Linux → `shadow-secret`
- macOS → `shadow-secret`

### 3. NPM Package Structure

**Published Contents:**
```
packages/cli-npm/
├── bin/
│   ├── shadow-secret.exe     # Windows
│   ├── shadow-secret         # Linux
│   └── shadow-secret         # macOS (universal x64/ARM64)
├── lib/
│   └── bridge.js             # Platform detection + spawning
└── package.json
```

---

## 🔄 Release Process v0.3.0

### Developer Workflow

**Step 1: Update Version**
```bash
# Update version in packages/core/Cargo.toml
# Update version in packages/cli-npm/package.json
```

**Step 2: Tag and Push**
```bash
git tag -a v0.3.0 -m "Release v0.3.0 - Multi-Platform Support"
git push origin main
git push origin v0.3.0
```

**Step 3: Automatic Build & Publish**
- GitHub Actions detects tag push
- Builds binaries for all platforms
- Publishes to NPM automatically
- Users can install with `npm install -g @oalacea/shadow-secret`

**Alternative: Manual Testing (Before Release)**
```bash
# Build locally for Windows (your platform)
cd packages/core
cargo build --release
cp target/release/shadow-secret.exe ../cli-npm/bin/

# Test locally
cd ../cli-npm
npm pack
npm install -g ./oalacea-shadow-secret-0.3.0.tgz
shadow-secret --version
```

---

## 📦 Binary Size Estimates

Expected binary sizes (stripped release builds):

| Platform | Size | Notes |
|----------|------|-------|
| Windows x64 | ~1.3 MB | Current (v0.2.1) |
| Linux x64 | ~1.2 MB | Estimated |
| macOS x64 | ~1.1 MB | Estimated |
| macOS ARM64 | ~1.0 MB | Estimated |
| **Total (all 4)** | **~4.6 MB** | NPM package size |

**Note:** NPM package will include all 4 binaries, users only download and use the one for their platform.

---

## 🧪 Testing Strategy

### Before Tagging v0.3.0

1. **Test GitHub Actions Workflow**
   ```bash
   # Create test tag
   git tag -a v0.3.0-test -m "Test multi-platform build"
   git push origin v0.3.0-test

   # Check Actions tab for build results
   # Verify all 4 platforms build successfully
   ```

2. **Verify Artifact Downloads**
   - Check that binaries are correctly uploaded
   - Test that downloads work on all platforms

3. **Test NPM Publication (Dry Run)**
   - Use `npm publish --dry-run` first
   - Verify tarball contents includes all binaries

4. **Manual Testing**
   - Install package on each platform
   - Run `shadow-secret --version`
   - Run `shadow-secret doctor`

---

## 📝 Post-Release Verification

After v0.3.0 is published:

1. **Install Test on Each Platform**
   ```bash
   # Windows
   npm install -g @oalacea/shadow-secret
   shadow-secret --version

   # Linux/macOS
   npm install -g @oalacea/shadow-secret
   shadow-secret --version
   ```

2. **Verify Platform Detection**
   - Binary should match platform
   - No "binary not found" errors
   - Terminal inherits correctly

3. **Check NPM Registry**
   - Visit https://www.npmjs.com/package/@oalacea/shadow-secret
   - Verify version 0.3.0 is published
   - Check file list includes binaries

---

## 🔧 Troubleshooting

### Binaries Not Included in Package

**Problem:** `npm pack` creates tarball without binaries

**Solution:**
- Check `packages/cli-npm/.gitignore` - should be at package level, NOT in `bin/`
- Verify `files` field in `package.json` includes `"bin"`

### Build Failures on macOS

**Problem:** macOS compilation fails in GitHub Actions

**Solution:**
- Check Rust target compatibility
- May need to update `rust-toolchain` version
- Consider using `macos-13` for older macOS compatibility

### Binary Not Executable

**Problem:** Linux/macOS binary not executable after download

**Solution:**
- Add `chmod +x` step in workflow before NPM publish
- Verify filesystem permissions preserved in tarball

### Wrong Platform Binary Detected

**Problem:** bridge.js uses wrong binary name

**Solution:**
- Test platform detection logic locally
- Verify `process.platform` values on each OS
- Test bridge.js with `node -e "console.log(process.platform)"`

---

## 📋 Checklist for v0.3.0

- [ ] GitHub Actions workflow created (`publish.yml`)
- [ ] Workflow tested with test tag
- [ ] All 4 platform builds successful
- [ ] Binaries correctly copied to `packages/cli-npm/bin/`
- [ ] NPM package tested locally
- [ ] Documentation updated (README, CHANGELOG, CLAUDE.md)
- [ ] Git tag `v0.3.0` pushed
- [ ] GitHub Actions publishes successfully
- [ ] NPM package verified on registry
- [ ] Installation tested on Windows/Linux/macOS

---

## 🎯 Success Criteria for v0.3.0

- ✅ All 4 platform binaries included in NPM package
- ✅ Automatic platform detection works on all OS
- ✅ Users can install with single command: `npm install -g @oalacea/shadow-secret`
- ✅ CI/CD fully automated (no manual binary copying required)
- ✅ Package size < 10 MB (target: ~5 MB with 4 binaries)

---

*Last Updated: 2026-02-15*
