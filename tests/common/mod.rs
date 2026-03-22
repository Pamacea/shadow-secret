//! Common testing utilities for Shadow Secret integration tests.

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tempfile::TempDir;

/// Test context that manages temporary files and directories.
pub struct TestContext {
    /// Path to temporary directory
    pub temp_path: PathBuf,
    /// The temporary directory (kept to prevent early deletion)
    _temp_dir: TempDir,
}

impl TestContext {
    /// Create a new test context with a temporary directory.
    pub fn new() -> anyhow::Result<Self> {
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path().to_path_buf();

        Ok(Self {
            temp_path,
            _temp_dir: temp_dir,
        })
    }

    /// Create a test file with content.
    pub fn create_file(&self, name: &str, content: &str) -> anyhow::Result<PathBuf> {
        let file_path = self.temp_path.join(name);
        let mut file = fs::File::create(&file_path)?;
        file.write_all(content.as_bytes())?;
        Ok(file_path)
    }

    /// Create a mock SOPS-encrypted ENV file.
    #[allow(dead_code)]
    pub fn create_sops_env(
        &self,
        name: &str,
        secrets: &[(impl AsRef<str>, impl AsRef<str>)],
    ) -> anyhow::Result<PathBuf> {
        let content = secrets
            .iter()
            .map(|(k, v)| format!("{}={}", k.as_ref(), v.as_ref()))
            .collect::<Vec<_>>()
            .join("\n");

        let file_path = self.temp_path.join(name);
        let mut file = fs::File::create(&file_path)?;
        file.write_all(content.as_bytes())?;
        Ok(file_path)
    }

    /// Create a mock SOPS-encrypted JSON file.
    #[allow(dead_code)]
    pub fn create_sops_json(
        &self,
        name: &str,
        secrets: &[(impl AsRef<str>, impl AsRef<str>)],
    ) -> anyhow::Result<PathBuf> {
        let mut json_data = serde_json::Map::new();
        for (key, value) in secrets {
            json_data.insert(key.as_ref().to_string(), serde_json::json!(value.as_ref()));
        }

        let content = serde_json::to_string_pretty(&json_data)?;
        let file_path = self.temp_path.join(name);
        let mut file = fs::File::create(&file_path)?;
        file.write_all(content.as_bytes())?;
        Ok(file_path)
    }

    /// Create a mock SOPS-encrypted YAML file.
    #[allow(dead_code)]
    pub fn create_sops_yaml(
        &self,
        name: &str,
        secrets: &[(impl AsRef<str>, impl AsRef<str>)],
    ) -> anyhow::Result<PathBuf> {
        let mut yaml_map = serde_yaml::Mapping::new();
        for (key, value) in secrets {
            yaml_map.insert(
                serde_yaml::Value::String(key.as_ref().to_string()),
                serde_yaml::Value::String(value.as_ref().to_string()),
            );
        }

        let content = serde_yaml::to_string(&yaml_map);
        let file_path = self.temp_path.join(name);
        let mut file = fs::File::create(&file_path)?;
        file.write_all(content.unwrap().as_bytes())?;
        Ok(file_path)
    }

    /// Get the path to a file in the temp directory.
    #[allow(dead_code)]
    pub fn path(&self, name: &str) -> PathBuf {
        self.temp_path.join(name)
    }
}

/// Mock SOPS command outputs for testing.
pub struct MockSops;

impl MockSops {
    /// Create mock ENV output (simulating `sops -d file.env`).
    pub fn env_output(secrets: &[(impl AsRef<str>, impl AsRef<str>)]) -> Vec<u8> {
        secrets
            .iter()
            .map(|(k, v)| format!("{}={}", k.as_ref(), v.as_ref()))
            .collect::<Vec<_>>()
            .join("\n")
            .into_bytes()
    }

    /// Create mock JSON output (simulating `sops -d file.json`).
    pub fn json_output(secrets: &[(impl AsRef<str>, impl AsRef<str>)]) -> Vec<u8> {
        let mut json_data = serde_json::Map::new();
        for (key, value) in secrets {
            json_data.insert(key.as_ref().to_string(), serde_json::json!(value.as_ref()));
        }

        serde_json::to_vec(&json_data).unwrap()
    }

