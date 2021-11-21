use crate::manifests::{ApixApi, ApixManifest};
use anyhow::Result;
use openapiv3::{OpenAPI, ReferenceOr};
use std::fs;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

pub enum OpenApiType {
  JSON,
  YAML,
}

pub async fn import_api(api_description: String, api_type: OpenApiType) -> Result<()> {
  let api: OpenAPI = load_api(api_description, api_type)?;
  let name = api.info.title.replace(" ", "-");
  let manifest = ApixManifest::new_api(
    name.clone(),
    Some(ApixApi {
      url: api.servers[0].url.clone(),
      description: api.info.description.or(Some(api.info.title)),
      version: api.info.version,
    }),
  );
  let dirname = std::env::current_dir()?.join(&name);
  let filename = dirname.join(format!("api.{}.yaml", &name));
  fs::create_dir_all(dirname)?;

  let mut file = File::create(filename).await?;
  let content = serde_yaml::to_string(&manifest)?;
  file.write_all(content.as_bytes()).await?;
  file.flush().await?;

  for (_path, item) in api.paths.paths {
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
