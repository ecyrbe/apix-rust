use super::http_display::{pretty_print, HttpDisplay};
use super::http_utils::Language;
use anyhow::Result;
use reqwest::{header::HeaderMap, Client, Method};
use std::collections::HashMap;
use std::str::FromStr;

pub async fn make_request(
  url: &str,
  method: &str,
  headers: HeaderMap,
  queries: &HashMap<String, String>,
  body: String,
  verbose: bool,
  theme: &str,
) -> Result<()> {
  let client = Client::new();
  let req = client
    .request(Method::from_str(&method.to_uppercase())?, url)
    .headers(headers)
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
