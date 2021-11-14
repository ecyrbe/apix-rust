use anyhow::Result;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ApixConfig {
  #[serde(default = "default_theme")]
  pub theme: String,
  #[serde(flatten, default)]
  pub extensions: IndexMap<String, String>,
}

fn default_theme() -> String {
  "Coldark-Dark".to_string()
}

impl Default for ApixConfig {
  fn default() -> Self {
    Self {
      theme: default_theme(),
      extensions: IndexMap::new(),
    }
  }
}

// read yaml config file from env::home_dir to ApixConfig
pub fn read_config() -> Result<ApixConfig> {
  //check if apix directory exists else create it
  let apix_dir = dirs::home_dir()
    .expect("Could not find HOME path, login as a user to use Apix")
    .join(".apix");
  let filename = apix_dir.join("config.yml");
  fs::create_dir_all(apix_dir)?;

  if let Ok(content) = fs::read_to_string(&filename) {
    if !content.is_empty() {
      let config: ApixConfig = serde_yaml::from_str(&content)
        .map_err(|e| anyhow::anyhow!("Could not parse config file {:?}: {}", &filename, e))?;
      return Ok(config);
    }
  }
  Ok(ApixConfig::default())
}
