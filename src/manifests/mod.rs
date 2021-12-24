pub use self::config::ApixConfiguration;
use indexmap::{indexmap, IndexMap};
use openapiv3::Schema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ApixApi {
    pub url: String,
    pub version: String,
    pub description: Option<String>,
}

impl ApixApi {
    pub fn new(url: String, version: String, description: Option<String>) -> Self {
        Self {
            url,
            version,
            description,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApixParameter {
    name: String,
    required: bool,
    description: Option<String>,
    schema: Option<Schema>,
}

impl ApixParameter {
    pub fn new(
        name: String,
        required: bool,
        description: Option<String>,
        schema: Option<Schema>,
    ) -> Self {
        Self {
            name,
            required,
            description,
            schema,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApixTemplate {
    method: String,
    url: String,
    headers: IndexMap<String, String>,
    body: Option<serde_json::Value>,
}

impl ApixTemplate {
    pub fn new(
        method: String,
        url: String,
        headers: IndexMap<String, String>,
        body: Option<serde_json::Value>,
    ) -> Self {
        Self {
            method,
            url,
            headers,
            body,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApixRequest {
    definitions: IndexMap<String, Schema>,
    parameters: Vec<ApixParameter>,
    template: ApixTemplate,
}

impl ApixRequest {
    pub fn new(
        definitions: IndexMap<String, Schema>,
        parameters: Vec<ApixParameter>,
        template: ApixTemplate,
    ) -> Self {
        Self {
            definitions,
            parameters,
            template,
        }
    }
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
    extensions: IndexMap<String, String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ApixManifestV1 {
    metadata: ApixMetadata,
    #[serde(flatten)]
    kind: ApixKind,
}

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
    pub fn new_api(name: String, api: Option<ApixApi>) -> Self {
        ApixManifest::V1(ApixManifestV1 {
            metadata: ApixMetadata {
                name,
                labels: indexmap! { "app".to_string() => "apix".to_string()},
                annotations: IndexMap::new(),
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

    pub fn get_metadata(&self, key: &String) -> Option<&String> {
        match self {
            ApixManifest::V1(manifest) => manifest.metadata.extensions.get(key),
            ApixManifest::None => None,
        }
    }

    pub fn insert_metadata(&mut self, key: String, value: String) {
        match self {
            ApixManifest::V1(manifest) => {
                manifest.metadata.extensions.insert(key, value);
                ()
            }
            ApixManifest::None => (),
        }
    }
}

pub mod config;
