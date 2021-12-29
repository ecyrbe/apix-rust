use crate::manifests::ApixRequest;

use super::dialog::Dialog;
use super::requests;
use super::template::{MapTemplate, StringTemplate, ValueTemplate};
use super::{ApixKind, ApixManifest};
use anyhow::Result;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde_json::Value;
use std::collections::HashMap;
use std::str::FromStr;
use tera::{Context, Tera};

struct RequestTemplate<'a> {
  request: &'a ApixRequest,
  engine: Tera,
  context: Context,
  convert_body_to_json: bool,
  file: String,
}

struct RequestParams {
  url: String,
  method: String,
  headers: HeaderMap,
  body: Option<String>,
}

// ask for all parameters in manifest request
fn ask_for_required_parameters(
  request: &ApixRequest,
) -> Result<serde_json::Map<String, serde_json::Value>, anyhow::Error> {
  request
    .parameters
    .iter()
    .filter(|param| param.required)
    .map(|parameter| Ok((parameter.name.clone(), parameter.ask()?)))
    .collect()
}

impl<'a> RequestTemplate<'a> {
  fn new(manifest: &'a ApixManifest, file: String) -> Result<Self> {
    match manifest.kind() {
      ApixKind::Request(request) => {
        let parameters = Value::Object(ask_for_required_parameters(&request)?);
        let env: HashMap<String, String> = std::env::vars().collect();
        let engine = Tera::default();
        let mut context = Context::new();

        context.insert("manifest", &manifest);
        context.insert("parameters", &parameters);
        context.insert("env", &env);

        let convert_body_to_json = manifest
          .get_annotation(&"apix.io/convert-body-string-to-json".to_string())
          .map(|v| bool::from_str(v).unwrap_or(false))
          .unwrap_or(false);

        Ok(Self {
          request,
          engine,
          context,
          convert_body_to_json,
          file,
        })
      }
      _ => Err(anyhow::anyhow!("Request manifest expected")),
    }
  }

  fn render_context(&mut self) -> Result<&mut Self> {
    let rendered_context = self.engine.render_value(
      &format!("{}#/context", self.file),
      &Value::Object(serde_json::Map::from_iter(self.request.context.clone().into_iter())),
      &self.context,
    )?;
    self.context.insert("context", &rendered_context);
    Ok(self)
  }

  fn render_url(&mut self) -> Result<String> {
    self
      .engine
      .add_raw_template(&format!("{}#/url", self.file), &self.request.request.url)?;
    let url = self.engine.render(&format!("{}#/url", self.file), &self.context)?;
    Ok(url)
  }

  fn render_method(&mut self) -> Result<String> {
    self
      .engine
      .add_raw_template(&format!("{}#/method", self.file), &self.request.request.method)?;
    let method = self.engine.render(&format!("{}#/method", self.file), &self.context)?;
    Ok(method)
  }

  fn render_headers(&mut self) -> Result<HeaderMap> {
    let headers = HeaderMap::from_iter(
      self
        .engine
        .render_map(
          &format!("{}#/headers", self.file),
          &self.request.request.headers,
          &self.context,
        )?
        .iter()
        .map(|(key, value)| {
          (
            HeaderName::from_str(key).unwrap(),
            HeaderValue::from_str(value).unwrap(),
          )
        }),
    );
    Ok(headers)
  }

  fn render_body(&mut self) -> Result<Option<Value>> {
    match (self.request.request.body.as_ref(), self.convert_body_to_json) {
      (Some(Value::String(body)), true) => {
        let string_body = self
          .engine
          .render_string(&format!("{}#/body", self.file), body, &self.context)?;
        // try to parse as json or return original string if it fails
        Ok(
          serde_json::from_str(&string_body)
            .or::<serde_json::Error>(Ok(Value::String(string_body)))
            .ok(),
        )
      }
      (Some(body), _) => Ok(Some(self.engine.render_value(
        &format!("{}#/body", self.file),
        body,
        &self.context,
      )?)),
      (None, _) => Ok(None),
    }
  }

  fn render_request_params(&mut self) -> Result<RequestParams> {
    let url = self.render_url()?;
    let method = self.render_method()?;
    let headers = self.render_headers()?;
    let body = match self.render_body()? {
      Some(body) => Some(serde_json::to_string(&body)?),
      None => None,
    };
    Ok(RequestParams {
      url,
      method,
      headers,
      body,
    })
  }
}

pub async fn handle_execute(file: &str, manifest: &ApixManifest, theme: &str, verbose: bool) -> Result<()> {
  let params = RequestTemplate::new(manifest, file.to_string())?
    .render_context()?
    .render_request_params()?;
  requests::make_request(
    &params.url,
    &params.method,
    Some(&params.headers),
    None,
    params.body,
    verbose,
    theme,
  )
  .await
}
