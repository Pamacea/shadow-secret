//! Integration tests for Vault module.
//!
//! These tests use mock SOPS output to verify the Vault functionality
//! without requiring actual SOPS installation or encrypted files.

mod common;

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_parse_env_from_mock_output() {
        let env_output = b"API_KEY=sk_test_12345\nDATABASE_URL=postgres://localhost:5432/test\n";
        let secrets = shadow_secret::vault::parse_env_for_testing(env_output);

        assert!(secrets.is_ok());
        let secrets = secrets.unwrap();
        assert_eq!(secrets.get("API_KEY"), Some(&"sk_test_12345".to_string()));
        assert_eq!(
            secrets.get("DATABASE_URL"),
            Some(&"postgres://localhost:5432/test".to_string())
        );
    }

    #[test]
    fn test_parse_json_from_mock_output() {
        let json_output =
            br#"{"API_KEY":"sk_test_12345","DATABASE_URL":"postgres://localhost:5432/test"}"#;
        let secrets = shadow_secret::vault::parse_json_for_testing(json_output);

        assert!(secrets.is_ok());
        let secrets = secrets.unwrap();
        assert_eq!(secrets.get("API_KEY"), Some(&"sk_test_12345".to_string()));
        assert_eq!(
            secrets.get("DATABASE_URL"),
            Some(&"postgres://localhost:5432/test".to_string())
        );
    }

    #[test]
    fn test_parse_yaml_from_mock_output() {
        let yaml_output = b"API_KEY: sk_test_12345\nDATABASE_URL: postgres://localhost:5432/test\n";
        let secrets = shadow_secret::vault::parse_yaml_for_testing(yaml_output);

        assert!(secrets.is_ok());
        let secrets = secrets.unwrap();
        assert_eq!(secrets.get("API_KEY"), Some(&"sk_test_12345".to_string()));
        assert_eq!(
            secrets.get("DATABASE_URL"),
            Some(&"postgres://localhost:5432/test".to_string())
        );
    }

    #[test]
    fn test_vault_get_secret() {
        use std::collections::HashMap;

        let mut secrets = HashMap::new();
        secrets.insert("API_KEY".to_string(), "sk_test_12345".to_string());
        secrets.insert(
            "DATABASE_URL".to_string(),
            "postgres://localhost".to_string(),
        );

        let vault = shadow_secret::vault::Vault::new(secrets);

        assert_eq!(vault.get("API_KEY"), Some(&"sk_test_12345".to_string()));
        assert_eq!(vault.get("NON_EXISTENT"), None);
    }

    #[test]
    fn test_vault_all_secrets() {
        use std::collections::HashMap;

        let mut secrets = HashMap::new();
        secrets.insert("API_KEY".to_string(), "sk_test_12345".to_string());
        secrets.insert(
            "DATABASE_URL".to_string(),
            "postgres://localhost".to_string(),
        );

        let vault = shadow_secret::vault::Vault::new(secrets);
        let all = vault.all();

        assert_eq!(all.len(), 2);
        assert!(all.contains_key("API_KEY"));
        assert!(all.contains_key("DATABASE_URL"));
    }
}
