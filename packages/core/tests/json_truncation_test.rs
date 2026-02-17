//! Test pour reproduire le bug de troncature JSON

use shadow_secret::injector::inject_secrets;
use std::collections::HashMap;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_json_no_truncation_large_file() {
    // Template très similaire à openclaw.json
    let template = r#"{
  "meta": {
    "lastTouchedVersion": "2026.2.14",
    "lastTouchedAt": "2026-02-16T23:21:57.718Z"
  },
  "wizard": {
    "lastRunAt": "2026-02-16T23:21:57.689Z",
    "lastRunVersion": "2026.2.14",
    "lastRunCommand": "doctor",
    "lastRunMode": "local"
  },
  "auth": {
    "profiles": {
      "zai:default": {
        "provider": "zai",
        "mode": "api_key"
      }
    }
  },
  "models": {
    "mode": "merge",
    "providers": {
      "zai": {
        "baseUrl": "https://api.z.ai/api/coding/paas/v4",
        "api": "openai-completions",
        "models": [
          {
            "id": "glm-4.7",
            "name": "GLM-4.7",
            "reasoning": true,
            "input": ["text"],
            "cost": {
              "input": 0,
              "output": 0,
              "cacheRead": 0,
              "cacheWrite": 0
            },
            "contextWindow": 204800,
            "maxTokens": 131072
          }
        ]
      }
    }
  },
  "agents": {
    "defaults": {
      "auth": {"persistence": "none"},
      "model": {
        "primary": "zai/glm-4.7",
        "fallbacks": ["zai/glm-4.7"]
      },
      "models": {
        "zai/glm-4.7": {"alias": "GLM"}
      },
      "workspace": "C:\\Users\\Yanis\\clawd",
      "compaction": {"mode": "safeguard"},
      "elevatedDefault": "off",
      "maxConcurrent": 8,
      "subagents": {
        "maxConcurrent": 16,
        "archiveAfterMinutes": 30,
        "model": "zai/glm-4.7"
      }
    }
  },
  "tools": {
    "profile": "full",
    "deny": [],
    "web": {
      "search": {
        "enabled": true,
        "apiKey": "$WEB_API_KEY"
      },
      "fetch": {"enabled": true}
    },
    "elevated": {
      "enabled": false,
      "allowFrom": {}
    },
    "exec": {
      "security": "allowlist",
      "ask": "always",
      "safeBins": ["git", "node", "npm", "pnpm", "yarn", "bun", "dir", "Get-ChildItem", "Set-Location", "Get-Location", "Get-Content", "Write-Output", "Select-String", "Where-Object", "grepai", "New-Item", "Remove-Item", "Copy-Item", "Move-Item", "python", "python3", "pip", "pip3", "curl", "wget", "docker", "docker-compose", "npx", "tsc", "vite"]
    }
  },
  "messages": {
    "ackReactionScope": "group-mentions"
  },
  "commands": {
    "native": "auto",
    "nativeSkills": "auto",
    "bash": false,
    "config": false,
    "debug": false,
    "restart": false
  },
  "hooks": {
    "enabled": true,
    "token": "$HOOK_TOKEN",
    "internal": {
      "enabled": true,
      "entries": {
        "boot-md": {"enabled": true},
        "command-logger": {"enabled": false},
        "session-memory": {"enabled": true}
      }
    }
  },
  "channels": {
    "discord": {
      "enabled": true,
      "token": "$DISCORD_TOKEN",
      "allowBots": true,
      "groupPolicy": "open",
      "dmPolicy": "pairing",
      "allowFrom": ["*"]
    }
  },
  "gateway": {
    "port": 18789,
    "mode": "local",
    "bind": "loopback",
    "auth": {
      "mode": "token",
      "token": "$GATEWAY_TOKEN"
    },
    "tailscale": {
      "mode": "off",
      "resetOnExit": false
    }
  },
  "skills": {
    "install": {"nodeManager": "pnpm"}
  },
  "plugins": {
    "entries": {
      "discord": {"enabled": true}
    }
  }
}"#;

    // Vérifier la taille du template
    println!("Template size: {} bytes", template.len());

    let mut secrets = HashMap::new();
    secrets.insert("WEB_API_KEY".to_string(), "test-key-123".to_string());
    secrets.insert("DISCORD_TOKEN".to_string(), "discord-token-123".to_string());
    secrets.insert("HOOK_TOKEN".to_string(), "hook-token-123".to_string());
    secrets.insert("GATEWAY_TOKEN".to_string(), "gateway-token-123".to_string());

    let placeholders = vec![
        "$WEB_API_KEY".to_string(),
        "$DISCORD_TOKEN".to_string(),
        "$HOOK_TOKEN".to_string(),
        "$GATEWAY_TOKEN".to_string(),
    ];

    // Créer un fichier temporaire
    let mut temp_file = NamedTempFile::with_suffix(".json").unwrap();
    temp_file.write_all(template.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    // Vérifier la taille écrite
    let written_size = std::fs::metadata(temp_file.path()).unwrap().len();
    println!("Written file size: {} bytes", written_size);

    // Effectuer l'injection
    let backup = inject_secrets(temp_file.path(), &secrets, &placeholders).unwrap();

    // Vérifier la taille après injection
    let injected_size = std::fs::metadata(temp_file.path()).unwrap().len();
    println!("Injected file size: {} bytes", injected_size);

    // Lire le résultat
    let result = std::fs::read_to_string(temp_file.path()).unwrap();

    println!("Result size: {} bytes", result.len());
    println!("Backup size: {} bytes", backup.content().len());

    // Compter les accolades fermantes pour vérifier l'intégrité
    let open_braces = result.matches('{').count();
    let close_braces = result.matches('}').count();
    println!("Open braces: {}, Close braces: {}", open_braces, close_braces);

    // Vérifier que le JSON est valide
    let parsed: serde_json::Value = serde_json::from_str(&result)
        .expect("Le JSON injecté n'est pas valide !");

    // Vérifier que toutes les clés racine sont présentes
    let obj = parsed.as_object().expect("Root should be an object");
    let expected_keys = vec![
        "meta", "wizard", "auth", "models", "agents", "tools",
        "messages", "commands", "hooks", "channels", "gateway",
        "skills", "plugins"
    ];

    for key in &expected_keys {
        assert!(obj.contains_key(*key), "La clé '{}' est manquante dans le JSON injecté !", key);
    }

    // Vérifier les secrets injectés
    assert!(result.contains("test-key-123"));
    assert!(result.contains("discord-token-123"));
    assert!(result.contains("hook-token-123"));
    assert!(result.contains("gateway-token-123"));

    println!("✅ Test passed - No truncation detected!");
}
