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

pub async fn handle_execute(
  file: &str,
  manifest: ApixManifest,
  theme: &str,
  verbose: bool,
) -> Result<()> {
  match manifest.kind() {
    ApixKind::Request(request) => {
      let parameters = Value::Object(ask_for_required_parameters(&request)?);
      let env: HashMap<String, String> = std::env::vars().collect();
      let mut engine = Tera::default();
      let mut context = Context::new();
      context.insert("manifest", &manifest);
      context.insert("parameters", &parameters);
      context.insert("env", &env);
      context.insert(
        "context",
        &engine.render_value(
          &format!("{}#/context", file),
          &Value::Object(serde_json::Map::from_iter(
            request.context.clone().into_iter(),
          )),
          &context,
        )?,
      );

      let convert_body_string_to_json = manifest
        .get_annotation(&"apix.io/convert-body-string-to-json".to_string())
        .map(|v| bool::from_str(v).unwrap_or(false))
        .unwrap_or(false);

      engine.add_raw_template(&format!("{}#/url", file), &request.request.url)?;
      engine.add_raw_template(&format!("{}#/method", file), &request.request.method)?;
      let url = &engine.render(&format!("{}#/url", file), &context)?;
      let method = &engine.render(&format!("{}#/method", file), &context)?;
      let headers = &HeaderMap::from_iter(
        engine
          .render_map(
            &format!("{}#/headers", file),
            &request.request.headers,
            &context,
          )?
          .iter()
          .map(|(key, value)| {
            (
              HeaderName::from_str(key).unwrap(),
              HeaderValue::from_str(value).unwrap(),
            )
          }),
      );
      let body = match (request.request.body.as_ref(), convert_body_string_to_json) {
        (Some(Value::String(body)), true) => {
          let string_body = engine.render_string(&format!("{}#/body", file), body, &context)?;
          // try to parse as json and return original string if it fails
          serde_json::from_str(&string_body)
            .or::<serde_json::Error>(Ok(Value::String(string_body)))
            .ok()
        }
        (Some(body), _) => Some(engine.render_value(&format!("{}#/body", file), body, &context)?),
        (None, _) => None,
      };
      requests::make_request(
        url,
        method,
        Some(headers),
        None,
        match body {
          Some(body) => Some(serde_json::to_string(&body)?),
          None => None,
        },
        verbose,
        theme,
      )
      .await
    }
    _ => todo!(),
  }
}
