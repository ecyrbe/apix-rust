use reqwest::{header::CONTENT_TYPE, Request, Response};

pub trait HttpHeaders {
  fn headers(&self) -> &reqwest::header::HeaderMap;
}

impl HttpHeaders for Request {
  #[inline]
  fn headers(&self) -> &reqwest::header::HeaderMap {
    self.headers()
  }
}

impl HttpHeaders for Response {
  #[inline]
  fn headers(&self) -> &reqwest::header::HeaderMap {
    self.headers()
  }
}

pub trait Language {
  fn get_language(&self) -> Option<&'static str>;
}

impl<T> Language for T
where
  T: HttpHeaders,
{
  fn get_language(&self) -> Option<&'static str> {
    match self.headers().get(CONTENT_TYPE) {
      Some(header) => match header.to_str() {
        Ok(content_type) if content_type.contains("json") => Some("json"),
        Ok(content_type) if content_type.contains("xml") => Some("xml"),
        Ok(content_type) if content_type.contains("html") => Some("html"),
        Ok(content_type) if content_type.contains("css") => Some("css"),
        Ok(content_type) if content_type.contains("javascript") => Some("js"),
        _ => Some("txt"),
      },
      _ => None,
    }
  }
}
