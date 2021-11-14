use super::cli_params::RequestParam;
use anyhow::Result;
use lazy_static::lazy_static;
use regex::Regex;
use url::Url;

pub fn validate_url(str_url: &str) -> Result<Url> {
  let parsed_url = Url::parse(str_url);
  match parsed_url {
    Ok(url) => {
      if !["https", "http"].contains(&url.scheme()) {
        Err(anyhow::anyhow!(
          "Apix only supports http(s) protocols for now",
        ))
      } else {
        Ok(url)
      }
    }
    Err(err) => Err(anyhow::anyhow!("{}", err)),
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
