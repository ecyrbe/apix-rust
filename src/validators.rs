use super::match_params::RequestParam;
use anyhow::Result;
use lazy_static::lazy_static;
use regex::Regex;
use url::Url;

pub fn validate_url(str_url: &str) -> Result<Url> {
  let url = Url::parse(str_url)?;
  if !["https", "http"].contains(&url.scheme()) {
    Err(anyhow::anyhow!(
      "Apix only supports http(s) protocols for now",
    ))
  } else {
    Ok(url)
  }
}

pub fn validate_param(param: &str, request_type: RequestParam) -> Result<()> {
  lazy_static! {
    static ref RE: Regex = Regex::new("^([\\w-]+):(.*)$").unwrap();
  }
  if RE.is_match(param) {
    Ok(())
  } else {
    Err(anyhow::anyhow!(
      "Bad {} format: \"{}\", should be of the form \"<name>:<value>\"",
      request_type,
      param
    ))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use test_case::test_case;
  use url::Url;

  // test validate url with test_case
  #[test_case("https://www.google.com")]
  #[test_case("http://www.google.com")]
  #[test_case("ftp://www.google.com" => panics )]
  fn test_validate_url(url: &str) {
    assert_eq!(validate_url(url).unwrap(), Url::parse(url).unwrap());
  }

  // test validate param with test_case
  #[test_case("name:value")]
  #[test_case("name-value" => panics)]
  fn test_validate_param(param: &str) {
    assert_eq!(validate_param(param, RequestParam::Header).unwrap(), ());
  }
}
