// use crate::manifests::{ApixApi, ApixManifest, ApixParameter, ApixRequest, ApixTemplate, Json};
// use anyhow::Result;
// use indexmap::IndexMap;
// use openapiv3::{OpenAPI, PathItem, ReferenceOr};
// use regex::Regex;
// use tokio::fs::File;
// use tokio::io::AsyncWriteExt;

// pub enum OpenApiType {
//     JSON,
//     YAML,
// }

// fn is_method(method: &str) -> bool {
//     ["get", "post", "put", "delete", "patch", "options", "head"]
//         .contains(&method.to_lowercase().as_ref())
// }

// // get parameter name from reference
// // example: #/components/parameters/id -> id
// fn get_reference_name(reference: &str) -> String {
//     reference.split('/').last().unwrap_or_default().to_string()
// }

// trait Replacable {
//     fn replace(&self, pattern: &str, replacement: &str) -> String;
// }

// impl Replacable for String {
//     fn replace(&self, pattern: &str, replacement: &str) -> String {
//         let re = Regex::new(pattern).unwrap();
//         re.replace_all(self, replacement).to_string()
//     }
// }

// trait ReferencableParameter {
//     fn get_parameter(&self, name: &str) -> Option<openapiv3::Parameter>;

//     fn resolve_parameter<'a>(
//         &'a self,
//         parameter: &'a ReferenceOr<openapiv3::Parameter>,
//     ) -> Option<openapiv3::Parameter>;
// }

// trait ReferencableSchema {
//     fn get_schema(&self, name: &str) -> Json<openapiv3::Schema>;

//     fn resolve_schema<'a>(
//         &'a self,
//         schema: &'a ReferenceOr<openapiv3::Schema>,
//     ) -> Json<openapiv3::Schema>;
// }

// trait ReferencableBody {
//     fn get_body(&self, name: &str) -> Option<openapiv3::RequestBody>;

//     fn resolve_body<'a>(
//         &'a self,
//         body: &'a ReferenceOr<openapiv3::RequestBody>,
//     ) -> Option<openapiv3::RequestBody>;
// }

// impl ReferencableParameter for OpenAPI {
//     fn get_parameter(&self, name: &str) -> Option<openapiv3::Parameter> {
//         match self.components.as_ref() {
//             Some(components) => match components.parameters.get(name)? {
//                 ReferenceOr::Reference { reference } => {
//                     self.get_parameter(&get_reference_name(reference))
//                 }
//                 ReferenceOr::Item(parameter) => Some(parameter.clone()),
//             },
//             None => None,
//         }
//     }

//     fn resolve_parameter<'a>(
//         &'a self,
//         parameter: &'a ReferenceOr<openapiv3::Parameter>,
//     ) -> Option<openapiv3::Parameter> {
//         match parameter {
//             ReferenceOr::Reference { reference } => {
//                 self.get_parameter(&get_reference_name(reference))
//             }
//             ReferenceOr::Item(parameter) => Some(parameter.clone()),
//         }
//     }
// }

// impl ReferencableSchema for OpenAPI {
//     fn get_schema(&self, name: &str) -> Json<openapiv3::Schema> {
//         match self.components.as_ref() {
//             Some(components) => match components.schemas.get(name)? {
//                 ReferenceOr::Reference { reference } => {
//                     self.get_schema(&get_reference_name(reference))
//                 }
//                 ReferenceOr::Item(schema) => Some(schema.clone()),
//             },
//             None => None,
//         }
//     }
//     fn resolve_schema<'a>(
//         &'a self,
//         schema: &'a ReferenceOr<openapiv3::Schema>,
//     ) -> Option<openapiv3::Schema> {
//         match schema {
//             ReferenceOr::Reference { reference } => self.get_schema(&get_reference_name(reference)),
//             ReferenceOr::Item(schema) => Some(schema.clone()),
//         }
//     }
// }

// impl ReferencableBody for OpenAPI {
//     fn get_body(&self, name: &str) -> Option<openapiv3::RequestBody> {
//         match self.components.as_ref() {
//             Some(components) => match components.request_bodies.get(name)? {
//                 ReferenceOr::Reference { reference } => {
//                     self.get_body(&get_reference_name(reference))
//                 }
//                 ReferenceOr::Item(body) => Some(body.clone()),
//             },
//             None => None,
//         }
//     }
//     fn resolve_body<'a>(
//         &'a self,
//         body: &'a ReferenceOr<openapiv3::RequestBody>,
//     ) -> Option<openapiv3::RequestBody> {
//         match body {
//             ReferenceOr::Reference { reference } => self.get_body(&get_reference_name(reference)),
//             ReferenceOr::Item(body) => Some(body.clone()),
//         }
//     }
// }

// pub fn openapi_operation_to_apix_request(operation: &openapiv3::Operation) -> Option<ApixRequest> {
//     todo!()
// }

// trait ToApixParameter {
//     fn to_apix_parameter(&self, api: &OpenAPI) -> Option<ApixParameter>;
// }

