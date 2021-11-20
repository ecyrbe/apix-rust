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
  headers: &HeaderMap,
  queries: &HashMap<String, String>,
  body: String,
  verbose: bool,
  theme: &str,
) -> Result<()> {
  let client = Client::builder().gzip(true).build()?;
  let req = client
    .request(Method::from_str(&method.to_uppercase())?, url)
    .headers(merge_with_defaults(&headers))
    .query(queries)
    .body(body)
    .build()?;
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
  let body = result.text().await?;
  pretty_print(body.as_bytes(), &theme, language.unwrap_or_default())
}
