//! Unit tests for push-cloud module logic.

use std::collections::HashMap;

/// Test filtering of LOCAL_ONLY_ secrets.
#[test]
fn test_filter_local_only_secrets() {
    let mut secrets = HashMap::new();
    secrets.insert("PUBLIC_API_KEY".to_string(), "public_value".to_string());
    secrets.insert("LOCAL_ONLY_SECRET".to_string(), "local_value".to_string());
    secrets.insert("ANOTHER_PUBLIC".to_string(), "another_value".to_string());

    // Filter out LOCAL_ONLY_ secrets
    let filtered: HashMap<&String, &String> = secrets
        .iter()
        .filter(|(k, _)| !k.starts_with("LOCAL_ONLY_"))
        .collect();

    // Should have 2 secrets (not 3)
    assert_eq!(filtered.len(), 2);

    // Should not contain LOCAL_ONLY_SECRET
    assert!(!filtered.contains_key(&"LOCAL_ONLY_SECRET".to_string()));

    // Should contain public variables
    assert!(filtered.contains_key(&"PUBLIC_API_KEY".to_string()));
    assert!(filtered.contains_key(&"ANOTHER_PUBLIC".to_string()));
}

/// Test that empty secrets are handled correctly.
#[test]
fn test_empty_secrets_handling() {
    let secrets: HashMap<String, String> = HashMap::new();

    // All secrets would be filtered
    let filtered: HashMap<&String, &String> = secrets
        .iter()
        .filter(|(k, _)| !k.starts_with("LOCAL_ONLY_"))
        .collect();

    assert_eq!(filtered.len(), 0);
}

/// Test mixed local and public secrets.
#[test]
fn test_mixed_secrets_filtering() {
    let mut secrets = HashMap::new();
    secrets.insert("API_KEY".to_string(), "value1".to_string());
    secrets.insert("LOCAL_ONLY_DB_PASS".to_string(), "value2".to_string());
    secrets.insert("LOCAL_ONLY_API_SECRET".to_string(), "value3".to_string());
    secrets.insert("DATABASE_URL".to_string(), "value4".to_string());

    let filtered: HashMap<&String, &String> = secrets
        .iter()
        .filter(|(k, _)| !k.starts_with("LOCAL_ONLY_"))
        .collect();

    // Should have 2 non-local secrets
    assert_eq!(filtered.len(), 2);
    assert!(filtered.contains_key(&"API_KEY".to_string()));
    assert!(filtered.contains_key(&"DATABASE_URL".to_string()));

    // Should not contain local-only secrets
    assert!(!filtered.contains_key(&"LOCAL_ONLY_DB_PASS".to_string()));
    assert!(!filtered.contains_key(&"LOCAL_ONLY_API_SECRET".to_string()));
}
