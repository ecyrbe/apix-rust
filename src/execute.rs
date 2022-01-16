use crate::manifests::ApixRequest;
use crate::requests::{make_request, AdvancedBody, RequestOptions};

use super::dialog::Dialog;
use super::template::{MapTemplate, StringTemplate, ValueTemplate};
use super::{ApixKind, ApixManifest};
use anyhow::Result;
use indexmap::IndexMap;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde_json::Value;
use std::collections::HashMap;
use std::str::FromStr;
use tera::{Context, Tera};

struct RequestTemplate<'a> {
  request: &'a ApixRequest,
  engine: Tera,
  context: Context,
  file: &'a str,
  annotations: IndexMap<String, String>,
}

#[derive(Debug, Clone)]
struct RequestParams<'a> {
  url: String,
  method: String,
  headers: HeaderMap,
  queries: IndexMap<String, String>,
  body: Option<AdvancedBody>,
  options: RequestOptions<'a>,
}

// ask for all parameters in manifest request
fn ask_for_required_parameters(
  request: &ApixRequest,
  params: &Option<IndexMap<String, String>>,
) -> Result<serde_json::Map<String, serde_json::Value>, anyhow::Error> {
  match params {
    Some(params) => request
      .parameters
      .iter()
      .filter(|param| param.required || params.get(&param.name).is_some())
      .map(|parameter| {
        if let Some(param) = params.get(&parameter.name) {
          Ok((parameter.name.clone(), Value::String(param.clone())))
        } else {
          Ok((parameter.name.clone(), parameter.ask()?))
        }
      })
      .collect(),
    None => request
      .parameters
      .iter()
      .filter(|param| param.required)
      .map(|parameter| Ok((parameter.name.clone(), parameter.ask()?)))
      .collect(),
  }
}

impl<'a> RequestTemplate<'a> {
  fn new(manifest: &'a ApixManifest, file: &'a str, params: &Option<IndexMap<String, String>>) -> Result<Self> {
    match manifest.kind() {
      ApixKind::Request(request) => {
        let parameters = Value::Object(ask_for_required_parameters(request, params)?);
        let env: HashMap<String, String> = std::env::vars().collect();
        let mut engine = Tera::default();
        let mut context = Context::new();

        context.insert("manifest", &manifest);
        context.insert("parameters", &parameters);
        context.insert("env", &env);

        let annotations = engine.render_map(
          &format!("{}#/annotations", file),
          manifest.get_annotations().unwrap_or(&IndexMap::<String, String>::new()),
          &context,
        )?;

        Ok(Self {
          request,
          engine,
          context,
          file,
          annotations,
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

  fn render_options(&mut self, options: &RequestOptions<'a>) -> RequestOptions<'a> {
    let output_filename = self.annotations.get("apix.io/output-file").map(String::to_owned);
    let proxy_url = self.annotations.get("apix.io/proxy-url").map(String::to_owned);
    let proxy_login = self.annotations.get("apix.io/proxy-login").map(String::to_owned);
    let proxy_password = self.annotations.get("apix.io/proxy-password").map(String::to_owned);
    let options = options.clone();
    RequestOptions {
      output_filename: options.output_filename.or(output_filename),
      proxy_url: options.proxy_url.or(proxy_url),
      proxy_login: options.proxy_login.or(proxy_login),
      proxy_password: options.proxy_password.or(proxy_password),
      ..options
    }
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

  fn render_queries(&mut self) -> Result<IndexMap<String, String>> {
    let queries = self.engine.render_map(
      &format!("{}#/queries", self.file),
      &self.request.request.queries,
      &self.context,
    )?;
    Ok(queries)
  }

  fn render_body(&mut self) -> Result<Option<AdvancedBody>> {
    match (
      self.request.request.body.as_ref(),
      self.annotations.get("apix.io/convert-body-to-json"),
      self.annotations.get("apix.io/body-file"),
    ) {
      (Some(Value::String(body)), Some(convert_to_json), _) if convert_to_json == "true" => {
        let string_body = self
          .engine
          .render_string(&format!("{}#/body", self.file), body, &self.context)?;
        // try to parse as json or return original string if it fails
        Ok(Some(AdvancedBody::Json(
          serde_json::from_str(&string_body).or::<serde_json::Error>(Ok(Value::String(string_body)))?,
        )))
      }
      (Some(body), _, _) => Ok(Some(AdvancedBody::Json(self.engine.render_value(
        &format!("{}#/body", self.file),
        body,
        &self.context,
      )?))),
      (None, _, Some(filepath)) => Ok(Some(AdvancedBody::File(filepath.to_owned()))),
      (None, _, None) => Ok(None),
    }
  }

  fn render_request_params(&mut self, options: &RequestOptions<'a>) -> Result<RequestParams> {
    let url = self.render_url()?;
    let method = self.render_method()?;
    let headers = self.render_headers()?;
    let queries = self.render_queries()?;
    let body = self.render_body()?;
    let options = self.render_options(options);
    Ok(RequestParams {
      url,
      method,
      headers,
      queries,
      body,
      options,
    })
  }
}

pub async fn handle_execute(
  file: &str,
  manifest: &ApixManifest,
  params: Option<IndexMap<String, String>>,
  options: RequestOptions<'_>,
) -> Result<()> {
  let mut template = RequestTemplate::new(manifest, file, &params)?;
  let params = template.render_context()?.render_request_params(&options)?;
  make_request(
    &params.url,
    &params.method,
    Some(&params.headers),
    Some(&params.queries),
    params.body,
    params.options,
  )
  .await
}
