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

//test get language for HttpHeaders
#[cfg(test)]
mod test_get_language {
  use super::*;
  use reqwest::header::CONTENT_TYPE;

  // Mock HttpHeaders
  struct MockHttpHeaders {
    headers: reqwest::header::HeaderMap,
  }

  // Mock HttpHeaders impl
  impl HttpHeaders for MockHttpHeaders {
    fn headers(&self) -> &reqwest::header::HeaderMap {
      &self.headers
    }
  }

  // Mock HttpHeaders impl
  impl MockHttpHeaders {
    fn new() -> MockHttpHeaders {
      MockHttpHeaders {
        headers: reqwest::header::HeaderMap::new(),
      }
    }

    fn set_content_type(&mut self, value: &str) {
      self.headers.insert(
        CONTENT_TYPE,
        reqwest::header::HeaderValue::from_str(value).unwrap(),
      );
    }
  }

  // test get language for json MockHttpHeaders
  #[test]
  fn test_get_language_for_json_mock_http_headers() {
    let mut mock_http_headers = MockHttpHeaders::new();
    mock_http_headers.set_content_type("application/json");
    let language = mock_http_headers.get_language();
    assert_eq!(language, Some("json"));
  }

  // test get language for xml MockHttpHeaders
  #[test]
  fn test_get_language_for_xml_mock_http_headers() {
    let mut mock_http_headers = MockHttpHeaders::new();
    mock_http_headers.set_content_type("application/xml");
    let language = mock_http_headers.get_language();
    assert_eq!(language, Some("xml"));
  }
  // test get language for html MockHttpHeaders
  #[test]
  fn test_get_language_for_html_mock_http_headers() {
    let mut mock_http_headers = MockHttpHeaders::new();
    mock_http_headers.set_content_type("text/html");
    let language = mock_http_headers.get_language();
    assert_eq!(language, Some("html"));
  }
  // test get language for css MockHttpHeaders
  #[test]
  fn test_get_language_for_css_mock_http_headers() {
    let mut mock_http_headers = MockHttpHeaders::new();
    mock_http_headers.set_content_type("text/css");
    let language = mock_http_headers.get_language();
    assert_eq!(language, Some("css"));
  }
  // test get language for javascript MockHttpHeaders
  #[test]
  fn test_get_language_for_javascript_mock_http_headers() {
    let mut mock_http_headers = MockHttpHeaders::new();
    mock_http_headers.set_content_type("application/javascript");
    let language = mock_http_headers.get_language();
    assert_eq!(language, Some("js"));
  }
  // test get language for txt MockHttpHeaders
  #[test]
  fn test_get_language_for_txt_mock_http_headers() {
    let mut mock_http_headers = MockHttpHeaders::new();
    mock_http_headers.set_content_type("text/plain");
    let language = mock_http_headers.get_language();
    assert_eq!(language, Some("txt"));
  }
  // test get language for other MockHttpHeaders
  #[test]
  fn test_get_language_for_other_mock_http_headers() {
    let mut mock_http_headers = MockHttpHeaders::new();
    mock_http_headers.set_content_type("application/other");
    let language = mock_http_headers.get_language();
    assert_eq!(language, Some("txt"));
  }
}
