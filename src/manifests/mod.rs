pub use self::config::ApixConfiguration;
pub mod config;

use indexmap::{indexmap, IndexMap};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

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

fn default_schema() -> Option<Value> {
    Some(json!({ "type": "string" }))
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApixParameter {
    name: String,
    required: bool,
    description: Option<String>,
    #[serde(default = "default_schema", skip_serializing_if = "Option::is_none")]
    schema: Option<Value>,
}

impl ApixParameter {
    pub fn new(
        name: String,
        required: bool,
        description: Option<String>,
        schema: Option<Value>,
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
pub struct ApixStep {
    name: String,
    description: Option<String>,
    context: IndexMap<String, String>,
    #[serde(rename = "if")]
    if_: Option<String>,
    request: ApixRequestTemplate,
}

/**
 * exemple of a story in yaml
 *
 * ```yaml
 * name: "get_user"
 * description: "Get a user by retriving a token first"
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
    needs: Option<String>,
    description: Option<String>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    context: IndexMap<String, IndexMap<String, Value>>,
    steps: Vec<ApixStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApixStories {
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub definitions: IndexMap<String, Value>,
    parameters: Option<Vec<ApixParameter>>,
    stories: Vec<ApixStory>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApixRequestTemplate {
    pub method: String,
    pub url: String,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub headers: IndexMap<String, String>,
    pub body: Option<Value>,
}

impl ApixRequestTemplate {
    pub fn new(
        method: String,
        url: String,
        headers: IndexMap<String, String>,
        body: Option<Value>,
    ) -> Self {
        Self {
            method,
            url,
            headers,
            body,
        }
    }
}

// exemple of an ApixRequest for a GET request in yaml
//
//  definitions:
//    param:
//      type: string
//  parameters:
//    - name: param
//      required: true
//      description: param description
//      schema:
//        $ref: '#/definitions/param'
//   template:
//     method: GET
//     url: /api/v1/resources/{param}
//     headers:
//       Accept: application/json

// exemple of an ApixRequest for a POST request with body template in yaml
//
//  definitions:
//    param:
//      type: string
//  parameters:
//    - name: param
//      required: true
//      description: param description
//      schema:
//        $ref: '#/definitions/param'
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
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub definitions: IndexMap<String, Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<ApixParameter>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub context: IndexMap<String, Value>,
    pub request: ApixRequestTemplate,
}

impl ApixRequest {
    pub fn new(
        definitions: IndexMap<String, Value>,
        parameters: Vec<ApixParameter>,
        context: IndexMap<String, Value>,
        request: ApixRequestTemplate,
    ) -> Self {
        Self {
            definitions,
            parameters,
            context,
            request,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

    pub fn get_annotation(&self, key: &String) -> Option<&String> {
        match self {
            ApixManifest::V1(manifest) => manifest.metadata.annotations.get(key),
            ApixManifest::None => None,
        }
    }

    pub fn get_label(&self, key: &String) -> Option<&String> {
        match self {
            ApixManifest::V1(manifest) => manifest.metadata.labels.get(key),
            ApixManifest::None => None,
        }
    }
}
