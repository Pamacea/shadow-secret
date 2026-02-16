# Global Setup with Encrypted Drives

Complete guide for configuring Shadow Secret with encrypted drives (VeraCrypt, FileVault, LUKS).

---

## Overview

Starting with **v0.3.8**, Shadow Secret supports storing encrypted vaults on separate drives, providing enhanced security for sensitive secrets.

### Why Use Encrypted Drives?

- **Enhanced Security**: Secrets only accessible when mounted
- **Portability**: Move vaults between systems
- **Isolation**: Keep sensitive data separate from development environment
- **Compliance**: Meet regulatory requirements for secret storage

---

## Quick Start

### 1. Initialize Global Configuration

```bash
shadow-secret init-global
```

This creates:
- `~/.config/shadow-secret/global.yaml` - Global configuration
- `~/.config/shadow-secret/global.enc.env` - Encrypted vault (on home drive)

### 2. Move to Encrypted Drive

**Choose your platform:**

#### Windows (VeraCrypt)

```bash
# Mount VeraCrypt volume to E:\ (example)

# Edit global configuration
notepad ~/.config/shadow-secret/global.yaml
```

Add `vault_path` field:
```yaml
vault:
  source: "global.enc.env"
  vault_path: "E:/encrypted-drive/global.enc.env"  # Add this line
  engine: "sops"
  age_key_path: "~/.config/shadow-secret/keys.txt"
```

Move vault:
```bash
mv ~/.config/shadow-secret/global.enc.env "E:/encrypted-drive/global.enc.env"
```

#### macOS (FileVault)

```bash
# Assume encrypted drive mounted at /Volumes/ShadowSecret

# Edit global configuration
nano ~/.config/shadow-secret/global.yaml
```

Add `vault_path` field:
```yaml
vault:
  source: "global.enc.env"
  vault_path: "/Volumes/ShadowSecret/global.enc.env"  # Add this line
  engine: "sops"
  age_key_path: "~/.config/shadow-secret/keys.txt"
```

Move vault:
```bash
mv ~/.config/shadow-secret/global.enc.env /Volumes/ShadowSecret/global.enc.env
```

#### Linux (LUKS)

```bash
# Assume encrypted drive mounted at /mnt/encrypted

# Edit global configuration
nano ~/.config/shadow-secret/global.yaml
```

Add `vault_path` field:
```yaml
vault:
  source: "global.enc.env"
  vault_path: "/mnt/encrypted/global.enc.env"  # Add this line
  engine: "sops"
  age_key_path: "~/.config/shadow-secret/keys.txt"
```

Move vault:
```bash
mv ~/.config/shadow-secret/global.enc.env /mnt/encrypted/global.enc.env
```

### 3. Unlock Global Secrets

```bash
# Unlock global secrets (works with encrypted drives)
shadow-secret unlock-global
```

---

## Configuration Reference

### vault_path Field

**Purpose**: Explicitly specify vault location (overrides `source` field)

**Syntax**:
- Absolute paths: `"C:/path/to/vault.enc.env"` (Windows)
- Absolute paths: `"/path/to/vault.enc.env"` (Unix)
- Home-relative: `"~/encrypted-drive/vault.enc.env"` (all platforms)

**Priority**:
1. If `vault_path` is set → use it
2. Otherwise → use `source` field (relative to config directory)

**Examples**:

```yaml
# Windows - VeraCrypt volume
vault_path: "E:/secure-drive/global.enc.env"

# macOS - FileVault or external drive
vault_path: "/Volumes/ShadowSecret/global.enc.env"

# Linux - LUKS encrypted partition
vault_path: "/mnt/encrypted/global.enc.env"

# Home-relative (cross-platform)
vault_path: "~/encrypted-drive/global.enc.env"

# Network share (not recommended, but supported)
vault_path: "//nas/secure/global.enc.env"
```

---

## Common Scenarios

### Scenario 1: VeraCrypt on Windows

**Setup:**

1. Create VeraCrypt volume mounted as `E:\`
2. Move vault to encrypted drive:
   ```bash
   mv ~/.config/shadow-secret/global.enc.env E:/global.enc.env
   ```
3. Update `~/.config/shadow-secret/global.yaml`:
   ```yaml
   vault:
     source: "global.enc.env"
     vault_path: "E:/global.enc.env"
     engine: "sops"
     age_key_path: "~/.config/shadow-secret/keys.txt"
   ```

**Usage:**
```bash
# Mount VeraCrypt volume first
# Then unlock
shadow-secret unlock-global
```

### Scenario 2: FileVault on macOS

**Setup:**

1. Create encrypted disk image (.dmg) or use external drive
2. Mount at `/Volumes/ShadowSecret`
3. Update `~/.config/shadow-secret/global.yaml`:
   ```yaml
   vault:
     source: "global.enc.env"
     vault_path: "/Volumes/ShadowSecret/global.enc.env"
     engine: "sops"
     age_key_path: "~/.config/shadow-secret/keys.txt"
   ```

**Usage:**
```bash
# Mount encrypted drive first
# Then unlock
shadow-secret unlock-global
```

### Scenario 3: LUKS on Linux

**Setup:**

1. Create LUKS-encrypted partition
2. Mount at `/mnt/secret`
3. Update `~/.config/shadow-secret/global.yaml`:
   ```yaml
   vault:
     source: "global.enc.env"
     vault_path: "/mnt/secret/global.enc.env"
     engine: "sops"
     age_key_path: "~/.config/shadow-secret/keys.txt"
   ```

**Usage:**
```bash
# Mount LUKS partition first
sudo cryptsetup luksOpen /dev/sdb1 secret
sudo mount /dev/mapper/secret /mnt/secret

