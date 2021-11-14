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
  use test_case::test_case;

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

    fn from_content_type(value: &str) -> MockHttpHeaders {
      let mut headers = MockHttpHeaders::new();
      headers.set_content_type(value);
      headers
    }
  }

  // test get language for all test cases
  #[test_case("application/json" => "json")]
  #[test_case("application/xml" => "xml")]
  #[test_case("text/html" => "html")]
  #[test_case("text/css" => "css")]
  #[test_case("text/javascript" => "js")]
  #[test_case("text/plain" => "txt")]
  fn test_get_language(content_type: &str) -> &str {
    MockHttpHeaders::from_content_type(content_type)
      .get_language()
      .unwrap()
  }
}
