use std::path::{Path, PathBuf};

pub use self::config::ApixConfiguration;
pub mod config;

use anyhow::Result;
use indexmap::{indexmap, IndexMap};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use strum_macros::Display as EnumDisplay;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ApixApi {
  pub url: String,
  pub version: String,
  pub description: Option<String>,
}

impl ApixApi {
  #[allow(dead_code)]
  pub fn new(url: String, version: String, description: Option<String>) -> Self {
    Self {
      url,
      version,
      description,
    }
  }
}

fn default_schema() -> Option<Value> {
  Some(json!({ "type": "string" }))
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApixParameter {
  pub name: String,
  #[serde(default)]
  pub required: bool,
  #[serde(default)]
  pub password: bool,
  pub description: Option<String>,
  #[serde(default = "default_schema", skip_serializing_if = "Option::is_none")]
  pub schema: Option<Value>,
}

impl ApixParameter {
  #[allow(dead_code)]
  pub fn new(name: String, required: bool, password: bool, description: Option<String>, schema: Option<Value>) -> Self {
    Self {
      name,
      required,
      password,
      description,
      schema,
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApixStep {
  name: String,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  description: Option<String>,
  #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
  context: IndexMap<String, String>,
  #[serde(default, skip_serializing_if = "Option::is_none", rename = "if")]
  if_: Option<String>,
  request: ApixRequestTemplate,
}

/**
 * exemple of a story in yaml
 *
 * ```yaml
 * name: get_user
 * description: Get a user by retriving a token first
 * context:
 *   dev:
 *      url: "https://dev.apix.io"
 *   prod:
 *      url: "https://prod.apix.io"
 * steps:
 *  - name: "get_token"
 *    description: "Get a token"
 *    request:
 *      method: "GET"
 *      url: "{{story.variables.url}}/token"
 *      headers:
 *          Authorization: "Basic {{parameters.credentials}}"
 *          Accept: "application/json"
 * - name: "get_user"
 *   description: "Get a user"
 *   request:
 *      method: "GET"
 *      url: "{{story.variables.url}}/user/{{parameters.user}}"
 *      headers:
 *          Authorization: "Bearer {{steps.get_token.response.body.token}}"
 *          Accept: "application/json"
 */
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApixStory {
  name: String,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  needs: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  description: Option<String>,
  #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
  context: IndexMap<String, IndexMap<String, Value>>,
  steps: Vec<ApixStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApixStories {
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub parameters: Vec<ApixParameter>,
  pub stories: Vec<ApixStory>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApixRequestTemplate {
  pub method: String,
  pub url: String,
  #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
  pub headers: IndexMap<String, String>,
  #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
  pub queries: IndexMap<String, String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub body: Option<Value>,
}

impl ApixRequestTemplate {
  pub fn new(
    method: String,
    url: String,
    headers: IndexMap<String, String>,
    queries: IndexMap<String, String>,
    body: Option<Value>,
  ) -> Self {
    Self {
      method,
      url,
      headers,
      queries,
      body,
    }
  }
}

// exemple of an ApixRequest for a GET request in yaml
//
//  parameters:
//    - name: param
//      required: true
//      description: param description
//      schema:
//      type: string
//   template:
//     method: GET
//     url: /api/v1/resources/{param}
//     headers:
//       Accept: application/json

// exemple of an ApixRequest for a POST request with body template in yaml
//
//  parameters:
//    - name: param
//      required: true
//      description: param description
//      schema:
//      type: string
//   template:
//     method: POST
//     url: /api/v1/resources
//     headers:
//       Accept: application/json
//       Content-Type: application/json
//     body: |-
//       {
//          "param": {{param}}
//       }
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApixRequest {
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub parameters: Vec<ApixParameter>,
  #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
  pub context: IndexMap<String, Value>,
  pub request: ApixRequestTemplate,
}

impl ApixRequest {
  pub fn new(parameters: Vec<ApixParameter>, context: IndexMap<String, Value>, request: ApixRequestTemplate) -> Self {
    Self {
      parameters,
      context,
      request,
    }
  }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, EnumDisplay)]
#[serde(tag = "kind", content = "spec")]
pub enum ApixKind {
  Api(ApixApi),
  Configuration(ApixConfiguration),
  Request(ApixRequest),
  Story(ApixStories),
  None,
}

impl Default for ApixKind {
  fn default() -> Self {
    ApixKind::None
  }
}

impl ApixKind {
  #[allow(dead_code)]
  pub fn as_api(&self) -> Option<&ApixApi> {
    match self {
      ApixKind::Api(api) => Some(api),
      _ => None,
    }
  }

  #[allow(dead_code)]
  pub fn as_configuration(&self) -> Option<&ApixConfiguration> {
    match self {
      ApixKind::Configuration(configuration) => Some(configuration),
      _ => None,
    }
  }

  #[allow(dead_code)]
  pub fn as_request(&self) -> Option<&ApixRequest> {
    match self {
      ApixKind::Request(request) => Some(request),
      _ => None,
    }
  }

  #[allow(dead_code)]
  pub fn as_story(&self) -> Option<&ApixStories> {
    match self {
      ApixKind::Story(story) => Some(story),
      _ => None,
    }
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
  extensions: IndexMap<String, String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ApixManifestV1 {
  metadata: ApixMetadata,
  #[serde(flatten)]
  kind: ApixKind,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "apiVersion")]
pub enum ApixManifest {
  #[serde(rename = "apix.io/v1")]
  V1(ApixManifestV1),
  None,
}

impl Default for ApixManifest {
  fn default() -> Self {
    ApixManifest::None
  }
}

impl ApixManifest {
  pub fn find_manifests() -> Result<impl Iterator<Item = (PathBuf, ApixManifest)>> {
    let current_dir = std::env::current_dir()?;
    let manifests = std::fs::read_dir(current_dir)?.filter_map(|entry| {
      if let Ok(entry) = entry {
        let path = entry.path();
        if path.is_file() {
          match path.extension() {
            Some(ext) if ext == "yaml" || ext == "yml" => {
              if let Ok(manifest) = ApixManifest::from_file(&path) {
                return Some((path, manifest));
              }
            }
            _ => {}
          }
        }
      }
      None
    });
    Ok(manifests)
  }

  pub fn find_manifests_by_kind(kind: &str) -> Result<impl Iterator<Item = (PathBuf, ApixManifest)> + '_> {
    Self::find_manifests().map(move |manifests| {
      manifests.filter(move |(_, manifest)| match manifest {
        ApixManifest::V1(manifestv1) => manifestv1.kind.to_string().to_lowercase() == kind,
        _ => false,
      })
    })
  }

  pub fn find_manifest(kind: &str, name: &str) -> Option<(PathBuf, ApixManifest)> {
    Self::find_manifests()
      .ok()
      .map(|mut manifests| {
        manifests.find(|(_, manifest)| match manifest {
          ApixManifest::V1(manifest) => {
            manifest.kind.to_string().to_lowercase() == kind && manifest.metadata.name == name
          }
          _ => false,
        })
      })
      .flatten()
  }

  pub fn find_manifest_filename(kind: &str, name: &str) -> Option<String> {
    Self::find_manifest(kind, name)
      .map(|(path, _)| path.to_str().map(str::to_string))
      .flatten()
  }

  #[allow(dead_code)]
  pub fn new_api(name: String, api: Option<ApixApi>) -> Self {
    ApixManifest::V1(ApixManifestV1 {
      metadata: ApixMetadata {
        name,
        labels: indexmap! { "app".to_string() => "apix".to_string()},
        annotations: indexmap! {
            "apix.io/created-by".to_string() => whoami::username(),
            "apix.io/created-at".to_string() => chrono::Utc::now().to_rfc3339(),
        },
        extensions: IndexMap::new(),
      },
      kind: ApixKind::Api(api.unwrap_or_default()),
    })
  }

  pub fn new_request(api: String, name: String, request: ApixRequest) -> Self {
    ApixManifest::V1(ApixManifestV1 {
      metadata: ApixMetadata {
        name,
        labels: indexmap! {
            "app".to_string() => "apix".to_string(),
            "apix.io/api".to_string() => api,
        },
        annotations: indexmap! {
            "apix.io/created-by".to_string() => whoami::username(),
            "apix.io/created-at".to_string() => chrono::Utc::now().to_rfc3339(),
        },
        extensions: IndexMap::new(),
      },
      kind: ApixKind::Request(request),
    })
  }

  #[allow(dead_code)]
  pub fn new_stories(api: String, name: String, stories: ApixStories) -> Self {
    ApixManifest::V1(ApixManifestV1 {
      metadata: ApixMetadata {
        name,
        labels: indexmap! {
            "app".to_string() => "apix".to_string(),
            "apix.io/api".to_string() => api,
        },
        annotations: indexmap! {
            "apix.io/created-by".to_string() => whoami::username(),
            "apix.io/created-at".to_string() => chrono::Utc::now().to_rfc3339(),
        },
        extensions: IndexMap::new(),
      },
      kind: ApixKind::Story(stories),
    })
  }

  pub fn from_file(path: &Path) -> Result<Self> {
    let content = std::fs::read_to_string(path)?;
    let manifest = serde_yaml::from_str::<ApixManifest>(&content)?;
    Ok(manifest)
  }

  #[allow(dead_code)]
  pub fn name(&self) -> &str {
    match self {
      ApixManifest::V1(manifest) => &manifest.metadata.name,
      ApixManifest::None => "",
    }
  }

  #[allow(dead_code)]
  pub fn version(&self) -> &str {
    match self {
      ApixManifest::V1(_) => "apix.io/v1",
      ApixManifest::None => "",
    }
  }

  pub fn kind(&self) -> &ApixKind {
    match self {
      ApixManifest::V1(manifest) => &manifest.kind,
      ApixManifest::None => &ApixKind::None,
    }
  }

  #[allow(dead_code)]
  pub fn get_metadata(&self, key: &str) -> Option<&String> {
    match self {
      ApixManifest::V1(manifest) => manifest.metadata.extensions.get(key),
      ApixManifest::None => None,
    }
  }

  #[allow(dead_code)]
  pub fn insert_metadata(&mut self, key: String, value: String) {
    match self {
      ApixManifest::V1(manifest) => {
        manifest.metadata.extensions.insert(key, value);
      }
      ApixManifest::None => (),
    }
  }

  #[allow(dead_code)]
  pub fn get_annotation(&self, key: &str) -> Option<&String> {
    match self {
      ApixManifest::V1(manifest) => manifest.metadata.annotations.get(key),
      ApixManifest::None => None,
    }
  }

  #[allow(dead_code)]
  pub fn get_annotations(&self) -> Option<&IndexMap<String, String>> {
    match self {
      ApixManifest::V1(manifest) => Some(&manifest.metadata.annotations),
      ApixManifest::None => None,
    }
  }

  #[allow(dead_code)]
  pub fn get_label(&self, key: &str) -> Option<&String> {
    match self {
      ApixManifest::V1(manifest) => manifest.metadata.labels.get(key),
      ApixManifest::None => None,
    }
  }
}