// impl ToApixParameter for openapiv3::Parameter {
//     fn to_apix_parameter(&self, api: &OpenAPI) -> Option<ApixParameter> {
//         let data = self.parameter_data_ref();
//         Some(ApixParameter::new(
//             data.name.clone(),
//             data.required,
//             data.description.clone(),
//             match &data.format {
//                 openapiv3::ParameterSchemaOrContent::Schema(schema) => api.resolve_schema(&schema),
//                 _ => return None,
//             },
//         ))
//     }
// }

// trait ToApixParameters {
//     fn to_apix_parameters(&self, api: &OpenAPI) -> Result<Vec<ApixParameter>>;
// }

// impl ToApixParameters for PathItem {
//     fn to_apix_parameters(&self, api: &OpenAPI) -> Result<Vec<ApixParameter>> {
//         let parameters = self
//             .parameters
//             .iter()
//             .filter_map(|maybe_ref_parameter| {
//                 Some(
//                     api.resolve_parameter(maybe_ref_parameter)?
//                         .to_apix_parameter(api)?,
//                 )
//             })
//             .collect();
//         Ok(parameters)
//     }
// }

// trait ToApixRequest {
//     fn to_apix_request(
//         &self,
//         method: &str,
//         operation: &openapiv3::Operation,
//     ) -> Option<ApixRequest>;
// }

// impl ToApixRequest for OpenAPI {
//     fn to_apix_request(
//         &self,
//         method: &str,
//         operation: &openapiv3::Operation,
//     ) -> Option<ApixRequest> {
//         let mut request = ApixRequest::new(
//             IndexMap::new(),
//             operation
//                 .parameters
//                 .iter()
//                 .filter_map(|maybe_ref_parameter| {
//                     Some(
//                         self.resolve_parameter(maybe_ref_parameter)?
//                             .to_apix_parameter(self)?,
//                     )
//                 })
//                 .collect(),
//             ApixTemplate::new(),
//         );
//         request.parameters = parameters;
//         request.body = operation.request_body.clone().map(|body| {
//             let body = api.resolve_body(&body)?;
//             ApixBody::new(
//                 body.description.clone(),
//                 body.content.clone(),
//                 body.required,
//             )
//         });
//         Some(request)
//     }
// }

// trait ToApixApiManifest {
//     fn to_apix_api(&self) -> Result<ApixManifest>;
// }

// impl ToApixApiManifest for OpenAPI {
//     fn to_apix_api(&self) -> Result<ApixManifest> {
//         //compute api name
//         let name = &self.info.title;
//         // create apixApi based on openapi
//         let url: Option<String> = {
//             let mut url = String::new();
//             for server in self.servers.iter() {
//                 if server.url.starts_with("http://") || server.url.starts_with("https://") {
//                     url = server.url.to_string();
//                     break;
//                 }
//             }
//             Some(url)
//         };
//         let api = ApixApi::new(
//             url.unwrap_or_default(),
//             self.info.version.clone(),
//             self.info.description.clone(),
//         );
//         Ok(ApixManifest::new_api(name.clone(), Some(api)))
//     }
// }

// trait ToApixRequestsManifest {
//     fn to_apix_requests(&self) -> Result<Vec<ApixManifest>>;
// }

// impl ToApixRequestsManifest for OpenAPI {
//     fn to_apix_requests(&self) -> Result<Vec<ApixManifest>> {
//         let mut apix_requests = Vec::new();
//         for (path, path_item) in self.paths.iter() {
//             match path_item {
//                 ReferenceOr::Item(path_item) => {
//                     for (method, operation) in path_item.iter() {
//                         if let Some(apix_request) = self.to_apix_request(method, operation) {
//                             apix_requests
//                                 .push(ApixManifest::new_request(path.clone(), apix_request));
//                         }
//                     }
//                 }
//                 ReferenceOr::Reference { .. } => {}
//             }
//         }
//         Ok(apix_requests)
//     }
// }

// // return an apix API and a vector of ApixManifest
// pub fn openapi_to_apix(api: &OpenAPI) -> Result<(ApixManifest, Vec<ApixManifest>)> {
//     let apix_api = api.to_apix_api()?;
//     let apix_requests = api.to_apix_requests()?;
//     Ok((apix_api, apix_requests))
// }

// pub async fn import_api(api_description: String, api_type: OpenApiType) -> Result<()> {
//     let api: OpenAPI = load_api(api_description, api_type)?;
//     // convert to apix
//     let (api, requests) = openapi_to_apix(&api)?;
//     // write apixApi to current directory with name of api
//     let mut file = File::create(format!("{}.index.yaml", &api.name())).await?;
//     file.write_all(serde_yaml::to_string(&api).unwrap().as_bytes())
//         .await?;
//     // write each request to current directory with name of request
//     for request in requests {
//         let mut file = File::create(format!("{}.{}.yaml", &api.name(), &request.name())).await?;
//         file.write_all(serde_yaml::to_string(&request).unwrap().as_bytes())
//             .await?;
//     }
//     Ok(())
// }

// fn load_api(api_description: String, api_type: OpenApiType) -> Result<OpenAPI> {
//     match api_type {
//         OpenApiType::JSON => {
//             let open_api: OpenAPI = serde_json::from_str(&api_description)?;
//             Ok(open_api)
//         }
//         OpenApiType::YAML => {
//             let open_api: OpenAPI = serde_yaml::from_str(&api_description)?;
//             Ok(open_api)
//         }
//     }
// }
