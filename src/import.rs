use anyhow::Result;
use indexmap::IndexMap;
use openapiv3::{OpenAPI, ReferenceOr};
use serde::{Deserialize, Serialize};
use std::fs;
// use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

pub enum OpenApiType {
  JSON,
  YAML,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApixApi {
  url: String,
  version: String,
  description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApixConfiguration {
  apis: Vec<String>,
  context: Option<String>,
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

pub async fn import_api(api_description: String, api_type: OpenApiType) -> Result<()> {
  let api: OpenAPI = load_api(api_description, api_type)?;
  let mut manifest: ApixManifest = Default::default();
  manifest.api_version = "apix/v1".to_owned();
  let name = api.info.title.replace(" ", "-");
  manifest.metadata.name = name.clone();
  manifest
    .metadata
    .labels
    .insert("app".to_owned(), "apix".to_owned());
  manifest.kind = ApixKind::Api(ApixApi {
    url: api.servers[0].url.clone(),
    description: api.info.description.or(Some(api.info.title)),
    version: api.info.version,
  });
  let dirname = std::env::current_dir()?.join(&name);
  let filename = dirname.join(format!("api.{}.yaml", &name));
  fs::create_dir_all(dirname)?;

  let mut file = File::create(filename).await?;
  let content = serde_yaml::to_string(&manifest)?;
  file.write_all(content.as_bytes()).await?;
  file.flush().await?;

  for (path, item) in api.paths.paths {
    match item {
      ReferenceOr::Item(path_item) => {
        path_item.get.as_ref();
        ()
      }
      ReferenceOr::Reference { reference: _ } => (),
    }
  }
  Ok(())
}

fn load_api(api_description: String, api_type: OpenApiType) -> Result<OpenAPI> {
  match api_type {
    OpenApiType::JSON => {
      let open_api: OpenAPI = serde_json::from_str(&api_description)?;
      Ok(open_api)
    }
    OpenApiType::YAML => {
      let open_api: OpenAPI = serde_yaml::from_str(&api_description)?;
      Ok(open_api)
    }
  }
}
