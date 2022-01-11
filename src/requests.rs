use super::display::{pretty_print, HttpDisplay};
use super::http_utils::Language;
use super::progress_component::FileProgressComponent;
use anyhow::Result;
use futures::stream::TryStreamExt;
use indexmap::IndexMap;
use once_cell::sync::Lazy;
use reqwest::{
  header::{HeaderMap, HeaderValue, ACCEPT, ACCEPT_ENCODING, CONTENT_TYPE, USER_AGENT},
  Body, Client, Method,
};
use std::fs::File;
use std::str::FromStr;
use tokio::fs::File as AsyncFile;
use tokio_util::codec::{BytesCodec, FramedRead};
use tokio_util::compat::FuturesAsyncReadCompatExt;
use url::Url;

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

static DEFAULT_HEADERS: Lazy<HeaderMap> = Lazy::new(|| {
  HeaderMap::from_iter([
    (USER_AGENT, HeaderValue::from_str(APP_USER_AGENT).unwrap()),
    (ACCEPT, HeaderValue::from_static("application/json")),
    (ACCEPT_ENCODING, HeaderValue::from_static("gzip")),
    (CONTENT_TYPE, HeaderValue::from_static("application/json")),
  ])
});

fn merge_with_defaults(headers: &HeaderMap) -> HeaderMap {
  let mut merged = DEFAULT_HEADERS.clone();
  for (key, value) in headers {
    merged.insert(key.clone(), value.clone());
  }
  merged
}

pub enum AdvancedBody {
  Json(serde_json::Value),
  String(String),
  File(String),
  None,
}

impl AdvancedBody {
  #[allow(dead_code)]
  pub fn to_string(&self) -> Result<String> {
    match self {
      AdvancedBody::Json(value) => Ok(serde_json::to_string(value)?),
      AdvancedBody::String(value) => Ok(value.to_string()),
      AdvancedBody::File(path) => Ok(std::fs::read_to_string(path)?),
      AdvancedBody::None => Ok(String::new()),
    }
  }
}

pub async fn make_request(
  url: &str,
  method: &str,
  headers: Option<&HeaderMap>,
  queries: Option<&IndexMap<String, String>>,
  body: AdvancedBody,
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
  match body {
    AdvancedBody::String(body) => {
      builder = builder.body(body);
    }
    AdvancedBody::File(file_path) => {
      let file =
        File::open(&file_path).map_err(|e| anyhow::anyhow!("Could not open File '{}'\nCause: {}", &file_path, e))?;
      let file_size = file.metadata()?.len();
      let progress_bar = FileProgressComponent::new_upload(file_path, file_size);
      let async_file = AsyncFile::from_std(file);
      let stream = FramedRead::new(async_file, BytesCodec::new()).inspect_ok(move |bytes| {
        progress_bar.update_progress(bytes.len() as u64);
      });
      builder = builder.body(Body::wrap_stream(stream));
    }
    AdvancedBody::Json(body) => {
      builder = builder.json(&body);
    }
    AdvancedBody::None => {}
  }
  let req = builder.build()?;
  if verbose {
    req.print(&theme)?;
    println!();
  }
  let result = client.execute(req).await?;
  if verbose {
    result.print(&theme)?;
    println!();
  }
  let language = result.get_language();
  if let Some("binary") = language {
    let url = Url::parse(url)?;
    let filename = url
      .path_segments()
      .and_then(|segments| segments.last())
      .unwrap_or("unknown.bin"); // Fallback to generic filename

    let progress_bar = FileProgressComponent::new_download(filename.to_owned(), result.content_length().unwrap_or(0));
    let mut stream = result
      .bytes_stream()
      .inspect_ok(move |bytes| {
        progress_bar.update_progress(bytes.len() as u64);
      })
      .map_err(|e| futures::io::Error::new(futures::io::ErrorKind::Other, e))
      .into_async_read()
      .compat();

    let mut file = AsyncFile::create(filename).await?;
    tokio::io::copy(&mut stream, &mut file).await?;
  } else {
    let response_body = result.text().await?;
    if !response_body.is_empty() {
      pretty_print(response_body.as_bytes(), &theme, language.unwrap_or_default())?;
      println!();
    }
  }
  Ok(())
}
