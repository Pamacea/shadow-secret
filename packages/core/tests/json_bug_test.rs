//! Test pour reproduire le bug de perte de données JSON

use shadow_secret::injector::inject_secrets;
use std::collections::HashMap;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_json_data_loss_bug() {
    // Template original qui pose problème
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
            "input": [
              "text"
            ],
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
      "auth": {
            "persistence": "none"
          },
      "model": {
        "primary": "zai/glm-4.7",
        "fallbacks": [
          "zai/glm-4.7"
        ]
      },
      "models": {
        "zai/glm-4.7": {
          "alias": "GLM"
        }
      },
      "workspace": "C:\\Users\\Yanis\\clawd",
      "compaction": {
        "mode": "safeguard"
      },
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
      "fetch": {
        "enabled": true
      }
    },
    "elevated": {
      "enabled": false,
      "allowFrom": {}
    },
    "exec": {
      "security": "allowlist",
      "ask": "always",
      "safeBins": [
        "git",
        "node",
        "npm",
        "pnpm",
        "yarn",
        "bun",
        "dir",
        "Get-ChildItem",
        "Set-Location",
        "Get-Location",
        "Get-Content",
        "Write-Output",
        "Select-String",
        "Where-Object",
        "grepai",
        "New-Item",
        "Remove-Item",
        "Copy-Item",
        "Move-Item",
        "python",
        "python3",
        "pip",
        "pip3",
        "curl",
        "wget",
        "docker",
        "docker-compose",
        "npx",
        "tsc",
        "vite"
      ]
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
        "boot-md": {
          "enabled": true
        },
        "command-logger": {
          "enabled": false
        },
        "session-memory": {
          "enabled": true
        }
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
      "allowFrom": [
        "*"
      ]
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
    "install": {
      "nodeManager": "pnpm"
    }
  },
  "plugins": {
    "entries": {
      "discord": {
        "enabled": true
      }
    }
  }
}"#;

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

    // Effectuer l'injection
    let _backup = inject_secrets(temp_file.path(), &secrets, &placeholders).unwrap();

    // Lire le résultat
    let result = std::fs::read_to_string(temp_file.path()).unwrap();

    println!("=== RÉSULTAT ===");
    println!("{}", result);
    println!("================");

    // Ces clés DOIVENT être présentes
    assert!(result.contains("\"meta\""), "La clé 'meta' est manquante!");
    assert!(result.contains("\"wizard\""), "La clé 'wizard' est manquante!");
    assert!(result.contains("\"auth\""), "La clé 'auth' est manquante!");
    assert!(result.contains("\"models\""), "La clé 'models' est manquante!");
    assert!(result.contains("\"agents\""), "La clé 'agents' est manquante!");
    assert!(result.contains("\"tools\""), "La clé 'tools' est manquante!");
    assert!(result.contains("\"messages\""), "La clé 'messages' est manquante!");
    assert!(result.contains("\"commands\""), "La clé 'commands' est manquante!");
    assert!(result.contains("\"hooks\""), "La clé 'hooks' est manquante!");
    assert!(result.contains("\"channels\""), "La clé 'channels' est manquante!");
    assert!(result.contains("\"gateway\""), "La clé 'gateway' est manquante!");
    assert!(result.contains("\"skills\""), "La clé 'skills' est manquante!");
    assert!(result.contains("\"plugins\""), "La clé 'plugins' est manquante!");

    // Vérifier que les secrets ont été injectés
    assert!(result.contains("test-key-123"), "WEB_API_KEY n'a pas été injecté!");
    assert!(result.contains("discord-token-123"), "DISCORD_TOKEN n'a pas été injecté!");
    assert!(result.contains("hook-token-123"), "HOOK_TOKEN n'a pas été injecté!");
    assert!(result.contains("gateway-token-123"), "GATEWAY_TOKEN n'a pas été injecté!");

    // Vérifier que les placeholders n'existent plus
    assert!(!result.contains("$WEB_API_KEY"), "Placeholder $WEB_API_KEY toujours présent!");
    assert!(!result.contains("$DISCORD_TOKEN"), "Placeholder $DISCORD_TOKEN toujours présent!");
    assert!(!result.contains("$HOOK_TOKEN"), "Placeholder $HOOK_TOKEN toujours présent!");
    assert!(!result.contains("$GATEWAY_TOKEN"), "Placeholder $GATEWAY_TOKEN toujours présent!");
}
