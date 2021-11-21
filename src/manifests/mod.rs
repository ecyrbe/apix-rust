pub use self::config::ApixConfiguration;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ApixApi {
  pub url: String,
  pub version: String,
  pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct ApixParameters {
  name: String,
  required: bool,
  description: Option<String>,
  schema: Option<serde_json::Value>, //todo use structured schema
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct ApixTemplate {
  method: String,
  url: String,
  headers: IndexMap<String, String>,
  body: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApixRequest {
  parameters: Vec<ApixParameters>,
  template: ApixTemplate,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", content = "spec")]
pub enum ApixKind {
  Api(ApixApi),
  Configuration(ApixConfiguration),
  Request(ApixRequest),
  None,
}

impl Default for ApixKind {
  fn default() -> Self {
    ApixKind::None
  }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ApixMetadata {
  name: String,
  #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
  labels: IndexMap<String, String>,
  #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
  annotations: IndexMap<String, String>,
  #[serde(flatten)]
  more: IndexMap<String, String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ApixManifest {
  api_version: String,
  metadata: ApixMetadata,
  #[serde(flatten)]
  kind: ApixKind,
}

impl ApixManifest {
  pub fn new() -> Self {
    ApixManifest {
      api_version: "apix.io/v1".to_string(),
      metadata: Default::default(),
      kind: Default::default(),
    }
  }

  pub fn new_api(name: String, api: Option<ApixApi>) -> Self {
    let mut manifest = ApixManifest::new();
    manifest.kind = ApixKind::Api(api.unwrap_or_default());
    manifest.metadata.name = name;
    manifest
      .metadata
      .labels
      .insert("app".to_string(), "apix".to_string());
    manifest
  }

  #[allow(dead_code)]
  pub fn version(&self) -> &str {
    &self.api_version
  }

  pub fn kind(&self) -> &ApixKind {
    &self.kind
  }
}

pub mod config;