# Then unlock
shadow-secret unlock-global
```

### Scenario 4: Removable USB Drive

**Setup:**

1. Format USB drive with encryption (BitLocker, VeraCrypt, LUKS)
2. Mount to system
3. Move vault to drive
4. Update `vault_path` with mount point

**Benefits:**
- **Physical security**: Remove drive when not in use
- **Portability**: Move between systems
- **Air-gap**: Keep secrets isolated from network

---

## Command Reference

### unlock

**Purpose**: Unlock project-specific secrets

**Usage:**
```bash
shadow-secret unlock
```

**Loads:** `shadow-secret.yaml` from current directory

**Falls back to:** Nothing (use `unlock-global` for global secrets)

### unlock-global

**Purpose**: Unlock global secrets (from encrypted drive)

**Usage:**
```bash
shadow-secret unlock-global
```

**Loads:** `~/.config/shadow-secret/global.yaml`

**Supports:** `vault_path` field for encrypted drives

---

## Path Resolution Behavior

### v0.3.8 and Later

**Resolution Order:**
1. If `vault_path` is set → use it directly
2. If `vault_path` not set → use `source` field
3. Relative paths → resolved from config directory (not CWD)

**Example:**

Given `~/.config/shadow-secret/global.yaml`:
```yaml
vault:
  source: "global.enc.env"           # Not used (vault_path takes priority)
  vault_path: "E:/vault.enc.env"     # Used instead
```

Result: Loads from `E:/vault.enc.env`

### Before v0.3.8

**Old Behavior:**
- Relative paths resolved from current working directory (CWD)
- No `vault_path` field
- `unlock` command tried both project and global configs

**Migration:** Add `vault_path` field if you moved configs to encrypted drives

---

## Troubleshooting

### Error: "Failed to load vault from: [path]"

**Cause:** Vault file not found at specified path

**Solutions:**

1. **Check vault_path is correct:**
   ```bash
   cat ~/.config/shadow-secret/global.yaml
   ```

2. **Verify encrypted drive is mounted:**
   - Windows: Check `E:\` exists
   - macOS: Check `/Volumes/ShadowSecret` exists
   - Linux: Check `/mnt/encrypted` exists

3. **Test path accessibility:**
   ```bash
   # Windows
   dir E:\vault.enc.env

   # Unix
   ls -la /mnt/encrypted/vault.enc.env
   ```

### Error: "No Shadow Secret configuration found"

**Cause:** Global config file missing

**Solution:**
```bash
# Reinitialize global config
shadow-secret init-global
```

### Error: "SOPS_AGE_KEY_FILE not set"

**Cause:** Age key file path not configured

**Solution:**

Check `global.yaml` has `age_key_path`:
```yaml
vault:
  engine: "sops"
  age_key_path: "~/.config/shadow-secret/keys.txt"  # Should be set
```

### Error: "Permission denied" when accessing vault

**Cause:** Insufficient permissions on encrypted drive

**Solution:**

```bash
# Fix permissions (Unix)
sudo chmod 600 /mnt/encrypted/vault.enc.env

# Check drive mounting (Windows)
# Ensure drive is mounted with proper permissions
```

---

## Security Best Practices

### DO ✅

- **Use encrypted drives** for production secrets
- **Keep age keys separate** from vaults
- **Unmount drives** when not in use
- **Backup vaults** securely (encrypted backups)
- **Use strong passphrases** for encrypted drives

### DON'T ❌

- **Store vaults on unencrypted drives**
- **Commit vaults** to version control
- **Share vault paths** via insecure channels
- **Use weak encryption** for encrypted drives
- **Forget encryption passwords** (use password manager)

---

## Performance Considerations

### Encrypted Drive Access

**Read Speed:**
- VeraCrypt: ~100-200 MB/s (depends on cipher)
- FileVault: ~500+ MB/s (hardware accelerated)
- LUKS: ~200-300 MB/s (depends on cipher)

**Recommendation:** For large vaults (1000+ secrets), consider:
- Using faster encryption (AES-NI hardware acceleration)
- Keeping frequently accessed secrets on local drive
- Caching secrets in session (Shadow Secret already does this)

### Network Drives

**Not recommended** due to:
- Latency impact
- Reliability concerns
- Security risks (untrusted networks)

**If required:**
- Use VPN for encrypted transport
- Ensure stable network connection
- Implement timeout handling

---

## Advanced Configuration

### Multiple Vaults

You can have multiple vaults on different drives:

```yaml
# ~/.config/shadow-secret/project-a.yaml
vault:
  vault_path: "E:/project-a/vault.enc.env"
  engine: "sops"