    /// Create mock YAML output (simulating `sops -d file.yaml`).
    pub fn yaml_output(secrets: &[(impl AsRef<str>, impl AsRef<str>)]) -> Vec<u8> {
        let mut yaml_map = serde_yaml::Mapping::new();
        for (key, value) in secrets {
            yaml_map.insert(
                serde_yaml::Value::String(key.as_ref().to_string()),
                serde_yaml::Value::String(value.as_ref().to_string()),
            );
        }

        serde_yaml::to_string(&yaml_map).unwrap().into_bytes()
    }

    /// Create mock SOPS JSON output with SOPS metadata.
    #[allow(dead_code)]
    pub fn sops_json_output(secrets: &[(impl AsRef<str>, impl AsRef<str>)]) -> Vec<u8> {
        let mut data = serde_json::Map::new();
        for (key, value) in secrets {
            data.insert(key.as_ref().to_string(), serde_json::json!(value.as_ref()));
        }

        let mut sops_output = serde_json::Map::new();
        sops_output.insert("data".to_string(), serde_json::Value::Object(data));
        sops_output.insert(
            "sops".to_string(),
            serde_json::json!({
                "kms": [],
                "gcp_kms": [],
                "azure_kv": [],
                "hc_vault": [],
                "lastmodified": "2024-01-01T00:00:00Z",
                "mac": "ENC[AES256_GCM,data:...,tag:...,type:str]",
                "pgp": [],
                "unencrypted_suffix": "_unencrypted",
                "version": "3.7.3"
            }),
        );

        serde_json::to_vec(&sops_output).unwrap()
    }

    /// Create mock SOPS YAML output with SOPS metadata.
    #[allow(dead_code)]
    pub fn sops_yaml_output(secrets: &[(impl AsRef<str>, impl AsRef<str>)]) -> Vec<u8> {
        let mut data = serde_yaml::Mapping::new();
        for (key, value) in secrets {
            data.insert(
                serde_yaml::Value::String(key.as_ref().to_string()),
                serde_yaml::Value::String(value.as_ref().to_string()),
            );
        }

        let mut sops_output = serde_yaml::Mapping::new();
        sops_output.insert(
            serde_yaml::Value::String("data".to_string()),
            serde_yaml::Value::Mapping(data),
        );
        sops_output.insert(
            serde_yaml::Value::String("sops".to_string()),
            serde_yaml::Value::Mapping(serde_yaml::Mapping::new()),
        );

        serde_yaml::to_string(&sops_output).unwrap().into_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_sops_env_output() {
        let output = MockSops::env_output(&[("KEY1", "value1"), ("KEY2", "value2")]);
        let content = String::from_utf8(output).unwrap();

        assert!(content.contains("KEY1=value1"));
        assert!(content.contains("KEY2=value2"));
    }

    #[test]
    fn test_mock_sops_json_output() {
        let output = MockSops::json_output(&[("KEY1", "value1"), ("KEY2", "value2")]);
        let json: serde_json::Value = serde_json::from_slice(&output).unwrap();

        assert_eq!(json["KEY1"], "value1");
        assert_eq!(json["KEY2"], "value2");
    }

    #[test]
    fn test_mock_sops_yaml_output() {
        let output = MockSops::yaml_output(&[("KEY1", "value1"), ("KEY2", "value2")]);
        let yaml: serde_yaml::Value = serde_yaml::from_slice(&output).unwrap();

        assert_eq!(yaml["KEY1"], "value1");
        assert_eq!(yaml["KEY2"], "value2");
    }

    #[test]
    fn test_test_context_create_file() {
        let ctx = TestContext::new().unwrap();
        let file_path = ctx.create_file("test.txt", "Hello, World!").unwrap();

        assert!(file_path.exists());
        assert_eq!(fs::read_to_string(&file_path).unwrap(), "Hello, World!");
    }
}
