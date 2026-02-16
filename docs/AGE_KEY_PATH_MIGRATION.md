# Migration Guide: Adding age_key_path to global.yaml

> **Version:** 0.3.8+
> **Issue:** Missing `age_key_path` field in global configuration

---

## 🐛 Problem

If you're seeing an error like:

```
Error: Failed to load global config
```

And your `~/.config/shadow-secret/global.yaml` looks like:

```yaml
vault:
  source: "V:/global.enc.env"
  engine: "sops"
  require_mount: true

targets:
  - name: "Openclaw"
    path: "C:/Users/Yanis/.openclaw/openclaw.json"
    placeholders:
      - "$ALL"
```

**You're missing the `age_key_path` field!**

---

## ✅ Solution

Add the `age_key_path` field to your `vault` section:

```yaml
vault:
  source: "V:/global.enc.env"
  engine: "sops"
  require_mount: true

  # ⬇️ ADD THIS LINE ⬇️
  age_key_path: "V:/-recovery/keys.txt"

targets:
  - name: "Openclaw"
    path: "C:/Users/Yanis/.openclaw/openclaw.json"
    placeholders:
      - "$ALL"
```

### What is age_key_path?

This field tells Shadow Secret where to find your AGE encryption key.

**Why it's better than $SOPS_AGE_KEY_FILE:**
- ✅ Stored in config (no need to set env var every time)
- ✅ Works with encrypted drives (like V:/)
- ✅ Per-config flexibility (different keys for different vaults)

---

## 🔧 Complete Example

Here's a complete `~/.config/shadow-secret/global.yaml`:

```yaml
vault:
  # Path to encrypted secrets file
  source: "V:/global.enc.env"

  # Optional: Explicit vault path (useful for encrypted drives)
  # vault_path: "V:/global.enc.env"  # Uncomment if needed

  # Encryption engine (sops with age)
  engine: "sops"

  # Path to age private key for SOPS encryption/decryption
  age_key_path: "V:/-recovery/keys.txt"

  # Whether to require vault mount (for VeraCrypt volumes)
  require_mount: true

# Example targets - modify as needed
targets:
  - name: "Openclaw"
    path: "C:/Users/Yanis/.openclaw/openclaw.json"
    placeholders:
      - "$ALL"

  - name: "Openclaw MCP"
    path: "C:/Users/Yanis/.openclaw/.mcp.json"
    placeholders:
      - "$ALL"

  - name: "Claude"
    path: "C:/Users/Yanis/.claude/settings.json"
    placeholders:
      - "$ALL"
```

---

## 🎯 Quick Fix (One-Liner)

If your `global.yaml` is at `~/.config/shadow-secret/global.yaml`:

**Windows (PowerShell):**
```powershell
Add-Content -Path $env:USERPROFILE\.config\shadow-secret\global.yaml -Value "`n  age_key_path: `"V:/-recovery/keys.txt"`""
```

**Or edit manually:**
```bash
notepad ~/.config/shadow-secret/global.yaml
```

Then add the line under `vault:` section.

---

## ✅ Verify

After updating, verify your config:

```bash
# Should now work!
shadow-secret unlock-global
```

Or run diagnostics:

```bash
shadow-secret doctor
```

---

**Need help?** See [docs/TROUBLESHOOTING.md](TROUBLESHOOTING.md)
