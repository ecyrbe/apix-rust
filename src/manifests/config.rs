use super::{ApixKind, ApixManifest, ApixManifestV1, ApixMetadata};
use anyhow::Result;
use indexmap::{indexmap, IndexMap};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::{fs, ops::DerefMut};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApixConfiguration {
  #[serde(flatten, default)]
  pub index: IndexMap<String, String>,
}

impl Default for ApixConfiguration {
  fn default() -> Self {
    let mut instance = Self { index: IndexMap::new() };
    instance.set_defaults();
    instance
  }
}

impl ApixConfiguration {
  pub fn once() -> &'static mut ApixConfiguration {
    static mut CONFIG: Lazy<ApixConfiguration> = Lazy::new(|| ApixConfiguration::load().unwrap());
    unsafe { CONFIG.deref_mut() }
  }

  // private function to create apix directory if it does not exist
  fn create_apix_dir_if_not_exists() -> Result<std::path::PathBuf> {
    let apix_dir = dirs::home_dir()
      .expect("Could not find HOME path, login as a user to use Apix")
      .join(".apix");
    fs::create_dir_all(&apix_dir)?;

    Ok(apix_dir)
  }

  // private function to load apix configuration from file when given a path
  fn load_from_path(path: &std::path::PathBuf) -> Result<Self> {
    if let Ok(content) = fs::read_to_string(path) {
      Self::load_from_string(&content, &format!("config file {:?}", &path))
    } else {
      Ok(Self::default())
    }
  }

  // private function to load apix configuration from string when given a content
  fn load_from_string(content: &str, err_msg: &str) -> Result<Self> {
    if !content.is_empty() {
      let manifest: ApixManifest =
        serde_yaml::from_str(&content).map_err(|e| anyhow::anyhow!("Could not parse {}: {:#}", &err_msg, e))?;
      return match manifest.kind() {
        ApixKind::Configuration(conf) => {
          let mut config = conf.clone();
          config.set_defaults();
          Ok(config)
        }
        _ => Err(anyhow::anyhow!("Invalid {}", &err_msg)),
      };
    }
    Ok(Self::default())
  }

  // private method to save apix configuration to file when given a path
  fn save_to_path(&self, path: &std::path::PathBuf) -> Result<()> {
    let manifest = ApixManifest::new_configuration(Some(self.clone()));
    let file = serde_yaml::to_string(&manifest)?;
    fs::write(path, file)?;
    Ok(())
  }

  // private method to set default values for apix configuration
  fn set_defaults(&mut self) {
    if let None = self.get("theme") {
      self.set("theme".to_string(), "Monokai Extended".to_string());
    }
  }

  // public function to load apix configuration from apix directory
  pub fn load() -> Result<Self> {
    let filename = Self::create_apix_dir_if_not_exists()?.join("config.yml");
    Self::load_from_path(&filename)
  }

  // public method to save apix configuration to apix directory
  pub fn save(&self) -> Result<()> {
    let filename = Self::create_apix_dir_if_not_exists()?.join("config.yml");
    self.save_to_path(&filename)
  }

  // public method to get apix configuration value by key
  pub fn get(&self, key: &str) -> Option<&String> {
    self.index.get(key)
  }

  // public method to set apix configuration value by key
  pub fn set(&mut self, key: String, value: String) -> Option<String> {
    self.index.insert(key, value)
  }

  // public method to remove apix configuration value by key
  pub fn delete(&mut self, key: &str) -> Option<String> {
    self.index.remove(key)
  }
}

impl ApixManifest {
  // There is only one configuration file per user, hence the name is hardcoded
  pub fn new_configuration(config: Option<ApixConfiguration>) -> Self {
    ApixManifest::V1(ApixManifestV1 {
      metadata: ApixMetadata {
        name: "configuration".to_string(),
        labels: indexmap! { "app".to_string() => "apix".to_string() },
        annotations: indexmap! {
            "apix.io/created-by".to_string() => whoami::username(),
            "apix.io/created-at".to_string() => chrono::Utc::now().to_rfc3339(),
        },
        extensions: IndexMap::new(),
      },
      kind: ApixKind::Configuration(config.unwrap_or_default()),
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  // static error message
  static ERROR_MSG: &str = "config string";

  // test apix has default config
  #[test]
  fn test_default_config() {
    let config = ApixConfiguration::default();
    assert_eq!(config.get("theme").unwrap(), "Monokai Extended");
  }
  // test ApixConfig default deserialize
  #[test]
  fn test_default_config_deserialize() {
    let config = r#"
      apiVersion: "apix.io/v1"
      kind: "Configuration"
      metadata:
        name: "configuration"
        labels:
          app: "apix"
      spec:
        rust: "rust"
    "#;
    let config = ApixConfiguration::load_from_string(config, ERROR_MSG).unwrap();
    assert_eq!(config.get("theme").unwrap(), "Monokai Extended");
    assert_eq!(config.get("rust").unwrap(), "rust");
  }
  // test ApixConfig deserialize
  #[test]
  fn test_config_deserialize() {
    let config = r#"
      apiVersion: "apix.io/v1"
      kind: "Configuration"
      metadata:
        name: "configuration"
        labels:
          app: "apix"
      spec:
        theme: "Coldark-Dark"
        rust: "rust"
    "#;
    let config = ApixConfiguration::load_from_string(config, ERROR_MSG).unwrap();
    assert_eq!(config.get("theme").unwrap(), "Coldark-Dark");
    assert_eq!(config.get("rust").unwrap(), "rust");
  }
}