```

```yaml
# ~/.config/shadow-secret/project-b.yaml
vault:
  vault_path: "F:/project-b/vault.enc.env"
  engine: "sops"
```

Usage:
```bash
# In project-a directory
shadow-secret unlock

# In project-b directory
shadow-secret unlock
```

### Conditional Vault Loading

```yaml
vault:
  source: "global.enc.env"
  # Override with environment variable
  vault_path: "${SHADOW_SECRET_VAULT_PATH}"
```

Set path at runtime:
```bash
export SHADOW_SECRET_VAULT_PATH="/mnt/backup/vault.enc.env"
shadow-secret unlock-global
```

---

## Migration Guide

### From v0.3.7 to v0.3.8

#### Scenario 1: Using `unlock` for global secrets

**Before (v0.3.7):**
```bash
shadow-secret unlock  # Tried both project and global
```

**After (v0.3.8):**
```bash
shadow-secret unlock-global  # Explicit global unlock
```

#### Scenario 2: Encrypted drive setup

**Before (v0.3.7):**
- No `vault_path` field
- Path resolution issues with encrypted drives

**After (v0.3.8):**
- Add `vault_path` to `global.yaml`
- Reliable encrypted drive support

**Steps:**

1. Update global config:
   ```bash
   nano ~/.config/shadow-secret/global.yaml
   ```

2. Add `vault_path`:
   ```yaml
   vault:
     source: "global.enc.env"
     vault_path: "/path/to/encrypted/drive/global.enc.env"
     engine: "sops"
     age_key_path: "~/.config/shadow-secret/keys.txt"
   ```

3. Move vault if needed:
   ```bash
   mv ~/.config/shadow-secret/global.enc.env /path/to/encrypted/drive/
   ```

4. Test:
   ```bash
   shadow-secret unlock-global
   ```

---

## Platform-Specific Notes

### Windows

**VeraCrypt Setup:**

1. Download: https://www.veracrypt.fr/
2. Create encrypted volume
3. Mount to drive letter (e.g., `E:\`)
4. Update `vault_path`: `"E:/vault.enc.env"`

**Path Format:**
- Use forward slashes: `"E:/path/to/vault.enc.env"`
- Or backslashes: `"E:\\path\\to\\vault.enc.env"`

### macOS

**FileVault Setup:**

1. System Preferences → Security & Privacy → FileVault
2. Create encrypted disk image (.dmg) if needed
3. Mount via Finder
4. Update `vault_path`: `"/Volumes/ShadowSecret/vault.enc.env"`

**Alternative:** Encrypted external drive (Disk Utility)

### Linux

**LUKS Setup:**

1. Create LUKS partition:
   ```bash
   sudo cryptsetup luksFormat /dev/sdb1
   sudo cryptsetup luksOpen /dev/sdb1 encrypted
   sudo mkfs.ext4 /dev/mapper/encrypted
   ```

2. Mount:
   ```bash
   sudo mount /dev/mapper/encrypted /mnt/secret
   ```

3. Update `vault_path`: `"/mnt/secret/vault.enc.env"`

**Automount:** Configure `/etc/fstab` for automatic mounting

---

## FAQ

### Q: Can I use both project and global vaults on encrypted drives?

**A:** Yes. Project vaults use `shadow-secret.yaml`, global uses `~/.config/shadow-secret/global.yaml`. Both can have `vault_path` pointing to different encrypted drives.

### Q: What happens if encrypted drive is not mounted?

**A:** Shadow Secret will show clear error: "Failed to load vault from: [path]". Mount the drive and retry.

### Q: Can vault_path point to a network share?

**A:** Yes, but not recommended due to latency and security concerns. Use VPN if required.

### Q: How do I backup my encrypted vault?

**A:** Copy the `.enc.env` file to another encrypted location. Example:
```bash
# Mount backup encrypted drive
cp /mnt/primary/vault.enc.env /mnt/backup/vault.enc.env
```

### Q: Can I use vault_path with relative paths?

**A:** No, `vault_path` should be absolute or start with `~`. Relative paths are only resolved for the `source` field (relative to config directory).

---

## References

- [VeraCrypt Documentation](https://www.veracrypt.fr/)
- [macOS FileVault](https://support.apple.com/guide/mac-help/secure-startup-mh35870/mac)
- [Linux LUKS](https://gitlab.com/cryptsetup/cryptsetup)
- [SOPS Documentation](https://github.com/getsops/sops)
- [Age Encryption](https://github.com/FiloSottile/age)

---

**Version:** 0.3.8+
**Last Updated:** 2026-02-16
