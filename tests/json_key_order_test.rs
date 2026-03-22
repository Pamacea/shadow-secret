//! Test pour vérifier que l'ordre des clés JSON est préservé après injection

use shadow_secret::injector::inject_secrets;
use std::collections::HashMap;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_json_key_order_preserved() {
    // Template avec un ordre de clés spécifique (non alphabétique)
    let template = r#"{
  "meta": {
    "lastTouchedVersion": "2026.2.14"
  },
  "wizard": {
    "lastRunCommand": "doctor"
  },
  "auth": {
    "profiles": {}
  },
  "models": {
    "mode": "merge"
  },
  "agents": {
    "defaults": {}
  },
  "tools": {
    "profile": "full"
  },
  "messages": {},
  "commands": {},
  "hooks": {
    "enabled": true
  },
  "channels": {},
  "gateway": {
    "port": 18789
  },
  "skills": {},
  "plugins": {}
}"#;

    let mut secrets = HashMap::new();
    secrets.insert("PLACEHOLDER".to_string(), "replaced".to_string());
    let placeholders = vec!["$PLACEHOLDER".to_string()];

    // Créer un fichier temporaire
    let mut temp_file = NamedTempFile::with_suffix(".json").unwrap();
    temp_file.write_all(template.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    // Effectuer l'injection
    let backup = inject_secrets(temp_file.path(), &secrets, &placeholders).unwrap();

    // Lire le résultat
    let result = std::fs::read_to_string(temp_file.path()).unwrap();

    // L'ordre des clés doit être préservé : meta, wizard, auth, models, agents, tools, messages, commands, hooks, channels, gateway, skills, plugins
    let key_order = vec![
        "meta", "wizard", "auth", "models", "agents", "tools",
        "messages", "commands", "hooks", "channels", "gateway",
        "skills", "plugins"
    ];

    let mut last_pos = 0;
    for key in &key_order {
        let pos = result.find(&format!("\"{}\"", key))
            .expect(&format!("Clé '{}' non trouvée dans le JSON", key));
        assert!(pos > last_pos,
            "L'ordre des clés n'est pas préservé : '{}' devrait être après la position {} mais est à {}",
            key, last_pos, pos);
        last_pos = pos;
    }

    // Vérifier que le JSON est toujours valide
    let parsed: serde_json::Value = serde_json::from_str(&result)
        .expect("Le JSON n'est pas valide après injection");

    assert_eq!(parsed["meta"]["lastTouchedVersion"], "2026.2.14");
    assert_eq!(parsed["wizard"]["lastRunCommand"], "doctor");
    assert_eq!(parsed["gateway"]["port"], 18789);

    println!("✅ L'ordre des clés JSON est préservé !");
}
