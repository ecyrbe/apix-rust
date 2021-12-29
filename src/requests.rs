use super::http_display::{pretty_print, HttpDisplay};
use super::http_utils::Language;
use anyhow::Result;
use lazy_static::lazy_static;
use reqwest::{
  header::{HeaderMap, HeaderValue, ACCEPT, ACCEPT_ENCODING, CONTENT_TYPE, USER_AGENT},
  Client, Method,
};
use std::collections::HashMap;
use std::str::FromStr;

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

lazy_static! {
  static ref DEFAULT_HEADERS: HeaderMap = HeaderMap::from_iter([
    (USER_AGENT, HeaderValue::from_str(APP_USER_AGENT).unwrap()),
    (ACCEPT, HeaderValue::from_static("application/json")),
    (ACCEPT_ENCODING, HeaderValue::from_static("gzip")),
    (CONTENT_TYPE, HeaderValue::from_static("application/json")),
  ]);
}

fn merge_with_defaults(headers: &HeaderMap) -> HeaderMap {
  let mut merged = DEFAULT_HEADERS.clone();
  for (key, value) in headers {
    merged.insert(key.clone(), value.clone());
  }
  merged
}

pub async fn make_request(
  url: &str,
  method: &str,
  headers: Option<&HeaderMap>,
  queries: Option<&HashMap<String, String>>,
  body: Option<String>,
  verbose: bool,
  theme: &str,
) -> Result<()> {
  let client = Client::builder().gzip(true).build()?;
  let mut builder = client.request(Method::from_str(&method.to_uppercase())?, url);
  if let Some(headers) = headers {
    builder = builder.headers(merge_with_defaults(&headers))
  } else {
    builder = builder.headers(DEFAULT_HEADERS.clone())
  }
  if let Some(query) = queries {
    builder = builder.query(query);
  }
  if let Some(body) = body {
    builder = builder.body(body);
  }
  let req = builder.build()?;
  if verbose {
    req.print(&theme)?;
    println!("");
  }
  let result = client.execute(req).await?;
  if verbose {
    result.print(&theme)?;
    println!("");
  }
  let language = result.get_language();
  let response_body = result.text().await?;
  pretty_print(
    response_body.as_bytes(),
    &theme,
    language.unwrap_or_default(),
  )?;
  println!("");
  Ok(())
}
