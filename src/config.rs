use anyhow::Result;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApixConfig {
  #[serde(flatten, default)]
  index: IndexMap<String, String>,
}

impl Default for ApixConfig {
  fn default() -> Self {
    let mut instance = Self {
      index: IndexMap::new(),
    };
    instance.set_defaults();
    instance
  }
}

// impl ApixConfig
impl ApixConfig {
  //private function to create apix directory if it does not exist
  fn create_apix_dir_if_not_exists() -> Result<std::path::PathBuf> {
    let apix_dir = dirs::home_dir()
      .expect("Could not find HOME path, login as a user to use Apix")
      .join(".apix");
    fs::create_dir_all(&apix_dir)?;

    Ok(apix_dir)
  }

  fn load(path: &std::path::PathBuf) -> Result<Self> {
    if let Ok(file) = fs::read_to_string(path) {
      if !file.is_empty() {
        let mut config: Self = serde_yaml::from_str(&file)
          .map_err(|e| anyhow::anyhow!("Could not parse config file {:?}: {}", &path, e))?;
        config.set_defaults();
        return Ok(config);
      }
    }
    Ok(Self::default())
  }

  #[allow(dead_code)]
  pub fn load_from_string(content: &str) -> Result<Self> {
    if !content.is_empty() {
      let mut config: Self = serde_yaml::from_str(content)
        .map_err(|e| anyhow::anyhow!("Could not parse config from string: {}", e))?;
      config.set_defaults();
      return Ok(config);
    }
    Ok(Self::default())
  }

  fn save(&self, path: &std::path::PathBuf) -> Result<()> {
    let file = serde_yaml::to_string(self)?;
    fs::write(path, file)?;
    Ok(())
  }

  fn set_defaults(&mut self) {
    if let None = self.get("theme") {
      self.set("theme".to_string(), "Coldark-Dark".to_string());
    }
  }

  pub fn read_config() -> Result<Self> {
    let filename = Self::create_apix_dir_if_not_exists()?.join("config.yml");
    Self::load(&filename)
  }
  pub fn save_config(&self) -> Result<()> {
    let filename = Self::create_apix_dir_if_not_exists()?.join("config.yml");
    self.save(&filename)
  }

  pub fn get(&self, key: &str) -> Option<&String> {
    self.index.get(key)
  }

  pub fn set(&mut self, key: String, value: String) -> Option<String> {
    self.index.insert(key, value)
  }

  pub fn delete(&mut self, key: &str) -> Option<String> {
    self.index.remove(key)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  // test apix has default config
  #[test]
  fn test_default_config() {
    let config = ApixConfig::default();
    assert_eq!(config.get("theme").unwrap(), "Coldark-Dark");
  }
  // test ApixConfig default deserialize
  #[test]
  fn test_default_config_deserialize() {
    let config = r#"
      rust: "rust"
    "#;
    let config = ApixConfig::load_from_string(config).unwrap();
    assert_eq!(config.get("theme").unwrap(), "Coldark-Dark");
    assert_eq!(config.get("rust").unwrap(), "rust");
  }
  // test ApixConfig deserialize
  #[test]
  fn test_config_deserialize() {
    let config = r#"
      theme: "Coldark-Dark"
      rust: "rust"
    "#;
    let config = ApixConfig::load_from_string(config).unwrap();
    assert_eq!(config.get("theme").unwrap(), "Coldark-Dark");
    assert_eq!(config.get("rust").unwrap(), "rust");
  }
}
