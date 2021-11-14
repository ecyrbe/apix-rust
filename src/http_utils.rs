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

//test get language for json reqwest::Request
#[cfg(test)]
mod test_get_language {
  use super::*;
  use reqwest::{header::CONTENT_TYPE, Client, Method};

  #[test]
  fn test_get_language_json() {
    let client = Client::new();
    let request = client
      .request(Method::GET, "http://localhost")
      .header(CONTENT_TYPE, "application/json")
      .build()
      .unwrap();
    assert_eq!(request.get_language().unwrap(), "json");
  }

  #[test]
  fn test_get_language_xml() {
    let client = Client::new();
    let request = client
      .request(Method::GET, "http://localhost")
      .header(CONTENT_TYPE, "application/xml")
      .build()
      .unwrap();
    assert_eq!(request.get_language().unwrap(), "xml");
  }
  #[test]
  fn test_get_language_html() {
    let client = Client::new();
    let request = client
      .request(Method::GET, "http://localhost")
      .header(CONTENT_TYPE, "text/html")
      .build()
      .unwrap();
    assert_eq!(request.get_language().unwrap(), "html");
  }
  #[test]
  fn test_get_language_css() {
    let client = Client::new();
    let request = client
      .request(Method::GET, "http://localhost")
      .header(CONTENT_TYPE, "text/css")
      .build()
      .unwrap();
    assert_eq!(request.get_language().unwrap(), "css");
  }
  #[test]
  fn test_get_language_js() {
    let client = Client::new();
    let request = client
      .request(Method::GET, "http://localhost")
      .header(CONTENT_TYPE, "application/javascript")
      .build()
      .unwrap();
    assert_eq!(request.get_language().unwrap(), "js");
  }
  #[test]
  fn test_get_language_txt() {
    let client = Client::new();
    let request = client
      .request(Method::GET, "http://localhost")
      .header(CONTENT_TYPE, "text/plain")
      .build()
      .unwrap();
    assert_eq!(request.get_language().unwrap(), "txt");
  }
}
