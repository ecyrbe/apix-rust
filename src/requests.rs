use super::display::{pretty_print, print_separator, HttpDisplay};
use super::http_utils::Language;
use super::progress_component::FileProgressComponent;
use anyhow::Result;
use futures::stream::TryStreamExt;
use indexmap::IndexMap;
use once_cell::sync::Lazy;
use reqwest::{
  header::{HeaderMap, HeaderValue, ACCEPT, ACCEPT_ENCODING, CONTENT_LENGTH, CONTENT_TYPE, USER_AGENT},
  Body, Client, Method,
};
use serde_json::Value;
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

#[derive(Debug, Clone)]
pub enum AdvancedBody {
  Json(Value),
  String(String),
  File(String),
}

impl AdvancedBody {
  #[allow(dead_code)]
  pub fn to_string(&self) -> Result<String> {
    match self {
      AdvancedBody::Json(value) => Ok(serde_json::to_string(value)?),
      AdvancedBody::String(value) => Ok(value.to_string()),
      AdvancedBody::File(path) => Ok(std::fs::read_to_string(path)?),
    }
  }
}

#[derive(Debug, Clone)]
pub struct RequestOptions<'a> {
  pub verbose: bool,
  pub theme: &'a str,
  pub is_output_terminal: bool,
  pub output_filename: Option<String>,
  pub proxy_url: Option<String>,
  pub proxy_login: Option<String>,
  pub proxy_password: Option<String>,
}

pub async fn make_request(
  url: &str,
  method: &str,
  headers: Option<&HeaderMap>,
  queries: Option<&IndexMap<String, String>>,
  body: Option<AdvancedBody>,
  options: RequestOptions<'_>,
) -> Result<()> {
  let mut client_builder = Client::builder();
  if let Some(proxy_url) = options.proxy_url {
    let mut proxy = reqwest::Proxy::all(&proxy_url)?;
    if let (Some(proxy_login), Some(proxy_password)) = (options.proxy_login, options.proxy_password) {
      proxy = proxy.basic_auth(&proxy_login, &proxy_password);
    }
    client_builder = client_builder.proxy(proxy);
  }
  let client = client_builder.gzip(true).build()?;
  let mut builder = client.request(Method::from_str(&method.to_uppercase())?, url);
  if let Some(headers) = headers {
    builder = builder.headers(merge_with_defaults(headers))
  } else {
    builder = builder.headers(DEFAULT_HEADERS.clone())
  }
  if let Some(query) = queries {
    builder = builder.query(query);
  }
  match body {
    Some(AdvancedBody::String(body)) => {
      builder = builder.body(body);
    }
    Some(AdvancedBody::File(file_path)) => {
      let file =
        File::open(&file_path).map_err(|e| anyhow::anyhow!("Could not open File '{}'\nCause: {}", &file_path, e))?;
      let file_size = file.metadata()?.len();
      let progress_bar = FileProgressComponent::new_upload(file_path, file_size);
      let async_file = AsyncFile::from_std(file);
      let stream = FramedRead::new(async_file, BytesCodec::new()).inspect_ok(move |bytes| {
        progress_bar.update_progress(bytes.len() as u64);
      });
      builder = builder
        .header(CONTENT_LENGTH, file_size)
        .body(Body::wrap_stream(stream));
    }
    Some(AdvancedBody::Json(body)) => {
      builder = builder.json(&body);
    }
    None => {}
  }
  let req = builder.build()?;
  if options.verbose {
    req.print(options.theme, options.is_output_terminal)?;
    println!();
    print_separator();
  }
  let result = client.execute(req).await?;
  if options.verbose {
    result.print(options.theme, options.is_output_terminal)?;
    println!();
  }
  let language = result.get_language();
  if let Some("binary") = language {
    let url = Url::parse(url)?;
    let filename = if let Some(output_filename) = options.output_filename {
      output_filename
    } else {
      url
        .path_segments()
        .and_then(|segments| segments.last())
        .unwrap_or("unknown.bin")
        .to_owned()
    };

    let progress_bar = FileProgressComponent::new_download(filename.to_owned(), result.content_length().unwrap_or(0));
    let mut stream = result
      .bytes_stream()
      .inspect_ok(move |bytes| {
        progress_bar.update_progress(bytes.len() as u64);
      })
      .map_err(|e| futures::io::Error::new(futures::io::ErrorKind::Other, e))
      .into_async_read()
      .compat();
    if !options.is_output_terminal {
      tokio::io::copy(&mut stream, &mut tokio::io::stdout()).await?;
    } else {
      let mut file = AsyncFile::create(filename).await?;
      tokio::io::copy(&mut stream, &mut file).await?;
    }
  } else {
    let response_body = result.text().await?;
    if !response_body.is_empty() {
      if let Some(output_filename) = options.output_filename {
        let mut file = AsyncFile::create(output_filename).await?;
        tokio::io::copy(&mut response_body.as_bytes(), &mut file).await?;
      } else {
        pretty_print(
          response_body,
          options.theme,
          language.unwrap_or_default(),
          options.is_output_terminal,
        )?;
        println!();
      }
    }
  }
  Ok(())
}
